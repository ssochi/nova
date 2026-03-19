use crate::frontend::ast::TypeRef;
use crate::semantic::analyzer::SemanticError;
use crate::semantic::model::{
    CheckedBlock, CheckedElseBranch, CheckedExpression, CheckedExpressionKind, CheckedForStatement,
    CheckedIfStatement, CheckedStatement, CheckedSwitchClause, CheckedSwitchStatement, Type,
};

pub fn resolve_type_ref(type_ref: &TypeRef) -> Option<Type> {
    match type_ref {
        TypeRef::Named(name) => match name.as_str() {
            "int" => Some(Type::Int),
            "byte" => Some(Type::Byte),
            "bool" => Some(Type::Bool),
            "string" => Some(Type::String),
            _ => None,
        },
        TypeRef::Slice(element) => Some(Type::Slice(Box::new(resolve_type_ref(element)?))),
        TypeRef::Chan(element) => Some(Type::Chan(Box::new(resolve_type_ref(element)?))),
        TypeRef::Map { key, value } => Some(Type::Map {
            key: Box::new(resolve_type_ref(key)?),
            value: Box::new(resolve_type_ref(value)?),
        }),
    }
}

pub fn is_supported_named_type(name: &str) -> bool {
    matches!(name, "int" | "byte" | "bool" | "string")
}

pub fn expect_type(expected: &Type, actual: &Type, context: &str) -> Result<(), SemanticError> {
    if expected == actual {
        Ok(())
    } else {
        Err(SemanticError::new(format!(
            "{context} requires `{}`, found `{}`",
            expected.render(),
            actual.render()
        )))
    }
}

pub fn expect_same_type(left: &Type, right: &Type, context: &str) -> Result<(), SemanticError> {
    if left == right {
        Ok(())
    } else {
        Err(SemanticError::new(format!(
            "{context} requires matching operand types, found `{}` and `{}`",
            left.render(),
            right.render()
        )))
    }
}

pub fn coerce_expression_to_type(
    expected: &Type,
    actual: CheckedExpression,
    context: &str,
) -> Result<CheckedExpression, SemanticError> {
    if &actual.ty == expected {
        return Ok(actual);
    }
    if actual.ty == Type::UntypedNil && expected.supports_nil() {
        return Ok(zero_value_expression(expected.clone()));
    }
    Err(SemanticError::new(format!(
        "{context} requires `{}`, found `{}`",
        expected.render(),
        actual.ty.render()
    )))
}

pub fn coerce_nil_equality_operands(
    left: CheckedExpression,
    right: CheckedExpression,
) -> Result<(CheckedExpression, CheckedExpression), SemanticError> {
    let left_type = left.ty.clone();
    let right_type = right.ty.clone();
    match (&left_type, &right_type) {
        (Type::UntypedNil, Type::UntypedNil) => Err(SemanticError::new(
            "equality expression does not support untyped `nil` operands",
        )),
        (Type::UntypedNil, right_type) if right_type.supports_nil() => {
            Ok((zero_value_expression(right_type.clone()), right))
        }
        (left_type, Type::UntypedNil) if left_type.supports_nil() => {
            Ok((left, zero_value_expression(left_type.clone())))
        }
        _ => {
            expect_same_type(&left_type, &right_type, "equality expression")?;
            if !left_type.supports_equality() {
                return Err(SemanticError::new(format!(
                    "equality expression does not support `{}` operands",
                    left_type.render()
                )));
            }
            Ok((left, right))
        }
    }
}

pub fn validate_runtime_type(ty: &Type, context: &str) -> Result<(), SemanticError> {
    match ty {
        Type::Slice(element) => validate_runtime_type(element, context),
        Type::Chan(element) => validate_runtime_type(element, context),
        Type::Map { key, value } => {
            validate_runtime_type(key, context)?;
            validate_runtime_type(value, context)?;
            if key.supports_map_key() {
                Ok(())
            } else {
                Err(SemanticError::new(format!(
                    "{context} requires a comparable map key type, found `{}`",
                    key.render()
                )))
            }
        }
        Type::Int | Type::Byte | Type::Bool | Type::String | Type::UntypedNil | Type::Void => {
            Ok(())
        }
    }
}

pub fn block_guarantees_return(block: &CheckedBlock) -> bool {
    for statement in &block.statements {
        if statement_guarantees_termination(statement) {
            return true;
        }
    }
    false
}

pub fn zero_value_expression(ty: Type) -> CheckedExpression {
    CheckedExpression {
        ty,
        kind: CheckedExpressionKind::ZeroValue,
    }
}

pub fn validate_make_literal_bounds(
    length: &CheckedExpression,
    capacity: &CheckedExpression,
) -> Result<(), SemanticError> {
    let CheckedExpressionKind::Integer(length_value) = &length.kind else {
        return Ok(());
    };
    let CheckedExpressionKind::Integer(capacity_value) = &capacity.kind else {
        return Ok(());
    };
    if length_value > capacity_value {
        return Err(SemanticError::new(format!(
            "builtin `make` length {} exceeds capacity {}",
            length_value, capacity_value
        )));
    }
    Ok(())
}

fn statement_guarantees_termination(statement: &CheckedStatement) -> bool {
    match statement {
        CheckedStatement::Return(_) => true,
        CheckedStatement::If(if_statement) => if_statement_guarantees_termination(if_statement),
        CheckedStatement::Switch(switch_statement) => {
            switch_statement_guarantees_termination(switch_statement)
        }
        CheckedStatement::For(for_statement) => for_statement_guarantees_termination(for_statement),
        CheckedStatement::RangeFor { .. } => false,
        CheckedStatement::Break | CheckedStatement::Continue => false,
        _ => false,
    }
}

fn if_statement_guarantees_termination(if_statement: &CheckedIfStatement) -> bool {
    block_guarantees_return(&if_statement.then_block)
        && match &if_statement.else_branch {
            Some(CheckedElseBranch::Block(else_block)) => block_guarantees_return(else_block),
            Some(CheckedElseBranch::If(else_if)) => if_statement_guarantees_termination(else_if),
            None => false,
        }
}

fn switch_statement_guarantees_termination(switch_statement: &CheckedSwitchStatement) -> bool {
    let mut has_default = false;
    for clause in &switch_statement.clauses {
        match clause {
            CheckedSwitchClause::Case { body, .. } => {
                if !block_guarantees_return(body) || block_contains_break_for_switch(body) {
                    return false;
                }
            }
            CheckedSwitchClause::Default(body) => {
                has_default = true;
                if !block_guarantees_return(body) || block_contains_break_for_switch(body) {
                    return false;
                }
            }
        }
    }
    has_default
}

fn for_statement_guarantees_termination(for_statement: &CheckedForStatement) -> bool {
    let Some(condition) = &for_statement.condition else {
        return !block_contains_break_for_loop(&for_statement.body);
    };
    expression_is_compile_time_true(condition)
        && !block_contains_break_for_loop(&for_statement.body)
}

fn expression_is_compile_time_true(expression: &CheckedExpression) -> bool {
    matches!(expression.kind, CheckedExpressionKind::Bool(true))
}

fn block_contains_break_for_loop(block: &CheckedBlock) -> bool {
    block
        .statements
        .iter()
        .any(statement_contains_break_for_loop)
}

fn statement_contains_break_for_loop(statement: &CheckedStatement) -> bool {
    match statement {
        CheckedStatement::Break => true,
        CheckedStatement::If(if_statement) => {
            block_contains_break_for_loop(&if_statement.then_block)
                || match &if_statement.else_branch {
                    Some(CheckedElseBranch::Block(else_block)) => {
                        block_contains_break_for_loop(else_block)
                    }
                    Some(CheckedElseBranch::If(else_if)) => {
                        if_statement_contains_break_for_loop(else_if)
                    }
                    None => false,
                }
        }
        CheckedStatement::Switch(_)
        | CheckedStatement::For(_)
        | CheckedStatement::RangeFor { .. } => false,
        _ => false,
    }
}

fn if_statement_contains_break_for_loop(if_statement: &CheckedIfStatement) -> bool {
    block_contains_break_for_loop(&if_statement.then_block)
        || match &if_statement.else_branch {
            Some(CheckedElseBranch::Block(else_block)) => block_contains_break_for_loop(else_block),
            Some(CheckedElseBranch::If(else_if)) => if_statement_contains_break_for_loop(else_if),
            None => false,
        }
}

fn block_contains_break_for_switch(block: &CheckedBlock) -> bool {
    block
        .statements
        .iter()
        .any(statement_contains_break_for_switch)
}

fn statement_contains_break_for_switch(statement: &CheckedStatement) -> bool {
    match statement {
        CheckedStatement::Break => true,
        CheckedStatement::If(if_statement) => {
            block_contains_break_for_switch(&if_statement.then_block)
                || match &if_statement.else_branch {
                    Some(CheckedElseBranch::Block(else_block)) => {
                        block_contains_break_for_switch(else_block)
                    }
                    Some(CheckedElseBranch::If(else_if)) => {
                        if_statement_contains_break_for_switch(else_if)
                    }
                    None => false,
                }
        }
        CheckedStatement::Switch(_)
        | CheckedStatement::For(_)
        | CheckedStatement::RangeFor { .. } => false,
        _ => false,
    }
}

fn if_statement_contains_break_for_switch(if_statement: &CheckedIfStatement) -> bool {
    block_contains_break_for_switch(&if_statement.then_block)
        || match &if_statement.else_branch {
            Some(CheckedElseBranch::Block(else_block)) => {
                block_contains_break_for_switch(else_block)
            }
            Some(CheckedElseBranch::If(else_if)) => if_statement_contains_break_for_switch(else_if),
            None => false,
        }
}
