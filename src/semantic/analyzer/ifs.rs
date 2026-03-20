use crate::frontend::ast::{ElseBranch, HeaderStatement, IfStatement};
use crate::semantic::analyzer::{FunctionAnalyzer, SemanticError};
use crate::semantic::model::{
    CheckedElseBranch, CheckedHeaderStatement, CheckedIfStatement, CheckedStatement, Type,
};
use crate::semantic::support::expect_type;

impl<'a> FunctionAnalyzer<'a> {
    pub(super) fn analyze_if_statement(
        &mut self,
        if_statement: &IfStatement,
    ) -> Result<CheckedStatement, SemanticError> {
        self.scopes.push(Default::default());
        let header = if_statement
            .header
            .as_ref()
            .map(|header| self.analyze_header_statement(header))
            .transpose()?;
        let condition = self.analyze_expression(&if_statement.condition)?;
        expect_type(&Type::Bool, &condition.ty, "if condition")?;
        let then_block = self.analyze_block(&if_statement.then_block, true)?;
        let else_branch = if_statement
            .else_branch
            .as_ref()
            .map(|else_branch| self.analyze_else_branch(else_branch))
            .transpose()?;
        self.scopes.pop();

        Ok(CheckedStatement::If(CheckedIfStatement {
            header,
            condition,
            then_block,
            else_branch,
        }))
    }

    pub(super) fn analyze_header_statement(
        &mut self,
        header: &HeaderStatement,
    ) -> Result<CheckedHeaderStatement, SemanticError> {
        match header {
            HeaderStatement::ShortVarDecl { bindings, values } => {
                checked_statement_to_header_statement(
                    self.analyze_short_var_decl_statement(bindings, values)?,
                )
            }
            HeaderStatement::MultiAssign { bindings, values } => {
                checked_statement_to_header_statement(
                    self.analyze_multi_assign_statement(bindings, values)?,
                )
            }
            HeaderStatement::VarDecl {
                name,
                type_ref,
                value,
            } => checked_statement_to_header_statement(self.analyze_var_decl_statement(
                name,
                type_ref.as_ref(),
                value.as_ref(),
            )?),
            HeaderStatement::Assign { target, value } => checked_statement_to_header_statement(
                self.analyze_assignment_statement(target, value)?,
            ),
            HeaderStatement::CompoundAssign {
                target,
                operator,
                value,
            } => checked_statement_to_header_statement(
                self.analyze_compound_assign_statement(target, *operator, value)?,
            ),
            HeaderStatement::Expr(expression) => checked_statement_to_header_statement(
                self.analyze_expression_statement(expression)?,
            ),
            HeaderStatement::MapLookup {
                bindings,
                binding_mode,
                target,
                key,
            } => self.analyze_map_lookup_initializer(bindings, *binding_mode, target, key),
            HeaderStatement::IncDec { target, operator } => checked_statement_to_header_statement(
                self.analyze_inc_dec_statement(target, *operator)?,
            ),
        }
    }

    fn analyze_else_branch(
        &mut self,
        else_branch: &ElseBranch,
    ) -> Result<CheckedElseBranch, SemanticError> {
        match else_branch {
            ElseBranch::Block(block) => {
                Ok(CheckedElseBranch::Block(self.analyze_block(block, true)?))
            }
            ElseBranch::If(if_statement) => {
                let checked_if = self.analyze_if_statement(if_statement)?;
                let CheckedStatement::If(if_statement) = checked_if else {
                    unreachable!("if statement analysis always returns checked if data");
                };
                Ok(CheckedElseBranch::If(Box::new(if_statement)))
            }
        }
    }
}

fn checked_statement_to_header_statement(
    statement: CheckedStatement,
) -> Result<CheckedHeaderStatement, SemanticError> {
    Ok(match statement {
        CheckedStatement::ShortVarDecl { bindings, values } => {
            CheckedHeaderStatement::ShortVarDecl { bindings, values }
        }
        CheckedStatement::MultiAssign { bindings, values } => {
            CheckedHeaderStatement::MultiAssign { bindings, values }
        }
        CheckedStatement::VarDecl { slot, name, value } => {
            CheckedHeaderStatement::VarDecl { slot, name, value }
        }
        CheckedStatement::Assign { target, value } => {
            CheckedHeaderStatement::Assign { target, value }
        }
        CheckedStatement::CompoundAssign {
            target,
            operator,
            value,
        } => CheckedHeaderStatement::CompoundAssign {
            target,
            operator,
            value,
        },
        CheckedStatement::Expr(expression) => CheckedHeaderStatement::Expr(expression),
        CheckedStatement::MapLookup {
            map,
            key,
            value_binding,
            ok_binding,
        } => CheckedHeaderStatement::MapLookup {
            map,
            key,
            value_binding,
            ok_binding,
        },
        CheckedStatement::IncDec {
            target,
            operator,
            operand_type,
        } => CheckedHeaderStatement::IncDec {
            target,
            operator,
            operand_type,
        },
        CheckedStatement::If(_)
        | CheckedStatement::Defer(_)
        | CheckedStatement::Send { .. }
        | CheckedStatement::Switch(_)
        | CheckedStatement::For(_)
        | CheckedStatement::RangeFor { .. }
        | CheckedStatement::Break
        | CheckedStatement::Continue
        | CheckedStatement::Return(_) => {
            return Err(SemanticError::new(
                "control-flow header requires a simple statement supported by the current frontend",
            ));
        }
    })
}
