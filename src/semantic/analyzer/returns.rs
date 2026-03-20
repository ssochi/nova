use crate::semantic::analyzer::{FunctionAnalyzer, SemanticError};
use crate::semantic::model::{
    CheckedExpression, CheckedExpressionKind, CheckedStatement, CheckedValueSource,
};
use crate::semantic::support::render_type_list;

impl<'a> FunctionAnalyzer<'a> {
    pub(super) fn analyze_return_statement(
        &mut self,
        values: &[crate::frontend::ast::Expression],
    ) -> Result<CheckedStatement, SemanticError> {
        let expected = self
            .registry
            .signature(self.function_index)
            .return_types
            .clone();
        if expected.is_empty() && values.is_empty() {
            return Ok(CheckedStatement::Return(CheckedValueSource::Expressions(
                Vec::new(),
            )));
        }
        if expected.is_empty() {
            return Err(SemanticError::new(format!(
                "function `{}` does not return a value",
                self.function.name
            )));
        }
        if values.is_empty() {
            if self.result_locals.is_empty() {
                return Err(SemanticError::new(format!(
                    "function `{}` must return a `{}` value",
                    self.function.name,
                    render_type_list(&expected)
                )));
            }
            self.ensure_named_results_visible_for_bare_return()?;
            let values = self
                .result_locals
                .iter()
                .map(|result| CheckedExpression {
                    ty: result.ty.clone(),
                    kind: CheckedExpressionKind::Local {
                        slot: result.slot,
                        name: result.local_name.clone(),
                    },
                })
                .collect();
            return Ok(CheckedStatement::Return(CheckedValueSource::Expressions(
                values,
            )));
        }

        let values = self.analyze_typed_value_source(values, &expected, "return statement")?;
        Ok(CheckedStatement::Return(values))
    }

    fn ensure_named_results_visible_for_bare_return(&self) -> Result<(), SemanticError> {
        for result in &self.result_locals {
            let Some(name) = result.name.as_deref() else {
                continue;
            };
            let binding = self.lookup_local(name).map_err(|_| {
                SemanticError::new(format!("result parameter `{name}` not in scope at return"))
            })?;
            if binding.slot != result.slot {
                return Err(SemanticError::new(format!(
                    "result parameter `{name}` not in scope at return"
                )));
            }
        }
        Ok(())
    }
}
