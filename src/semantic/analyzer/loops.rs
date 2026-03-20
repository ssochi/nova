use crate::frontend::ast::{ForPostStatement, ForStatement};
use crate::semantic::analyzer::{ControlFlowContext, FunctionAnalyzer, SemanticError};
use crate::semantic::model::{
    CheckedForPostStatement, CheckedForStatement, CheckedStatement, Type,
};
use crate::semantic::support::expect_type;

impl<'a> FunctionAnalyzer<'a> {
    pub(super) fn analyze_for_statement(
        &mut self,
        for_statement: &ForStatement,
    ) -> Result<CheckedStatement, SemanticError> {
        self.scopes.push(Default::default());
        let init = for_statement
            .init
            .as_ref()
            .map(|header| self.analyze_header_statement(header))
            .transpose()?;
        let condition = for_statement
            .condition
            .as_ref()
            .map(|condition| self.analyze_expression(condition))
            .transpose()?;
        if let Some(condition) = &condition {
            expect_type(&Type::Bool, &condition.ty, "for condition")?;
        }
        let post = for_statement
            .post
            .as_ref()
            .map(|post| self.analyze_for_post_statement(post))
            .transpose()?;

        self.control_flow_stack.push(ControlFlowContext::Loop);
        let body = self.analyze_block(&for_statement.body, true)?;
        self.control_flow_stack.pop();
        self.scopes.pop();

        Ok(CheckedStatement::For(CheckedForStatement {
            init,
            condition,
            post,
            body,
        }))
    }

    pub(super) fn analyze_break_statement(&mut self) -> Result<CheckedStatement, SemanticError> {
        if self.control_flow_stack.is_empty() {
            return Err(SemanticError::new(
                "`break` requires an enclosing `for`, `range`, or `switch`",
            ));
        }
        Ok(CheckedStatement::Break)
    }

    pub(super) fn analyze_continue_statement(&mut self) -> Result<CheckedStatement, SemanticError> {
        if !self
            .control_flow_stack
            .iter()
            .any(|context| *context == ControlFlowContext::Loop)
        {
            return Err(SemanticError::new(
                "`continue` requires an enclosing `for` or `range` loop",
            ));
        }
        Ok(CheckedStatement::Continue)
    }

    fn analyze_for_post_statement(
        &mut self,
        post: &ForPostStatement,
    ) -> Result<CheckedForPostStatement, SemanticError> {
        match post {
            ForPostStatement::Assign { target, value } => {
                let statement = self.analyze_assignment_statement(target, value)?;
                let CheckedStatement::Assign { target, value } = statement else {
                    unreachable!("assignment analysis always returns checked assignment data");
                };
                Ok(CheckedForPostStatement::Assign { target, value })
            }
            ForPostStatement::MultiAssign { bindings, values } => {
                let statement = self.analyze_multi_assign_statement(bindings, values)?;
                let CheckedStatement::MultiAssign { bindings, values } = statement else {
                    unreachable!(
                        "multi-assignment analysis always returns checked multi-assignment data"
                    );
                };
                Ok(CheckedForPostStatement::MultiAssign { bindings, values })
            }
            ForPostStatement::CompoundAssign {
                target,
                operator,
                value,
            } => {
                let statement = self.analyze_compound_assign_statement(target, *operator, value)?;
                let CheckedStatement::CompoundAssign {
                    target,
                    operator,
                    value,
                } = statement
                else {
                    unreachable!(
                        "compound-assignment analysis always returns checked compound-assignment data"
                    );
                };
                Ok(CheckedForPostStatement::CompoundAssign {
                    target,
                    operator,
                    value,
                })
            }
            ForPostStatement::Expr(expression) => {
                let statement = self.analyze_expression_statement(expression)?;
                let CheckedStatement::Expr(expression) = statement else {
                    unreachable!("expression analysis always returns checked expression data");
                };
                Ok(CheckedForPostStatement::Expr(expression))
            }
            ForPostStatement::MapLookup {
                bindings,
                target,
                key,
            } => {
                let statement = self.analyze_map_lookup_statement(
                    bindings,
                    crate::frontend::ast::BindingMode::Assign,
                    target,
                    key,
                )?;
                let CheckedStatement::MapLookup {
                    map,
                    key,
                    value_binding,
                    ok_binding,
                } = statement
                else {
                    unreachable!("map lookup analysis always returns checked map lookup data");
                };
                Ok(CheckedForPostStatement::MapLookup {
                    map,
                    key,
                    value_binding,
                    ok_binding,
                })
            }
            ForPostStatement::TypeAssert {
                bindings,
                target,
                asserted_type,
            } => {
                let statement = self.analyze_type_assert_statement(
                    bindings,
                    crate::frontend::ast::BindingMode::Assign,
                    target,
                    asserted_type,
                )?;
                let CheckedStatement::TypeAssert {
                    interface,
                    asserted_type,
                    value_binding,
                    ok_binding,
                } = statement
                else {
                    unreachable!("type-assert analysis always returns checked type-assert data");
                };
                Ok(CheckedForPostStatement::TypeAssert {
                    interface,
                    asserted_type,
                    value_binding,
                    ok_binding,
                })
            }
            ForPostStatement::IncDec { target, operator } => {
                let statement = self.analyze_inc_dec_statement(target, *operator)?;
                let CheckedStatement::IncDec {
                    target,
                    operator,
                    operand_type,
                } = statement
                else {
                    unreachable!("inc/dec analysis always returns checked inc/dec data");
                };
                Ok(CheckedForPostStatement::IncDec {
                    target,
                    operator,
                    operand_type,
                })
            }
        }
    }
}
