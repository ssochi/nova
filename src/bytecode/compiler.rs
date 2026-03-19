use std::fmt;

use crate::bytecode::instruction::{CompiledFunction, Instruction, Program};
use crate::semantic::model::{
    CallTarget, CheckedAssignmentTarget, CheckedBinaryOperator, CheckedBlock, CheckedExpression,
    CheckedExpressionKind, CheckedFunction, CheckedProgram, CheckedStatement, Type,
};

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

pub fn compile_program(program: &CheckedProgram) -> Result<Program, CompileError> {
    let mut compiled_functions = Vec::with_capacity(program.functions.len());
    for function in &program.functions {
        compiled_functions.push(FunctionCompiler::new(function).compile()?);
    }

    Ok(Program {
        package_name: program.package_name.clone(),
        entry_function: program.functions[program.entry_function].name.clone(),
        entry_function_index: program.entry_function,
        functions: compiled_functions,
    })
}

struct FunctionCompiler<'a> {
    function: &'a CheckedFunction,
    instructions: Vec<Instruction>,
}

impl<'a> FunctionCompiler<'a> {
    fn new(function: &'a CheckedFunction) -> Self {
        Self {
            function,
            instructions: Vec::new(),
        }
    }

    fn compile(mut self) -> Result<CompiledFunction, CompileError> {
        self.compile_block(&self.function.body)?;
        if !matches!(self.instructions.last(), Some(Instruction::Return)) {
            self.instructions.push(Instruction::Return);
        }

        Ok(CompiledFunction {
            name: self.function.name.clone(),
            parameter_count: self.function.parameter_count,
            returns_value: self.function.return_type.produces_value(),
            local_names: self.function.local_names.clone(),
            instructions: self.instructions,
        })
    }

    fn compile_block(&mut self, block: &CheckedBlock) -> Result<(), CompileError> {
        for statement in &block.statements {
            self.compile_statement(statement)?;
        }
        Ok(())
    }

    fn compile_statement(&mut self, statement: &CheckedStatement) -> Result<(), CompileError> {
        match statement {
            CheckedStatement::VarDecl { slot, value, .. } => {
                self.compile_expression(value)?;
                self.expect_value(&value.ty, "variable declaration")?;
                self.instructions.push(Instruction::StoreLocal(*slot));
            }
            CheckedStatement::Assign { target, value } => match target {
                CheckedAssignmentTarget::Local { slot, .. } => {
                    self.compile_expression(value)?;
                    self.expect_value(&value.ty, "assignment")?;
                    self.instructions.push(Instruction::StoreLocal(*slot));
                }
                CheckedAssignmentTarget::Index { target, index } => {
                    self.compile_expression(target)?;
                    self.expect_value(&target.ty, "index assignment")?;
                    self.compile_expression(index)?;
                    self.expect_value(&index.ty, "index assignment")?;
                    self.compile_expression(value)?;
                    self.expect_value(&value.ty, "index assignment")?;
                    self.instructions.push(Instruction::SetIndex);
                }
            },
            CheckedStatement::Expr(expression) => {
                self.compile_expression(expression)?;
                if expression.ty.produces_value() {
                    self.instructions.push(Instruction::Pop);
                }
            }
            CheckedStatement::If {
                condition,
                then_block,
                else_block,
            } => {
                self.compile_expression(condition)?;
                let jump_to_else = self.push_instruction(Instruction::JumpIfFalse(usize::MAX));
                self.compile_block(then_block)?;
                if let Some(else_block) = else_block {
                    let jump_to_end = self.push_instruction(Instruction::Jump(usize::MAX));
                    let else_start = self.instructions.len();
                    self.patch_jump(jump_to_else, Instruction::JumpIfFalse(else_start));
                    self.compile_block(else_block)?;
                    let end = self.instructions.len();
                    self.patch_jump(jump_to_end, Instruction::Jump(end));
                } else {
                    let end = self.instructions.len();
                    self.patch_jump(jump_to_else, Instruction::JumpIfFalse(end));
                }
            }
            CheckedStatement::For { condition, body } => {
                let loop_start = self.instructions.len();
                self.compile_expression(condition)?;
                let jump_to_end = self.push_instruction(Instruction::JumpIfFalse(usize::MAX));
                self.compile_block(body)?;
                self.instructions.push(Instruction::Jump(loop_start));
                let loop_end = self.instructions.len();
                self.patch_jump(jump_to_end, Instruction::JumpIfFalse(loop_end));
            }
            CheckedStatement::Return(value) => {
                if let Some(expression) = value {
                    self.compile_expression(expression)?;
                    self.expect_value(&expression.ty, "return")?;
                }
                self.instructions.push(Instruction::Return);
            }
        }

        Ok(())
    }

    fn compile_expression(&mut self, expression: &CheckedExpression) -> Result<(), CompileError> {
        match &expression.kind {
            CheckedExpressionKind::Integer(value) => {
                self.instructions.push(Instruction::PushInt(*value));
            }
            CheckedExpressionKind::Bool(value) => {
                self.instructions.push(Instruction::PushBool(*value));
            }
            CheckedExpressionKind::String(value) => {
                self.instructions
                    .push(Instruction::PushString(value.clone()));
            }
            CheckedExpressionKind::SliceLiteral { elements } => {
                for element in elements {
                    self.compile_expression(element)?;
                    self.expect_value(&element.ty, "slice literal element")?;
                }
                self.instructions
                    .push(Instruction::BuildSlice(elements.len()));
            }
            CheckedExpressionKind::Local { slot, .. } => {
                self.instructions.push(Instruction::LoadLocal(*slot));
            }
            CheckedExpressionKind::Index { target, index } => {
                self.compile_expression(target)?;
                self.expect_value(&target.ty, "index expression")?;
                self.compile_expression(index)?;
                self.expect_value(&index.ty, "index expression")?;
                self.instructions.push(Instruction::Index);
            }
            CheckedExpressionKind::Slice { target, low, high } => {
                self.compile_expression(target)?;
                self.expect_value(&target.ty, "slice expression")?;
                if let Some(low) = low {
                    self.compile_expression(low)?;
                    self.expect_value(&low.ty, "slice expression")?;
                }
                if let Some(high) = high {
                    self.compile_expression(high)?;
                    self.expect_value(&high.ty, "slice expression")?;
                }
                self.instructions.push(Instruction::Slice {
                    has_low: low.is_some(),
                    has_high: high.is_some(),
                });
            }
            CheckedExpressionKind::Binary {
                left,
                operator,
                right,
            } => {
                self.compile_expression(left)?;
                self.expect_value(&left.ty, "binary expression")?;
                self.compile_expression(right)?;
                self.expect_value(&right.ty, "binary expression")?;
                self.instructions.push(match operator {
                    CheckedBinaryOperator::Add => Instruction::Add,
                    CheckedBinaryOperator::Concat => Instruction::Concat,
                    CheckedBinaryOperator::Subtract => Instruction::Subtract,
                    CheckedBinaryOperator::Multiply => Instruction::Multiply,
                    CheckedBinaryOperator::Divide => Instruction::Divide,
                    CheckedBinaryOperator::Equal => Instruction::Equal,
                    CheckedBinaryOperator::NotEqual => Instruction::NotEqual,
                    CheckedBinaryOperator::Less => Instruction::Less,
                    CheckedBinaryOperator::LessEqual => Instruction::LessEqual,
                    CheckedBinaryOperator::Greater => Instruction::Greater,
                    CheckedBinaryOperator::GreaterEqual => Instruction::GreaterEqual,
                });
            }
            CheckedExpressionKind::Call { target, arguments } => {
                for argument in arguments {
                    self.compile_expression(argument)?;
                    self.expect_value(&argument.ty, "function call")?;
                }

                match target {
                    CallTarget::Builtin(builtin) => {
                        self.instructions
                            .push(Instruction::CallBuiltin(*builtin, arguments.len()));
                    }
                    CallTarget::PackageFunction(function) => {
                        self.instructions
                            .push(Instruction::CallPackage(*function, arguments.len()));
                    }
                    CallTarget::UserDefined { function_index, .. } => {
                        self.instructions
                            .push(Instruction::CallFunction(*function_index, arguments.len()));
                    }
                }
            }
        }

        Ok(())
    }

    fn expect_value(&self, ty: &Type, context: &str) -> Result<(), CompileError> {
        if ty.produces_value() {
            Ok(())
        } else {
            Err(CompileError::new(format!(
                "{context} requires a value-producing expression"
            )))
        }
    }

    fn push_instruction(&mut self, instruction: Instruction) -> usize {
        let index = self.instructions.len();
        self.instructions.push(instruction);
        index
    }

    fn patch_jump(&mut self, index: usize, instruction: Instruction) {
        self.instructions[index] = instruction;
    }
}
