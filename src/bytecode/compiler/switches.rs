use super::{CompileError, ControlFlowContext, FunctionCompiler, lower_value_type};
use crate::bytecode::instruction::Instruction;
use crate::semantic::model::{
    CheckedBinding, CheckedSwitchClause, CheckedSwitchStatement, CheckedTypeSwitchBinding,
    CheckedTypeSwitchBindingSource, CheckedTypeSwitchCase, CheckedTypeSwitchClause,
    CheckedTypeSwitchStatement,
};

impl<'a> FunctionCompiler<'a> {
    pub(super) fn compile_switch_statement(
        &mut self,
        switch_statement: &CheckedSwitchStatement,
    ) -> Result<(), CompileError> {
        if let Some(header) = &switch_statement.header {
            self.compile_header_statement(header, "switch header")?;
        }

        let tag_slot = if let Some(expression) = &switch_statement.expression {
            self.compile_expression(expression)?;
            self.expect_value(&expression.ty, "switch expression")?;
            let slot = self.allocate_hidden_local("switch$tag");
            self.instructions.push(Instruction::StoreLocal(slot));
            Some(slot)
        } else {
            None
        };

        let mut end_jumps = Vec::new();
        let default_body = switch_statement
            .clauses
            .iter()
            .find_map(|clause| match clause {
                CheckedSwitchClause::Default(body) => Some(body),
                CheckedSwitchClause::Case { .. } => None,
            });
        self.control_flow_stack.push(ControlFlowContext::Switch {
            break_jumps: Vec::new(),
        });

        for clause in &switch_statement.clauses {
            let CheckedSwitchClause::Case { expressions, body } = clause else {
                continue;
            };
            let mut next_test_jump = None;
            let mut success_jumps = Vec::new();
            for expression in expressions {
                let test_start = self.instructions.len();
                if let Some(previous_jump) = next_test_jump.take() {
                    self.patch_jump(previous_jump, Instruction::JumpIfFalse(test_start));
                }
                if let Some(tag_slot) = tag_slot {
                    self.instructions.push(Instruction::LoadLocal(tag_slot));
                    self.compile_expression(expression)?;
                    self.expect_value(&expression.ty, "switch case expression")?;
                    self.instructions.push(Instruction::Equal);
                } else {
                    self.compile_expression(expression)?;
                    self.expect_value(&expression.ty, "switch case expression")?;
                }
                next_test_jump = Some(self.push_instruction(Instruction::JumpIfFalse(usize::MAX)));
                success_jumps.push(self.push_instruction(Instruction::Jump(usize::MAX)));
            }

            let body_start = self.instructions.len();
            for jump in success_jumps {
                self.patch_jump(jump, Instruction::Jump(body_start));
            }
            self.compile_block(body)?;
            end_jumps.push(self.push_instruction(Instruction::Jump(usize::MAX)));

            let next_clause_start = self.instructions.len();
            if let Some(jump) = next_test_jump {
                self.patch_jump(jump, Instruction::JumpIfFalse(next_clause_start));
            }
        }

        if let Some(default_body) = default_body {
            self.compile_block(default_body)?;
        }

        let end = self.instructions.len();
        for jump in end_jumps {
            self.patch_jump(jump, Instruction::Jump(end));
        }
        self.patch_switch_break_jumps(end)?;
        self.control_flow_stack.pop();
        Ok(())
    }

    pub(super) fn compile_type_switch_statement(
        &mut self,
        type_switch_statement: &CheckedTypeSwitchStatement,
    ) -> Result<(), CompileError> {
        if let Some(header) = &type_switch_statement.header {
            self.compile_header_statement(header, "type switch header")?;
        }

        self.compile_expression(&type_switch_statement.guard)?;
        self.expect_value(&type_switch_statement.guard.ty, "type switch guard")?;
        let guard_slot = self.allocate_hidden_local("type-switch$guard");
        self.instructions.push(Instruction::StoreLocal(guard_slot));
        let value_slot = self.allocate_hidden_local("type-switch$value");
        let ok_slot = self.allocate_hidden_local("type-switch$ok");

        let mut end_jumps = Vec::new();
        let default_clause = type_switch_statement
            .clauses
            .iter()
            .find_map(|clause| match clause {
                CheckedTypeSwitchClause::Default { binding, body } => Some((binding, body)),
                CheckedTypeSwitchClause::Case { .. } => None,
            });
        self.control_flow_stack.push(ControlFlowContext::Switch {
            break_jumps: Vec::new(),
        });

        for clause in &type_switch_statement.clauses {
            let CheckedTypeSwitchClause::Case {
                cases,
                binding,
                body,
            } = clause
            else {
                continue;
            };

            let mut next_test_jump = None;
            let mut success_jumps = Vec::new();
            for case in cases {
                let test_start = self.instructions.len();
                if let Some(previous_jump) = next_test_jump.take() {
                    self.patch_jump(previous_jump, Instruction::JumpIfFalse(test_start));
                }
                self.compile_type_switch_case_test(guard_slot, value_slot, ok_slot, case)?;
                next_test_jump = Some(self.push_instruction(Instruction::JumpIfFalse(usize::MAX)));
                success_jumps.push(self.push_instruction(Instruction::Jump(usize::MAX)));
            }

            let body_start = self.instructions.len();
            for jump in success_jumps {
                self.patch_jump(jump, Instruction::Jump(body_start));
            }
            if let Some(binding) = binding {
                self.compile_type_switch_binding_store(guard_slot, value_slot, binding);
            }
            self.compile_block(body)?;
            end_jumps.push(self.push_instruction(Instruction::Jump(usize::MAX)));

            let next_clause_start = self.instructions.len();
            if let Some(jump) = next_test_jump {
                self.patch_jump(jump, Instruction::JumpIfFalse(next_clause_start));
            }
        }

        if let Some((binding, body)) = default_clause {
            if let Some(binding) = binding {
                self.compile_type_switch_binding_store(guard_slot, value_slot, binding);
            }
            self.compile_block(body)?;
        }

        let end = self.instructions.len();
        for jump in end_jumps {
            self.patch_jump(jump, Instruction::Jump(end));
        }
        self.patch_switch_break_jumps(end)?;
        self.control_flow_stack.pop();
        Ok(())
    }

    fn compile_type_switch_case_test(
        &mut self,
        guard_slot: usize,
        value_slot: usize,
        ok_slot: usize,
        case: &CheckedTypeSwitchCase,
    ) -> Result<(), CompileError> {
        match case {
            CheckedTypeSwitchCase::Nil => {
                self.instructions.push(Instruction::LoadLocal(guard_slot));
                self.instructions.push(Instruction::PushNilInterface);
                self.instructions.push(Instruction::Equal);
            }
            CheckedTypeSwitchCase::Type(ty) => {
                self.instructions.push(Instruction::LoadLocal(guard_slot));
                self.instructions
                    .push(Instruction::TypeAssertOk(lower_value_type(ty)?));
                self.instructions.push(Instruction::StoreLocal(ok_slot));
                self.instructions.push(Instruction::StoreLocal(value_slot));
                self.instructions.push(Instruction::LoadLocal(ok_slot));
            }
        }
        Ok(())
    }

    fn compile_type_switch_binding_store(
        &mut self,
        guard_slot: usize,
        value_slot: usize,
        binding: &CheckedTypeSwitchBinding,
    ) {
        let CheckedBinding::Local { slot, .. } = &binding.binding else {
            return;
        };
        match &binding.source {
            CheckedTypeSwitchBindingSource::Interface => {
                self.instructions.push(Instruction::LoadLocal(guard_slot));
            }
            CheckedTypeSwitchBindingSource::Asserted(_) => {
                self.instructions.push(Instruction::LoadLocal(value_slot));
            }
        }
        self.instructions.push(Instruction::StoreLocal(*slot));
    }
}
