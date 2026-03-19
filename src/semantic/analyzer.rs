use std::collections::HashMap;
use std::fmt;

use crate::config::ExecutionConfig;
use crate::frontend::ast::{BinaryOperator, Block, Expression, FunctionDecl, SourceFileAst, Statement};
use crate::semantic::model::{
    BuiltinFunction, CallTarget, CheckedBinaryOperator, CheckedBlock, CheckedExpression,
    CheckedExpressionKind, CheckedFunction, CheckedProgram, CheckedStatement, Type,
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
    let mut checked_functions = Vec::with_capacity(ast.functions.len());
    for (index, function) in ast.functions.iter().enumerate() {
        checked_functions.push(FunctionAnalyzer::new(index, function, &registry).analyze()?);
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
    let entry_function = registry
        .lookup(&config.entry_function)
        .ok_or_else(|| {
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
                    resolve_type_name(&parameter.type_name).ok_or_else(|| {
                        SemanticError::new(format!(
                            "unsupported parameter type `{}` in function `{}`",
                            parameter.type_name, function.name
                        ))
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            let return_type = match &function.return_type {
                Some(type_name) => resolve_type_name(type_name).ok_or_else(|| {
                    SemanticError::new(format!(
                        "unsupported return type `{type_name}` in function `{}`",
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
    scopes: Vec<HashMap<String, LocalBinding>>,
    local_names: Vec<String>,
}

impl<'a> FunctionAnalyzer<'a> {
    fn new(
        function_index: usize,
        function: &'a FunctionDecl,
        registry: &'a FunctionRegistry,
    ) -> Self {
        let mut scopes = vec![HashMap::new()];
        let mut local_names = Vec::new();
        for parameter in &function.parameters {
            let ty = resolve_type_name(&parameter.type_name)
                .expect("function registry only keeps validated type names");
            let slot = local_names.len();
            scopes[0].insert(parameter.name.clone(), LocalBinding { slot, ty });
            local_names.push(parameter.name.clone());
        }

        Self {
            function_index,
            function,
            registry,
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
            return_type: signature.return_type,
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

    fn analyze_statement(&mut self, statement: &Statement) -> Result<CheckedStatement, SemanticError> {
        match statement {
            Statement::VarDecl { name, value } => {
                if self.current_scope().contains_key(name) {
                    return Err(SemanticError::new(format!(
                        "variable `{name}` is already defined in the current scope"
                    )));
                }

                let value = self.analyze_expression(value)?;
                if !value.ty.produces_value() {
                    return Err(SemanticError::new(format!(
                        "variable `{name}` requires a value-producing expression"
                    )));
                }

                let slot = self.allocate_local(name.clone(), value.ty);
                Ok(CheckedStatement::VarDecl {
                    slot,
                    name: name.clone(),
                    value,
                })
            }
            Statement::Assign { name, value } => {
                let binding = self.lookup_local(name)?.clone();
                let value = self.analyze_expression(value)?;
                expect_type(binding.ty, value.ty, &format!("assignment to `{name}`"))?;
                Ok(CheckedStatement::Assign {
                    slot: binding.slot,
                    name: name.clone(),
                    value,
                })
            }
            Statement::Expr(expression) => Ok(CheckedStatement::Expr(
                self.analyze_expression(expression)?,
            )),
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                let condition = self.analyze_expression(condition)?;
                expect_type(Type::Bool, condition.ty, "if condition")?;
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
            Statement::Return(value) => {
                let expected = self.registry.signature(self.function_index).return_type;
                match (expected, value) {
                    (Type::Void, None) => Ok(CheckedStatement::Return(None)),
                    (Type::Void, Some(_)) => Err(SemanticError::new(format!(
                        "function `{}` does not return a value",
                        self.function.name
                    ))),
                    (expected, Some(expression)) => {
                        let expression = self.analyze_expression(expression)?;
                        expect_type(expected, expression.ty, "return statement")?;
                        Ok(CheckedStatement::Return(Some(expression)))
                    }
                    (expected, None) => Err(SemanticError::new(format!(
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
            Expression::Identifier(name) => {
                let binding = self.lookup_local(name)?.clone();
                Ok(CheckedExpression {
                    ty: binding.ty,
                    kind: CheckedExpressionKind::Local {
                        slot: binding.slot,
                        name: name.clone(),
                    },
                })
            }
            Expression::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.analyze_expression(left)?;
                let right = self.analyze_expression(right)?;
                let (operator, ty) = analyze_binary_operator(*operator, left.ty, right.ty)?;
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

                if let Some(builtin) = resolve_builtin(callee) {
                    validate_builtin_call(builtin, &checked_arguments)?;
                    return Ok(CheckedExpression {
                        ty: Type::Void,
                        kind: CheckedExpressionKind::Call {
                            target: CallTarget::Builtin(builtin),
                            arguments: checked_arguments,
                        },
                    });
                }

                let function_index = self.registry.lookup(callee).ok_or_else(|| {
                    SemanticError::new(format!("unknown function `{callee}`"))
                })?;
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
                        *expected,
                        argument.ty,
                        &format!("argument {} in call to `{callee}`", index + 1),
                    )?;
                }

                Ok(CheckedExpression {
                    ty: signature.return_type,
                    kind: CheckedExpressionKind::Call {
                        target: CallTarget::UserDefined {
                            function_index,
                            name: signature.name.clone(),
                        },
                        arguments: checked_arguments,
                    },
                })
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
}

#[derive(Clone)]
struct LocalBinding {
    slot: usize,
    ty: Type,
}

fn analyze_binary_operator(
    operator: BinaryOperator,
    left: Type,
    right: Type,
) -> Result<(CheckedBinaryOperator, Type), SemanticError> {
    match operator {
        BinaryOperator::Add
        | BinaryOperator::Subtract
        | BinaryOperator::Multiply
        | BinaryOperator::Divide => {
            expect_type(Type::Int, left, "left side of arithmetic expression")?;
            expect_type(Type::Int, right, "right side of arithmetic expression")?;
            Ok((
                match operator {
                    BinaryOperator::Add => CheckedBinaryOperator::Add,
                    BinaryOperator::Subtract => CheckedBinaryOperator::Subtract,
                    BinaryOperator::Multiply => CheckedBinaryOperator::Multiply,
                    BinaryOperator::Divide => CheckedBinaryOperator::Divide,
                    _ => unreachable!("arithmetic operators already matched"),
                },
                Type::Int,
            ))
        }
        BinaryOperator::Equal | BinaryOperator::NotEqual => {
            expect_same_type(left, right, "equality expression")?;
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
            expect_type(Type::Int, left, "left side of comparison expression")?;
            expect_type(Type::Int, right, "right side of comparison expression")?;
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

fn validate_builtin_call(
    builtin: BuiltinFunction,
    arguments: &[CheckedExpression],
) -> Result<(), SemanticError> {
    match builtin {
        BuiltinFunction::Println => {
            for argument in arguments {
                if !matches!(argument.ty, Type::Int | Type::Bool) {
                    return Err(SemanticError::new(format!(
                        "builtin `println` does not support `{}` values yet",
                        argument.ty.render()
                    )));
                }
            }
            Ok(())
        }
    }
}

fn resolve_type_name(name: &str) -> Option<Type> {
    match name {
        "int" => Some(Type::Int),
        "bool" => Some(Type::Bool),
        _ => None,
    }
}

fn resolve_builtin(name: &str) -> Option<BuiltinFunction> {
    match name {
        "println" => Some(BuiltinFunction::Println),
        _ => None,
    }
}

fn expect_type(expected: Type, actual: Type, context: &str) -> Result<(), SemanticError> {
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

fn expect_same_type(left: Type, right: Type, context: &str) -> Result<(), SemanticError> {
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
        if statement_guarantees_return(statement) {
            return true;
        }
    }
    false
}

fn statement_guarantees_return(statement: &CheckedStatement) -> bool {
    match statement {
        CheckedStatement::Return(_) => true,
        CheckedStatement::If {
            then_block,
            else_block: Some(else_block),
            ..
        } => block_guarantees_return(then_block) && block_guarantees_return(else_block),
        _ => false,
    }
}
