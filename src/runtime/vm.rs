use std::fmt;

use self::support::{normalize_slice_bound, slice_bounds_error_message, zero_value_for_type};
use self::unwind::{CallFrame, DeferredCall, PanicPayload, ReturnMode};
use crate::bytecode::instruction::{Instruction, Program, SequenceKind, ValueType};
use crate::conversion::ConversionKind;
use crate::runtime::value::{
    ChannelReceiveError, ChannelReceiveResult, ChannelSendError, ChannelValue, MapKey, MapValue,
    SliceValue, StringValue, Value,
};

mod builtins;
mod calls;
mod interfaces;
mod packages;
mod support;
mod unwind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeError {
    kind: RuntimeErrorKind,
}

impl RuntimeError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            kind: RuntimeErrorKind::Message(message.into()),
        }
    }

    fn user_panic_message(message: impl Into<String>) -> Self {
        Self {
            kind: RuntimeErrorKind::Panic(PanicPayload::Message(message.into())),
        }
    }

    fn panic_value(value: Value, value_type: ValueType) -> Self {
        Self {
            kind: RuntimeErrorKind::Panic(PanicPayload::Value { value, value_type }),
        }
    }

    fn panic_nil() -> Self {
        Self {
            kind: RuntimeErrorKind::Panic(PanicPayload::NilArgument),
        }
    }

    fn from_panic_payload(payload: PanicPayload) -> Self {
        Self {
            kind: RuntimeErrorKind::Panic(payload),
        }
    }

    fn into_panic_payload(self) -> Option<PanicPayload> {
        match self.kind {
            RuntimeErrorKind::Message(_) => None,
            RuntimeErrorKind::Panic(payload) => Some(payload),
        }
    }

    fn with_output(self, output: String) -> Self {
        if output.is_empty() {
            return self;
        }

        Self::new(format!("{output}{self}"))
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            RuntimeErrorKind::Message(message) => f.write_str(message),
            RuntimeErrorKind::Panic(payload) => write!(f, "panic: {}", payload.render_message()),
        }
    }
}

impl std::error::Error for RuntimeError {}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RuntimeErrorKind {
    Message(String),
    Panic(PanicPayload),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionResult {
    pub output: String,
}

impl ExecutionResult {
    pub fn render_output(&self) -> String {
        self.output.clone()
    }
}

#[derive(Debug)]
pub struct VirtualMachine {
    stack: Vec<Value>,
    frames: Vec<CallFrame>,
    output: String,
    pending_panic: Option<PanicPayload>,
    panic_depth: usize,
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            frames: Vec::new(),
            output: String::new(),
            pending_panic: None,
            panic_depth: 0,
        }
    }

    pub fn execute(&mut self, program: &Program) -> Result<ExecutionResult, RuntimeError> {
        self.stack.clear();
        self.frames.clear();
        self.output.clear();
        self.pending_panic = None;
        self.panic_depth = 0;

        let entry = program
            .functions
            .get(program.entry_function_index)
            .ok_or_else(|| RuntimeError::new("entry function index is invalid"))?;
        self.frames.push(CallFrame::new(
            program.entry_function_index,
            entry.local_names.len(),
            entry.parameter_count,
            ReturnMode::Normal,
            entry.return_types.clone(),
            false,
        ));

        while let Some(frame_index) = self.frames.len().checked_sub(1) {
            if self.panic_ready_to_unwind() {
                if let Some(error) = self.resume_panic_unwind(program)? {
                    return Err(self.decorate_runtime_error(error));
                }
                continue;
            }
            if self.frames[frame_index].returning {
                match self.resume_return(program) {
                    Ok(Some(result)) => return Ok(result),
                    Ok(None) => {}
                    Err(error) => {
                        if let Some(payload) = error.clone().into_panic_payload() {
                            self.begin_panic(payload);
                            continue;
                        }
                        return Err(self.decorate_runtime_error(error));
                    }
                }
                continue;
            }

            let function_index = self.frames[frame_index].function_index;
            let pc = self.frames[frame_index].pc;
            let function = program.functions.get(function_index).ok_or_else(|| {
                RuntimeError::new(format!("invalid function index {function_index}"))
            })?;
            let instruction = function.instructions.get(pc).cloned().ok_or_else(|| {
                RuntimeError::new(format!(
                    "instruction pointer {pc} is out of bounds in function `{}`",
                    function.name
                ))
            })?;

            let mut advance_pc = true;
            let step_result = (|| -> Result<(), RuntimeError> {
                match instruction {
                    Instruction::PushInt(value) => self.stack.push(Value::Integer(value)),
                    Instruction::PushByte(value) => self.stack.push(Value::Byte(value)),
                    Instruction::PushBool(value) => self.stack.push(Value::Boolean(value)),
                    Instruction::PushString(value) => {
                        self.stack.push(Value::String(StringValue::from(value)))
                    }
                    Instruction::PushNilInterface => self.stack.push(Value::nil_any()),
                    Instruction::PushNilSlice => self.stack.push(Value::Slice(SliceValue::nil())),
                    Instruction::PushNilChan => self.stack.push(Value::Chan(ChannelValue::nil())),
                    Instruction::PushNilMap => self.stack.push(Value::Map(MapValue::nil())),
                    Instruction::BuildSlice(count) => {
                        let mut elements = Vec::with_capacity(count);
                        for _ in 0..count {
                            elements.push(self.pop_value()?);
                        }
                        elements.reverse();
                        self.stack.push(Value::Slice(SliceValue::new(elements)));
                    }
                    Instruction::BuildMap {
                        map_type,
                        entry_count,
                    } => self.build_map(&map_type, entry_count)?,
                    Instruction::MakeSlice {
                        element_type,
                        has_capacity,
                    } => self.make_slice(&element_type, has_capacity)?,
                    Instruction::MakeChan {
                        element_type,
                        has_buffer,
                    } => self.make_chan(&element_type, has_buffer)?,
                    Instruction::MakeMap { map_type, has_hint } => {
                        self.make_map(&map_type, has_hint)?
                    }
                    Instruction::Convert(conversion) => self.convert_value(conversion)?,
                    Instruction::BoxAny(value_type) => self.box_any(value_type)?,
                    Instruction::LoadLocal(index) => {
                        let value = self.frames[frame_index]
                            .locals
                            .get(index)
                            .cloned()
                            .ok_or_else(|| {
                                RuntimeError::new(format!("invalid local slot {index}"))
                            })?;
                        self.stack.push(value);
                    }
                    Instruction::StoreLocal(index) => {
                        let value = self.pop_value()?;
                        let slot =
                            self.frames[frame_index]
                                .locals
                                .get_mut(index)
                                .ok_or_else(|| {
                                    RuntimeError::new(format!("invalid local slot {index}"))
                                })?;
                        *slot = value;
                    }
                    Instruction::Add => self.binary_numeric_op(
                        |left, right| left + right,
                        |left, right| left.wrapping_add(right),
                    )?,
                    Instruction::Concat => self.concat_strings()?,
                    Instruction::Subtract => self.binary_numeric_op(
                        |left, right| left - right,
                        |left, right| left.wrapping_sub(right),
                    )?,
                    Instruction::Multiply => self.binary_numeric_op(
                        |left, right| left * right,
                        |left, right| left.wrapping_mul(right),
                    )?,
                    Instruction::Divide => self.binary_numeric_op_checked(
                        |left, right| {
                            if right == 0 {
                                Err(RuntimeError::new("division by zero"))
                            } else {
                                Ok(left / right)
                            }
                        },
                        |left, right| {
                            if right == 0 {
                                Err(RuntimeError::new("division by zero"))
                            } else {
                                Ok(left / right)
                            }
                        },
                    )?,
                    Instruction::Equal => self.binary_compare(true)?,
                    Instruction::NotEqual => self.binary_compare(false)?,
                    Instruction::Less => self.binary_integer_compare(|left, right| left < right)?,
                    Instruction::LessEqual => {
                        self.binary_integer_compare(|left, right| left <= right)?
                    }
                    Instruction::Greater => {
                        self.binary_integer_compare(|left, right| left > right)?
                    }
                    Instruction::GreaterEqual => {
                        self.binary_integer_compare(|left, right| left >= right)?
                    }
                    Instruction::Index(target_kind) => self.index_value(target_kind)?,
                    Instruction::Slice {
                        target_kind,
                        has_low,
                        has_high,
                    } => self.slice_value(target_kind, has_low, has_high)?,
                    Instruction::Receive(element_type) => self.receive(&element_type)?,
                    Instruction::IndexMap(map_type) => self.index_map(&map_type)?,
                    Instruction::LookupMap(map_type) => self.lookup_map(&map_type)?,
                    Instruction::MapKeys(key_type) => self.map_keys(&key_type)?,
                    Instruction::SetIndex => self.set_slice_index()?,
                    Instruction::SetMapIndex => self.set_map_index()?,
                    Instruction::Send => self.send()?,
                    Instruction::Jump(target) => {
                        self.frames[frame_index].pc = target;
                        advance_pc = false;
                    }
                    Instruction::JumpIfFalse(target) => {
                        if !self.pop_bool()? {
                            self.frames[frame_index].pc = target;
                            advance_pc = false;
                        }
                    }
                    Instruction::Pop => {
                        self.pop_value()?;
                    }
                    Instruction::CallBuiltin(builtin, arity) => {
                        self.call_builtin(builtin, arity)?
                    }
                    Instruction::CallBuiltinSpread(builtin, prefix_arity) => {
                        self.call_builtin_spread(builtin, prefix_arity)?
                    }
                    Instruction::Panic(value_type) => {
                        let value = self.pop_value()?;
                        return Err(RuntimeError::panic_value(value, value_type));
                    }
                    Instruction::PanicNil => return Err(RuntimeError::panic_nil()),
                    Instruction::CallPackage(function, arity) => {
                        self.call_package_function(function, arity)?
                    }
                    Instruction::CallPackageSpread(function, prefix_arity) => {
                        self.call_package_function_spread(function, prefix_arity)?
                    }
                    Instruction::CallFunction(function_index, arity) => {
                        let arguments = self.pop_arguments(arity)?;
                        self.call_user_function(
                            program,
                            frame_index,
                            function_index,
                            arguments,
                            ReturnMode::Normal,
                            true,
                            None,
                        )?;
                        advance_pc = false;
                    }
                    Instruction::CallFunctionSpread(function_index, prefix_arity) => {
                        let spread = self.pop_value()?;
                        let mut arguments = self.pop_arguments(prefix_arity)?;
                        let spread_arguments =
                            self.expand_user_function_spread_arguments(spread)?;
                        arguments.extend(spread_arguments);
                        self.call_user_function(
                            program,
                            frame_index,
                            function_index,
                            arguments,
                            ReturnMode::Normal,
                            true,
                            None,
                        )?;
                        advance_pc = false;
                    }
                    Instruction::DeferBuiltin(builtin, arity) => {
                        let arguments = self.pop_arguments(arity)?;
                        self.frames[frame_index]
                            .deferred_calls
                            .push(DeferredCall::Builtin { builtin, arguments });
                    }
                    Instruction::DeferPanic(value_type) => {
                        let value = self.pop_value()?;
                        self.frames[frame_index]
                            .deferred_calls
                            .push(DeferredCall::PanicValue { value, value_type });
                    }
                    Instruction::DeferPanicNil => {
                        self.frames[frame_index]
                            .deferred_calls
                            .push(DeferredCall::PanicNil);
                    }
                    Instruction::DeferPackage(function, arity) => {
                        let arguments = self.pop_arguments(arity)?;
                        self.frames[frame_index]
                            .deferred_calls
                            .push(DeferredCall::Package {
                                function,
                                arguments,
                            });
                    }
                    Instruction::DeferPackageSpread(function, prefix_arity) => {
                        let spread = self.pop_value()?;
                        let arguments = self.pop_arguments(prefix_arity)?;
                        let arguments = self.expand_package_spread_arguments(arguments, spread)?;
                        self.frames[frame_index]
                            .deferred_calls
                            .push(DeferredCall::Package {
                                function,
                                arguments,
                            });
                    }
                    Instruction::DeferFunction(function_index, arity) => {
                        let arguments = self.pop_arguments(arity)?;
                        self.frames[frame_index]
                            .deferred_calls
                            .push(DeferredCall::UserDefined {
                                function_index,
                                arguments,
                            });
                    }
                    Instruction::DeferFunctionSpread(function_index, prefix_arity) => {
                        let spread = self.pop_value()?;
                        let mut arguments = self.pop_arguments(prefix_arity)?;
                        let spread_arguments =
                            self.expand_user_function_spread_arguments(spread)?;
                        arguments.extend(spread_arguments);
                        self.frames[frame_index]
                            .deferred_calls
                            .push(DeferredCall::UserDefined {
                                function_index,
                                arguments,
                            });
                    }
                    Instruction::Return => {
                        if self.frames[frame_index].pending_return_values.is_none() {
                            let mut return_values = Vec::with_capacity(function.return_types.len());
                            for _ in 0..function.return_types.len() {
                                return_values.push(self.pop_value()?);
                            }
                            return_values.reverse();
                            self.frames[frame_index].pending_return_values = Some(return_values);
                        }
                        self.frames[frame_index].returning = true;
                        advance_pc = false;
                    }
                }
                Ok(())
            })();

            match step_result {
                Ok(()) => {
                    if advance_pc {
                        self.frames[frame_index].pc += 1;
                    }
                }
                Err(error) => {
                    if let Some(payload) = error.clone().into_panic_payload() {
                        self.begin_panic(payload);
                        continue;
                    }
                    return Err(self.decorate_runtime_error(error));
                }
            }
        }

        if let Some(error) = self.finish_pending_panic() {
            return Err(self.decorate_runtime_error(error));
        }

        Ok(ExecutionResult {
            output: self.output.clone(),
        })
    }

    fn binary_numeric_op(
        &mut self,
        integer_operation: impl FnOnce(i64, i64) -> i64,
        byte_operation: impl FnOnce(u8, u8) -> u8,
    ) -> Result<(), RuntimeError> {
        self.binary_numeric_op_checked(
            |left, right| Ok(integer_operation(left, right)),
            |left, right| Ok(byte_operation(left, right)),
        )
    }

    fn binary_numeric_op_checked(
        &mut self,
        integer_operation: impl FnOnce(i64, i64) -> Result<i64, RuntimeError>,
        byte_operation: impl FnOnce(u8, u8) -> Result<u8, RuntimeError>,
    ) -> Result<(), RuntimeError> {
        let right = self.pop_value()?;
        let left = self.pop_value()?;
        let value = match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => {
                Value::Integer(integer_operation(left, right)?)
            }
            (Value::Byte(left), Value::Byte(right)) => Value::Byte(byte_operation(left, right)?),
            (left, right) => {
                return Err(RuntimeError::new(format!(
                    "arithmetic expected matching `int` or `byte` operands, found `{}` and `{}`",
                    support::runtime_type_name(&left),
                    support::runtime_type_name(&right)
                )));
            }
        };
        self.stack.push(value);
        Ok(())
    }

    fn binary_integer_compare(
        &mut self,
        operation: impl FnOnce(i64, i64) -> bool,
    ) -> Result<(), RuntimeError> {
        let right = self.pop_integer()?;
        let left = self.pop_integer()?;
        self.stack.push(Value::Boolean(operation(left, right)));
        Ok(())
    }

    fn binary_compare(&mut self, expect_equal: bool) -> Result<(), RuntimeError> {
        let right = self.pop_value()?;
        let left = self.pop_value()?;
        let equals = self.compare_values_for_equality(left, right)?;
        self.stack
            .push(Value::Boolean(if expect_equal { equals } else { !equals }));
        Ok(())
    }

    fn concat_strings(&mut self) -> Result<(), RuntimeError> {
        let right = self.pop_string()?;
        let left = self.pop_string()?;
        self.stack.push(Value::String(left.concat(&right)));
        Ok(())
    }

    fn expand_user_function_spread_arguments(
        &self,
        spread: Value,
    ) -> Result<Vec<Value>, RuntimeError> {
        match spread {
            Value::Slice(slice) => Ok(slice.visible_elements()),
            _ => Err(RuntimeError::new(
                "function call with `...` requires a slice spread argument",
            )),
        }
    }

    fn index_value(&mut self, target_kind: SequenceKind) -> Result<(), RuntimeError> {
        let index = self.pop_integer()?;
        if index < 0 {
            return Err(RuntimeError::new(format!(
                "{} index {index} is out of bounds",
                target_kind.render()
            )));
        }
        let index = index as usize;
        let target = self.pop_value()?;
        let value = match (target_kind, target) {
            (SequenceKind::Slice, Value::Slice(slice)) => slice.get(index).ok_or_else(|| {
                RuntimeError::new(format!("slice index {} is out of bounds", index))
            })?,
            (SequenceKind::String, Value::String(value)) => {
                value.byte_at(index).map(Value::Byte).ok_or_else(|| {
                    RuntimeError::new(format!("string index {} is out of bounds", index))
                })?
            }
            (SequenceKind::Slice, _) => {
                return Err(RuntimeError::new("index target is not a slice"));
            }
            (SequenceKind::String, _) => {
                return Err(RuntimeError::new("index target is not a string"));
            }
        };
        self.stack.push(value);
        Ok(())
    }

    fn slice_value(
        &mut self,
        target_kind: SequenceKind,
        has_low: bool,
        has_high: bool,
    ) -> Result<(), RuntimeError> {
        let high = if has_high {
            Some(self.pop_integer()?)
        } else {
            None
        };
        let low = if has_low {
            Some(self.pop_integer()?)
        } else {
            None
        };
        let target = self.pop_value()?;

        match (target_kind, target) {
            (SequenceKind::Slice, Value::Slice(slice)) => {
                let low = normalize_slice_bound(low, 0, "lower")?;
                let high_default = i64::try_from(slice.len())
                    .map_err(|_| RuntimeError::new("slice length could not convert to integer"))?;
                let high = normalize_slice_bound(high, high_default, "upper")?;
                let result = slice
                    .slice(low, high)
                    .map_err(|_| RuntimeError::new(slice_bounds_error_message(low, high)))?;
                self.stack.push(Value::Slice(result));
            }
            (SequenceKind::String, Value::String(value)) => {
                let low = normalize_slice_bound(low, 0, "lower")?;
                let high_default = i64::try_from(value.len())
                    .map_err(|_| RuntimeError::new("string length could not convert to integer"))?;
                let high = normalize_slice_bound(high, high_default, "upper")?;
                let result = value
                    .slice(low, high)
                    .map_err(|_| RuntimeError::new(slice_bounds_error_message(low, high)))?;
                self.stack.push(Value::String(result));
            }
            (SequenceKind::Slice, _) => {
                return Err(RuntimeError::new("slice expression target is not a slice"));
            }
            (SequenceKind::String, _) => {
                return Err(RuntimeError::new("slice expression target is not a string"));
            }
        }

        Ok(())
    }

    fn set_slice_index(&mut self) -> Result<(), RuntimeError> {
        let value = self.pop_value()?;
        let index = self.pop_integer()?;
        if index < 0 {
            return Err(RuntimeError::new(format!(
                "slice index {index} is out of bounds"
            )));
        }
        let target = self.pop_value()?;
        let slice = match target {
            Value::Slice(slice) => slice,
            _ => return Err(RuntimeError::new("index assignment target is not a slice")),
        };
        slice
            .set(index as usize, value)
            .map_err(|_| RuntimeError::new(format!("slice index {index} is out of bounds")))?;
        Ok(())
    }

    fn set_map_index(&mut self) -> Result<(), RuntimeError> {
        let value = self.pop_value()?;
        let key = self.pop_map_key()?;
        let target = self.pop_value()?;
        let map = match target {
            Value::Map(map) => map,
            _ => return Err(RuntimeError::new("index assignment target is not a map")),
        };
        map.insert(key, value)
            .map_err(|_| RuntimeError::user_panic_message("assignment to entry in nil map"))?;
        Ok(())
    }

    fn send(&mut self) -> Result<(), RuntimeError> {
        let value = self.pop_value()?;
        let target = self.pop_value()?;
        let channel = match target {
            Value::Chan(channel) => channel,
            _ => return Err(RuntimeError::new("send target is not a channel")),
        };
        channel.send(value).map_err(|error| match error {
            ChannelSendError::Nil => RuntimeError::new("send on nil channel would block"),
            ChannelSendError::Closed => RuntimeError::user_panic_message("send on closed channel"),
            ChannelSendError::WouldBlock => {
                RuntimeError::new("send would block in the current single-threaded VM")
            }
        })
    }

    fn make_slice(
        &mut self,
        element_type: &ValueType,
        has_capacity: bool,
    ) -> Result<(), RuntimeError> {
        let capacity = if has_capacity { self.pop_integer()? } else { 0 };
        let length = self.pop_integer()?;
        if length < 0 {
            return Err(RuntimeError::new(format!(
                "builtin `make` length must be non-negative, found {length}"
            )));
        }
        let capacity = if has_capacity { capacity } else { length };
        if capacity < 0 {
            return Err(RuntimeError::new(format!(
                "builtin `make` capacity must be non-negative, found {capacity}"
            )));
        }
        if length > capacity {
            return Err(RuntimeError::new(format!(
                "builtin `make` length {length} exceeds capacity {capacity}"
            )));
        }
        let length = usize::try_from(length).map_err(|_| {
            RuntimeError::new("builtin `make` length could not convert to runtime size")
        })?;
        let capacity = usize::try_from(capacity).map_err(|_| {
            RuntimeError::new("builtin `make` capacity could not convert to runtime size")
        })?;
        self.stack
            .push(Value::Slice(SliceValue::with_len_and_capacity(
                zero_value_for_type(element_type),
                length,
                capacity,
            )));
        Ok(())
    }

    fn make_chan(
        &mut self,
        element_type: &ValueType,
        has_buffer: bool,
    ) -> Result<(), RuntimeError> {
        let buffer = if has_buffer { self.pop_integer()? } else { 0 };
        if buffer < 0 {
            return Err(RuntimeError::new(format!(
                "builtin `make` buffer size must be non-negative, found {buffer}"
            )));
        }
        let buffer = usize::try_from(buffer).map_err(|_| {
            RuntimeError::new("builtin `make` buffer size could not convert to runtime size")
        })?;

        let _ = zero_value_for_type(element_type);
        self.stack
            .push(Value::Chan(ChannelValue::with_capacity(buffer)));
        Ok(())
    }

    fn make_map(&mut self, map_type: &ValueType, has_hint: bool) -> Result<(), RuntimeError> {
        let hint = if has_hint {
            let hint = self.pop_integer()?;
            if hint < 0 {
                return Err(RuntimeError::new(format!(
                    "builtin `make` hint must be non-negative, found {hint}"
                )));
            }
            usize::try_from(hint).map_err(|_| {
                RuntimeError::new("builtin `make` hint could not convert to runtime size")
            })?
        } else {
            0
        };

        if !matches!(map_type, ValueType::Map { .. }) {
            return Err(RuntimeError::new(
                "map allocation expected a map runtime type",
            ));
        }

        self.stack.push(Value::Map(MapValue::with_hint(hint)));
        Ok(())
    }

    fn build_map(&mut self, map_type: &ValueType, entry_count: usize) -> Result<(), RuntimeError> {
        if !matches!(map_type, ValueType::Map { .. }) {
            return Err(RuntimeError::new("build-map expected a map runtime type"));
        }

        let mut entries = Vec::with_capacity(entry_count);
        for _ in 0..entry_count {
            let value = self.pop_value()?;
            let key = self.pop_map_key()?;
            entries.push((key, value));
        }
        entries.reverse();

        let map = MapValue::with_hint(entry_count);
        for (key, value) in entries {
            map.insert(key, value)
                .map_err(|_| RuntimeError::new("map literal construction encountered a nil map"))?;
        }
        self.stack.push(Value::Map(map));
        Ok(())
    }

    fn convert_value(&mut self, conversion: ConversionKind) -> Result<(), RuntimeError> {
        let value = self.pop_value()?;
        let converted = match (conversion, value) {
            (ConversionKind::StringToBytes, Value::String(value)) => {
                Value::Slice(SliceValue::from_string(&value))
            }
            (ConversionKind::BytesToString, Value::Slice(slice)) => {
                Value::String(StringValue::from_byte_slice(&slice).map_err(|_| {
                    RuntimeError::new("conversion `string([]byte)` encountered a non-byte slice")
                })?)
            }
            (ConversionKind::StringToBytes, _) => {
                return Err(RuntimeError::new(
                    "conversion `[]byte(string)` expected a string value",
                ));
            }
            (ConversionKind::BytesToString, _) => {
                return Err(RuntimeError::new(
                    "conversion `string([]byte)` expected a byte slice value",
                ));
            }
        };
        self.stack.push(converted);
        Ok(())
    }

    fn receive(&mut self, element_type: &ValueType) -> Result<(), RuntimeError> {
        let target = self.pop_value()?;
        let channel = match target {
            Value::Chan(channel) => channel,
            _ => return Err(RuntimeError::new("receive target is not a channel")),
        };
        let value = match channel.receive() {
            Ok(ChannelReceiveResult::Value(value)) => value,
            Ok(ChannelReceiveResult::ClosedEmpty) => zero_value_for_type(element_type),
            Err(ChannelReceiveError::Nil) => {
                return Err(RuntimeError::new("receive from nil channel would block"));
            }
            Err(ChannelReceiveError::WouldBlock) => {
                return Err(RuntimeError::new(
                    "receive would block in the current single-threaded VM",
                ));
            }
        };
        self.stack.push(value);
        Ok(())
    }

    fn index_map(&mut self, map_type: &ValueType) -> Result<(), RuntimeError> {
        let (value, _) = self.read_map(map_type)?;
        self.stack.push(value);
        Ok(())
    }

    fn lookup_map(&mut self, map_type: &ValueType) -> Result<(), RuntimeError> {
        let (value, present) = self.read_map(map_type)?;
        self.stack.push(value);
        self.stack.push(Value::Boolean(present));
        Ok(())
    }

    fn read_map(&mut self, map_type: &ValueType) -> Result<(Value, bool), RuntimeError> {
        let key = self.pop_map_key()?;
        let target = self.pop_value()?;
        let map = match target {
            Value::Map(map) => map,
            _ => return Err(RuntimeError::new("index target is not a map")),
        };
        let value_type = map_type
            .map_value_type()
            .ok_or_else(|| RuntimeError::new("index-map expected a map runtime type"))?;
        let value = map.get(&key);
        Ok((
            value
                .clone()
                .unwrap_or_else(|| zero_value_for_type(value_type)),
            value.is_some(),
        ))
    }

    fn map_keys(&mut self, key_type: &ValueType) -> Result<(), RuntimeError> {
        let target = self.pop_value()?;
        if !matches!(
            key_type,
            ValueType::Int | ValueType::Byte | ValueType::Bool | ValueType::String
        ) {
            return Err(RuntimeError::new(
                "map-keys expected a comparable scalar runtime key type",
            ));
        }
        let map = match target {
            Value::Map(map) => map,
            _ => return Err(RuntimeError::new("map-keys target is not a map")),
        };
        self.stack
            .push(Value::Slice(SliceValue::new(map.keys_as_values())));
        Ok(())
    }

    fn pop_integer(&mut self) -> Result<i64, RuntimeError> {
        match self.pop_value()? {
            Value::Integer(value) => Ok(value),
            Value::Byte(_)
            | Value::Boolean(_)
            | Value::String(_)
            | Value::Interface(_)
            | Value::Slice(_)
            | Value::Chan(_)
            | Value::Map(_) => Err(RuntimeError::new("expected integer value on the stack")),
        }
    }

    fn pop_bool(&mut self) -> Result<bool, RuntimeError> {
        match self.pop_value()? {
            Value::Boolean(value) => Ok(value),
            Value::Integer(_)
            | Value::Byte(_)
            | Value::String(_)
            | Value::Interface(_)
            | Value::Slice(_)
            | Value::Chan(_)
            | Value::Map(_) => Err(RuntimeError::new("expected boolean value on the stack")),
        }
    }

    fn pop_string(&mut self) -> Result<StringValue, RuntimeError> {
        match self.pop_value()? {
            Value::String(value) => Ok(value),
            Value::Integer(_)
            | Value::Byte(_)
            | Value::Boolean(_)
            | Value::Interface(_)
            | Value::Slice(_)
            | Value::Chan(_)
            | Value::Map(_) => Err(RuntimeError::new("expected string value on the stack")),
        }
    }

    fn pop_map_key(&mut self) -> Result<MapKey, RuntimeError> {
        match self.pop_value()? {
            Value::Integer(value) => Ok(MapKey::Integer(value)),
            Value::Byte(value) => Ok(MapKey::Byte(value)),
            Value::Boolean(value) => Ok(MapKey::Boolean(value)),
            Value::String(value) => Ok(MapKey::String(value)),
            Value::Interface(_) | Value::Slice(_) | Value::Chan(_) | Value::Map(_) => Err(
                RuntimeError::new("map index key is not a comparable scalar value"),
            ),
        }
    }

    fn pop_value(&mut self) -> Result<Value, RuntimeError> {
        self.stack
            .pop()
            .ok_or_else(|| RuntimeError::new("stack underflow"))
    }
}

#[cfg(test)]
mod tests;
