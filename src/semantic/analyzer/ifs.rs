use crate::frontend::ast::{ElseBranch, IfInitializer, IfStatement};
use crate::semantic::analyzer::{FunctionAnalyzer, SemanticError};
use crate::semantic::model::{
    CheckedElseBranch, CheckedIfInitializer, CheckedIfStatement, CheckedStatement, Type,
};
use crate::semantic::support::expect_type;

impl<'a> FunctionAnalyzer<'a> {
    pub(super) fn analyze_if_statement(
        &mut self,
        if_statement: &IfStatement,
    ) -> Result<CheckedStatement, SemanticError> {
        self.scopes.push(Default::default());
        let initializer = if_statement
            .initializer
            .as_ref()
            .map(|initializer| self.analyze_if_initializer(initializer))
            .transpose()?;
        let condition = self.analyze_expression(&if_statement.condition)?;
        expect_type(&Type::Bool, &condition.ty, "if condition")?;
        let then_block = self.analyze_block(&if_statement.then_block, true)?;
        let else_branch = if_statement
            .else_branch
            .as_ref()
            .map(|else_branch| self.analyze_else_branch(else_branch))
            .transpose()?;
        self.scopes.pop();

        Ok(CheckedStatement::If(CheckedIfStatement {
            initializer,
            condition,
            then_block,
            else_branch,
        }))
    }

    fn analyze_if_initializer(
        &mut self,
        initializer: &IfInitializer,
    ) -> Result<CheckedIfInitializer, SemanticError> {
        match initializer {
            IfInitializer::VarDecl {
                name,
                type_ref,
                value,
            } => checked_statement_to_if_initializer(self.analyze_var_decl_statement(
                name,
                type_ref.as_ref(),
                value.as_ref(),
            )?),
            IfInitializer::Assign { target, value } => checked_statement_to_if_initializer(
                self.analyze_assignment_statement(target, value)?,
            ),
            IfInitializer::Expr(expression) => {
                checked_statement_to_if_initializer(self.analyze_expression_statement(expression)?)
            }
            IfInitializer::MapLookup {
                bindings,
                binding_mode,
                target,
                key,
            } => self.analyze_map_lookup_initializer(bindings, *binding_mode, target, key),
        }
    }

    fn analyze_else_branch(
        &mut self,
        else_branch: &ElseBranch,
    ) -> Result<CheckedElseBranch, SemanticError> {
        match else_branch {
            ElseBranch::Block(block) => {
                Ok(CheckedElseBranch::Block(self.analyze_block(block, true)?))
            }
            ElseBranch::If(if_statement) => {
                let checked_if = self.analyze_if_statement(if_statement)?;
                let CheckedStatement::If(if_statement) = checked_if else {
                    unreachable!("if statement analysis always returns checked if data");
                };
                Ok(CheckedElseBranch::If(Box::new(if_statement)))
            }
        }
    }
}

fn checked_statement_to_if_initializer(
    statement: CheckedStatement,
) -> Result<CheckedIfInitializer, SemanticError> {
    Ok(match statement {
        CheckedStatement::VarDecl { slot, name, value } => {
            CheckedIfInitializer::VarDecl { slot, name, value }
        }
        CheckedStatement::Assign { target, value } => {
            CheckedIfInitializer::Assign { target, value }
        }
        CheckedStatement::Expr(expression) => CheckedIfInitializer::Expr(expression),
        CheckedStatement::MapLookup {
            map,
            key,
            value_binding,
            ok_binding,
        } => CheckedIfInitializer::MapLookup {
            map,
            key,
            value_binding,
            ok_binding,
        },
        CheckedStatement::If(_)
        | CheckedStatement::For { .. }
        | CheckedStatement::RangeFor { .. }
        | CheckedStatement::Return(_) => {
            return Err(SemanticError::new(
                "if initializer requires a simple statement supported by the current frontend",
            ));
        }
    })
}
