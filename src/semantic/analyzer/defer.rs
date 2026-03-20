use crate::frontend::ast::Expression;
use crate::semantic::analyzer::{FunctionAnalyzer, SemanticError};
use crate::semantic::builtins::builtin_permits_statement_context;
use crate::semantic::model::{CallTarget, CheckedStatement};

impl<'a> FunctionAnalyzer<'a> {
    pub(super) fn analyze_defer_statement(
        &mut self,
        expression: &Expression,
    ) -> Result<CheckedStatement, SemanticError> {
        let Expression::Call { callee, arguments } = expression else {
            return Err(SemanticError::new("defer requires a function call"));
        };

        let call = self.analyze_call(callee, arguments)?;
        if let CallTarget::Builtin(builtin) = call.target {
            if !builtin_permits_statement_context(builtin) {
                return Err(SemanticError::new(format!(
                    "builtin `{}` is not permitted in defer statement context",
                    builtin.render()
                )));
            }
        }

        Ok(CheckedStatement::Defer(call))
    }
}
