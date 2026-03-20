use crate::conversion::ConversionKind;
use std::collections::HashSet;

use crate::frontend::ast::{Binding, BindingMode, Expression, TypeRef};
use crate::semantic::analyzer::{FunctionAnalyzer, SemanticError};
use crate::semantic::model::{
    CheckedBinding, CheckedExpression, CheckedExpressionKind, CheckedHeaderStatement,
    CheckedStatement, Type,
};
use crate::semantic::support::{
    coerce_expression_to_type, resolve_type_ref, validate_runtime_type,
};

impl<'a> FunctionAnalyzer<'a> {
    pub(super) fn analyze_conversion_expression(
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

    pub(super) fn analyze_type_assertion_expression(
        &mut self,
        target: &Expression,
        asserted_type_ref: &TypeRef,
    ) -> Result<CheckedExpression, SemanticError> {
        let value = self.analyze_expression(target)?;
        if value.ty != Type::Any {
            return Err(SemanticError::new(format!(
                "type assertion requires interface operand, found `{}`",
                value.ty.render()
            )));
        }

        let asserted_type = resolve_type_ref(asserted_type_ref).ok_or_else(|| {
            SemanticError::new(format!(
                "type assertion does not support target type `{}`",
                asserted_type_ref.render()
            ))
        })?;
        validate_runtime_type(&asserted_type, "type assertion target")?;

        Ok(CheckedExpression {
            ty: asserted_type.clone(),
            kind: CheckedExpressionKind::TypeAssertion {
                value: Box::new(value),
                asserted_type,
            },
        })
    }

    pub(super) fn analyze_type_assert_initializer(
        &mut self,
        bindings: &[Binding],
        binding_mode: BindingMode,
        target: &Expression,
        asserted_type_ref: &TypeRef,
    ) -> Result<CheckedHeaderStatement, SemanticError> {
        let statement =
            self.analyze_type_assert_statement(bindings, binding_mode, target, asserted_type_ref)?;
        let CheckedStatement::TypeAssert {
            interface,
            asserted_type,
            value_binding,
            ok_binding,
        } = statement
        else {
            unreachable!("type-assert analysis always returns checked type-assert data");
        };
        Ok(CheckedHeaderStatement::TypeAssert {
            interface,
            asserted_type,
            value_binding,
            ok_binding,
        })
    }

    pub(super) fn analyze_type_assert_statement(
        &mut self,
        bindings: &[Binding],
        binding_mode: BindingMode,
        target: &Expression,
        asserted_type_ref: &TypeRef,
    ) -> Result<CheckedStatement, SemanticError> {
        if bindings.len() != 2 {
            return Err(SemanticError::new(
                "comma-ok type assertion requires exactly two left-hand-side bindings",
            ));
        }

        let asserted = self.analyze_type_assertion_expression(target, asserted_type_ref)?;
        let value_type = asserted.ty.clone();
        let CheckedExpressionKind::TypeAssertion {
            value,
            asserted_type,
        } = asserted.kind
        else {
            unreachable!("type assertion analysis always returns checked assertion data");
        };

        let mut resolved =
            self.resolve_type_assert_bindings(bindings, binding_mode, &value_type)?;
        let ok_binding = resolved
            .pop()
            .expect("type-assert binding resolution returns two entries");
        let value_binding = resolved
            .pop()
            .expect("type-assert binding resolution returns two entries");
        Ok(CheckedStatement::TypeAssert {
            interface: *value,
            asserted_type,
            value_binding,
            ok_binding,
        })
    }

    fn resolve_type_assert_bindings(
        &mut self,
        bindings: &[Binding],
        binding_mode: BindingMode,
        asserted_type: &Type,
    ) -> Result<Vec<CheckedBinding>, SemanticError> {
        let expected_types = [asserted_type.clone(), Type::Bool];
        let mut seen = HashSet::new();
        let mut has_new_named_binding = false;
        let mut resolved = Vec::with_capacity(bindings.len());

        for (binding, expected_type) in bindings.iter().zip(expected_types.iter()) {
            resolved.push(match binding {
                Binding::Blank => CheckedBinding::Discard,
                Binding::Identifier(name) => {
                    if !seen.insert(name.clone()) {
                        return Err(SemanticError::new(format!(
                            "comma-ok type assertion variable `{name}` is declared more than once"
                        )));
                    }

                    match binding_mode {
                        BindingMode::Assign => {
                            let binding = self.lookup_local(name)?;
                            if binding.ty != *expected_type {
                                return Err(SemanticError::new(format!(
                                    "comma-ok type assertion assignment to `{name}` requires `{}`, found `{}`",
                                    binding.ty.render(),
                                    expected_type.render()
                                )));
                            }
                            CheckedBinding::Local {
                                slot: binding.slot,
                                name: name.clone(),
                            }
                        }
                        BindingMode::Define => {
                            if let Some(binding) = self.current_scope().get(name).cloned() {
                                if binding.ty != *expected_type {
                                    return Err(SemanticError::new(format!(
                                        "comma-ok type assertion redeclaration of `{name}` requires `{}`, found `{}`",
                                        binding.ty.render(),
                                        expected_type.render()
                                    )));
                                }
                                CheckedBinding::Local {
                                    slot: binding.slot,
                                    name: name.clone(),
                                }
                            } else {
                                has_new_named_binding = true;
                                let slot =
                                    self.allocate_local(name.clone(), expected_type.clone());
                                CheckedBinding::Local {
                                    slot,
                                    name: name.clone(),
                                }
                            }
                        }
                    }
                }
            });
        }

        if binding_mode == BindingMode::Define && !has_new_named_binding {
            return Err(SemanticError::new(
                "comma-ok type assertion `:=` requires at least one new named variable",
            ));
        }

        Ok(resolved)
    }
}
