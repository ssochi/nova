use std::collections::HashMap;
use std::fmt;

use crate::bytecode::instruction::{Builtin, Instruction, Program};
use crate::config::ExecutionConfig;
use crate::frontend::ast::{BinaryOperator, Expression, FunctionDecl, SourceFileAst, Statement};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileError {
    message: String,
}

impl CompileError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CompileError {}

pub fn compile_program(ast: &SourceFileAst, config: &ExecutionConfig) -> Result<Program, CompileError> {
    if ast.package_name != config.entry_package {
        return Err(CompileError::new(format!(
            "entry package mismatch: expected `{}`, found `{}`",
            config.entry_package, ast.package_name
        )));
    }

    let function = ast
        .functions
        .iter()
        .find(|function| function.name == config.entry_function)
        .ok_or_else(|| {
            CompileError::new(format!(
                "entry function `{}` was not found in package `{}`",
                config.entry_function, ast.package_name
            ))
        })?;

    Compiler::new(ast.package_name.clone(), function.name.clone()).compile(function)
}

struct Compiler {
    package_name: String,
    entry_function: String,
    locals: HashMap<String, usize>,
    local_names: Vec<String>,
    instructions: Vec<Instruction>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ValueMode {
    ProducesValue,
    ProducesNoValue,
}

impl Compiler {
    fn new(package_name: String, entry_function: String) -> Self {
        Self {
            package_name,
            entry_function,
            locals: HashMap::new(),
            local_names: Vec::new(),
            instructions: Vec::new(),
        }
    }

    fn compile(mut self, function: &FunctionDecl) -> Result<Program, CompileError> {
        for statement in &function.body.statements {
            self.compile_statement(statement)?;
        }

        if !matches!(self.instructions.last(), Some(Instruction::Return)) {
            self.instructions.push(Instruction::Return);
        }

        Ok(Program {
            package_name: self.package_name,
            entry_function: self.entry_function,
            local_names: self.local_names,
            instructions: self.instructions,
        })
    }

    fn compile_statement(&mut self, statement: &Statement) -> Result<(), CompileError> {
        match statement {
            Statement::VarDecl { name, value } => {
                if self.locals.contains_key(name) {
                    return Err(CompileError::new(format!(
                        "variable `{name}` is already defined"
                    )));
                }

                let mode = self.compile_expression(value)?;
                self.expect_value(mode, "variable declaration")?;
                let slot = self.allocate_local(name.clone());
                self.instructions.push(Instruction::StoreLocal(slot));
            }
            Statement::Assign { name, value } => {
                let slot = self.lookup_local(name)?;
                let mode = self.compile_expression(value)?;
                self.expect_value(mode, "assignment")?;
                self.instructions.push(Instruction::StoreLocal(slot));
            }
            Statement::Expr(expression) => {
                if self.compile_expression(expression)? == ValueMode::ProducesValue {
                    self.instructions.push(Instruction::Pop);
                }
            }
            Statement::Return(value) => {
                if let Some(expression) = value {
                    let mode = self.compile_expression(expression)?;
                    self.expect_value(mode, "return")?;
                    self.instructions.push(Instruction::Pop);
                }
                self.instructions.push(Instruction::Return);
            }
        }

        Ok(())
    }

    fn compile_expression(&mut self, expression: &Expression) -> Result<ValueMode, CompileError> {
        match expression {
            Expression::Integer(value) => {
                self.instructions.push(Instruction::PushInt(*value));
                Ok(ValueMode::ProducesValue)
            }
            Expression::Identifier(name) => {
                let slot = self.lookup_local(name)?;
                self.instructions.push(Instruction::LoadLocal(slot));
                Ok(ValueMode::ProducesValue)
            }
            Expression::Binary {
                left,
                operator,
                right,
            } => {
                let left_mode = self.compile_expression(left)?;
                self.expect_value(left_mode, "binary expression")?;
                let right_mode = self.compile_expression(right)?;
                self.expect_value(right_mode, "binary expression")?;
                self.instructions.push(match operator {
                    BinaryOperator::Add => Instruction::Add,
                    BinaryOperator::Subtract => Instruction::Subtract,
                    BinaryOperator::Multiply => Instruction::Multiply,
                    BinaryOperator::Divide => Instruction::Divide,
                });
                Ok(ValueMode::ProducesValue)
            }
            Expression::Call { callee, arguments } => {
                let builtin = resolve_builtin(callee).ok_or_else(|| {
                    CompileError::new(format!(
                        "unsupported function call `{callee}`; only builtins are available in the bootstrap VM"
                    ))
                })?;

                for argument in arguments {
                    let mode = self.compile_expression(argument)?;
                    self.expect_value(mode, "builtin call")?;
                }

                self.instructions
                    .push(Instruction::CallBuiltin(builtin, arguments.len()));
                Ok(ValueMode::ProducesNoValue)
            }
        }
    }

    fn expect_value(&self, mode: ValueMode, context: &str) -> Result<(), CompileError> {
        if mode == ValueMode::ProducesValue {
            Ok(())
        } else {
            Err(CompileError::new(format!(
                "{context} requires a value-producing expression"
            )))
        }
    }

    fn allocate_local(&mut self, name: String) -> usize {
        let slot = self.local_names.len();
        self.locals.insert(name.clone(), slot);
        self.local_names.push(name);
        slot
    }

    fn lookup_local(&self, name: &str) -> Result<usize, CompileError> {
        self.locals
            .get(name)
            .copied()
            .ok_or_else(|| CompileError::new(format!("unknown variable `{name}`")))
    }
}

fn resolve_builtin(name: &str) -> Option<Builtin> {
    match name {
        "println" => Some(Builtin::Println),
        _ => None,
    }
}
