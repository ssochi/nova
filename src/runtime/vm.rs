use std::fmt;

use crate::bytecode::instruction::{Builtin, Instruction, Program};
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
    pub output: Vec<String>,
}

impl ExecutionResult {
    pub fn render_output(&self) -> String {
        if self.output.is_empty() {
            String::new()
        } else {
            format!("{}\n", self.output.join("\n"))
        }
    }
}

#[derive(Debug)]
pub struct VirtualMachine {
    stack: Vec<Value>,
    locals: Vec<Value>,
    output: Vec<String>,
}

impl VirtualMachine {
    pub fn new(local_count: usize) -> Self {
        Self {
            stack: Vec::new(),
            locals: vec![Value::default(); local_count],
            output: Vec::new(),
        }
    }

    pub fn execute(&mut self, program: &Program) -> Result<ExecutionResult, RuntimeError> {
        let mut pc = 0usize;

        while let Some(instruction) = program.instructions.get(pc) {
            match instruction {
                Instruction::PushInt(value) => self.stack.push(Value::Integer(*value)),
                Instruction::LoadLocal(index) => {
                    let value = self
                        .locals
                        .get(*index)
                        .cloned()
                        .ok_or_else(|| RuntimeError::new(format!("invalid local slot {index}")))?;
                    self.stack.push(value);
                }
                Instruction::StoreLocal(index) => {
                    let value = self.pop_value()?;
                    let slot = self
                        .locals
                        .get_mut(*index)
                        .ok_or_else(|| RuntimeError::new(format!("invalid local slot {index}")))?;
                    *slot = value;
                }
                Instruction::Add => self.binary_integer_op(|left, right| left + right)?,
                Instruction::Subtract => self.binary_integer_op(|left, right| left - right)?,
                Instruction::Multiply => self.binary_integer_op(|left, right| left * right)?,
                Instruction::Divide => self.binary_integer_op_checked(|left, right| {
                    if right == 0 {
                        Err(RuntimeError::new("division by zero"))
                    } else {
                        Ok(left / right)
                    }
                })?,
                Instruction::Pop => {
                    self.pop_value()?;
                }
                Instruction::CallBuiltin(Builtin::Println, arity) => {
                    let mut arguments = Vec::with_capacity(*arity);
                    for _ in 0..*arity {
                        arguments.push(self.pop_value()?);
                    }
                    arguments.reverse();
                    let rendered = arguments
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(" ");
                    self.output.push(rendered);
                }
                Instruction::Return => {
                    return Ok(ExecutionResult {
                        output: self.output.clone(),
                    });
                }
            }

            pc += 1;
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

    fn pop_integer(&mut self) -> Result<i64, RuntimeError> {
        match self.pop_value()? {
            Value::Integer(value) => Ok(value),
        }
    }

    fn pop_value(&mut self) -> Result<Value, RuntimeError> {
        self.stack
            .pop()
            .ok_or_else(|| RuntimeError::new("stack underflow"))
    }
}
