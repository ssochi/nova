use super::{CompileError, FunctionCompiler, lower_sequence_kind, lower_value_type};
use crate::bytecode::instruction::Instruction;
use crate::semantic::model::{
    CheckedAssignmentTarget, CheckedBinding, CheckedCompoundAssignOperator, CheckedExpression,
    CheckedForPostStatement, CheckedHeaderStatement, CheckedIncDecOperator, Type,
};

impl<'a> FunctionCompiler<'a> {
    pub(super) fn compile_header_statement(
        &mut self,
        header: &CheckedHeaderStatement,
        context: &str,
    ) -> Result<(), CompileError> {
        match header {
            CheckedHeaderStatement::ShortVarDecl { slot, value, .. } => {
                self.compile_local_store(*slot, value, &format!("{context} short declaration"))?;
            }
            CheckedHeaderStatement::VarDecl { slot, value, .. } => {
                if let Some(value) = value {
                    self.compile_local_store(
                        *slot,
                        value,
                        &format!("{context} variable declaration"),
                    )?;
                }
            }
            CheckedHeaderStatement::Assign { target, value } => {
                self.compile_assignment(target, value, &format!("{context} assignment"))?
            }
            CheckedHeaderStatement::CompoundAssign {
                target,
                operator,
                value,
            } => self.compile_compound_assignment(
                target,
                *operator,
                value,
                &format!("{context} compound assignment"),
            )?,
            CheckedHeaderStatement::Expr(expression) => {
                self.compile_expression(expression)?;
                if expression.ty.produces_value() {
                    self.instructions.push(Instruction::Pop);
                }
            }
            CheckedHeaderStatement::MapLookup {
                map,
                key,
                value_binding,
                ok_binding,
            } => self.compile_map_lookup_statement(map, key, value_binding, ok_binding)?,
            CheckedHeaderStatement::IncDec {
                target,
                operator,
                operand_type,
            } => self.compile_inc_dec_statement(target, *operator, operand_type)?,
        }
        Ok(())
    }

    pub(super) fn compile_map_lookup_statement(
        &mut self,
        map: &CheckedExpression,
        key: &CheckedExpression,
        value_binding: &CheckedBinding,
        ok_binding: &CheckedBinding,
    ) -> Result<(), CompileError> {
        self.compile_expression(map)?;
        self.expect_value(&map.ty, "comma-ok lookup")?;
        self.compile_expression(key)?;
        self.expect_value(&key.ty, "comma-ok lookup")?;
        self.instructions
            .push(Instruction::LookupMap(lower_value_type(&map.ty)?));
        self.consume_binding_value(ok_binding);
        self.consume_binding_value(value_binding);
        Ok(())
    }

    pub(super) fn compile_for_post_statement(
        &mut self,
        post: &CheckedForPostStatement,
    ) -> Result<(), CompileError> {
        match post {
            CheckedForPostStatement::Assign { target, value } => {
                self.compile_assignment(target, value, "for post assignment")?
            }
            CheckedForPostStatement::CompoundAssign {
                target,
                operator,
                value,
            } => self.compile_compound_assignment(
                target,
                *operator,
                value,
                "for post compound assignment",
            )?,
            CheckedForPostStatement::Expr(expression) => {
                self.compile_expression(expression)?;
                if expression.ty.produces_value() {
                    self.instructions.push(Instruction::Pop);
                }
            }
            CheckedForPostStatement::MapLookup {
                map,
                key,
                value_binding,
                ok_binding,
            } => self.compile_map_lookup_statement(map, key, value_binding, ok_binding)?,
            CheckedForPostStatement::IncDec {
                target,
                operator,
                operand_type,
            } => self.compile_inc_dec_statement(target, *operator, operand_type)?,
        }
        Ok(())
    }

    pub(super) fn compile_local_store(
        &mut self,
        slot: usize,
        value: &CheckedExpression,
        context: &str,
    ) -> Result<(), CompileError> {
        self.compile_expression(value)?;
        self.expect_value(&value.ty, context)?;
        self.instructions.push(Instruction::StoreLocal(slot));
        Ok(())
    }

    pub(super) fn compile_assignment(
        &mut self,
        target: &CheckedAssignmentTarget,
        value: &CheckedExpression,
        context: &str,
    ) -> Result<(), CompileError> {
        match target {
            CheckedAssignmentTarget::Local { slot, .. } => {
                self.compile_local_store(*slot, value, context)?;
            }
            CheckedAssignmentTarget::Index { target, index } => {
                self.compile_expression(target)?;
                self.expect_value(&target.ty, context)?;
                self.compile_expression(index)?;
                self.expect_value(&index.ty, context)?;
                self.compile_expression(value)?;
                self.expect_value(&value.ty, context)?;
                if matches!(target.ty, Type::Map { .. }) {
                    self.instructions.push(Instruction::SetMapIndex);
                } else {
                    self.instructions.push(Instruction::SetIndex);
                }
            }
        }
        Ok(())
    }

    pub(super) fn compile_inc_dec_statement(
        &mut self,
        target: &CheckedAssignmentTarget,
        operator: CheckedIncDecOperator,
        operand_type: &Type,
    ) -> Result<(), CompileError> {
        match target {
            CheckedAssignmentTarget::Local { slot, .. } => {
                self.instructions.push(Instruction::LoadLocal(*slot));
                self.push_inc_dec_delta(operand_type)?;
                self.instructions
                    .push(self.inc_dec_instruction(operator, operand_type)?);
                self.instructions.push(Instruction::StoreLocal(*slot));
            }
            CheckedAssignmentTarget::Index { target, index } => {
                let target_slot = self.allocate_hidden_local("incdec$target");
                let index_slot = self.allocate_hidden_local("incdec$index");
                let value_slot = self.allocate_hidden_local("incdec$value");

                self.compile_expression(target)?;
                self.expect_value(&target.ty, "inc/dec target")?;
                self.instructions.push(Instruction::StoreLocal(target_slot));

                self.compile_expression(index)?;
                self.expect_value(&index.ty, "inc/dec target")?;
                self.instructions.push(Instruction::StoreLocal(index_slot));

                self.instructions.push(Instruction::LoadLocal(target_slot));
                self.instructions.push(Instruction::LoadLocal(index_slot));
                if matches!(target.ty, Type::Map { .. }) {
                    self.instructions
                        .push(Instruction::IndexMap(lower_value_type(&target.ty)?));
                } else {
                    self.instructions
                        .push(Instruction::Index(lower_sequence_kind(&target.ty)?));
                }
                self.push_inc_dec_delta(operand_type)?;
                self.instructions
                    .push(self.inc_dec_instruction(operator, operand_type)?);
                self.instructions.push(Instruction::StoreLocal(value_slot));

                self.instructions.push(Instruction::LoadLocal(target_slot));
                self.instructions.push(Instruction::LoadLocal(index_slot));
                self.instructions.push(Instruction::LoadLocal(value_slot));
                if matches!(target.ty, Type::Map { .. }) {
                    self.instructions.push(Instruction::SetMapIndex);
                } else {
                    self.instructions.push(Instruction::SetIndex);
                }
            }
        }
        Ok(())
    }

    pub(super) fn compile_compound_assignment(
        &mut self,
        target: &CheckedAssignmentTarget,
        operator: CheckedCompoundAssignOperator,
        value: &CheckedExpression,
        context: &str,
    ) -> Result<(), CompileError> {
        match target {
            CheckedAssignmentTarget::Local { slot, .. } => {
                self.instructions.push(Instruction::LoadLocal(*slot));
                self.compile_expression(value)?;
                self.expect_value(&value.ty, context)?;
                self.instructions
                    .push(self.compound_assign_instruction(operator)?);
                self.instructions.push(Instruction::StoreLocal(*slot));
            }
            CheckedAssignmentTarget::Index { target, index } => {
                let target_slot = self.allocate_hidden_local("compound$target");
                let index_slot = self.allocate_hidden_local("compound$index");
                let value_slot = self.allocate_hidden_local("compound$value");

                self.compile_expression(target)?;
                self.expect_value(&target.ty, context)?;
                self.instructions.push(Instruction::StoreLocal(target_slot));

                self.compile_expression(index)?;
                self.expect_value(&index.ty, context)?;
                self.instructions.push(Instruction::StoreLocal(index_slot));

                self.instructions.push(Instruction::LoadLocal(target_slot));
                self.instructions.push(Instruction::LoadLocal(index_slot));
                if matches!(target.ty, Type::Map { .. }) {
                    self.instructions
                        .push(Instruction::IndexMap(lower_value_type(&target.ty)?));
                } else {
                    self.instructions
                        .push(Instruction::Index(lower_sequence_kind(&target.ty)?));
                }
                self.compile_expression(value)?;
                self.expect_value(&value.ty, context)?;
                self.instructions
                    .push(self.compound_assign_instruction(operator)?);
                self.instructions.push(Instruction::StoreLocal(value_slot));

                self.instructions.push(Instruction::LoadLocal(target_slot));
                self.instructions.push(Instruction::LoadLocal(index_slot));
                self.instructions.push(Instruction::LoadLocal(value_slot));
                if matches!(target.ty, Type::Map { .. }) {
                    self.instructions.push(Instruction::SetMapIndex);
                } else {
                    self.instructions.push(Instruction::SetIndex);
                }
            }
        }
        Ok(())
    }

    fn push_inc_dec_delta(&mut self, operand_type: &Type) -> Result<(), CompileError> {
        match operand_type {
            Type::Int => self.instructions.push(Instruction::PushInt(1)),
            Type::Byte => self.instructions.push(Instruction::PushByte(1)),
            _ => {
                return Err(CompileError::new(format!(
                    "inc/dec lowering does not support `{}`",
                    operand_type.render()
                )));
            }
        }
        Ok(())
    }

    fn inc_dec_instruction(
        &self,
        operator: CheckedIncDecOperator,
        operand_type: &Type,
    ) -> Result<Instruction, CompileError> {
        match operand_type {
            Type::Int | Type::Byte => Ok(match operator {
                CheckedIncDecOperator::Increment => Instruction::Add,
                CheckedIncDecOperator::Decrement => Instruction::Subtract,
            }),
            _ => Err(CompileError::new(format!(
                "inc/dec lowering does not support `{}`",
                operand_type.render()
            ))),
        }
    }

    fn compound_assign_instruction(
        &self,
        operator: CheckedCompoundAssignOperator,
    ) -> Result<Instruction, CompileError> {
        Ok(match operator {
            CheckedCompoundAssignOperator::Add => Instruction::Add,
            CheckedCompoundAssignOperator::Concat => Instruction::Concat,
            CheckedCompoundAssignOperator::Subtract => Instruction::Subtract,
            CheckedCompoundAssignOperator::Multiply => Instruction::Multiply,
            CheckedCompoundAssignOperator::Divide => Instruction::Divide,
        })
    }
}
