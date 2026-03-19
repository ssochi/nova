use std::fmt;

use crate::builtin::BuiltinFunction;
use crate::bytecode::instruction::{Instruction, Program};
use crate::runtime::value::Value;

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
            let function = program
                .functions
                .get(function_index)
                .ok_or_else(|| RuntimeError::new(format!("invalid function index {function_index}")))?;
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
                Instruction::LoadLocal(index) => {
                    let value = self
                        .frames[frame_index]
                        .locals
                        .get(index)
                        .cloned()
                        .ok_or_else(|| RuntimeError::new(format!("invalid local slot {index}")))?;
                    self.stack.push(value);
                }
                Instruction::StoreLocal(index) => {
                    let value = self.pop_value()?;
                    let slot = self
                        .frames[frame_index]
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
                Instruction::LessEqual => self.binary_integer_compare(|left, right| left <= right)?,
                Instruction::Greater => self.binary_integer_compare(|left, right| left > right)?,
                Instruction::GreaterEqual => {
                    self.binary_integer_compare(|left, right| left >= right)?
                }
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

    fn binary_integer_op(&mut self, operation: impl FnOnce(i64, i64) -> i64) -> Result<(), RuntimeError> {
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

    fn call_builtin(
        &mut self,
        builtin: BuiltinFunction,
        arity: usize,
    ) -> Result<(), RuntimeError> {
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
                    Value::String(value) => value,
                    _ => {
                        return Err(RuntimeError::new(
                            "builtin `len` expected a string argument",
                        ));
                    }
                };
                self.stack.push(Value::Integer(value.len() as i64));
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

    fn pop_integer(&mut self) -> Result<i64, RuntimeError> {
        match self.pop_value()? {
            Value::Integer(value) => Ok(value),
            Value::Boolean(_) => Err(RuntimeError::new("expected integer value on the stack")),
            Value::String(_) => Err(RuntimeError::new("expected integer value on the stack")),
        }
    }

    fn pop_bool(&mut self) -> Result<bool, RuntimeError> {
        match self.pop_value()? {
            Value::Boolean(value) => Ok(value),
            Value::Integer(_) => Err(RuntimeError::new("expected boolean value on the stack")),
            Value::String(_) => Err(RuntimeError::new("expected boolean value on the stack")),
        }
    }

    fn pop_string(&mut self) -> Result<String, RuntimeError> {
        match self.pop_value()? {
            Value::String(value) => Ok(value),
            Value::Integer(_) | Value::Boolean(_) => {
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
