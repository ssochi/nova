use std::collections::HashMap;

use crate::frontend::ast::{FunctionDecl, SourceFileAst};
use crate::package::ImportedPackage;
use crate::semantic::analyzer::SemanticError;
use crate::semantic::builtins::resolve_builtin;
use crate::semantic::model::Type;
use crate::semantic::packages::resolve_import_path;
use crate::semantic::support::{is_supported_named_type, resolve_type_ref, validate_runtime_type};

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

            let parameters = function
                .parameters
                .iter()
                .map(|parameter| {
                    let ty = resolve_type_ref(&parameter.type_ref).ok_or_else(|| {
                        SemanticError::new(format!(
                            "unsupported parameter type `{}` in function `{}`",
                            parameter.type_ref.render(),
                            function.name
                        ))
                    })?;
                    validate_runtime_type(
                        &ty,
                        &format!(
                            "parameter `{}` in function `{}`",
                            parameter.name, function.name
                        ),
                    )?;
                    Ok(ty)
                })
                .collect::<Result<Vec<_>, SemanticError>>()?;

            let return_type = match &function.return_type {
                Some(type_ref) => {
                    let ty = resolve_type_ref(type_ref).ok_or_else(|| {
                        SemanticError::new(format!(
                            "unsupported return type `{}` in function `{}`",
                            type_ref.render(),
                            function.name
                        ))
                    })?;
                    validate_runtime_type(
                        &ty,
                        &format!("return type in function `{}`", function.name),
                    )?;
                    ty
                }
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
    pub(super) return_type: Type,
}

pub(super) struct ImportRegistry {
    bindings: HashMap<String, ImportedPackage>,
}

impl ImportRegistry {
    pub(super) fn from_ast(ast: &SourceFileAst) -> Result<Self, SemanticError> {
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

    pub(super) fn lookup(&self, name: &str) -> Option<ImportedPackage> {
        self.bindings.get(name).copied()
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
