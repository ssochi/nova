use std::collections::HashSet;

use crate::builtin::BuiltinFunction;
use crate::conversion::ConversionKind;
use crate::frontend::ast::{BinaryOperator, CallArgument, Expression, TypeRef};
use crate::semantic::analyzer::{FunctionAnalyzer, SemanticError};
use crate::semantic::builtins::{
    resolve_builtin, validate_append_spread_call, validate_builtin_call, validate_make_call,
};
use crate::semantic::model::{
    CallTarget, CheckedBinaryOperator, CheckedCall, CheckedCallArguments, CheckedExpression,
    CheckedExpressionKind, CheckedMapLiteralEntry, Type,
};
use crate::semantic::packages::{
    resolve_package_function, validate_package_call, variadic_element_type,
};
use crate::semantic::support::{
    coerce_expression_to_type, coerce_nil_equality_operands, expect_type, render_type_list,
    resolve_type_ref, validate_make_literal_bounds, validate_runtime_type,
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
            Expression::Receive { channel } => {
                let channel = self.analyze_expression(channel)?;
                let element_type = channel.ty.channel_element_type().cloned().ok_or_else(|| {
                    SemanticError::new(format!(
                        "receive expression requires `chan` target, found `{}`",
                        channel.ty.render()
                    ))
                })?;
                Ok(CheckedExpression {
                    ty: element_type,
                    kind: CheckedExpressionKind::Receive {
                        channel: Box::new(channel),
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
                self.analyze_call_expression(callee, arguments)
            }
        }
    }

    pub(super) fn try_analyze_multi_result_call(
        &mut self,
        expression: &Expression,
    ) -> Result<Option<CheckedCall>, SemanticError> {
        let Expression::Call { callee, arguments } = expression else {
            return Ok(None);
        };
        if arguments
            .iter()
            .any(|argument| matches!(argument, CallArgument::Spread(_)))
        {
            return Ok(None);
        }
        let call = self.analyze_call(callee, arguments)?;
        if call.result_types.len() > 1 {
            Ok(Some(call))
        } else {
            Ok(None)
        }
    }

    fn analyze_call_expression(
        &mut self,
        callee: &Expression,
        arguments: &[CallArgument],
    ) -> Result<CheckedExpression, SemanticError> {
        let call = self.analyze_call(callee, arguments)?;
        if call.result_types.len() != 1 {
            return Err(SemanticError::new(format!(
                "call to `{}` produces `{}` and cannot be used in a single-value expression",
                render_call_target(&call.target),
                render_type_list(&call.result_types)
            )));
        }
        Ok(CheckedExpression {
            ty: call.result_types[0].clone(),
            kind: CheckedExpressionKind::Call(call),
        })
    }

    pub(super) fn analyze_call(
        &mut self,
        callee: &Expression,
        arguments: &[CallArgument],
    ) -> Result<CheckedCall, SemanticError> {
        let checked_arguments = self.analyze_call_arguments(arguments)?;
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
        &mut self,
        callee: &str,
        checked_arguments: CheckedCallArguments,
    ) -> Result<CheckedCall, SemanticError> {
        if let Some(builtin) = resolve_builtin(callee) {
            return self.analyze_builtin_call(builtin, checked_arguments);
        }

        let function_index = self
            .registry
            .lookup(callee)
            .ok_or_else(|| SemanticError::new(format!("unknown function `{callee}`")))?;
        let signature = self.registry.signature(function_index);
        let checked_arguments = self.coerce_user_defined_call_arguments(
            callee,
            &signature.parameters,
            signature.variadic_element_type.as_ref(),
            checked_arguments,
        )?;

        Ok(CheckedCall {
            target: CallTarget::UserDefined {
                function_index,
                name: signature.name.clone(),
            },
            arguments: checked_arguments,
            result_types: signature.return_types.clone(),
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
            Type::Chan(element_type) => Ok(CheckedExpression {
                ty: result_type.clone(),
                kind: CheckedExpressionKind::MakeChan {
                    element_type: element_type.as_ref().clone(),
                    buffer: checked_arguments.into_iter().next().map(Box::new),
                },
            }),
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
        &mut self,
        target: &Expression,
        member: &str,
        checked_arguments: CheckedCallArguments,
    ) -> Result<CheckedCall, SemanticError> {
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
                    package_name
                ))
            })?;
        let checked_arguments =
            self.coerce_package_call_arguments(package_function, checked_arguments)?;
        let argument_types = checked_call_argument_types(&checked_arguments);
        let result_types =
            validate_package_call(package_function, &argument_types).map_err(SemanticError::new)?;
        Ok(CheckedCall {
            target: CallTarget::PackageFunction(package_function),
            arguments: checked_arguments,
            result_types,
        })
    }

    fn analyze_call_arguments(
        &mut self,
        arguments: &[CallArgument],
    ) -> Result<CheckedCallArguments, SemanticError> {
        if let [CallArgument::Expression(argument)] = arguments {
            if let Some(call) = self.try_analyze_multi_result_call(argument)? {
                return Ok(CheckedCallArguments::ExpandedCall(Box::new(call)));
            }
        }

        if let Some(CallArgument::Spread(spread)) = arguments.last() {
            let checked_arguments = arguments[..arguments.len() - 1]
                .iter()
                .map(|argument| match argument {
                    CallArgument::Expression(argument) => self.analyze_expression(argument),
                    CallArgument::Spread(_) => Err(SemanticError::new(
                        "spread argument must be the final call argument",
                    )),
                })
                .collect::<Result<Vec<_>, _>>()?;
            let spread = self.analyze_expression(spread)?;
            return Ok(CheckedCallArguments::Spread {
                arguments: checked_arguments,
                spread: Box::new(spread),
            });
        }

        let checked_arguments = arguments
            .iter()
            .map(|argument| match argument {
                CallArgument::Expression(argument) => self.analyze_expression(argument),
                CallArgument::Spread(_) => Err(SemanticError::new(
                    "spread argument must be the final call argument",
                )),
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(CheckedCallArguments::Expressions(checked_arguments))
    }

    fn coerce_user_defined_call_arguments(
        &mut self,
        callee: &str,
        expected_arguments: &[Type],
        variadic_element_type: Option<&Type>,
        checked_arguments: CheckedCallArguments,
    ) -> Result<CheckedCallArguments, SemanticError> {
        match checked_arguments {
            CheckedCallArguments::Expressions(arguments) => {
                if let Some(variadic_element_type) = variadic_element_type {
                    let fixed_count = expected_arguments.len() - 1;
                    if arguments.len() < fixed_count {
                        return Err(SemanticError::new(format!(
                            "function `{callee}` expects at least {} arguments, found {}",
                            fixed_count,
                            arguments.len()
                        )));
                    }
                    let mut coerced_arguments = Vec::with_capacity(arguments.len());
                    for (index, (argument, expected)) in arguments
                        .iter()
                        .take(fixed_count)
                        .cloned()
                        .zip(expected_arguments.iter().take(fixed_count))
                        .enumerate()
                    {
                        coerced_arguments.push(coerce_expression_to_type(
                            expected,
                            argument,
                            &format!("argument {} in call to `{callee}`", index + 1),
                        )?);
                    }
                    for (index, argument) in arguments.into_iter().enumerate().skip(fixed_count) {
                        coerced_arguments.push(coerce_expression_to_type(
                            variadic_element_type,
                            argument,
                            &format!("argument {} in call to `{callee}`", index + 1),
                        )?);
                    }
                    return Ok(CheckedCallArguments::Expressions(coerced_arguments));
                }
                if arguments.len() != expected_arguments.len() {
                    return Err(SemanticError::new(format!(
                        "function `{callee}` expects {} arguments, found {}",
                        expected_arguments.len(),
                        arguments.len()
                    )));
                }
                let arguments = arguments
                    .into_iter()
                    .zip(expected_arguments.iter())
                    .enumerate()
                    .map(|(index, (argument, expected))| {
                        coerce_expression_to_type(
                            expected,
                            argument,
                            &format!("argument {} in call to `{callee}`", index + 1),
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(CheckedCallArguments::Expressions(arguments))
            }
            CheckedCallArguments::ExpandedCall(call) => {
                if let Some(variadic_element_type) = variadic_element_type {
                    let fixed_count = expected_arguments.len() - 1;
                    if call.result_types.len() < fixed_count {
                        return Err(SemanticError::new(format!(
                            "function `{callee}` expects at least {} arguments, found {}",
                            fixed_count,
                            call.result_types.len()
                        )));
                    }
                    for (index, (actual, expected)) in call
                        .result_types
                        .iter()
                        .take(fixed_count)
                        .zip(expected_arguments.iter().take(fixed_count))
                        .enumerate()
                    {
                        expect_type(
                            expected,
                            actual,
                            &format!("argument {} in call to `{callee}`", index + 1),
                        )?;
                    }
                    for (index, actual) in call.result_types.iter().enumerate().skip(fixed_count) {
                        expect_type(
                            variadic_element_type,
                            actual,
                            &format!("argument {} in call to `{callee}`", index + 1),
                        )?;
                    }
                    return Ok(CheckedCallArguments::ExpandedCall(call));
                }
                if call.result_types.len() != expected_arguments.len() {
                    return Err(SemanticError::new(format!(
                        "function `{callee}` expects {} arguments, found {}",
                        expected_arguments.len(),
                        call.result_types.len()
                    )));
                }
                for (index, (actual, expected)) in call
                    .result_types
                    .iter()
                    .zip(expected_arguments.iter())
                    .enumerate()
                {
                    expect_type(
                        expected,
                        actual,
                        &format!("argument {} in call to `{callee}`", index + 1),
                    )?;
                }
                Ok(CheckedCallArguments::ExpandedCall(call))
            }
            CheckedCallArguments::Spread { arguments, spread } => {
                let Some(variadic_element_type) = variadic_element_type else {
                    return Err(SemanticError::new(format!(
                        "function `{callee}` does not support explicit `...` arguments"
                    )));
                };
                let fixed_count = expected_arguments.len() - 1;
                if arguments.len() != fixed_count {
                    return Err(SemanticError::new(format!(
                        "function `{callee}` with `...` requires {} fixed arguments before the spread value, found {}",
                        fixed_count,
                        arguments.len()
                    )));
                }
                let arguments = arguments
                    .into_iter()
                    .zip(expected_arguments.iter().take(fixed_count))
                    .enumerate()
                    .map(|(index, (argument, expected))| {
                        coerce_expression_to_type(
                            expected,
                            argument,
                            &format!("argument {} in call to `{callee}`", index + 1),
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let spread = coerce_expression_to_type(
                    &Type::Slice(Box::new(variadic_element_type.clone())),
                    *spread,
                    &format!("spread argument in call to `{callee}`"),
                )?;
                Ok(CheckedCallArguments::Spread {
                    arguments,
                    spread: Box::new(spread),
                })
            }
        }
    }

    fn coerce_package_call_arguments(
        &mut self,
        function: crate::package::PackageFunction,
        checked_arguments: CheckedCallArguments,
    ) -> Result<CheckedCallArguments, SemanticError> {
        if let Some(variadic_element_type) = variadic_element_type(function) {
            return match checked_arguments {
                CheckedCallArguments::Expressions(arguments) => {
                    Ok(CheckedCallArguments::Expressions(arguments))
                }
                CheckedCallArguments::ExpandedCall(call) => {
                    Ok(CheckedCallArguments::ExpandedCall(call))
                }
                CheckedCallArguments::Spread { arguments, spread } => {
                    if !arguments.is_empty() {
                        return Err(SemanticError::new(format!(
                            "package function `{}` with `...` requires 0 fixed arguments before the spread value, found {}",
                            function.render(),
                            arguments.len()
                        )));
                    }
                    let spread = coerce_expression_to_type(
                        &Type::Slice(Box::new(variadic_element_type)),
                        *spread,
                        &format!("spread argument in call to `{}`", function.render()),
                    )?;
                    Ok(CheckedCallArguments::Spread {
                        arguments,
                        spread: Box::new(spread),
                    })
                }
            };
        }
        let Some(expected_arguments) = crate::semantic::packages::expected_argument_types(function)
        else {
            return match checked_arguments {
                CheckedCallArguments::Spread { .. } => Err(SemanticError::new(format!(
                    "package function `{}` does not support explicit `...` arguments in the current subset",
                    function.render()
                ))),
                _ => Ok(checked_arguments),
            };
        };
        match checked_arguments {
            CheckedCallArguments::Expressions(arguments) => {
                if arguments.len() != expected_arguments.len() {
                    return Ok(CheckedCallArguments::Expressions(arguments));
                }
                let arguments = arguments
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
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(CheckedCallArguments::Expressions(arguments))
            }
            CheckedCallArguments::ExpandedCall(call) => {
                Ok(CheckedCallArguments::ExpandedCall(call))
            }
            CheckedCallArguments::Spread { .. } => Err(SemanticError::new(format!(
                "package function `{}` does not support explicit `...` arguments in the current subset",
                function.render()
            ))),
        }
    }

    fn analyze_builtin_call(
        &mut self,
        builtin: BuiltinFunction,
        checked_arguments: CheckedCallArguments,
    ) -> Result<CheckedCall, SemanticError> {
        let result_types = match &checked_arguments {
            CheckedCallArguments::Expressions(_) | CheckedCallArguments::ExpandedCall(_) => {
                validate_builtin_call(builtin, &checked_call_argument_types(&checked_arguments))
                    .map_err(SemanticError::new)?
            }
            CheckedCallArguments::Spread { arguments, spread } => {
                if builtin != BuiltinFunction::Append {
                    return Err(SemanticError::new(format!(
                        "builtin `{}` does not support explicit `...` arguments",
                        builtin.render()
                    )));
                }
                validate_append_spread_call(
                    &arguments
                        .iter()
                        .map(|argument| argument.ty.clone())
                        .collect::<Vec<_>>(),
                    &coerce_append_spread_argument(arguments, spread.as_ref())?.ty,
                )
                .map_err(SemanticError::new)?
            }
        };

        if let CheckedCallArguments::Spread { arguments, spread } = checked_arguments {
            let spread = coerce_append_spread_argument(&arguments, spread.as_ref())?;
            return Ok(CheckedCall {
                target: CallTarget::Builtin(builtin),
                arguments: CheckedCallArguments::Spread {
                    arguments,
                    spread: Box::new(spread),
                },
                result_types,
            });
        }

        Ok(CheckedCall {
            target: CallTarget::Builtin(builtin),
            arguments: checked_arguments,
            result_types,
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
        if target_type == Type::Any {
            return coerce_expression_to_type(&Type::Any, value, "conversion to `any`");
        }
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

fn checked_call_argument_types(arguments: &CheckedCallArguments) -> Vec<Type> {
    match arguments {
        CheckedCallArguments::Expressions(arguments) => arguments
            .iter()
            .map(|argument| argument.ty.clone())
            .collect(),
        CheckedCallArguments::ExpandedCall(call) => call.result_types.clone(),
        CheckedCallArguments::Spread { arguments, spread } => arguments
            .iter()
            .map(|argument| argument.ty.clone())
            .chain(std::iter::once(spread.ty.clone()))
            .collect(),
    }
}

fn coerce_append_spread_argument(
    arguments: &[CheckedExpression],
    spread: &CheckedExpression,
) -> Result<CheckedExpression, SemanticError> {
    let Some(target) = arguments.first() else {
        return Err(SemanticError::new(
            "builtin `append` with `...` requires a destination slice argument",
        ));
    };
    let Some(element_type) = target.ty.slice_element_type().cloned() else {
        return Err(SemanticError::new(format!(
            "argument 1 in call to builtin `append` requires `slice`, found `{}`",
            target.ty.render()
        )));
    };
    if element_type == Type::Byte && spread.ty == Type::String {
        return Ok(spread.clone());
    }
    coerce_expression_to_type(
        &Type::Slice(Box::new(element_type)),
        spread.clone(),
        "spread argument in call to builtin `append`",
    )
}

fn render_call_target(target: &CallTarget) -> String {
    match target {
        CallTarget::Builtin(builtin) => builtin.render().to_string(),
        CallTarget::PackageFunction(function) => function.render().to_string(),
        CallTarget::UserDefined { name, .. } => name.clone(),
    }
}
