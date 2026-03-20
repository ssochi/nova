use std::collections::HashMap;

use crate::frontend::ast::{FunctionDecl, ImportDecl, ImportSpec, SourceFileAst};
use crate::package::ImportedPackage;
use crate::semantic::analyzer::SemanticError;
use crate::semantic::builtins::resolve_builtin;
use crate::semantic::model::Type;
use crate::semantic::packages::resolve_import_path;
use crate::semantic::support::{
    flatten_function_parameters, flatten_function_results, is_supported_named_type,
    resolve_type_ref, validate_runtime_type,
};

pub(super) struct FunctionRegistry {
    name_to_index: HashMap<String, usize>,
    signatures: Vec<FunctionSignature>,
}

impl FunctionRegistry {
    pub(super) fn from_ast(ast: &SourceFileAst) -> Result<Self, SemanticError> {
        let mut name_to_index = HashMap::new();
        let mut signatures = Vec::with_capacity(ast.functions.len());

        for function in &ast.functions {
            validate_function_name(function)?;
            if name_to_index.contains_key(&function.name) {
                return Err(SemanticError::new(format!(
                    "function `{}` is already defined",
                    function.name
                )));
            }

            let flattened_parameters = flatten_function_parameters(function);
            let parameters = flattened_parameters
                .iter()
                .map(|parameter| {
                    let parameter_type =
                        resolve_type_ref(&parameter.type_ref).ok_or_else(|| {
                            SemanticError::new(format!(
                                "unsupported parameter type `{}` in function `{}`",
                                parameter.type_ref.render(),
                                function.name
                            ))
                        })?;
                    let ty = if parameter.variadic {
                        Type::Slice(Box::new(parameter_type.clone()))
                    } else {
                        parameter_type.clone()
                    };
                    validate_runtime_type(
                        &parameter_type,
                        &format!(
                            "parameter `{}` in function `{}`",
                            parameter.name, function.name
                        ),
                    )?;
                    Ok(ty)
                })
                .collect::<Result<Vec<_>, SemanticError>>()?;

            let return_types = flatten_function_results(function)
                .iter()
                .map(|result| {
                    let ty = resolve_type_ref(&result.type_ref).ok_or_else(|| {
                        SemanticError::new(format!(
                            "unsupported return type `{}` in function `{}`",
                            result.type_ref.render(),
                            function.name
                        ))
                    })?;
                    validate_runtime_type(
                        &ty,
                        &format!("return type in function `{}`", function.name),
                    )?;
                    Ok(ty)
                })
                .collect::<Result<Vec<_>, SemanticError>>()?;

            let index = signatures.len();
            name_to_index.insert(function.name.clone(), index);
            signatures.push(FunctionSignature {
                name: function.name.clone(),
                parameters,
                variadic_element_type: flattened_parameters
                    .last()
                    .and_then(|parameter| {
                        parameter
                            .variadic
                            .then(|| resolve_type_ref(&parameter.type_ref))
                    })
                    .flatten(),
                return_types,
            });
        }

        Ok(Self {
            name_to_index,
            signatures,
        })
    }

    pub(super) fn lookup(&self, name: &str) -> Option<usize> {
        self.name_to_index.get(name).copied()
    }

    pub(super) fn signature(&self, index: usize) -> &FunctionSignature {
        &self.signatures[index]
    }
}

pub(super) struct FunctionSignature {
    pub(super) name: String,
    pub(super) parameters: Vec<Type>,
    pub(super) variadic_element_type: Option<Type>,
    pub(super) return_types: Vec<Type>,
}

pub(super) struct ImportRegistry {
    bindings: HashMap<String, ImportedPackage>,
}

impl ImportRegistry {
    pub(super) fn from_ast(ast: &SourceFileAst) -> Result<Self, SemanticError> {
        let mut bindings = HashMap::new();
        for import in &ast.imports {
            for spec in import_specs(import) {
                let package = resolve_import_path(&spec.path).ok_or_else(|| {
                    SemanticError::new(format!("unsupported import path `{}`", spec.path))
                })?;
                let binding = spec
                    .binding
                    .clone()
                    .unwrap_or_else(|| package.default_binding_name().to_string());
                if bindings.insert(binding.clone(), package).is_some() {
                    return Err(SemanticError::new(format!(
                        "import binding `{binding}` is already defined"
                    )));
                }
            }
        }

        Ok(Self { bindings })
    }

    pub(super) fn lookup(&self, name: &str) -> Option<ImportedPackage> {
        self.bindings.get(name).copied()
    }
}

fn import_specs(import: &ImportDecl) -> &[ImportSpec] {
    match import {
        ImportDecl::Single(spec) => std::slice::from_ref(spec),
        ImportDecl::Group(specs) => specs.as_slice(),
    }
}

fn validate_function_name(function: &FunctionDecl) -> Result<(), SemanticError> {
    if resolve_builtin(function.name.as_str()).is_some() {
        return Err(SemanticError::new(format!(
            "function `{}` conflicts with a builtin name",
            function.name
        )));
    }
    if is_supported_named_type(function.name.as_str()) {
        return Err(SemanticError::new(format!(
            "function `{}` conflicts with a predeclared type name",
            function.name
        )));
    }

    Ok(())
}
