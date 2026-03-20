use std::convert::TryFrom;
use std::fmt;

use self::support::{
    execute_bytes_package_function, expect_exact_builtin_arguments, expect_exact_package_arguments,
    expect_integer_package_argument, expect_string_package_argument,
    expect_string_slice_package_argument, normalize_slice_bound, render_builtin_arguments,
    render_package_arguments, slice_bounds_error_message, zero_value_for_type,
};
use crate::builtin::BuiltinFunction;
use crate::bytecode::instruction::{Instruction, Program, SequenceKind, ValueType};
use crate::conversion::ConversionKind;
use crate::package::PackageFunction;
use crate::runtime::value::{
    ChannelCloseError, ChannelReceiveError, ChannelReceiveResult, ChannelSendError, ChannelValue,
    MapKey, MapValue, SliceValue, StringValue, Value,
};

mod support;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeError {
    message: String,
}

impl RuntimeError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for RuntimeError {}

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
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            frames: Vec::new(),
            output: String::new(),
        }
    }

    pub fn execute(&mut self, program: &Program) -> Result<ExecutionResult, RuntimeError> {
        self.stack.clear();
        self.frames.clear();
        self.output.clear();

        let entry = program
            .functions
            .get(program.entry_function_index)
            .ok_or_else(|| RuntimeError::new("entry function index is invalid"))?;
        self.frames.push(CallFrame::new(
            program.entry_function_index,
            entry.local_names.len(),
        ));

        while let Some(frame_index) = self.frames.len().checked_sub(1) {
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
            match instruction {
                Instruction::PushInt(value) => self.stack.push(Value::Integer(value)),
                Instruction::PushByte(value) => self.stack.push(Value::Byte(value)),
                Instruction::PushBool(value) => self.stack.push(Value::Boolean(value)),
                Instruction::PushString(value) => {
                    self.stack.push(Value::String(StringValue::from(value)))
                }
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
                Instruction::LoadLocal(index) => {
                    let value = self.frames[frame_index]
                        .locals
                        .get(index)
                        .cloned()
                        .ok_or_else(|| RuntimeError::new(format!("invalid local slot {index}")))?;
                    self.stack.push(value);
                }
                Instruction::StoreLocal(index) => {
                    let value = self.pop_value()?;
                    let slot = self.frames[frame_index]
                        .locals
                        .get_mut(index)
                        .ok_or_else(|| RuntimeError::new(format!("invalid local slot {index}")))?;
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
                Instruction::Equal => self.binary_compare(|left, right| left == right)?,
                Instruction::NotEqual => self.binary_compare(|left, right| left != right)?,
                Instruction::Less => self.binary_integer_compare(|left, right| left < right)?,
                Instruction::LessEqual => {
                    self.binary_integer_compare(|left, right| left <= right)?
                }
                Instruction::Greater => self.binary_integer_compare(|left, right| left > right)?,
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
                Instruction::CallBuiltin(builtin, arity) => self.call_builtin(builtin, arity)?,
                Instruction::CallPackage(function, arity) => {
                    self.call_package_function(function, arity)?
                }
                Instruction::CallFunction(function_index, arity) => {
                    let function = program.functions.get(function_index).ok_or_else(|| {
                        RuntimeError::new(format!("invalid function index {function_index}"))
                    })?;
                    if arity != function.parameter_count {
                        return Err(RuntimeError::new(format!(
                            "function `{}` expected {} arguments, received {}",
                            function.name, function.parameter_count, arity
                        )));
                    }

                    let mut arguments = Vec::with_capacity(arity);
                    for _ in 0..arity {
                        arguments.push(self.pop_value()?);
                    }
                    arguments.reverse();

                    self.frames[frame_index].pc += 1;
                    let mut frame = CallFrame::new(function_index, function.local_names.len());
                    for (index, argument) in arguments.into_iter().enumerate() {
                        frame.locals[index] = argument;
                    }
                    self.frames.push(frame);
                    advance_pc = false;
                }
                Instruction::Return => {
                    let return_value = if function.returns_value {
                        Some(self.pop_value()?)
                    } else {
                        None
                    };
                    self.frames.pop();
                    if let Some(value) = return_value {
                        self.stack.push(value);
                    }
                    if self.frames.is_empty() {
                        return Ok(ExecutionResult {
                            output: self.output.clone(),
                        });
                    }

                    advance_pc = false;
                }
            }

            if advance_pc {
                self.frames[frame_index].pc += 1;
            }
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

    fn binary_compare(
        &mut self,
        operation: impl FnOnce(Value, Value) -> bool,
    ) -> Result<(), RuntimeError> {
        let right = self.pop_value()?;
        let left = self.pop_value()?;
        self.stack.push(Value::Boolean(operation(left, right)));
        Ok(())
    }

    fn concat_strings(&mut self) -> Result<(), RuntimeError> {
        let right = self.pop_string()?;
        let left = self.pop_string()?;
        self.stack.push(Value::String(left.concat(&right)));
        Ok(())
    }

    fn call_builtin(&mut self, builtin: BuiltinFunction, arity: usize) -> Result<(), RuntimeError> {
        let arguments = self.pop_arguments(arity)?;

        match builtin {
            BuiltinFunction::Print => {
                self.output.push_str(&render_builtin_arguments(&arguments));
            }
            BuiltinFunction::Println => {
                self.output.push_str(&render_builtin_arguments(&arguments));
                self.output.push('\n');
            }
            BuiltinFunction::Len => {
                let [argument] = expect_exact_builtin_arguments(arguments, 1, "len")?;
                let value = match argument {
                    Value::String(value) => value.len() as i64,
                    Value::Slice(slice) => slice.len() as i64,
                    Value::Chan(channel) => channel.len() as i64,
                    Value::Map(map) => map.len() as i64,
                    _ => {
                        return Err(RuntimeError::new(
                            "builtin `len` expected a string, slice, chan, or map argument",
                        ));
                    }
                };
                self.stack.push(Value::Integer(value));
            }
            BuiltinFunction::Cap => {
                let [argument] = expect_exact_builtin_arguments(arguments, 1, "cap")?;
                let value = match argument {
                    Value::Slice(slice) => slice.capacity() as i64,
                    Value::Chan(channel) => channel.capacity() as i64,
                    _ => {
                        return Err(RuntimeError::new(
                            "builtin `cap` expected a slice or chan argument",
                        ));
                    }
                };
                self.stack.push(Value::Integer(value));
            }
            BuiltinFunction::Copy => {
                let [destination, source] = expect_exact_builtin_arguments(arguments, 2, "copy")?;
                let destination = match destination {
                    Value::Slice(slice) => slice,
                    _ => {
                        return Err(RuntimeError::new(
                            "builtin `copy` expected a slice as argument 1",
                        ));
                    }
                };
                let copied = match source {
                    Value::Slice(slice) => destination.copy_from(&slice),
                    Value::String(value) => destination.copy_from_string(&value),
                    _ => {
                        return Err(RuntimeError::new(
                            "builtin `copy` expected a slice or string as argument 2",
                        ));
                    }
                };
                self.stack.push(Value::Integer(copied as i64));
            }
            BuiltinFunction::Append => {
                let Some((first, rest)) = arguments.split_first() else {
                    return Err(RuntimeError::new(
                        "builtin `append` expected at least 1 argument",
                    ));
                };
                let slice = match first {
                    Value::Slice(slice) => slice.clone(),
                    _ => {
                        return Err(RuntimeError::new(
                            "builtin `append` expected a slice as the first argument",
                        ));
                    }
                };
                self.stack.push(Value::Slice(slice.append(rest)));
            }
            BuiltinFunction::Make => {
                return Err(RuntimeError::new(
                    "builtin `make` is lowered into dedicated allocation bytecode",
                ));
            }
            BuiltinFunction::Delete => {
                let [target, key] = expect_exact_builtin_arguments(arguments, 2, "delete")?;
                let map = match target {
                    Value::Map(map) => map,
                    _ => {
                        return Err(RuntimeError::new(
                            "builtin `delete` expected a map as argument 1",
                        ));
                    }
                };
                let key = match key {
                    Value::Integer(value) => MapKey::Integer(value),
                    Value::Byte(value) => MapKey::Byte(value),
                    Value::Boolean(value) => MapKey::Boolean(value),
                    Value::String(value) => MapKey::String(value),
                    Value::Slice(_) | Value::Chan(_) | Value::Map(_) => {
                        return Err(RuntimeError::new(
                            "builtin `delete` expected a comparable scalar key as argument 2",
                        ));
                    }
                };
                map.remove(&key);
            }
            BuiltinFunction::Close => {
                let [target] = expect_exact_builtin_arguments(arguments, 1, "close")?;
                let channel = match target {
                    Value::Chan(channel) => channel,
                    _ => {
                        return Err(RuntimeError::new(
                            "builtin `close` expected a chan as argument 1",
                        ));
                    }
                };
                channel.close().map_err(|error| match error {
                    ChannelCloseError::Nil => RuntimeError::new("close of nil channel"),
                    ChannelCloseError::Closed => RuntimeError::new("close of closed channel"),
                })?;
            }
        }

        Ok(())
    }

    fn call_package_function(
        &mut self,
        function: PackageFunction,
        arity: usize,
    ) -> Result<(), RuntimeError> {
        let arguments = self.pop_arguments(arity)?;

        match function {
            PackageFunction::FmtPrint => {
                self.output
                    .push_str(&render_package_arguments(&arguments, ""));
            }
            PackageFunction::FmtPrintln => {
                self.output
                    .push_str(&render_package_arguments(&arguments, " "));
                self.output.push('\n');
            }
            PackageFunction::FmtSprint => {
                self.stack
                    .push(Value::String(StringValue::from(render_package_arguments(
                        &arguments, "",
                    ))));
            }
            PackageFunction::StringsContains => {
                let [haystack, needle] = expect_exact_package_arguments(function, arguments, 2)?;
                let haystack = expect_string_package_argument(function, 1, haystack)?;
                let needle = expect_string_package_argument(function, 2, needle)?;
                self.stack.push(Value::Boolean(haystack.contains(&needle)));
            }
            PackageFunction::StringsHasPrefix => {
                let [value, prefix] = expect_exact_package_arguments(function, arguments, 2)?;
                let value = expect_string_package_argument(function, 1, value)?;
                let prefix = expect_string_package_argument(function, 2, prefix)?;
                self.stack.push(Value::Boolean(value.has_prefix(&prefix)));
            }
            PackageFunction::StringsJoin => {
                let [elements, separator] = expect_exact_package_arguments(function, arguments, 2)?;
                let elements = expect_string_slice_package_argument(function, 1, elements)?;
                let separator = expect_string_package_argument(function, 2, separator)?;
                self.stack
                    .push(Value::String(StringValue::join(&elements, &separator)));
            }
            PackageFunction::StringsRepeat => {
                let [value, count] = expect_exact_package_arguments(function, arguments, 2)?;
                let value = expect_string_package_argument(function, 1, value)?;
                let count = expect_integer_package_argument(function, 2, count)?;
                if count < 0 {
                    return Err(RuntimeError::new(format!(
                        "package function `{}` requires a non-negative repeat count",
                        function.render()
                    )));
                }
                let repeat_count = usize::try_from(count).map_err(|_| {
                    RuntimeError::new(format!(
                        "package function `{}` could not convert repeat count",
                        function.render()
                    ))
                })?;
                value.len().checked_mul(repeat_count).ok_or_else(|| {
                    RuntimeError::new(format!(
                        "package function `{}` overflowed the repeated string size",
                        function.render()
                    ))
                })?;
                self.stack.push(Value::String(value.repeat(repeat_count)));
            }
            PackageFunction::BytesEqual
            | PackageFunction::BytesContains
            | PackageFunction::BytesHasPrefix
            | PackageFunction::BytesJoin
            | PackageFunction::BytesRepeat => {
                self.stack
                    .push(execute_bytes_package_function(function, arguments)?);
            }
        }

        Ok(())
    }

    fn pop_arguments(&mut self, arity: usize) -> Result<Vec<Value>, RuntimeError> {
        let mut arguments = Vec::with_capacity(arity);
        for _ in 0..arity {
            arguments.push(self.pop_value()?);
        }
        arguments.reverse();
        Ok(arguments)
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
            .map_err(|_| RuntimeError::new("assignment to entry in nil map"))?;
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
            ChannelSendError::Closed => RuntimeError::new("send on closed channel"),
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
            Value::Slice(_) | Value::Chan(_) | Value::Map(_) => Err(RuntimeError::new(
                "map index key is not a comparable scalar value",
            )),
        }
    }

    fn pop_value(&mut self) -> Result<Value, RuntimeError> {
        self.stack
            .pop()
            .ok_or_else(|| RuntimeError::new("stack underflow"))
    }
}

#[derive(Debug)]
struct CallFrame {
    function_index: usize,
    pc: usize,
    locals: Vec<Value>,
}

impl CallFrame {
    fn new(function_index: usize, local_count: usize) -> Self {
        Self {
            function_index,
            pc: 0,
            locals: vec![Value::default(); local_count],
        }
    }
}

#[cfg(test)]
mod tests;
