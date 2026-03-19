use std::collections::HashMap;
use std::fmt;

use crate::config::ExecutionConfig;
use crate::frontend::ast::{AssignmentTarget, Block, FunctionDecl, SourceFileAst, Statement};
use crate::semantic::model::{
    CheckedAssignmentTarget, CheckedBlock, CheckedFunction, CheckedProgram, CheckedStatement, Type,
};
use crate::semantic::registry::{FunctionRegistry, ImportRegistry};
use crate::semantic::support::{
    block_guarantees_return, coerce_expression_to_type, expect_type, resolve_type_ref,
    validate_runtime_type, zero_value_expression,
};

mod expressions;
mod range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticError {
    message: String,
}

impl SemanticError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for SemanticError {}

pub fn analyze_package(ast: &SourceFileAst) -> Result<CheckedProgram, SemanticError> {
    let registry = FunctionRegistry::from_ast(ast)?;
    let imports = ImportRegistry::from_ast(ast)?;
    let mut checked_functions = Vec::with_capacity(ast.functions.len());
    for (index, function) in ast.functions.iter().enumerate() {
        checked_functions
            .push(FunctionAnalyzer::new(index, function, &registry, &imports).analyze()?);
    }

    Ok(CheckedProgram {
        package_name: ast.package_name.clone(),
        entry_function: 0,
        functions: checked_functions,
    })
}

pub fn analyze_program(
    ast: &SourceFileAst,
    config: &ExecutionConfig,
) -> Result<CheckedProgram, SemanticError> {
    if ast.package_name != config.entry_package {
        return Err(SemanticError::new(format!(
            "entry package mismatch: expected `{}`, found `{}`",
            config.entry_package, ast.package_name
        )));
    }

    let mut program = analyze_package(ast)?;
    let registry = FunctionRegistry::from_ast(ast)?;
    let entry_function = registry.lookup(&config.entry_function).ok_or_else(|| {
        SemanticError::new(format!(
            "entry function `{}` was not found in package `{}`",
            config.entry_function, ast.package_name
        ))
    })?;
    let entry_signature = registry.signature(entry_function);
    if !entry_signature.parameters.is_empty() {
        return Err(SemanticError::new(format!(
            "entry function `{}` must not declare parameters",
            config.entry_function
        )));
    }
    if entry_signature.return_type != Type::Void {
        return Err(SemanticError::new(format!(
            "entry function `{}` must not return a value",
            config.entry_function
        )));
    }

    program.entry_function = entry_function;
    Ok(program)
}

struct FunctionAnalyzer<'a> {
    function_index: usize,
    function: &'a FunctionDecl,
    registry: &'a FunctionRegistry,
    imports: &'a ImportRegistry,
    scopes: Vec<HashMap<String, LocalBinding>>,
    local_names: Vec<String>,
}

impl<'a> FunctionAnalyzer<'a> {
    fn new(
        function_index: usize,
        function: &'a FunctionDecl,
        registry: &'a FunctionRegistry,
        imports: &'a ImportRegistry,
    ) -> Self {
        let mut scopes = vec![HashMap::new()];
        let mut local_names = Vec::new();
        for parameter in &function.parameters {
            let ty = resolve_type_ref(&parameter.type_ref)
                .expect("function registry only keeps validated type names");
            let slot = local_names.len();
            scopes[0].insert(parameter.name.clone(), LocalBinding { slot, ty });
            local_names.push(parameter.name.clone());
        }

        Self {
            function_index,
            function,
            registry,
            imports,
            scopes,
            local_names,
        }
    }

    fn analyze(mut self) -> Result<CheckedFunction, SemanticError> {
        self.ensure_unique_parameters()?;
        let body = self.analyze_block(&self.function.body, false)?;
        let signature = self.registry.signature(self.function_index);

        if signature.return_type != Type::Void && !block_guarantees_return(&body) {
            return Err(SemanticError::new(format!(
                "function `{}` must return a `{}` on every path",
                self.function.name,
                signature.return_type.render()
            )));
        }

        Ok(CheckedFunction {
            name: self.function.name.clone(),
            parameter_count: self.function.parameters.len(),
            return_type: signature.return_type.clone(),
            local_names: self.local_names,
            body,
        })
    }

    fn ensure_unique_parameters(&self) -> Result<(), SemanticError> {
        let mut seen = HashMap::new();
        for parameter in &self.function.parameters {
            if seen.insert(parameter.name.as_str(), ()).is_some() {
                return Err(SemanticError::new(format!(
                    "parameter `{}` is already defined in function `{}`",
                    parameter.name, self.function.name
                )));
            }
        }
        Ok(())
    }

    fn analyze_block(
        &mut self,
        block: &Block,
        create_scope: bool,
    ) -> Result<CheckedBlock, SemanticError> {
        if create_scope {
            self.scopes.push(HashMap::new());
        }

        let mut statements = Vec::with_capacity(block.statements.len());
        for statement in &block.statements {
            statements.push(self.analyze_statement(statement)?);
        }

        if create_scope {
            self.scopes.pop();
        }

        Ok(CheckedBlock { statements })
    }

    fn analyze_statement(
        &mut self,
        statement: &Statement,
    ) -> Result<CheckedStatement, SemanticError> {
        match statement {
            Statement::VarDecl {
                name,
                type_ref,
                value,
            } => {
                if self.current_scope().contains_key(name) {
                    return Err(SemanticError::new(format!(
                        "variable `{name}` is already defined in the current scope"
                    )));
                }

                let declared_type = type_ref
                    .as_ref()
                    .map(|type_ref| {
                        let ty = resolve_type_ref(type_ref).ok_or_else(|| {
                            SemanticError::new(format!(
                                "unsupported variable type `{}` in declaration of `{name}`",
                                type_ref.render()
                            ))
                        })?;
                        validate_runtime_type(&ty, &format!("variable `{name}`"))?;
                        Ok(ty)
                    })
                    .transpose()?;
                let value = match value {
                    Some(expression) => {
                        let value = self.analyze_expression(expression)?;
                        if value.ty == Type::UntypedNil && declared_type.is_none() {
                            return Err(SemanticError::new(format!(
                                "variable `{name}` requires an explicit type when initialized with `nil`"
                            )));
                        }
                        if !value.ty.produces_value() && value.ty != Type::UntypedNil {
                            return Err(SemanticError::new(format!(
                                "variable `{name}` requires a value-producing expression"
                            )));
                        }
                        Some(if let Some(declared_type) = &declared_type {
                            coerce_expression_to_type(
                                declared_type,
                                value,
                                &format!("variable `{name}`"),
                            )?
                        } else {
                            value
                        })
                    }
                    None => None,
                };
                let local_type = declared_type.unwrap_or_else(|| {
                    value
                        .as_ref()
                        .expect("untyped variable declarations require an initializer")
                        .ty
                        .clone()
                });
                let value = if value.is_some() {
                    value
                } else {
                    Some(zero_value_expression(local_type.clone()))
                };
                let slot = self.allocate_local(name.clone(), local_type);
                Ok(CheckedStatement::VarDecl {
                    slot,
                    name: name.clone(),
                    value,
                })
            }
            Statement::Assign { target, value } => {
                let target = self.analyze_assignment_target(target)?;
                let value = self.analyze_expression(value)?;
                let value = match &target {
                    CheckedAssignmentTarget::Local { name, .. } => {
                        let binding = self.lookup_local(name)?;
                        coerce_expression_to_type(
                            &binding.ty,
                            value,
                            &format!("assignment to `{name}`"),
                        )?
                    }
                    CheckedAssignmentTarget::Index {
                        target: target_expression,
                        ..
                    } => match &target_expression.ty {
                        Type::Slice(element) => coerce_expression_to_type(
                            element.as_ref(),
                            value,
                            "slice element assignment",
                        )?,
                        Type::Map { value: element, .. } => coerce_expression_to_type(
                            element.as_ref(),
                            value,
                            "map element assignment",
                        )?,
                        _ => {
                            return Err(SemanticError::new(format!(
                                "index assignment requires `slice` or `map` target, found `{}`",
                                target_expression.ty.render()
                            )));
                        }
                    },
                };
                Ok(CheckedStatement::Assign { target, value })
            }
            Statement::Expr(expression) => {
                let expression = self.analyze_expression(expression)?;
                if expression.ty == Type::UntypedNil {
                    return Err(SemanticError::new(
                        "expression statement requires a typed value, found `nil`",
                    ));
                }
                Ok(CheckedStatement::Expr(expression))
            }
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                let condition = self.analyze_expression(condition)?;
                expect_type(&Type::Bool, &condition.ty, "if condition")?;
                let then_block = self.analyze_block(then_block, true)?;
                let else_block = else_block
                    .as_ref()
                    .map(|block| self.analyze_block(block, true))
                    .transpose()?;
                Ok(CheckedStatement::If {
                    condition,
                    then_block,
                    else_block,
                })
            }
            Statement::For { condition, body } => {
                let condition = self.analyze_expression(condition)?;
                expect_type(&Type::Bool, &condition.ty, "for condition")?;
                let body = self.analyze_block(body, true)?;
                Ok(CheckedStatement::For { condition, body })
            }
            Statement::RangeFor {
                bindings,
                binding_mode,
                target,
                body,
            } => self.analyze_range_statement(bindings, *binding_mode, target, body),
            Statement::Return(value) => {
                let expected = self
                    .registry
                    .signature(self.function_index)
                    .return_type
                    .clone();
                match (&expected, value) {
                    (Type::Void, None) => Ok(CheckedStatement::Return(None)),
                    (Type::Void, Some(_)) => Err(SemanticError::new(format!(
                        "function `{}` does not return a value",
                        self.function.name
                    ))),
                    (_, Some(expression)) => {
                        let expression = coerce_expression_to_type(
                            &expected,
                            self.analyze_expression(expression)?,
                            "return statement",
                        )?;
                        Ok(CheckedStatement::Return(Some(expression)))
                    }
                    (_, None) => Err(SemanticError::new(format!(
                        "function `{}` must return a `{}` value",
                        self.function.name,
                        expected.render()
                    ))),
                }
            }
        }
    }

    fn current_scope(&self) -> &HashMap<String, LocalBinding> {
        self.scopes.last().expect("at least one scope exists")
    }

    fn current_scope_mut(&mut self) -> &mut HashMap<String, LocalBinding> {
        self.scopes.last_mut().expect("at least one scope exists")
    }

    fn allocate_local(&mut self, name: String, ty: Type) -> usize {
        let slot = self.local_names.len();
        self.current_scope_mut()
            .insert(name.clone(), LocalBinding { slot, ty });
        self.local_names.push(name);
        slot
    }

    fn lookup_local(&self, name: &str) -> Result<&LocalBinding, SemanticError> {
        for scope in self.scopes.iter().rev() {
            if let Some(binding) = scope.get(name) {
                return Ok(binding);
            }
        }

        Err(SemanticError::new(format!("unknown variable `{name}`")))
    }

    fn analyze_assignment_target(
        &mut self,
        target: &AssignmentTarget,
    ) -> Result<CheckedAssignmentTarget, SemanticError> {
        match target {
            AssignmentTarget::Identifier(name) => {
                let binding = self.lookup_local(name)?;
                Ok(CheckedAssignmentTarget::Local {
                    slot: binding.slot,
                    name: name.clone(),
                })
            }
            AssignmentTarget::Index { target, index } => {
                let target = self.analyze_expression(target)?;
                let index = self.analyze_expression(index)?;
                match &target.ty {
                    Type::Slice(_) => expect_type(&Type::Int, &index.ty, "index assignment")?,
                    Type::Map { key, .. } => {
                        expect_type(key.as_ref(), &index.ty, "map index assignment")?
                    }
                    _ => {
                        return Err(SemanticError::new(format!(
                            "index assignment requires `slice` or `map` target, found `{}`",
                            target.ty.render()
                        )));
                    }
                }
                Ok(CheckedAssignmentTarget::Index {
                    target: Box::new(target),
                    index: Box::new(index),
                })
            }
        }
    }
}

#[derive(Clone)]
struct LocalBinding {
    slot: usize,
    ty: Type,
}

#[cfg(test)]
mod tests;
