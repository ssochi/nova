use std::collections::HashSet;

use crate::frontend::ast::{Block, Expression, RangeBinding, RangeBindingMode};
use crate::semantic::analyzer::{FunctionAnalyzer, SemanticError};
use crate::semantic::model::{CheckedRangeBinding, CheckedStatement, Type};

impl<'a> FunctionAnalyzer<'a> {
    pub(super) fn analyze_range_statement(
        &mut self,
        bindings: &[RangeBinding],
        binding_mode: Option<RangeBindingMode>,
        target: &Expression,
        body: &Block,
    ) -> Result<CheckedStatement, SemanticError> {
        let source = self.analyze_expression(target)?;
        let (key_type, value_type) = range_value_types(&source.ty)?;

        if bindings.len() > 2 {
            return Err(SemanticError::new(
                "range loop supports at most two iteration variables",
            ));
        }
        if bindings.is_empty() && binding_mode.is_some() {
            return Err(SemanticError::new(
                "range loop without iteration variables must omit assignment syntax",
            ));
        }
        if !bindings.is_empty() && binding_mode.is_none() {
            return Err(SemanticError::new(
                "range loop iteration variables require `=` or `:=`",
            ));
        }

        self.scopes.push(Default::default());
        let bindings =
            self.resolve_range_bindings(bindings, binding_mode, &key_type, &value_type)?;
        let body = self.analyze_block(body, true)?;
        self.scopes.pop();

        let mut bindings = bindings.into_iter();
        Ok(CheckedStatement::RangeFor {
            source,
            key_binding: bindings.next().flatten(),
            value_binding: bindings.next().flatten(),
            body,
        })
    }

    fn resolve_range_bindings(
        &mut self,
        bindings: &[RangeBinding],
        binding_mode: Option<RangeBindingMode>,
        key_type: &Type,
        value_type: &Type,
    ) -> Result<Vec<Option<CheckedRangeBinding>>, SemanticError> {
        if bindings.is_empty() {
            return Ok(Vec::new());
        }

        let binding_mode = binding_mode.expect("non-empty range bindings must carry a mode");
        let mut seen = HashSet::new();
        let mut has_named_define = false;
        let mut resolved = Vec::with_capacity(bindings.len());
        for (index, binding) in bindings.iter().enumerate() {
            let expected_type = if index == 0 { key_type } else { value_type };
            resolved.push(match binding {
                RangeBinding::Blank => None,
                RangeBinding::Identifier(name) => {
                    if !seen.insert(name.clone()) {
                        return Err(SemanticError::new(format!(
                            "range loop variable `{name}` is declared more than once"
                        )));
                    }
                    match binding_mode {
                        RangeBindingMode::Define => {
                            has_named_define = true;
                            let slot = self.allocate_local(name.clone(), expected_type.clone());
                            Some(CheckedRangeBinding::Local {
                                slot,
                                name: name.clone(),
                            })
                        }
                        RangeBindingMode::Assign => {
                            let binding = self.lookup_local(name)?;
                            if binding.ty != *expected_type {
                                return Err(SemanticError::new(format!(
                                    "range loop assignment to `{name}` requires `{}`, found `{}`",
                                    binding.ty.render(),
                                    expected_type.render()
                                )));
                            }
                            Some(CheckedRangeBinding::Local {
                                slot: binding.slot,
                                name: name.clone(),
                            })
                        }
                    }
                }
            });
        }

        if binding_mode == RangeBindingMode::Define && !has_named_define {
            return Err(SemanticError::new(
                "range loop `:=` requires at least one named iteration variable",
            ));
        }

        Ok(resolved
            .into_iter()
            .map(|binding| binding.or(Some(CheckedRangeBinding::Discard)))
            .collect())
    }
}

fn range_value_types(source_type: &Type) -> Result<(Type, Type), SemanticError> {
    match source_type {
        Type::Slice(element) => Ok((Type::Int, element.as_ref().clone())),
        Type::Map { key, value } => Ok((key.as_ref().clone(), value.as_ref().clone())),
        _ => Err(SemanticError::new(format!(
            "range loop requires `slice` or `map` source, found `{}`",
            source_type.render()
        ))),
    }
}
