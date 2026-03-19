use std::collections::HashSet;

use crate::frontend::ast::{Binding, BindingMode, Expression};
use crate::semantic::analyzer::{FunctionAnalyzer, SemanticError};
use crate::semantic::model::{CheckedBinding, CheckedHeaderStatement, CheckedStatement, Type};
use crate::semantic::support::expect_type;

impl<'a> FunctionAnalyzer<'a> {
    pub(super) fn analyze_map_lookup_initializer(
        &mut self,
        bindings: &[Binding],
        binding_mode: BindingMode,
        target: &Expression,
        key: &Expression,
    ) -> Result<CheckedHeaderStatement, SemanticError> {
        let statement = self.analyze_map_lookup_statement(bindings, binding_mode, target, key)?;
        let CheckedStatement::MapLookup {
            map,
            key,
            value_binding,
            ok_binding,
        } = statement
        else {
            unreachable!("map lookup analysis always returns a map lookup statement");
        };
        Ok(CheckedHeaderStatement::MapLookup {
            map,
            key,
            value_binding,
            ok_binding,
        })
    }

    pub(super) fn analyze_map_lookup_statement(
        &mut self,
        bindings: &[Binding],
        binding_mode: BindingMode,
        target: &Expression,
        key: &Expression,
    ) -> Result<CheckedStatement, SemanticError> {
        if bindings.len() != 2 {
            return Err(SemanticError::new(
                "comma-ok lookup requires exactly two left-hand-side bindings",
            ));
        }

        let map = self.analyze_expression(target)?;
        let key = self.analyze_expression(key)?;
        let value_type = match &map.ty {
            Type::Map {
                key: key_type,
                value,
            } => {
                expect_type(key_type.as_ref(), &key.ty, "comma-ok map lookup key")?;
                value.as_ref().clone()
            }
            _ => {
                return Err(SemanticError::new(format!(
                    "comma-ok lookup requires `map` target, found `{}`",
                    map.ty.render()
                )));
            }
        };

        let mut resolved = self.resolve_map_lookup_bindings(bindings, binding_mode, &value_type)?;
        let ok_binding = resolved
            .pop()
            .expect("map lookup binding resolution returns two entries");
        let value_binding = resolved
            .pop()
            .expect("map lookup binding resolution returns two entries");
        Ok(CheckedStatement::MapLookup {
            map,
            key,
            value_binding,
            ok_binding,
        })
    }

    fn resolve_map_lookup_bindings(
        &mut self,
        bindings: &[Binding],
        binding_mode: BindingMode,
        value_type: &Type,
    ) -> Result<Vec<CheckedBinding>, SemanticError> {
        let expected_types = [value_type.clone(), Type::Bool];
        let mut seen = HashSet::new();
        let mut has_new_named_binding = false;
        let mut resolved = Vec::with_capacity(bindings.len());

        for (binding, expected_type) in bindings.iter().zip(expected_types.iter()) {
            resolved.push(match binding {
                Binding::Blank => CheckedBinding::Discard,
                Binding::Identifier(name) => {
                    if !seen.insert(name.clone()) {
                        return Err(SemanticError::new(format!(
                            "comma-ok lookup variable `{name}` is declared more than once"
                        )));
                    }

                    match binding_mode {
                        BindingMode::Assign => {
                            let binding = self.lookup_local(name)?;
                            if binding.ty != *expected_type {
                                return Err(SemanticError::new(format!(
                                    "comma-ok lookup assignment to `{name}` requires `{}`, found `{}`",
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
                                        "comma-ok lookup redeclaration of `{name}` requires `{}`, found `{}`",
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
                                let slot = self.allocate_local(name.clone(), expected_type.clone());
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
                "comma-ok lookup `:=` requires at least one new named variable",
            ));
        }

        Ok(resolved)
    }
}
