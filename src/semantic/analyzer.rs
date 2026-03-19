use std::collections::HashMap;
use std::fmt;

use crate::config::ExecutionConfig;
use crate::conversion::ConversionKind;
use crate::frontend::ast::{
    AssignmentTarget, BinaryOperator, Block, Expression, FunctionDecl, SourceFileAst, Statement,
    TypeRef,
};
use crate::semantic::builtins::{resolve_builtin, validate_builtin_call, validate_make_call};
use crate::semantic::model::{
    CallTarget, CheckedAssignmentTarget, CheckedBinaryOperator, CheckedBlock, CheckedExpression,
    CheckedExpressionKind, CheckedFunction, CheckedMapLiteralEntry, CheckedProgram,
    CheckedStatement, Type,
};
use crate::semantic::packages::{resolve_package_function, validate_package_call};
use crate::semantic::registry::{FunctionRegistry, ImportRegistry};
use crate::semantic::support::{
    block_guarantees_return, coerce_expression_to_type, coerce_nil_equality_operands, expect_type,
    resolve_type_ref, validate_make_literal_bounds, validate_runtime_type, zero_value_expression,
};

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

        let checked_arguments = checked_arguments
            .into_iter()
            .zip(signature.parameters.iter())
            .enumerate()
            .map(|(index, (argument, expected))| {
                coerce_expression_to_type(
                    expected,
                    argument,
                    &format!("argument {} in call to `{callee}`", index + 1),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;

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
        let checked_arguments = coerce_package_call_arguments(package_function, checked_arguments)?;
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
        let checked_entries = entries
            .iter()
            .enumerate()
            .map(|(index, entry)| {
                let key = self.analyze_expression(&entry.key)?;
                expect_type(key_type, &key.ty, &format!("map literal key {}", index + 1))?;
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

fn coerce_package_call_arguments(
    function: crate::package::PackageFunction,
    checked_arguments: Vec<CheckedExpression>,
) -> Result<Vec<CheckedExpression>, SemanticError> {
    let Some(expected_arguments) = crate::semantic::packages::expected_argument_types(function)
    else {
        return Ok(checked_arguments);
    };
    if checked_arguments.len() != expected_arguments.len() {
        return Ok(checked_arguments);
    }
    checked_arguments
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
        .collect()
}

#[cfg(test)]
mod tests;
