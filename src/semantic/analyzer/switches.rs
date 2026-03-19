use std::collections::HashMap;

use crate::frontend::ast::{SwitchClause, SwitchStatement};
use crate::semantic::analyzer::{ControlFlowContext, FunctionAnalyzer, SemanticError};
use crate::semantic::model::{
    CheckedExpression, CheckedExpressionKind, CheckedStatement, CheckedSwitchClause,
    CheckedSwitchStatement, Type,
};
use crate::semantic::support::coerce_expression_to_type;

impl<'a> FunctionAnalyzer<'a> {
    pub(super) fn analyze_switch_statement(
        &mut self,
        switch_statement: &SwitchStatement,
    ) -> Result<CheckedStatement, SemanticError> {
        self.scopes.push(Default::default());
        let header = switch_statement
            .header
            .as_ref()
            .map(|header| self.analyze_header_statement(header))
            .transpose()?;
        let expression = switch_statement
            .expression
            .as_ref()
            .map(|expression| self.analyze_expression(expression))
            .transpose()?;
        let match_type = self.validate_switch_expression(expression.as_ref())?;
        self.control_flow_stack.push(ControlFlowContext::Switch);
        let clauses = self.analyze_switch_clauses(&switch_statement.clauses, &match_type)?;
        self.control_flow_stack.pop();
        self.scopes.pop();

        Ok(CheckedStatement::Switch(CheckedSwitchStatement {
            header,
            expression,
            clauses,
        }))
    }

    fn validate_switch_expression(
        &self,
        expression: Option<&CheckedExpression>,
    ) -> Result<Type, SemanticError> {
        match expression {
            Some(expression) => {
                if expression.ty == Type::UntypedNil {
                    return Err(SemanticError::new(
                        "switch expression requires a typed comparable value, found `nil`",
                    ));
                }
                if !expression.ty.supports_equality() {
                    return Err(SemanticError::new(format!(
                        "switch expression requires a comparable value, found `{}`",
                        expression.ty.render()
                    )));
                }
                Ok(expression.ty.clone())
            }
            None => Ok(Type::Bool),
        }
    }

    fn analyze_switch_clauses(
        &mut self,
        clauses: &[SwitchClause],
        match_type: &Type,
    ) -> Result<Vec<CheckedSwitchClause>, SemanticError> {
        let mut checked = Vec::with_capacity(clauses.len());
        let mut default_seen = false;
        let mut literal_cases = HashMap::new();

        for clause in clauses {
            match clause {
                SwitchClause::Case { expressions, body } => {
                    let checked_expressions = expressions
                        .iter()
                        .enumerate()
                        .map(|(index, expression)| {
                            let checked_expression = coerce_expression_to_type(
                                match_type,
                                self.analyze_expression(expression)?,
                                &format!("switch case expression {}", index + 1),
                            )?;
                            register_literal_case(&mut literal_cases, &checked_expression)?;
                            Ok(checked_expression)
                        })
                        .collect::<Result<Vec<_>, SemanticError>>()?;
                    let body = self.analyze_block(body, true)?;
                    checked.push(CheckedSwitchClause::Case {
                        expressions: checked_expressions,
                        body,
                    });
                }
                SwitchClause::Default(body) => {
                    if default_seen {
                        return Err(SemanticError::new(
                            "switch statement may only contain one `default` clause",
                        ));
                    }
                    default_seen = true;
                    checked.push(CheckedSwitchClause::Default(
                        self.analyze_block(body, true)?,
                    ));
                }
            }
        }

        Ok(checked)
    }
}

fn register_literal_case(
    seen: &mut HashMap<LiteralCaseKey, ()>,
    expression: &CheckedExpression,
) -> Result<(), SemanticError> {
    let Some(key) = literal_case_key(expression) else {
        return Ok(());
    };
    if seen.insert(key.clone(), ()).is_some() {
        return Err(SemanticError::new(format!(
            "duplicate switch case literal {}",
            key.render()
        )));
    }
    Ok(())
}

fn literal_case_key(expression: &CheckedExpression) -> Option<LiteralCaseKey> {
    match &expression.kind {
        CheckedExpressionKind::Integer(value) => Some(LiteralCaseKey::Int(*value)),
        CheckedExpressionKind::Bool(value) => Some(LiteralCaseKey::Bool(*value)),
        CheckedExpressionKind::String(value) => Some(LiteralCaseKey::String(value.clone())),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum LiteralCaseKey {
    Int(i64),
    Bool(bool),
    String(String),
}

impl LiteralCaseKey {
    fn render(&self) -> String {
        match self {
            LiteralCaseKey::Int(value) => value.to_string(),
            LiteralCaseKey::Bool(value) => value.to_string(),
            LiteralCaseKey::String(value) => format!("{value:?}"),
        }
    }
}
