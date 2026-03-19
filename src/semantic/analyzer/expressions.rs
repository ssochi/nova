use std::collections::HashSet;

use crate::conversion::ConversionKind;
use crate::frontend::ast::{BinaryOperator, Expression, TypeRef};
use crate::semantic::analyzer::{FunctionAnalyzer, SemanticError};
use crate::semantic::builtins::{resolve_builtin, validate_builtin_call, validate_make_call};
use crate::semantic::model::{
    CallTarget, CheckedBinaryOperator, CheckedExpression, CheckedExpressionKind,
    CheckedMapLiteralEntry, Type,
};
use crate::semantic::packages::{resolve_package_function, validate_package_call};
use crate::semantic::support::{
    coerce_expression_to_type, coerce_nil_equality_operands, expect_type, resolve_type_ref,
    validate_make_literal_bounds, validate_runtime_type,
};

impl<'a> FunctionAnalyzer<'a> {
    pub(super) fn analyze_expression(
        &mut self,
        expression: &Expression,
    ) -> Result<CheckedExpression, SemanticError> {
        match expression {
            Expression::Integer(value) => Ok(CheckedExpression {
                ty: Type::Int,
                kind: CheckedExpressionKind::Integer(*value),
            }),
            Expression::Bool(value) => Ok(CheckedExpression {
                ty: Type::Bool,
                kind: CheckedExpressionKind::Bool(*value),
            }),
            Expression::String(value) => Ok(CheckedExpression {
                ty: Type::String,
                kind: CheckedExpressionKind::String(value.clone()),
            }),
            Expression::Nil => Ok(CheckedExpression {
                ty: Type::UntypedNil,
                kind: CheckedExpressionKind::UntypedNil,
            }),
            Expression::SliceLiteral {
                element_type,
                elements,
            } => self.analyze_slice_literal_expression(element_type, elements),
            Expression::MapLiteral { map_type, entries } => {
                self.analyze_map_literal_expression(map_type, entries)
            }
            Expression::Identifier(name) => {
                let binding = self.lookup_local(name)?.clone();
                Ok(CheckedExpression {
                    ty: binding.ty.clone(),
                    kind: CheckedExpressionKind::Local {
                        slot: binding.slot,
                        name: name.clone(),
                    },
                })
            }
            Expression::Index { target, index } => {
                let target = self.analyze_expression(target)?;
                let index = self.analyze_expression(index)?;
                let element_type = match &target.ty {
                    Type::Slice(element) => {
                        expect_type(&Type::Int, &index.ty, "index expression")?;
                        element.as_ref().clone()
                    }
                    Type::String => {
                        expect_type(&Type::Int, &index.ty, "index expression")?;
                        Type::Byte
                    }
                    Type::Map { key, value } => {
                        expect_type(key.as_ref(), &index.ty, "map index")?;
                        value.as_ref().clone()
                    }
                    _ => {
                        return Err(SemanticError::new(format!(
                            "index expression requires `slice`, `string`, or `map` target, found `{}`",
                            target.ty.render()
                        )));
                    }
                };
                Ok(CheckedExpression {
                    ty: element_type,
                    kind: CheckedExpressionKind::Index {
                        target: Box::new(target),
                        index: Box::new(index),
                    },
                })
            }
            Expression::Slice { target, low, high } => {
                let target = self.analyze_expression(target)?;
                if !matches!(target.ty, Type::Slice(_) | Type::String) {
                    return Err(SemanticError::new(format!(
                        "slice expression requires `slice` or `string` target, found `{}`",
                        target.ty.render()
                    )));
                }
                let low = low
                    .as_ref()
                    .map(|value| self.analyze_expression(value))
                    .transpose()?;
                if let Some(low) = &low {
                    expect_type(&Type::Int, &low.ty, "slice expression lower bound")?;
                }
                let high = high
                    .as_ref()
                    .map(|value| self.analyze_expression(value))
                    .transpose()?;
                if let Some(high) = &high {
                    expect_type(&Type::Int, &high.ty, "slice expression upper bound")?;
                }
                Ok(CheckedExpression {
                    ty: if target.ty == Type::String {
                        Type::String
                    } else {
                        target.ty.clone()
                    },
                    kind: CheckedExpressionKind::Slice {
                        target: Box::new(target),
                        low: low.map(Box::new),
                        high: high.map(Box::new),
                    },
                })
            }
            Expression::Selector { .. } => Err(SemanticError::new(
                "selector expressions are only supported as imported package call targets",
            )),
            Expression::Make {
                type_ref,
                arguments,
            } => self.analyze_make_expression(type_ref, arguments),
            Expression::Conversion { type_ref, value } => {
                self.analyze_conversion_expression(type_ref, value)
            }
            Expression::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.analyze_expression(left)?;
                let right = self.analyze_expression(right)?;
                let (operator, ty, left, right) = analyze_binary_operator(*operator, left, right)?;
                Ok(CheckedExpression {
                    ty,
                    kind: CheckedExpressionKind::Binary {
                        left: Box::new(left),
                        operator,
                        right: Box::new(right),
                    },
                })
            }
            Expression::Call { callee, arguments } => {
                let checked_arguments = arguments
                    .iter()
                    .map(|argument| self.analyze_expression(argument))
                    .collect::<Result<Vec<_>, _>>()?;
                self.analyze_call(callee, checked_arguments)
            }
        }
    }

    fn analyze_call(
        &self,
        callee: &Expression,
        checked_arguments: Vec<CheckedExpression>,
    ) -> Result<CheckedExpression, SemanticError> {
        match callee {
            Expression::Identifier(name) => self.analyze_identifier_call(name, checked_arguments),
            Expression::Selector { target, member } => {
                self.analyze_package_call(target, member, checked_arguments)
            }
            _ => Err(SemanticError::new(
                "call target must be a function name or imported package member",
            )),
        }
    }

    fn analyze_identifier_call(
        &self,
        callee: &str,
        checked_arguments: Vec<CheckedExpression>,
    ) -> Result<CheckedExpression, SemanticError> {
        if let Some(builtin) = resolve_builtin(callee) {
            let argument_types = checked_arguments
                .iter()
                .map(|argument| argument.ty.clone())
                .collect::<Vec<_>>();
            let return_type =
                validate_builtin_call(builtin, &argument_types).map_err(SemanticError::new)?;
            return Ok(CheckedExpression {
                ty: return_type,
                kind: CheckedExpressionKind::Call {
                    target: CallTarget::Builtin(builtin),
                    arguments: checked_arguments,
                },
            });
        }

        let function_index = self
            .registry
            .lookup(callee)
            .ok_or_else(|| SemanticError::new(format!("unknown function `{callee}`")))?;
        let signature = self.registry.signature(function_index);
        if checked_arguments.len() != signature.parameters.len() {
            return Err(SemanticError::new(format!(
                "function `{callee}` expects {} arguments, found {}",
                signature.parameters.len(),
                checked_arguments.len()
            )));
        }

        let checked_arguments = checked_arguments
            .into_iter()
            .zip(signature.parameters.iter())
            .enumerate()
            .map(|(index, (argument, expected))| {
                coerce_expression_to_type(
                    expected,
                    argument,
                    &format!("argument {} in call to `{callee}`", index + 1),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(CheckedExpression {
            ty: signature.return_type.clone(),
            kind: CheckedExpressionKind::Call {
                target: CallTarget::UserDefined {
                    function_index,
                    name: signature.name.clone(),
                },
                arguments: checked_arguments,
            },
        })
    }

    fn analyze_make_expression(
        &mut self,
        type_ref: &TypeRef,
        arguments: &[Expression],
    ) -> Result<CheckedExpression, SemanticError> {
        let allocated_type = resolve_type_ref(type_ref).ok_or_else(|| {
            SemanticError::new(format!(
                "builtin `make` does not support type `{}`",
                type_ref.render()
            ))
        })?;
        validate_runtime_type(&allocated_type, "builtin `make` type argument")?;
        let checked_arguments = arguments
            .iter()
            .map(|expression| self.analyze_expression(expression))
            .collect::<Result<Vec<_>, _>>()?;
        let argument_types = checked_arguments
            .iter()
            .map(|argument| argument.ty.clone())
            .collect::<Vec<_>>();
        let result_type =
            validate_make_call(&allocated_type, &argument_types).map_err(SemanticError::new)?;
        match result_type.clone() {
            Type::Slice(element_type) => {
                let length = checked_arguments[0].clone();
                let capacity = checked_arguments.get(1).cloned();
                if let Some(capacity) = &capacity {
                    validate_make_literal_bounds(&length, capacity)?;
                }
                Ok(CheckedExpression {
                    ty: Type::Slice(element_type.clone()),
                    kind: CheckedExpressionKind::MakeSlice {
                        element_type: element_type.as_ref().clone(),
                        length: Box::new(length),
                        capacity: capacity.map(Box::new),
                    },
                })
            }
            Type::Map { .. } => Ok(CheckedExpression {
                ty: result_type.clone(),
                kind: CheckedExpressionKind::MakeMap {
                    map_type: result_type,
                    hint: checked_arguments.into_iter().next().map(Box::new),
                },
            }),
            _ => Err(SemanticError::new(
                "builtin `make` lowered into an unsupported result kind",
            )),
        }
    }

    fn analyze_package_call(
        &self,
        target: &Expression,
        member: &str,
        checked_arguments: Vec<CheckedExpression>,
    ) -> Result<CheckedExpression, SemanticError> {
        let package_name = match target {
            Expression::Identifier(name) => name.as_str(),
            _ => {
                return Err(SemanticError::new(
                    "selector target must be an imported package name",
                ));
            }
        };
        let imported_package = self.imports.lookup(package_name).ok_or_else(|| {
            SemanticError::new(format!("package `{package_name}` is not imported"))
        })?;
        let package_function =
            resolve_package_function(imported_package, member).ok_or_else(|| {
                SemanticError::new(format!(
                    "package `{}` does not export supported member `{member}`",
                    imported_package.binding_name()
                ))
            })?;
        let checked_arguments = coerce_package_call_arguments(package_function, checked_arguments)?;
        let argument_types = checked_arguments
            .iter()
            .map(|argument| argument.ty.clone())
            .collect::<Vec<_>>();
        let return_type =
            validate_package_call(package_function, &argument_types).map_err(SemanticError::new)?;
        Ok(CheckedExpression {
            ty: return_type,
            kind: CheckedExpressionKind::Call {
                target: CallTarget::PackageFunction(package_function),
                arguments: checked_arguments,
            },
        })
    }

    fn analyze_conversion_expression(
        &mut self,
        type_ref: &TypeRef,
        value: &Expression,
    ) -> Result<CheckedExpression, SemanticError> {
        let target_type = resolve_type_ref(type_ref).ok_or_else(|| {
            SemanticError::new(format!(
                "conversion does not support target type `{}`",
                type_ref.render()
            ))
        })?;
        let value = self.analyze_expression(value)?;
        let conversion = match (&target_type, &value.ty) {
            (Type::Slice(element), Type::String) if element.as_ref() == &Type::Byte => {
                ConversionKind::StringToBytes
            }
            (Type::String, source) if source.is_byte_slice() => ConversionKind::BytesToString,
            (Type::Slice(element), _) if element.as_ref() == &Type::Byte => {
                return Err(SemanticError::new(format!(
                    "conversion to `[]byte` requires `string`, found `{}`",
                    value.ty.render()
                )));
            }
            (Type::String, _) => {
                return Err(SemanticError::new(format!(
                    "conversion to `string` requires `[]byte`, found `{}`",
                    value.ty.render()
                )));
            }
            _ => {
                return Err(SemanticError::new(format!(
                    "conversion to `{}` is not supported",
                    target_type.render()
                )));
            }
        };
        Ok(CheckedExpression {
            ty: target_type,
            kind: CheckedExpressionKind::Conversion {
                conversion,
                value: Box::new(value),
            },
        })
    }

    fn analyze_slice_literal_expression(
        &mut self,
        element_type: &TypeRef,
        elements: &[Expression],
    ) -> Result<CheckedExpression, SemanticError> {
        let slice_type = resolve_type_ref(element_type).ok_or_else(|| {
            SemanticError::new(format!(
                "unsupported slice literal type `{}`",
                element_type.render()
            ))
        })?;
        validate_runtime_type(&slice_type, "slice literal type")?;
        let element_type = slice_type.slice_element_type().cloned().ok_or_else(|| {
            SemanticError::new(format!(
                "slice literal requires `[]T` type syntax, found `{}`",
                element_type.render()
            ))
        })?;
        let checked_elements = elements
            .iter()
            .enumerate()
            .map(|(index, element)| {
                let checked = self.analyze_expression(element)?;
                coerce_expression_to_type(
                    &element_type,
                    checked,
                    &format!("slice literal element {}", index + 1),
                )
            })
            .collect::<Result<Vec<_>, SemanticError>>()?;
        Ok(CheckedExpression {
            ty: slice_type,
            kind: CheckedExpressionKind::SliceLiteral {
                elements: checked_elements,
            },
        })
    }

    fn analyze_map_literal_expression(
        &mut self,
        map_type_ref: &TypeRef,
        entries: &[crate::frontend::ast::MapLiteralEntry],
    ) -> Result<CheckedExpression, SemanticError> {
        let map_type = resolve_type_ref(map_type_ref).ok_or_else(|| {
            SemanticError::new(format!(
                "unsupported map literal type `{}`",
                map_type_ref.render()
            ))
        })?;
        validate_runtime_type(&map_type, "map literal type")?;
        let (key_type, value_type) = map_type.map_parts().ok_or_else(|| {
            SemanticError::new(format!(
                "map literal requires `map[K]V` type syntax, found `{}`",
                map_type_ref.render()
            ))
        })?;
        let mut constant_keys = HashSet::new();
        let checked_entries = entries
            .iter()
            .enumerate()
            .map(|(index, entry)| {
                let key = self.analyze_expression(&entry.key)?;
                expect_type(key_type, &key.ty, &format!("map literal key {}", index + 1))?;
                if let Some(constant_key) = constant_map_key(&key) {
                    if !constant_keys.insert(constant_key.clone()) {
                        return Err(SemanticError::new(format!(
                            "map literal contains duplicate constant key {}",
                            constant_key.render()
                        )));
                    }
                }
                let value = coerce_expression_to_type(
                    value_type,
                    self.analyze_expression(&entry.value)?,
                    &format!("map literal value {}", index + 1),
                )?;
                Ok(CheckedMapLiteralEntry { key, value })
            })
            .collect::<Result<Vec<_>, SemanticError>>()?;
        Ok(CheckedExpression {
            ty: map_type,
            kind: CheckedExpressionKind::MapLiteral {
                entries: checked_entries,
            },
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum ConstantMapKey {
    Integer(i64),
    Bool(bool),
    String(String),
}

impl ConstantMapKey {
    fn render(&self) -> String {
        match self {
            ConstantMapKey::Integer(value) => value.to_string(),
            ConstantMapKey::Bool(value) => value.to_string(),
            ConstantMapKey::String(value) => format!("{value:?}"),
        }
    }
}

fn constant_map_key(expression: &CheckedExpression) -> Option<ConstantMapKey> {
    match &expression.kind {
        CheckedExpressionKind::Integer(value) => Some(ConstantMapKey::Integer(*value)),
        CheckedExpressionKind::Bool(value) => Some(ConstantMapKey::Bool(*value)),
        CheckedExpressionKind::String(value) => Some(ConstantMapKey::String(value.clone())),
        _ => None,
    }
}

fn analyze_binary_operator(
    operator: BinaryOperator,
    left: CheckedExpression,
    right: CheckedExpression,
) -> Result<
    (
        CheckedBinaryOperator,
        Type,
        CheckedExpression,
        CheckedExpression,
    ),
    SemanticError,
> {
    match operator {
        BinaryOperator::Add if left.ty == Type::Int && right.ty == Type::Int => {
            Ok((CheckedBinaryOperator::Add, Type::Int, left, right))
        }
        BinaryOperator::Add if left.ty == Type::String && right.ty == Type::String => {
            Ok((CheckedBinaryOperator::Concat, Type::String, left, right))
        }
        BinaryOperator::Add => Err(SemanticError::new(format!(
            "addition requires matching `int` or `string` operands, found `{}` and `{}`",
            left.ty.render(),
            right.ty.render()
        ))),
        BinaryOperator::Subtract | BinaryOperator::Multiply | BinaryOperator::Divide => {
            expect_type(&Type::Int, &left.ty, "left side of arithmetic expression")?;
            expect_type(&Type::Int, &right.ty, "right side of arithmetic expression")?;
            Ok((
                match operator {
                    BinaryOperator::Subtract => CheckedBinaryOperator::Subtract,
                    BinaryOperator::Multiply => CheckedBinaryOperator::Multiply,
                    BinaryOperator::Divide => CheckedBinaryOperator::Divide,
                    _ => unreachable!("non-add arithmetic operators already matched"),
                },
                Type::Int,
                left,
                right,
            ))
        }
        BinaryOperator::Equal | BinaryOperator::NotEqual => {
            let (left, right) = coerce_nil_equality_operands(left, right)?;
            Ok((
                match operator {
                    BinaryOperator::Equal => CheckedBinaryOperator::Equal,
                    BinaryOperator::NotEqual => CheckedBinaryOperator::NotEqual,
                    _ => unreachable!("equality operators already matched"),
                },
                Type::Bool,
                left,
                right,
            ))
        }
        BinaryOperator::Less
        | BinaryOperator::LessEqual
        | BinaryOperator::Greater
        | BinaryOperator::GreaterEqual => {
            expect_type(&Type::Int, &left.ty, "left side of comparison expression")?;
            expect_type(&Type::Int, &right.ty, "right side of comparison expression")?;
            Ok((
                match operator {
                    BinaryOperator::Less => CheckedBinaryOperator::Less,
                    BinaryOperator::LessEqual => CheckedBinaryOperator::LessEqual,
                    BinaryOperator::Greater => CheckedBinaryOperator::Greater,
                    BinaryOperator::GreaterEqual => CheckedBinaryOperator::GreaterEqual,
                    _ => unreachable!("comparison operators already matched"),
                },
                Type::Bool,
                left,
                right,
            ))
        }
    }
}

fn coerce_package_call_arguments(
    function: crate::package::PackageFunction,
    checked_arguments: Vec<CheckedExpression>,
) -> Result<Vec<CheckedExpression>, SemanticError> {
    let Some(expected_arguments) = crate::semantic::packages::expected_argument_types(function)
    else {
        return Ok(checked_arguments);
    };
    if checked_arguments.len() != expected_arguments.len() {
        return Ok(checked_arguments);
    }
    checked_arguments
        .into_iter()
        .zip(expected_arguments.iter())
        .enumerate()
        .map(|(index, (argument, expected))| {
            coerce_expression_to_type(
                expected,
                argument,
                &format!("argument {} in call to `{}`", index + 1, function.render()),
            )
        })
        .collect()
}
