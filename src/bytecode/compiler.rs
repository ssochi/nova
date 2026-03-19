use std::fmt;

use crate::builtin::BuiltinFunction;
use crate::bytecode::instruction::{
    CompiledFunction, Instruction, Program, SequenceKind, ValueType,
};
use crate::semantic::model::{
    CallTarget, CheckedAssignmentTarget, CheckedBinaryOperator, CheckedBlock, CheckedExpression,
    CheckedExpressionKind, CheckedFunction, CheckedMapLiteralEntry, CheckedProgram,
    CheckedRangeBinding, CheckedStatement, Type,
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
    local_names: Vec<String>,
    instructions: Vec<Instruction>,
}

impl<'a> FunctionCompiler<'a> {
    fn new(function: &'a CheckedFunction) -> Self {
        Self {
            function,
            local_names: function.local_names.clone(),
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
            local_names: self.local_names,
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
                if let Some(value) = value {
                    self.compile_expression(value)?;
                    self.expect_value(&value.ty, "variable declaration")?;
                    self.instructions.push(Instruction::StoreLocal(*slot));
                }
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
                    if matches!(target.ty, Type::Map { .. }) {
                        self.instructions.push(Instruction::SetMapIndex);
                    } else {
                        self.instructions.push(Instruction::SetIndex);
                    }
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
            CheckedStatement::RangeFor {
                source,
                key_binding,
                value_binding,
                body,
            } => self.compile_range_statement(
                source,
                key_binding.as_ref(),
                value_binding.as_ref(),
                body,
            )?,
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
            CheckedExpressionKind::UntypedNil => {
                return Err(CompileError::new(
                    "untyped `nil` must be resolved before bytecode lowering",
                ));
            }
            CheckedExpressionKind::ZeroValue => match &expression.ty {
                Type::Int => self.instructions.push(Instruction::PushInt(0)),
                Type::Byte => self.instructions.push(Instruction::PushByte(0)),
                Type::Bool => self.instructions.push(Instruction::PushBool(false)),
                Type::String => self
                    .instructions
                    .push(Instruction::PushString(String::new())),
                Type::UntypedNil => {
                    return Err(CompileError::new(
                        "zero-value synthesis does not support untyped `nil`",
                    ));
                }
                Type::Slice(_) => self.instructions.push(Instruction::PushNilSlice),
                Type::Map { .. } => self.instructions.push(Instruction::PushNilMap),
                Type::Void => {
                    return Err(CompileError::new(
                        "zero-value synthesis does not support `void` locals",
                    ));
                }
            },
            CheckedExpressionKind::SliceLiteral { elements } => {
                for element in elements {
                    self.compile_expression(element)?;
                    self.expect_value(&element.ty, "slice literal element")?;
                }
                self.instructions
                    .push(Instruction::BuildSlice(elements.len()));
            }
            CheckedExpressionKind::MapLiteral { entries } => {
                for CheckedMapLiteralEntry { key, value } in entries {
                    self.compile_expression(key)?;
                    self.expect_value(&key.ty, "map literal key")?;
                    self.compile_expression(value)?;
                    self.expect_value(&value.ty, "map literal value")?;
                }
                self.instructions.push(Instruction::BuildMap {
                    map_type: lower_value_type(&expression.ty)?,
                    entry_count: entries.len(),
                });
            }
            CheckedExpressionKind::MakeSlice {
                element_type,
                length,
                capacity,
            } => {
                self.compile_expression(length)?;
                self.expect_value(&length.ty, "make expression")?;
                if let Some(capacity) = capacity {
                    self.compile_expression(capacity)?;
                    self.expect_value(&capacity.ty, "make expression")?;
                }
                self.instructions.push(Instruction::MakeSlice {
                    element_type: lower_value_type(element_type)?,
                    has_capacity: capacity.is_some(),
                });
            }
            CheckedExpressionKind::MakeMap { map_type, hint } => {
                if let Some(hint) = hint {
                    self.compile_expression(hint)?;
                    self.expect_value(&hint.ty, "make expression")?;
                }
                self.instructions.push(Instruction::MakeMap {
                    map_type: lower_value_type(map_type)?,
                    has_hint: hint.is_some(),
                });
            }
            CheckedExpressionKind::Conversion { conversion, value } => {
                self.compile_expression(value)?;
                self.expect_value(&value.ty, "conversion expression")?;
                self.instructions.push(Instruction::Convert(*conversion));
            }
            CheckedExpressionKind::Local { slot, .. } => {
                self.instructions.push(Instruction::LoadLocal(*slot));
            }
            CheckedExpressionKind::Index { target, index } => {
                self.compile_expression(target)?;
                self.expect_value(&target.ty, "index expression")?;
                self.compile_expression(index)?;
                self.expect_value(&index.ty, "index expression")?;
                if matches!(target.ty, Type::Map { .. }) {
                    self.instructions
                        .push(Instruction::IndexMap(lower_value_type(&target.ty)?));
                } else {
                    self.instructions
                        .push(Instruction::Index(lower_sequence_kind(&target.ty)?));
                }
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
                    target_kind: lower_sequence_kind(&target.ty)?,
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

    fn allocate_hidden_local(&mut self, label: &str) -> usize {
        let slot = self.local_names.len();
        self.local_names.push(format!("range${label}{slot}"));
        slot
    }

    fn compile_range_statement(
        &mut self,
        source: &CheckedExpression,
        key_binding: Option<&CheckedRangeBinding>,
        value_binding: Option<&CheckedRangeBinding>,
        body: &CheckedBlock,
    ) -> Result<(), CompileError> {
        self.compile_expression(source)?;
        self.expect_value(&source.ty, "range loop")?;
        let source_slot = self.allocate_hidden_local("source");
        self.instructions.push(Instruction::StoreLocal(source_slot));

        match &source.ty {
            Type::Slice(_) => {
                let index_slot = self.allocate_hidden_local("index");
                self.instructions.push(Instruction::PushInt(0));
                self.instructions.push(Instruction::StoreLocal(index_slot));

                let loop_start = self.instructions.len();
                self.instructions.push(Instruction::LoadLocal(index_slot));
                self.instructions.push(Instruction::LoadLocal(source_slot));
                self.instructions
                    .push(Instruction::CallBuiltin(BuiltinFunction::Len, 1));
                self.instructions.push(Instruction::Less);
                let jump_to_end = self.push_instruction(Instruction::JumpIfFalse(usize::MAX));

                self.compile_range_binding_value(key_binding, |compiler| {
                    compiler
                        .instructions
                        .push(Instruction::LoadLocal(index_slot));
                    Ok(())
                })?;
                self.compile_range_binding_value(value_binding, |compiler| {
                    compiler
                        .instructions
                        .push(Instruction::LoadLocal(source_slot));
                    compiler
                        .instructions
                        .push(Instruction::LoadLocal(index_slot));
                    compiler
                        .instructions
                        .push(Instruction::Index(SequenceKind::Slice));
                    Ok(())
                })?;

                self.compile_block(body)?;
                self.instructions.push(Instruction::LoadLocal(index_slot));
                self.instructions.push(Instruction::PushInt(1));
                self.instructions.push(Instruction::Add);
                self.instructions.push(Instruction::StoreLocal(index_slot));
                self.instructions.push(Instruction::Jump(loop_start));
                let loop_end = self.instructions.len();
                self.patch_jump(jump_to_end, Instruction::JumpIfFalse(loop_end));
            }
            Type::Map { key, .. } => {
                let keys_slot = self.allocate_hidden_local("keys");
                let index_slot = self.allocate_hidden_local("index");
                self.instructions.push(Instruction::LoadLocal(source_slot));
                self.instructions
                    .push(Instruction::MapKeys(lower_value_type(key.as_ref())?));
                self.instructions.push(Instruction::StoreLocal(keys_slot));
                self.instructions.push(Instruction::PushInt(0));
                self.instructions.push(Instruction::StoreLocal(index_slot));

                let loop_start = self.instructions.len();
                self.instructions.push(Instruction::LoadLocal(index_slot));
                self.instructions.push(Instruction::LoadLocal(keys_slot));
                self.instructions
                    .push(Instruction::CallBuiltin(BuiltinFunction::Len, 1));
                self.instructions.push(Instruction::Less);
                let jump_to_end = self.push_instruction(Instruction::JumpIfFalse(usize::MAX));

                self.compile_range_binding_value(key_binding, |compiler| {
                    compiler
                        .instructions
                        .push(Instruction::LoadLocal(keys_slot));
                    compiler
                        .instructions
                        .push(Instruction::LoadLocal(index_slot));
                    compiler
                        .instructions
                        .push(Instruction::Index(SequenceKind::Slice));
                    Ok(())
                })?;
                self.compile_range_binding_value(value_binding, |compiler| {
                    compiler
                        .instructions
                        .push(Instruction::LoadLocal(source_slot));
                    compiler
                        .instructions
                        .push(Instruction::LoadLocal(keys_slot));
                    compiler
                        .instructions
                        .push(Instruction::LoadLocal(index_slot));
                    compiler
                        .instructions
                        .push(Instruction::Index(SequenceKind::Slice));
                    compiler
                        .instructions
                        .push(Instruction::IndexMap(lower_value_type(&source.ty)?));
                    Ok(())
                })?;

                self.compile_block(body)?;
                self.instructions.push(Instruction::LoadLocal(index_slot));
                self.instructions.push(Instruction::PushInt(1));
                self.instructions.push(Instruction::Add);
                self.instructions.push(Instruction::StoreLocal(index_slot));
                self.instructions.push(Instruction::Jump(loop_start));
                let loop_end = self.instructions.len();
                self.patch_jump(jump_to_end, Instruction::JumpIfFalse(loop_end));
            }
            _ => {
                return Err(CompileError::new(format!(
                    "range lowering does not support `{}`",
                    source.ty.render()
                )));
            }
        }

        Ok(())
    }

    fn compile_range_binding_value(
        &mut self,
        binding: Option<&CheckedRangeBinding>,
        emit_value: impl FnOnce(&mut Self) -> Result<(), CompileError>,
    ) -> Result<(), CompileError> {
        let Some(binding) = binding else {
            return Ok(());
        };
        emit_value(self)?;
        match binding {
            CheckedRangeBinding::Local { slot, .. } => {
                self.instructions.push(Instruction::StoreLocal(*slot));
            }
            CheckedRangeBinding::Discard => self.instructions.push(Instruction::Pop),
        }
        Ok(())
    }
}

fn lower_value_type(ty: &Type) -> Result<ValueType, CompileError> {
    match ty {
        Type::Int => Ok(ValueType::Int),
        Type::Byte => Ok(ValueType::Byte),
        Type::Bool => Ok(ValueType::Bool),
        Type::String => Ok(ValueType::String),
        Type::UntypedNil => Err(CompileError::new(
            "runtime value types do not support untyped `nil`",
        )),
        Type::Slice(element) => Ok(ValueType::Slice(Box::new(lower_value_type(element)?))),
        Type::Map { key, value } => Ok(ValueType::Map {
            key: Box::new(lower_value_type(key)?),
            value: Box::new(lower_value_type(value)?),
        }),
        Type::Void => Err(CompileError::new(
            "runtime value types do not support `void`",
        )),
    }
}

fn lower_sequence_kind(ty: &Type) -> Result<SequenceKind, CompileError> {
    match ty {
        Type::Slice(_) => Ok(SequenceKind::Slice),
        Type::String => Ok(SequenceKind::String),
        _ => Err(CompileError::new(format!(
            "sequence operations do not support `{}`",
            ty.render()
        ))),
    }
}
