use std::collections::HashMap;

use crate::frontend::ast::{
    SwitchClause, SwitchStatement, TypeSwitchCase, TypeSwitchClause, TypeSwitchStatement,
};
use crate::semantic::analyzer::{ControlFlowContext, FunctionAnalyzer, SemanticError};
use crate::semantic::model::{
    CheckedExpression, CheckedExpressionKind, CheckedStatement, CheckedSwitchClause,
    CheckedSwitchStatement, CheckedTypeSwitchBinding, CheckedTypeSwitchBindingSource,
    CheckedTypeSwitchCase, CheckedTypeSwitchClause, CheckedTypeSwitchStatement, Type,
};
use crate::semantic::support::{
    coerce_expression_to_type, resolve_type_ref, validate_runtime_type,
};

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

    pub(super) fn analyze_type_switch_statement(
        &mut self,
        type_switch_statement: &TypeSwitchStatement,
    ) -> Result<CheckedStatement, SemanticError> {
        self.scopes.push(Default::default());
        let header = type_switch_statement
            .header
            .as_ref()
            .map(|header| self.analyze_header_statement(header))
            .transpose()?;
        let guard = self.analyze_expression(&type_switch_statement.guard.expression)?;
        if guard.ty != Type::Any {
            return Err(SemanticError::new(format!(
                "type switch guard requires interface operand, found `{}`",
                guard.ty.render()
            )));
        }

        self.control_flow_stack.push(ControlFlowContext::Switch);
        let clauses = self.analyze_type_switch_clauses(
            &type_switch_statement.clauses,
            type_switch_statement.guard.binding.as_deref(),
        )?;
        self.control_flow_stack.pop();
        self.scopes.pop();

        Ok(CheckedStatement::TypeSwitch(CheckedTypeSwitchStatement {
            header,
            guard,
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

    fn analyze_type_switch_clauses(
        &mut self,
        clauses: &[TypeSwitchClause],
        binding_name: Option<&str>,
    ) -> Result<Vec<CheckedTypeSwitchClause>, SemanticError> {
        let mut checked = Vec::with_capacity(clauses.len());
        let mut default_seen = false;
        let mut seen_cases = HashMap::new();

        for clause in clauses {
            match clause {
                TypeSwitchClause::Case { cases, body } => {
                    let checked_cases = self.analyze_type_switch_cases(cases, &mut seen_cases)?;
                    let binding_source = binding_name
                        .map(|_| type_switch_binding_source(&checked_cases))
                        .unwrap_or(CheckedTypeSwitchBindingSource::Interface);
                    let (binding, body) =
                        self.analyze_type_switch_clause_body(body, binding_name, &binding_source)?;
                    checked.push(CheckedTypeSwitchClause::Case {
                        cases: checked_cases,
                        binding,
                        body,
                    });
                }
                TypeSwitchClause::Default(body) => {
                    if default_seen {
                        return Err(SemanticError::new(
                            "type switch may only contain one `default` clause",
                        ));
                    }
                    default_seen = true;
                    let binding_source = CheckedTypeSwitchBindingSource::Interface;
                    let (binding, body) =
                        self.analyze_type_switch_clause_body(body, binding_name, &binding_source)?;
                    checked.push(CheckedTypeSwitchClause::Default { binding, body });
                }
            }
        }

        Ok(checked)
    }

    fn analyze_type_switch_cases(
        &mut self,
        cases: &[TypeSwitchCase],
        seen_cases: &mut HashMap<TypeSwitchCaseKey, ()>,
    ) -> Result<Vec<CheckedTypeSwitchCase>, SemanticError> {
        let mut checked = Vec::with_capacity(cases.len());
        for case in cases {
            let checked_case = match case {
                TypeSwitchCase::Nil => CheckedTypeSwitchCase::Nil,
                TypeSwitchCase::Type(type_ref) => {
                    let ty = resolve_type_ref(type_ref).ok_or_else(|| {
                        SemanticError::new(format!(
                            "type switch case does not support type `{}`",
                            type_ref.render()
                        ))
                    })?;
                    validate_runtime_type(&ty, "type switch case")?;
                    CheckedTypeSwitchCase::Type(ty)
                }
            };
            register_type_switch_case(seen_cases, &checked_case)?;
            checked.push(checked_case);
        }
        Ok(checked)
    }

    fn analyze_type_switch_clause_body(
        &mut self,
        body: &crate::frontend::ast::Block,
        binding_name: Option<&str>,
        binding_source: &CheckedTypeSwitchBindingSource,
    ) -> Result<
        (
            Option<CheckedTypeSwitchBinding>,
            crate::semantic::model::CheckedBlock,
        ),
        SemanticError,
    > {
        self.scopes.push(Default::default());
        let binding = match binding_name {
            Some(name) => {
                let ty = match binding_source {
                    CheckedTypeSwitchBindingSource::Interface => Type::Any,
                    CheckedTypeSwitchBindingSource::Asserted(ty) => ty.clone(),
                };
                let slot = self.allocate_local(name.to_string(), ty);
                Some(CheckedTypeSwitchBinding {
                    binding: crate::semantic::model::CheckedBinding::Local {
                        slot,
                        name: name.to_string(),
                    },
                    source: binding_source.clone(),
                })
            }
            None => None,
        };
        let body = self.analyze_block(body, false)?;
        self.scopes.pop();
        Ok((binding, body))
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

fn type_switch_binding_source(cases: &[CheckedTypeSwitchCase]) -> CheckedTypeSwitchBindingSource {
    if let [CheckedTypeSwitchCase::Type(ty)] = cases {
        CheckedTypeSwitchBindingSource::Asserted(ty.clone())
    } else {
        CheckedTypeSwitchBindingSource::Interface
    }
}

fn register_type_switch_case(
    seen: &mut HashMap<TypeSwitchCaseKey, ()>,
    case: &CheckedTypeSwitchCase,
) -> Result<(), SemanticError> {
    let key = match case {
        CheckedTypeSwitchCase::Type(ty) => TypeSwitchCaseKey::Type(ty.render()),
        CheckedTypeSwitchCase::Nil => TypeSwitchCaseKey::Nil,
    };
    if seen.insert(key.clone(), ()).is_some() {
        return Err(SemanticError::new(format!(
            "duplicate case {} in type switch",
            key.render()
        )));
    }
    Ok(())
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum TypeSwitchCaseKey {
    Type(String),
    Nil,
}

impl TypeSwitchCaseKey {
    fn render(&self) -> String {
        match self {
            TypeSwitchCaseKey::Type(ty) => ty.clone(),
            TypeSwitchCaseKey::Nil => "nil".to_string(),
        }
    }
}
