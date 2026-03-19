use std::convert::TryFrom;
use std::fmt;

use crate::builtin::BuiltinFunction;
use crate::bytecode::instruction::{Instruction, Program};
use crate::package::PackageFunction;
use crate::runtime::value::{SliceValue, Value};

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
                Instruction::PushBool(value) => self.stack.push(Value::Boolean(value)),
                Instruction::PushString(value) => self.stack.push(Value::String(value)),
                Instruction::BuildSlice(count) => {
                    let mut elements = Vec::with_capacity(count);
                    for _ in 0..count {
                        elements.push(self.pop_value()?);
                    }
                    elements.reverse();
                    self.stack.push(Value::Slice(SliceValue::new(elements)));
                }
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
                Instruction::Add => self.binary_integer_op(|left, right| left + right)?,
                Instruction::Concat => self.concat_strings()?,
                Instruction::Subtract => self.binary_integer_op(|left, right| left - right)?,
                Instruction::Multiply => self.binary_integer_op(|left, right| left * right)?,
                Instruction::Divide => self.binary_integer_op_checked(|left, right| {
                    if right == 0 {
                        Err(RuntimeError::new("division by zero"))
                    } else {
                        Ok(left / right)
                    }
                })?,
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
                Instruction::Index => self.index_slice()?,
                Instruction::Slice { has_low, has_high } => self.slice_value(has_low, has_high)?,
                Instruction::SetIndex => self.set_slice_index()?,
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

    fn binary_integer_op(
        &mut self,
        operation: impl FnOnce(i64, i64) -> i64,
    ) -> Result<(), RuntimeError> {
        self.binary_integer_op_checked(|left, right| Ok(operation(left, right)))
    }

    fn binary_integer_op_checked(
        &mut self,
        operation: impl FnOnce(i64, i64) -> Result<i64, RuntimeError>,
    ) -> Result<(), RuntimeError> {
        let right = self.pop_integer()?;
        let left = self.pop_integer()?;
        self.stack.push(Value::Integer(operation(left, right)?));
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
        self.stack.push(Value::String(format!("{left}{right}")));
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
                if arguments.len() != 1 {
                    return Err(RuntimeError::new(format!(
                        "builtin `len` expected 1 argument, received {}",
                        arguments.len()
                    )));
                }
                let value = match arguments.into_iter().next().expect("arity checked above") {
                    Value::String(value) => value.len() as i64,
                    Value::Slice(slice) => slice.len() as i64,
                    _ => {
                        return Err(RuntimeError::new(
                            "builtin `len` expected a string or slice argument",
                        ));
                    }
                };
                self.stack.push(Value::Integer(value));
            }
            BuiltinFunction::Cap => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::new(format!(
                        "builtin `cap` expected 1 argument, received {}",
                        arguments.len()
                    )));
                }
                let value = match arguments.into_iter().next().expect("arity checked above") {
                    Value::Slice(slice) => slice.capacity() as i64,
                    _ => {
                        return Err(RuntimeError::new("builtin `cap` expected a slice argument"));
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
                let source = match source {
                    Value::Slice(slice) => slice,
                    _ => {
                        return Err(RuntimeError::new(
                            "builtin `copy` expected a slice as argument 2",
                        ));
                    }
                };
                self.stack
                    .push(Value::Integer(destination.copy_from(&source) as i64));
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
                    .push(Value::String(render_package_arguments(&arguments, "")));
            }
            PackageFunction::StringsContains => {
                let [haystack, needle] = expect_exact_package_arguments(function, arguments, 2)?;
                let haystack = expect_string_package_argument(function, 1, haystack)?;
                let needle = expect_string_package_argument(function, 2, needle)?;
                self.stack
                    .push(Value::Boolean(haystack.contains(needle.as_str())));
            }
            PackageFunction::StringsHasPrefix => {
                let [value, prefix] = expect_exact_package_arguments(function, arguments, 2)?;
                let value = expect_string_package_argument(function, 1, value)?;
                let prefix = expect_string_package_argument(function, 2, prefix)?;
                self.stack
                    .push(Value::Boolean(value.starts_with(prefix.as_str())));
            }
            PackageFunction::StringsJoin => {
                let [elements, separator] = expect_exact_package_arguments(function, arguments, 2)?;
                let elements = expect_string_slice_package_argument(function, 1, elements)?;
                let separator = expect_string_package_argument(function, 2, separator)?;
                self.stack
                    .push(Value::String(elements.join(separator.as_str())));
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

    fn index_slice(&mut self) -> Result<(), RuntimeError> {
        let index = self.pop_integer()?;
        if index < 0 {
            return Err(RuntimeError::new(format!(
                "slice index {index} is out of bounds"
            )));
        }
        let target = self.pop_value()?;
        let slice = match target {
            Value::Slice(slice) => slice,
            _ => return Err(RuntimeError::new("index target is not a slice")),
        };
        let value = slice
            .get(index as usize)
            .ok_or_else(|| RuntimeError::new(format!("slice index {index} is out of bounds")))?;
        self.stack.push(value);
        Ok(())
    }

    fn slice_value(&mut self, has_low: bool, has_high: bool) -> Result<(), RuntimeError> {
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
        let slice = match target {
            Value::Slice(slice) => slice,
            _ => return Err(RuntimeError::new("slice expression target is not a slice")),
        };
        let low = normalize_slice_bound(low, 0, "lower")?;
        let high_default = i64::try_from(slice.len())
            .map_err(|_| RuntimeError::new("slice length could not convert to integer"))?;
        let high = normalize_slice_bound(high, high_default, "upper")?;
        let result = slice
            .slice(low, high)
            .map_err(|_| RuntimeError::new(slice_bounds_error_message(low, high)))?;
        self.stack.push(Value::Slice(result));
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

    fn pop_integer(&mut self) -> Result<i64, RuntimeError> {
        match self.pop_value()? {
            Value::Integer(value) => Ok(value),
            Value::Boolean(_) => Err(RuntimeError::new("expected integer value on the stack")),
            Value::String(_) => Err(RuntimeError::new("expected integer value on the stack")),
            Value::Slice(_) => Err(RuntimeError::new("expected integer value on the stack")),
        }
    }

    fn pop_bool(&mut self) -> Result<bool, RuntimeError> {
        match self.pop_value()? {
            Value::Boolean(value) => Ok(value),
            Value::Integer(_) => Err(RuntimeError::new("expected boolean value on the stack")),
            Value::String(_) => Err(RuntimeError::new("expected boolean value on the stack")),
            Value::Slice(_) => Err(RuntimeError::new("expected boolean value on the stack")),
        }
    }

    fn pop_string(&mut self) -> Result<String, RuntimeError> {
        match self.pop_value()? {
            Value::String(value) => Ok(value),
            Value::Integer(_) | Value::Boolean(_) | Value::Slice(_) => {
                Err(RuntimeError::new("expected string value on the stack"))
            }
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

fn render_builtin_arguments(arguments: &[Value]) -> String {
    arguments
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(" ")
}

fn render_package_arguments(arguments: &[Value], separator: &str) -> String {
    arguments
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(separator)
}

#[cfg(test)]
mod tests {
    use super::VirtualMachine;
    use crate::builtin::BuiltinFunction;
    use crate::bytecode::instruction::{CompiledFunction, Instruction, Program};
    use crate::package::PackageFunction;

    #[test]
    fn execute_builds_and_indexes_slices() {
        let program = Program {
            package_name: "main".to_string(),
            entry_function: "main".to_string(),
            entry_function_index: 0,
            functions: vec![CompiledFunction {
                name: "main".to_string(),
                parameter_count: 0,
                returns_value: false,
                local_names: vec!["values".to_string()],
                instructions: vec![
                    Instruction::PushInt(1),
                    Instruction::PushInt(2),
                    Instruction::BuildSlice(2),
                    Instruction::StoreLocal(0),
                    Instruction::LoadLocal(0),
                    Instruction::CallBuiltin(BuiltinFunction::Len, 1),
                    Instruction::LoadLocal(0),
                    Instruction::PushInt(1),
                    Instruction::Index,
                    Instruction::CallBuiltin(BuiltinFunction::Println, 2),
                    Instruction::Return,
                ],
            }],
        };

        let output = VirtualMachine::new()
            .execute(&program)
            .expect("program should execute")
            .render_output();
        assert_eq!(output, "2 2\n");
    }

    #[test]
    fn execute_strings_package_functions() {
        let program = Program {
            package_name: "main".to_string(),
            entry_function: "main".to_string(),
            entry_function_index: 0,
            functions: vec![CompiledFunction {
                name: "main".to_string(),
                parameter_count: 0,
                returns_value: false,
                local_names: vec![],
                instructions: vec![
                    Instruction::PushString("nova".to_string()),
                    Instruction::PushString("go".to_string()),
                    Instruction::PushString("go".to_string()),
                    Instruction::BuildSlice(3),
                    Instruction::PushString("-".to_string()),
                    Instruction::CallPackage(PackageFunction::StringsJoin, 2),
                    Instruction::PushString("gogo".to_string()),
                    Instruction::CallPackage(PackageFunction::StringsContains, 2),
                    Instruction::CallBuiltin(BuiltinFunction::Println, 1),
                    Instruction::PushString("vm".to_string()),
                    Instruction::PushInt(2),
                    Instruction::CallPackage(PackageFunction::StringsRepeat, 2),
                    Instruction::PushString("vmvm".to_string()),
                    Instruction::CallPackage(PackageFunction::StringsHasPrefix, 2),
                    Instruction::CallBuiltin(BuiltinFunction::Println, 1),
                    Instruction::Return,
                ],
            }],
        };

        let output = VirtualMachine::new()
            .execute(&program)
            .expect("strings package functions should execute")
            .render_output();

        assert_eq!(output, "false\ntrue\n");
    }

    #[test]
    fn execute_slice_windows_and_index_assignment() {
        let program = Program {
            package_name: "main".to_string(),
            entry_function: "main".to_string(),
            entry_function_index: 0,
            functions: vec![CompiledFunction {
                name: "main".to_string(),
                parameter_count: 0,
                returns_value: false,
                local_names: vec!["values".to_string(), "window".to_string()],
                instructions: vec![
                    Instruction::PushInt(1),
                    Instruction::PushInt(2),
                    Instruction::PushInt(3),
                    Instruction::BuildSlice(3),
                    Instruction::StoreLocal(0),
                    Instruction::LoadLocal(0),
                    Instruction::PushInt(1),
                    Instruction::Slice {
                        has_low: true,
                        has_high: false,
                    },
                    Instruction::StoreLocal(1),
                    Instruction::LoadLocal(1),
                    Instruction::PushInt(0),
                    Instruction::PushInt(9),
                    Instruction::SetIndex,
                    Instruction::LoadLocal(0),
                    Instruction::PushInt(1),
                    Instruction::Index,
                    Instruction::CallBuiltin(BuiltinFunction::Println, 1),
                    Instruction::Return,
                ],
            }],
        };

        let output = VirtualMachine::new()
            .execute(&program)
            .expect("slice window program should execute")
            .render_output();

        assert_eq!(output, "9\n");
    }

    #[test]
    fn execute_slice_builtins_and_capacity_aware_append() {
        let program = Program {
            package_name: "main".to_string(),
            entry_function: "main".to_string(),
            entry_function_index: 0,
            functions: vec![CompiledFunction {
                name: "main".to_string(),
                parameter_count: 0,
                returns_value: false,
                local_names: vec![
                    "values".to_string(),
                    "head".to_string(),
                    "grown".to_string(),
                ],
                instructions: vec![
                    Instruction::PushInt(1),
                    Instruction::PushInt(2),
                    Instruction::PushInt(3),
                    Instruction::PushInt(4),
                    Instruction::BuildSlice(4),
                    Instruction::StoreLocal(0),
                    Instruction::LoadLocal(0),
                    Instruction::PushInt(2),
                    Instruction::Slice {
                        has_low: false,
                        has_high: true,
                    },
                    Instruction::StoreLocal(1),
                    Instruction::LoadLocal(1),
                    Instruction::PushInt(9),
                    Instruction::CallBuiltin(BuiltinFunction::Append, 2),
                    Instruction::StoreLocal(2),
                    Instruction::LoadLocal(2),
                    Instruction::CallBuiltin(BuiltinFunction::Cap, 1),
                    Instruction::CallBuiltin(BuiltinFunction::Println, 1),
                    Instruction::LoadLocal(0),
                    Instruction::LoadLocal(0),
                    Instruction::PushInt(1),
                    Instruction::Slice {
                        has_low: true,
                        has_high: false,
                    },
                    Instruction::CallBuiltin(BuiltinFunction::Copy, 2),
                    Instruction::LoadLocal(0),
                    Instruction::PushInt(0),
                    Instruction::Index,
                    Instruction::CallBuiltin(BuiltinFunction::Println, 2),
                    Instruction::Return,
                ],
            }],
        };

        let output = VirtualMachine::new()
            .execute(&program)
            .expect("slice builtin program should execute")
            .render_output();

        assert_eq!(output, "4\n3 2\n");
    }
}

fn expect_exact_package_arguments<const N: usize>(
    function: PackageFunction,
    arguments: Vec<Value>,
    expected: usize,
) -> Result<[Value; N], RuntimeError> {
    if arguments.len() != expected {
        return Err(RuntimeError::new(format!(
            "package function `{}` expected {} arguments, received {}",
            function.render(),
            expected,
            arguments.len()
        )));
    }

    arguments.try_into().map_err(|arguments: Vec<Value>| {
        RuntimeError::new(format!(
            "package function `{}` expected {} arguments, received {}",
            function.render(),
            expected,
            arguments.len()
        ))
    })
}

fn expect_string_package_argument(
    function: PackageFunction,
    position: usize,
    value: Value,
) -> Result<String, RuntimeError> {
    match value {
        Value::String(value) => Ok(value),
        other => Err(RuntimeError::new(format!(
            "argument {} in call to `{}` expected `string`, found `{}`",
            position,
            function.render(),
            runtime_type_name(&other)
        ))),
    }
}

fn expect_integer_package_argument(
    function: PackageFunction,
    position: usize,
    value: Value,
) -> Result<i64, RuntimeError> {
    match value {
        Value::Integer(value) => Ok(value),
        other => Err(RuntimeError::new(format!(
            "argument {} in call to `{}` expected `int`, found `{}`",
            position,
            function.render(),
            runtime_type_name(&other)
        ))),
    }
}

fn expect_string_slice_package_argument(
    function: PackageFunction,
    position: usize,
    value: Value,
) -> Result<Vec<String>, RuntimeError> {
    let elements = match value {
        Value::Slice(slice) => slice.visible_elements(),
        other => {
            return Err(RuntimeError::new(format!(
                "argument {} in call to `{}` expected `[]string`, found `{}`",
                position,
                function.render(),
                runtime_type_name(&other)
            )));
        }
    };

    let mut strings = Vec::with_capacity(elements.len());
    for element in elements {
        match element {
            Value::String(value) => strings.push(value),
            other => {
                return Err(RuntimeError::new(format!(
                    "argument {} in call to `{}` expected `[]string`, found `[]{}`",
                    position,
                    function.render(),
                    runtime_type_name(&other)
                )));
            }
        }
    }

    Ok(strings)
}

fn expect_exact_builtin_arguments<const N: usize>(
    arguments: Vec<Value>,
    expected: usize,
    builtin: &str,
) -> Result<[Value; N], RuntimeError> {
    if arguments.len() != expected {
        return Err(RuntimeError::new(format!(
            "builtin `{builtin}` expected {expected} arguments, received {}",
            arguments.len()
        )));
    }

    arguments.try_into().map_err(|arguments: Vec<Value>| {
        RuntimeError::new(format!(
            "builtin `{builtin}` expected {expected} arguments, received {}",
            arguments.len()
        ))
    })
}

fn runtime_type_name(value: &Value) -> &'static str {
    match value {
        Value::Integer(_) => "int",
        Value::Boolean(_) => "bool",
        Value::String(_) => "string",
        Value::Slice(_) => "slice",
    }
}

fn normalize_slice_bound(
    bound: Option<i64>,
    default: i64,
    position: &str,
) -> Result<usize, RuntimeError> {
    let value = bound.unwrap_or(default);
    if value < 0 {
        return Err(RuntimeError::new(format!(
            "slice {position} bound {value} is out of bounds"
        )));
    }
    usize::try_from(value)
        .map_err(|_| RuntimeError::new(format!("slice {position} bound {value} is out of bounds")))
}

fn slice_bounds_error_message(low: usize, high: usize) -> String {
    format!("slice bounds [{low}:{high}] are out of range")
}
