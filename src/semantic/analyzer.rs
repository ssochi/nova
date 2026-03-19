use std::collections::HashMap;
use std::fmt;

use crate::config::ExecutionConfig;
use crate::frontend::ast::{
    AssignmentTarget, BinaryOperator, Block, Expression, FunctionDecl, SourceFileAst, Statement,
    TypeRef,
};
use crate::package::ImportedPackage;
use crate::semantic::builtins::{resolve_builtin, validate_builtin_call, validate_make_call};
use crate::semantic::model::{
    CallTarget, CheckedAssignmentTarget, CheckedBinaryOperator, CheckedBlock, CheckedExpression,
    CheckedExpressionKind, CheckedFunction, CheckedProgram, CheckedStatement, Type,
};
use crate::semantic::packages::{
    resolve_import_path, resolve_package_function, validate_package_call,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticError {
    message: String,
}

impl SemanticError {
    fn new(message: impl Into<String>) -> Self {
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

struct FunctionRegistry {
    name_to_index: HashMap<String, usize>,
    signatures: Vec<FunctionSignature>,
}

impl FunctionRegistry {
    fn from_ast(ast: &SourceFileAst) -> Result<Self, SemanticError> {
        let mut name_to_index = HashMap::new();
        let mut signatures = Vec::with_capacity(ast.functions.len());

        for function in &ast.functions {
            if resolve_builtin(function.name.as_str()).is_some() {
                return Err(SemanticError::new(format!(
                    "function `{}` conflicts with a builtin name",
                    function.name
                )));
            }

            if name_to_index.contains_key(&function.name) {
                return Err(SemanticError::new(format!(
                    "function `{}` is already defined",
                    function.name
                )));
            }

            let parameters = function
                .parameters
                .iter()
                .map(|parameter| {
                    resolve_type_ref(&parameter.type_ref).ok_or_else(|| {
                        SemanticError::new(format!(
                            "unsupported parameter type `{}` in function `{}`",
                            parameter.type_ref.render(),
                            function.name
                        ))
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            let return_type = match &function.return_type {
                Some(type_ref) => resolve_type_ref(type_ref).ok_or_else(|| {
                    SemanticError::new(format!(
                        "unsupported return type `{}` in function `{}`",
                        type_ref.render(),
                        function.name
                    ))
                })?,
                None => Type::Void,
            };

            let index = signatures.len();
            name_to_index.insert(function.name.clone(), index);
            signatures.push(FunctionSignature {
                name: function.name.clone(),
                parameters,
                return_type,
            });
        }

        Ok(Self {
            name_to_index,
            signatures,
        })
    }

    fn lookup(&self, name: &str) -> Option<usize> {
        self.name_to_index.get(name).copied()
    }

    fn signature(&self, index: usize) -> &FunctionSignature {
        &self.signatures[index]
    }
}

struct FunctionSignature {
    name: String,
    parameters: Vec<Type>,
    return_type: Type,
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
                        resolve_type_ref(type_ref).ok_or_else(|| {
                            SemanticError::new(format!(
                                "unsupported variable type `{}` in declaration of `{name}`",
                                type_ref.render()
                            ))
                        })
                    })
                    .transpose()?;
                let value = match value {
                    Some(expression) => {
                        let value = self.analyze_expression(expression)?;
                        if !value.ty.produces_value() {
                            return Err(SemanticError::new(format!(
                                "variable `{name}` requires a value-producing expression"
                            )));
                        }
                        if let Some(declared_type) = &declared_type {
                            expect_type(declared_type, &value.ty, &format!("variable `{name}`"))?;
                        }
                        Some(value)
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
                match &target {
                    CheckedAssignmentTarget::Local { name, .. } => {
                        let binding = self.lookup_local(name)?;
                        expect_type(&binding.ty, &value.ty, &format!("assignment to `{name}`"))?;
                    }
                    CheckedAssignmentTarget::Index { target, .. } => {
                        let element_type = target.ty.slice_element_type().ok_or_else(|| {
                            SemanticError::new(format!(
                                "index assignment requires `slice` target, found `{}`",
                                target.ty.render()
                            ))
                        })?;
                        expect_type(element_type, &value.ty, "slice element assignment")?;
                    }
                }
                Ok(CheckedStatement::Assign { target, value })
            }
            Statement::Expr(expression) => {
                Ok(CheckedStatement::Expr(self.analyze_expression(expression)?))
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
                        let expression = self.analyze_expression(expression)?;
                        expect_type(&expected, &expression.ty, "return statement")?;
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

    fn analyze_expression(
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
            Expression::SliceLiteral {
                element_type,
                elements,
            } => {
                let slice_type = resolve_type_ref(element_type).ok_or_else(|| {
                    SemanticError::new(format!(
                        "unsupported slice literal type `{}`",
                        element_type.render()
                    ))
                })?;
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
                        expect_type(
                            &element_type,
                            &checked.ty,
                            &format!("slice literal element {}", index + 1),
                        )?;
                        Ok(checked)
                    })
                    .collect::<Result<Vec<_>, SemanticError>>()?;
                Ok(CheckedExpression {
                    ty: slice_type,
                    kind: CheckedExpressionKind::SliceLiteral {
                        elements: checked_elements,
                    },
                })
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
                expect_type(&Type::Int, &index.ty, "index expression")?;
                let element_type = target.ty.slice_element_type().cloned().ok_or_else(|| {
                    SemanticError::new(format!(
                        "index expression requires `slice` target, found `{}`",
                        target.ty.render()
                    ))
                })?;
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
                if !matches!(target.ty, Type::Slice(_)) {
                    return Err(SemanticError::new(format!(
                        "slice expression requires `slice` target, found `{}`",
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
                    ty: target.ty.clone(),
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
                length,
                capacity,
            } => self.analyze_make_expression(type_ref, length, capacity.as_deref()),
            Expression::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.analyze_expression(left)?;
                let right = self.analyze_expression(right)?;
                let (operator, ty) = analyze_binary_operator(*operator, &left.ty, &right.ty)?;
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

        for (index, (argument, expected)) in checked_arguments
            .iter()
            .zip(signature.parameters.iter())
            .enumerate()
        {
            expect_type(
                expected,
                &argument.ty,
                &format!("argument {} in call to `{callee}`", index + 1),
            )?;
        }

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
        length: &Expression,
        capacity: Option<&Expression>,
    ) -> Result<CheckedExpression, SemanticError> {
        let allocated_type = resolve_type_ref(type_ref).ok_or_else(|| {
            SemanticError::new(format!(
                "builtin `make` does not support type `{}`",
                type_ref.render()
            ))
        })?;
        let length = self.analyze_expression(length)?;
        let capacity = capacity
            .map(|expression| self.analyze_expression(expression))
            .transpose()?;
        let mut argument_types = vec![length.ty.clone()];
        if let Some(capacity) = &capacity {
            argument_types.push(capacity.ty.clone());
        }
        let result_type =
            validate_make_call(&allocated_type, &argument_types).map_err(SemanticError::new)?;
        if let Some(capacity) = &capacity {
            validate_make_literal_bounds(&length, capacity)?;
        }
        let element_type = result_type
            .slice_element_type()
            .cloned()
            .expect("slice validation ensures make result has an element type");
        Ok(CheckedExpression {
            ty: result_type,
            kind: CheckedExpressionKind::MakeSlice {
                element_type,
                length: Box::new(length),
                capacity: capacity.map(Box::new),
            },
        })
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
                expect_type(&Type::Int, &index.ty, "index assignment")?;
                if !matches!(target.ty, Type::Slice(_)) {
                    return Err(SemanticError::new(format!(
                        "index assignment requires `slice` target, found `{}`",
                        target.ty.render()
                    )));
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

struct ImportRegistry {
    bindings: HashMap<String, ImportedPackage>,
}

impl ImportRegistry {
    fn from_ast(ast: &SourceFileAst) -> Result<Self, SemanticError> {
        let mut bindings = HashMap::new();
        for import in &ast.imports {
            let package = resolve_import_path(&import.path).ok_or_else(|| {
                SemanticError::new(format!("unsupported import path `{}`", import.path))
            })?;
            let binding = package.binding_name().to_string();
            if bindings.insert(binding.clone(), package).is_some() {
                return Err(SemanticError::new(format!(
                    "import binding `{binding}` is already defined"
                )));
            }
        }

        Ok(Self { bindings })
    }

    fn lookup(&self, name: &str) -> Option<ImportedPackage> {
        self.bindings.get(name).copied()
    }
}

fn analyze_binary_operator(
    operator: BinaryOperator,
    left: &Type,
    right: &Type,
) -> Result<(CheckedBinaryOperator, Type), SemanticError> {
    match operator {
        BinaryOperator::Add if left == &Type::Int && right == &Type::Int => {
            Ok((CheckedBinaryOperator::Add, Type::Int))
        }
        BinaryOperator::Add if left == &Type::String && right == &Type::String => {
            Ok((CheckedBinaryOperator::Concat, Type::String))
        }
        BinaryOperator::Add => Err(SemanticError::new(format!(
            "addition requires matching `int` or `string` operands, found `{}` and `{}`",
            left.render(),
            right.render()
        ))),
        BinaryOperator::Subtract | BinaryOperator::Multiply | BinaryOperator::Divide => {
            expect_type(&Type::Int, left, "left side of arithmetic expression")?;
            expect_type(&Type::Int, right, "right side of arithmetic expression")?;
            Ok((
                match operator {
                    BinaryOperator::Subtract => CheckedBinaryOperator::Subtract,
                    BinaryOperator::Multiply => CheckedBinaryOperator::Multiply,
                    BinaryOperator::Divide => CheckedBinaryOperator::Divide,
                    _ => unreachable!("non-add arithmetic operators already matched"),
                },
                Type::Int,
            ))
        }
        BinaryOperator::Equal | BinaryOperator::NotEqual => {
            expect_same_type(left, right, "equality expression")?;
            if !left.supports_equality() {
                return Err(SemanticError::new(format!(
                    "equality expression does not support `{}` operands",
                    left.render()
                )));
            }
            Ok((
                match operator {
                    BinaryOperator::Equal => CheckedBinaryOperator::Equal,
                    BinaryOperator::NotEqual => CheckedBinaryOperator::NotEqual,
                    _ => unreachable!("equality operators already matched"),
                },
                Type::Bool,
            ))
        }
        BinaryOperator::Less
        | BinaryOperator::LessEqual
        | BinaryOperator::Greater
        | BinaryOperator::GreaterEqual => {
            expect_type(&Type::Int, left, "left side of comparison expression")?;
            expect_type(&Type::Int, right, "right side of comparison expression")?;
            Ok((
                match operator {
                    BinaryOperator::Less => CheckedBinaryOperator::Less,
                    BinaryOperator::LessEqual => CheckedBinaryOperator::LessEqual,
                    BinaryOperator::Greater => CheckedBinaryOperator::Greater,
                    BinaryOperator::GreaterEqual => CheckedBinaryOperator::GreaterEqual,
                    _ => unreachable!("comparison operators already matched"),
                },
                Type::Bool,
            ))
        }
    }
}

fn resolve_type_ref(type_ref: &TypeRef) -> Option<Type> {
    match type_ref {
        TypeRef::Named(name) => match name.as_str() {
            "int" => Some(Type::Int),
            "bool" => Some(Type::Bool),
            "string" => Some(Type::String),
            _ => None,
        },
        TypeRef::Slice(element) => Some(Type::Slice(Box::new(resolve_type_ref(element)?))),
    }
}

fn expect_type(expected: &Type, actual: &Type, context: &str) -> Result<(), SemanticError> {
    if expected == actual {
        Ok(())
    } else {
        Err(SemanticError::new(format!(
            "{context} requires `{}`, found `{}`",
            expected.render(),
            actual.render()
        )))
    }
}

fn expect_same_type(left: &Type, right: &Type, context: &str) -> Result<(), SemanticError> {
    if left == right {
        Ok(())
    } else {
        Err(SemanticError::new(format!(
            "{context} requires matching operand types, found `{}` and `{}`",
            left.render(),
            right.render()
        )))
    }
}

fn block_guarantees_return(block: &CheckedBlock) -> bool {
    for statement in &block.statements {
        if statement_guarantees_termination(statement) {
            return true;
        }
    }
    false
}

fn statement_guarantees_termination(statement: &CheckedStatement) -> bool {
    match statement {
        CheckedStatement::Return(_) => true,
        CheckedStatement::If {
            then_block,
            else_block: Some(else_block),
            ..
        } => block_guarantees_return(then_block) && block_guarantees_return(else_block),
        CheckedStatement::For { condition, .. } => expression_is_compile_time_true(condition),
        _ => false,
    }
}

fn expression_is_compile_time_true(expression: &CheckedExpression) -> bool {
    matches!(expression.kind, CheckedExpressionKind::Bool(true))
}

#[cfg(test)]
mod tests {
    use super::analyze_package;
    use crate::frontend::{lexer::lex, parser::parse_source_file};
    use crate::source::SourceFile;

    #[test]
    fn analyze_slice_index_and_append() {
        let source = SourceFile {
            path: "test.go".into(),
            contents:
                "package main\n\nfunc main() {\n\tvar values = []int{1, 2}\n\tvalues = append(values, 3)\n\tprintln(values[1])\n}\n"
                    .to_string(),
        };

        let tokens = lex(&source).expect("lexing should succeed");
        let ast = parse_source_file(&tokens).expect("parsing should succeed");
        let program = analyze_package(&ast).expect("analysis should succeed");

        assert_eq!(program.functions.len(), 1);
    }

    #[test]
    fn reject_slice_equality() {
        let source = SourceFile {
            path: "test.go".into(),
            contents:
                "package main\n\nfunc main() {\n\tvar left = []int{1}\n\tvar right = []int{1}\n\tprintln(left == right)\n}\n"
                    .to_string(),
        };

        let tokens = lex(&source).expect("lexing should succeed");
        let ast = parse_source_file(&tokens).expect("parsing should succeed");
        let error = analyze_package(&ast).expect_err("slice equality should fail");

        assert!(
            error
                .to_string()
                .contains("does not support `[]int` operands")
        );
    }

    #[test]
    fn analyze_slice_expression_and_index_assignment() {
        let source = SourceFile {
            path: "test.go".into(),
            contents: "package main\n\nfunc main() {\n\tvar values = []int{1, 2, 3}\n\tvar middle = values[1:3]\n\tmiddle[0] = 9\n\tprintln(values[1], middle[0])\n}\n"
                .to_string(),
        };

        let tokens = lex(&source).expect("lexing should succeed");
        let ast = parse_source_file(&tokens).expect("parsing should succeed");
        let program = analyze_package(&ast).expect("analysis should succeed");

        assert_eq!(program.functions.len(), 1);
    }

    #[test]
    fn analyze_typed_var_declarations_without_initializers() {
        let source = SourceFile {
            path: "test.go".into(),
            contents:
                "package main\n\nfunc main() {\n\tvar total int\n\tvar ready bool\n\tvar label string\n\tvar values []int\n\tprintln(total, ready, len(label), len(values), cap(values))\n}\n"
                    .to_string(),
        };

        let tokens = lex(&source).expect("lexing should succeed");
        let ast = parse_source_file(&tokens).expect("parsing should succeed");
        let program = analyze_package(&ast).expect("analysis should succeed");

        assert_eq!(program.functions.len(), 1);
    }

    #[test]
    fn analyze_make_slice_expression() {
        let source = SourceFile {
            path: "test.go".into(),
            contents: "package main\n\nfunc main() {\n\tvar values = make([]int, 2, 4)\n\tvalues[1] = 9\n\tprintln(len(values), cap(values), values[1])\n}\n"
                .to_string(),
        };

        let tokens = lex(&source).expect("lexing should succeed");
        let ast = parse_source_file(&tokens).expect("parsing should succeed");
        let program = analyze_package(&ast).expect("analysis should succeed");

        assert_eq!(program.functions.len(), 1);
    }

    #[test]
    fn reject_make_with_constant_length_exceeding_capacity() {
        let source = SourceFile {
            path: "test.go".into(),
            contents: "package main\n\nfunc main() {\n\tvar values = make([]int, 3, 2)\n\tprintln(values)\n}\n"
                .to_string(),
        };

        let tokens = lex(&source).expect("lexing should succeed");
        let ast = parse_source_file(&tokens).expect("parsing should succeed");
        let error = analyze_package(&ast).expect_err("analysis should reject invalid make bounds");

        assert!(error.to_string().contains("length 3 exceeds capacity 2"));
    }
}

fn zero_value_expression(ty: Type) -> CheckedExpression {
    CheckedExpression {
        ty,
        kind: CheckedExpressionKind::ZeroValue,
    }
}

fn validate_make_literal_bounds(
    length: &CheckedExpression,
    capacity: &CheckedExpression,
) -> Result<(), SemanticError> {
    let CheckedExpressionKind::Integer(length_value) = &length.kind else {
        return Ok(());
    };
    let CheckedExpressionKind::Integer(capacity_value) = &capacity.kind else {
        return Ok(());
    };
    if length_value > capacity_value {
        return Err(SemanticError::new(format!(
            "builtin `make` length {} exceeds capacity {}",
            length_value, capacity_value
        )));
    }
    Ok(())
}
