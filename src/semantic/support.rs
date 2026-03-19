use crate::frontend::ast::TypeRef;
use crate::semantic::analyzer::SemanticError;
use crate::semantic::model::{
    CheckedBlock, CheckedExpression, CheckedExpressionKind, CheckedStatement, Type,
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

pub fn validate_runtime_type(ty: &Type, context: &str) -> Result<(), SemanticError> {
    match ty {
        Type::Slice(element) => validate_runtime_type(element, context),
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
        Type::Int | Type::Byte | Type::Bool | Type::String | Type::Void => Ok(()),
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
        CheckedStatement::If {
            then_block,
            else_block: Some(else_block),
            ..
        } => block_guarantees_return(then_block) && block_guarantees_return(else_block),
        CheckedStatement::For { condition, .. } => expression_is_compile_time_true(condition),
        _ => false,
    }
}

fn expression_is_compile_time_true(expression: &CheckedExpression) -> bool {
    matches!(expression.kind, CheckedExpressionKind::Bool(true))
}
