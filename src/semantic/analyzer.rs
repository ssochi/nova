use std::collections::HashMap;
use std::fmt;

use crate::config::ExecutionConfig;
use crate::frontend::ast::{
    AssignmentTarget, Block, CompoundAssignOperator, FunctionDecl, IncDecOperator, SourceFileAst,
    Statement, TypeRef,
};
use crate::semantic::model::{
    CheckedAssignmentTarget, CheckedBinding, CheckedBlock, CheckedCompoundAssignOperator,
    CheckedExpression, CheckedFunction, CheckedIncDecOperator, CheckedProgram, CheckedStatement,
    CheckedValueSource, Type,
};
use crate::semantic::registry::{FunctionRegistry, ImportRegistry};
use crate::semantic::support::{
    block_guarantees_return, coerce_expression_to_type, expect_type, render_type_list,
    resolve_type_ref, validate_runtime_type, zero_value_expression,
};

mod expressions;
mod ifs;
mod lookup;
mod loops;
mod range;
mod switches;

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
    if !entry_signature.return_types.is_empty() {
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
    control_flow_stack: Vec<ControlFlowContext>,
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
            control_flow_stack: Vec::new(),
            local_names,
        }
    }

    fn analyze(mut self) -> Result<CheckedFunction, SemanticError> {
        self.ensure_unique_parameters()?;
        let body = self.analyze_block(&self.function.body, false)?;
        let signature = self.registry.signature(self.function_index);

        if !signature.return_types.is_empty() && !block_guarantees_return(&body) {
            return Err(SemanticError::new(format!(
                "function `{}` must return a `{}` on every path",
                self.function.name,
                render_type_list(&signature.return_types)
            )));
        }

        Ok(CheckedFunction {
            name: self.function.name.clone(),
            parameter_count: self.function.parameters.len(),
            return_types: signature.return_types.clone(),
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
            Statement::ShortVarDecl { bindings, values } => {
                self.analyze_short_var_decl_statement(bindings, values)
            }
            Statement::MultiAssign { bindings, values } => {
                self.analyze_multi_assign_statement(bindings, values)
            }
            Statement::VarDecl {
                name,
                type_ref,
                value,
            } => self.analyze_var_decl_statement(name, type_ref.as_ref(), value.as_ref()),
            Statement::Assign { target, value } => self.analyze_assignment_statement(target, value),
            Statement::Send { channel, value } => self.analyze_send_statement(channel, value),
            Statement::CompoundAssign {
                target,
                operator,
                value,
            } => self.analyze_compound_assign_statement(target, *operator, value),
            Statement::Expr(expression) => self.analyze_expression_statement(expression),
            Statement::If(if_statement) => self.analyze_if_statement(if_statement),
            Statement::Switch(switch_statement) => self.analyze_switch_statement(switch_statement),
            Statement::For(for_statement) => self.analyze_for_statement(for_statement),
            Statement::RangeFor {
                bindings,
                binding_mode,
                target,
                body,
            } => self.analyze_range_statement(bindings, *binding_mode, target, body),
            Statement::MapLookup {
                bindings,
                binding_mode,
                target,
                key,
            } => self.analyze_map_lookup_statement(bindings, *binding_mode, target, key),
            Statement::IncDec { target, operator } => {
                self.analyze_inc_dec_statement(target, *operator)
            }
            Statement::Break => self.analyze_break_statement(),
            Statement::Continue => self.analyze_continue_statement(),
            Statement::Return(values) => self.analyze_return_statement(values),
        }
    }

    pub(super) fn analyze_var_decl_statement(
        &mut self,
        name: &str,
        type_ref: Option<&TypeRef>,
        value: Option<&crate::frontend::ast::Expression>,
    ) -> Result<CheckedStatement, SemanticError> {
        if self.current_scope().contains_key(name) {
            return Err(SemanticError::new(format!(
                "variable `{name}` is already defined in the current scope"
            )));
        }

        let declared_type = type_ref
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
        let value = self.analyze_var_initializer(name, declared_type.as_ref(), value)?;
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
        let slot = self.allocate_local(name.to_string(), local_type);
        Ok(CheckedStatement::VarDecl {
            slot,
            name: name.to_string(),
            value,
        })
    }

    pub(super) fn analyze_short_var_decl_statement(
        &mut self,
        bindings: &[crate::frontend::ast::Binding],
        values: &[crate::frontend::ast::Expression],
    ) -> Result<CheckedStatement, SemanticError> {
        let value_source =
            self.analyze_value_source(values, Some(bindings.len()), "short declaration")?;
        let result_types = value_source.result_types();
        let bindings =
            self.resolve_short_decl_bindings(bindings, &result_types, "short declaration `:=`")?;
        Ok(CheckedStatement::ShortVarDecl {
            bindings,
            values: value_source,
        })
    }

    pub(super) fn analyze_multi_assign_statement(
        &mut self,
        bindings: &[crate::frontend::ast::Binding],
        values: &[crate::frontend::ast::Expression],
    ) -> Result<CheckedStatement, SemanticError> {
        let binding_targets = self.resolve_assignment_bindings(bindings, "assignment")?;
        let expected_types = binding_targets
            .iter()
            .map(|binding| match binding {
                CheckedBinding::Local { slot, name: _ } => self.local_type(*slot).clone(),
                CheckedBinding::Discard => Type::Void,
            })
            .collect::<Vec<_>>();
        let value_source =
            self.analyze_typed_value_source(values, &expected_types, "assignment")?;
        Ok(CheckedStatement::MultiAssign {
            bindings: binding_targets,
            values: value_source,
        })
    }

    pub(super) fn analyze_assignment_statement(
        &mut self,
        target: &AssignmentTarget,
        value: &crate::frontend::ast::Expression,
    ) -> Result<CheckedStatement, SemanticError> {
        let target = self.analyze_assignment_target(target)?;
        let value = self.analyze_assignment_value(&target, value)?;
        Ok(CheckedStatement::Assign { target, value })
    }

    pub(super) fn analyze_expression_statement(
        &mut self,
        expression: &crate::frontend::ast::Expression,
    ) -> Result<CheckedStatement, SemanticError> {
        if let crate::frontend::ast::Expression::Call { callee, arguments } = expression {
            let call = self.analyze_call(callee, arguments)?;
            if call.result_types.len() > 1 {
                return Err(SemanticError::new(format!(
                    "expression statement cannot discard multi-result call to `{}`",
                    match &call.target {
                        crate::semantic::model::CallTarget::Builtin(builtin) => {
                            builtin.render().to_string()
                        }
                        crate::semantic::model::CallTarget::PackageFunction(function) => {
                            function.render().to_string()
                        }
                        crate::semantic::model::CallTarget::UserDefined { name, .. } => {
                            name.clone()
                        }
                    }
                )));
            }
            let ty = call.result_types.first().cloned().unwrap_or(Type::Void);
            return Ok(CheckedStatement::Expr(CheckedExpression {
                ty,
                kind: crate::semantic::model::CheckedExpressionKind::Call(call),
            }));
        }
        let expression = self.analyze_expression(expression)?;
        if expression.ty == Type::UntypedNil {
            return Err(SemanticError::new(
                "expression statement requires a typed value, found `nil`",
            ));
        }
        Ok(CheckedStatement::Expr(expression))
    }

    pub(super) fn analyze_send_statement(
        &mut self,
        channel: &crate::frontend::ast::Expression,
        value: &crate::frontend::ast::Expression,
    ) -> Result<CheckedStatement, SemanticError> {
        let channel = self.analyze_expression(channel)?;
        let element_type = channel.ty.channel_element_type().cloned().ok_or_else(|| {
            SemanticError::new(format!(
                "send statement requires `chan` target, found `{}`",
                channel.ty.render()
            ))
        })?;
        let value = coerce_expression_to_type(
            &element_type,
            self.analyze_expression(value)?,
            "send statement",
        )?;
        Ok(CheckedStatement::Send { channel, value })
    }

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
            return Err(SemanticError::new(format!(
                "function `{}` must return a `{}` value",
                self.function.name,
                render_type_list(&expected)
            )));
        }

        let values = self.analyze_typed_value_source(values, &expected, "return statement")?;
        Ok(CheckedStatement::Return(values))
    }

    pub(super) fn analyze_compound_assign_statement(
        &mut self,
        target: &AssignmentTarget,
        operator: CompoundAssignOperator,
        value: &crate::frontend::ast::Expression,
    ) -> Result<CheckedStatement, SemanticError> {
        let target = self.analyze_assignment_target(target)?;
        let target_type = self.checked_assignment_target_type(&target)?;
        let value = self.analyze_expression(value)?;
        let (operator, value) = analyze_compound_assign(operator, &target_type, value)?;
        Ok(CheckedStatement::CompoundAssign {
            target,
            operator,
            value,
        })
    }

    pub(super) fn analyze_inc_dec_statement(
        &mut self,
        target: &AssignmentTarget,
        operator: IncDecOperator,
    ) -> Result<CheckedStatement, SemanticError> {
        let target = self.analyze_assignment_target(target)?;
        let operand_type = self.checked_assignment_target_type(&target)?;
        if !operand_type.supports_inc_dec() {
            return Err(SemanticError::new(format!(
                "`{}` requires `int` or `byte`, found `{}`",
                render_inc_dec_operator(operator),
                operand_type.render()
            )));
        }

        Ok(CheckedStatement::IncDec {
            target,
            operator: checked_inc_dec_operator(operator),
            operand_type,
        })
    }

    fn analyze_var_initializer(
        &mut self,
        name: &str,
        declared_type: Option<&Type>,
        value: Option<&crate::frontend::ast::Expression>,
    ) -> Result<Option<CheckedExpression>, SemanticError> {
        match value {
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
                Ok(Some(if let Some(declared_type) = declared_type {
                    coerce_expression_to_type(declared_type, value, &format!("variable `{name}`"))?
                } else {
                    value
                }))
            }
            None => Ok(None),
        }
    }

    fn analyze_assignment_value(
        &mut self,
        target: &CheckedAssignmentTarget,
        value: &crate::frontend::ast::Expression,
    ) -> Result<CheckedExpression, SemanticError> {
        let value = self.analyze_expression(value)?;
        match target {
            CheckedAssignmentTarget::Local { name, .. } => {
                let binding = self.lookup_local(name)?;
                coerce_expression_to_type(&binding.ty, value, &format!("assignment to `{name}`"))
            }
            CheckedAssignmentTarget::Index {
                target: target_expression,
                ..
            } => match &target_expression.ty {
                Type::Slice(element) => {
                    coerce_expression_to_type(element.as_ref(), value, "slice element assignment")
                }
                Type::Map { value: element, .. } => {
                    coerce_expression_to_type(element.as_ref(), value, "map element assignment")
                }
                _ => Err(SemanticError::new(format!(
                    "index assignment requires `slice` or `map` target, found `{}`",
                    target_expression.ty.render()
                ))),
            },
        }
    }

    fn analyze_value_source(
        &mut self,
        values: &[crate::frontend::ast::Expression],
        expected_count: Option<usize>,
        context: &str,
    ) -> Result<CheckedValueSource, SemanticError> {
        if let Some(expected_count) = expected_count {
            if let [expression] = values {
                if let Some(call) = self.try_analyze_multi_result_call(expression)? {
                    if call.result_types.len() != expected_count {
                        return Err(SemanticError::new(format!(
                            "{context} expects {} values, found {}",
                            expected_count,
                            call.result_types.len()
                        )));
                    }
                    return Ok(CheckedValueSource::Call(call));
                }
            }
            if values.len() != expected_count {
                return Err(SemanticError::new(format!(
                    "{context} expects {} values, found {}",
                    expected_count,
                    values.len()
                )));
            }
        }

        let checked_values = values
            .iter()
            .map(|value| self.analyze_expression(value))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(CheckedValueSource::Expressions(checked_values))
    }

    fn analyze_typed_value_source(
        &mut self,
        values: &[crate::frontend::ast::Expression],
        expected_types: &[Type],
        context: &str,
    ) -> Result<CheckedValueSource, SemanticError> {
        if let [expression] = values {
            if let Some(call) = self.try_analyze_multi_result_call(expression)? {
                if call.result_types.len() != expected_types.len() {
                    return Err(SemanticError::new(format!(
                        "{context} expects `{}`, found `{}`",
                        render_type_list(expected_types),
                        render_type_list(&call.result_types)
                    )));
                }
                for (actual, expected) in call.result_types.iter().zip(expected_types) {
                    expect_type(expected, actual, context)?;
                }
                return Ok(CheckedValueSource::Call(call));
            }
        }

        if values.len() != expected_types.len() {
            return Err(SemanticError::new(format!(
                "{context} expects {} values, found {}",
                expected_types.len(),
                values.len()
            )));
        }

        let checked_values = values
            .iter()
            .zip(expected_types.iter())
            .enumerate()
            .map(|(index, (value, expected))| {
                let checked = self.analyze_expression(value)?;
                if expected == &Type::Void {
                    if checked.ty == Type::UntypedNil {
                        return Err(SemanticError::new(format!(
                            "{context} value {} requires a typed value, found `nil`",
                            index + 1
                        )));
                    }
                    if !checked.ty.produces_value() {
                        return Err(SemanticError::new(format!(
                            "{context} value {} requires a value-producing expression",
                            index + 1
                        )));
                    }
                    Ok(checked)
                } else {
                    coerce_expression_to_type(
                        expected,
                        checked,
                        &format!("{context} value {}", index + 1),
                    )
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(CheckedValueSource::Expressions(checked_values))
    }

    fn resolve_short_decl_bindings(
        &mut self,
        bindings: &[crate::frontend::ast::Binding],
        result_types: &[Type],
        context: &str,
    ) -> Result<Vec<CheckedBinding>, SemanticError> {
        if bindings.len() != result_types.len() {
            return Err(SemanticError::new(format!(
                "{context} expects {} bindings, found {}",
                result_types.len(),
                bindings.len()
            )));
        }

        let mut seen = HashMap::new();
        let mut has_new_named_binding = false;
        let mut resolved = Vec::with_capacity(bindings.len());
        for (binding, result_type) in bindings.iter().zip(result_types.iter()) {
            resolved.push(match binding {
                crate::frontend::ast::Binding::Blank => CheckedBinding::Discard,
                crate::frontend::ast::Binding::Identifier(name) => {
                    if seen.insert(name.as_str(), ()).is_some() {
                        return Err(SemanticError::new(format!(
                            "{context} declares `{name}` more than once"
                        )));
                    }
                    if result_type == &Type::UntypedNil {
                        return Err(SemanticError::new(format!(
                            "{context} `{name}` requires a typed value, found `nil`"
                        )));
                    }
                    if let Some(binding) = self.current_scope().get(name).cloned() {
                        if binding.ty != *result_type {
                            return Err(SemanticError::new(format!(
                                "{context} redeclaration of `{name}` requires `{}`, found `{}`",
                                binding.ty.render(),
                                result_type.render()
                            )));
                        }
                        CheckedBinding::Local {
                            slot: binding.slot,
                            name: name.clone(),
                        }
                    } else {
                        has_new_named_binding = true;
                        let slot = self.allocate_local(name.clone(), result_type.clone());
                        CheckedBinding::Local {
                            slot,
                            name: name.clone(),
                        }
                    }
                }
            });
        }

        if !has_new_named_binding {
            return Err(SemanticError::new(
                "short declaration `:=` requires at least one new named variable",
            ));
        }

        Ok(resolved)
    }

    fn resolve_assignment_bindings(
        &mut self,
        bindings: &[crate::frontend::ast::Binding],
        context: &str,
    ) -> Result<Vec<CheckedBinding>, SemanticError> {
        let mut seen = HashMap::new();
        let mut resolved = Vec::with_capacity(bindings.len());
        for binding in bindings {
            resolved.push(match binding {
                crate::frontend::ast::Binding::Blank => CheckedBinding::Discard,
                crate::frontend::ast::Binding::Identifier(name) => {
                    if seen.insert(name.as_str(), ()).is_some() {
                        return Err(SemanticError::new(format!(
                            "{context} assigns `{name}` more than once"
                        )));
                    }
                    let binding = self.lookup_local(name)?;
                    CheckedBinding::Local {
                        slot: binding.slot,
                        name: name.clone(),
                    }
                }
            });
        }
        Ok(resolved)
    }

    fn checked_assignment_target_type(
        &self,
        target: &CheckedAssignmentTarget,
    ) -> Result<Type, SemanticError> {
        match target {
            CheckedAssignmentTarget::Local { name, .. } => Ok(self.lookup_local(name)?.ty.clone()),
            CheckedAssignmentTarget::Index {
                target: target_expression,
                ..
            } => match &target_expression.ty {
                Type::Slice(element) => Ok(element.as_ref().clone()),
                Type::Map { value, .. } => Ok(value.as_ref().clone()),
                _ => Err(SemanticError::new(format!(
                    "index assignment requires `slice` or `map` target, found `{}`",
                    target_expression.ty.render()
                ))),
            },
        }
    }

    fn current_scope(&self) -> &HashMap<String, LocalBinding> {
        self.scopes.last().expect("at least one scope exists")
    }

    fn current_scope_mut(&mut self) -> &mut HashMap<String, LocalBinding> {
        self.scopes.last_mut().expect("at least one scope exists")
    }

    fn local_type(&self, slot: usize) -> &Type {
        self.scopes
            .iter()
            .flat_map(|scope| scope.values())
            .find(|binding| binding.slot == slot)
            .map(|binding| &binding.ty)
            .expect("local slot should resolve to a tracked type")
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

fn checked_inc_dec_operator(operator: IncDecOperator) -> CheckedIncDecOperator {
    match operator {
        IncDecOperator::Increment => CheckedIncDecOperator::Increment,
        IncDecOperator::Decrement => CheckedIncDecOperator::Decrement,
    }
}

fn analyze_compound_assign(
    operator: CompoundAssignOperator,
    target_type: &Type,
    value: CheckedExpression,
) -> Result<(CheckedCompoundAssignOperator, CheckedExpression), SemanticError> {
    match operator {
        CompoundAssignOperator::Add => match target_type {
            Type::Int | Type::Byte => Ok((
                CheckedCompoundAssignOperator::Add,
                coerce_expression_to_type(target_type, value, "compound assignment `+=`")?,
            )),
            Type::String => Ok((
                CheckedCompoundAssignOperator::Concat,
                coerce_expression_to_type(target_type, value, "compound assignment `+=`")?,
            )),
            _ => Err(SemanticError::new(format!(
                "`+=` requires `int`, `byte`, or `string`, found `{}`",
                target_type.render()
            ))),
        },
        CompoundAssignOperator::Subtract => analyze_numeric_compound_assign(
            target_type,
            value,
            "`-=`",
            CheckedCompoundAssignOperator::Subtract,
        ),
        CompoundAssignOperator::Multiply => analyze_numeric_compound_assign(
            target_type,
            value,
            "`*=`",
            CheckedCompoundAssignOperator::Multiply,
        ),
        CompoundAssignOperator::Divide => analyze_numeric_compound_assign(
            target_type,
            value,
            "`/=`",
            CheckedCompoundAssignOperator::Divide,
        ),
    }
}

fn analyze_numeric_compound_assign(
    target_type: &Type,
    value: CheckedExpression,
    operator: &str,
    checked_operator: CheckedCompoundAssignOperator,
) -> Result<(CheckedCompoundAssignOperator, CheckedExpression), SemanticError> {
    if !matches!(target_type, Type::Int | Type::Byte) {
        return Err(SemanticError::new(format!(
            "{operator} requires `int` or `byte`, found `{}`",
            target_type.render()
        )));
    }
    Ok((
        checked_operator,
        coerce_expression_to_type(
            target_type,
            value,
            &format!("compound assignment {operator}"),
        )?,
    ))
}

fn render_inc_dec_operator(operator: IncDecOperator) -> &'static str {
    match operator {
        IncDecOperator::Increment => "++",
        IncDecOperator::Decrement => "--",
    }
}

#[derive(Clone)]
struct LocalBinding {
    slot: usize,
    ty: Type,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ControlFlowContext {
    Loop,
    Switch,
}

#[cfg(test)]
mod tests;
