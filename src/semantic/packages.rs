use crate::package::{ImportedPackage, PackageFunction};
use crate::semantic::model::Type;

#[derive(Clone, Copy)]
pub struct PackageFunctionContract {
    pub function: PackageFunction,
    pub member_name: &'static str,
    pub arity: PackageArity,
    pub arguments: PackageArguments,
    pub return_type: Type,
}

#[derive(Clone, Copy)]
pub enum PackageArity {
    Variadic,
}

#[derive(Clone, Copy)]
pub enum PackageArguments {
    AnyValue,
}

const FMT_FUNCTIONS: [PackageFunctionContract; 3] = [
    PackageFunctionContract {
        function: PackageFunction::FmtPrint,
        member_name: "Print",
        arity: PackageArity::Variadic,
        arguments: PackageArguments::AnyValue,
        return_type: Type::Void,
    },
    PackageFunctionContract {
        function: PackageFunction::FmtPrintln,
        member_name: "Println",
        arity: PackageArity::Variadic,
        arguments: PackageArguments::AnyValue,
        return_type: Type::Void,
    },
    PackageFunctionContract {
        function: PackageFunction::FmtSprint,
        member_name: "Sprint",
        arity: PackageArity::Variadic,
        arguments: PackageArguments::AnyValue,
        return_type: Type::String,
    },
];

pub fn resolve_import_path(path: &str) -> Option<ImportedPackage> {
    match path {
        "fmt" => Some(ImportedPackage::Fmt),
        _ => None,
    }
}

pub fn resolve_package_function(
    package: ImportedPackage,
    member_name: &str,
) -> Option<PackageFunction> {
    package_functions(package)
        .iter()
        .find(|contract| contract.member_name == member_name)
        .map(|contract| contract.function)
}

pub fn validate_package_call(
    function: PackageFunction,
    argument_types: &[Type],
) -> Result<Type, String> {
    let contract = package_function_contract(function);
    validate_package_arity(contract, argument_types.len())?;

    match contract.arguments {
        PackageArguments::AnyValue => {
            for (index, argument) in argument_types.iter().enumerate() {
                if !argument.produces_value() {
                    return Err(format!(
                        "argument {} in call to `{}` must produce a value",
                        index + 1,
                        function.render()
                    ));
                }
            }
        }
    }

    Ok(contract.return_type)
}

fn package_functions(package: ImportedPackage) -> &'static [PackageFunctionContract] {
    match package {
        ImportedPackage::Fmt => &FMT_FUNCTIONS,
    }
}

fn package_function_contract(function: PackageFunction) -> &'static PackageFunctionContract {
    FMT_FUNCTIONS
        .iter()
        .find(|contract| contract.function == function)
        .expect("all package functions must have contracts")
}

fn validate_package_arity(
    contract: &PackageFunctionContract,
    _actual: usize,
) -> Result<(), String> {
    match contract.arity {
        PackageArity::Variadic => Ok(()),
    }
}
