use crate::builtin::BuiltinFunction;
use crate::semantic::model::Type;

#[derive(Clone, Copy)]
pub struct BuiltinContract {
    pub builtin: BuiltinFunction,
    pub name: &'static str,
    pub arity: BuiltinArity,
    pub arguments: BuiltinArguments,
    pub return_type: Type,
}

#[derive(Clone, Copy)]
pub enum BuiltinArity {
    Variadic,
    Exact(usize),
}

#[derive(Clone, Copy)]
pub enum BuiltinArguments {
    AnyValue,
    Exact(&'static [Type]),
}

const LEN_ARGUMENTS: [Type; 1] = [Type::String];

const BUILTIN_CONTRACTS: [BuiltinContract; 3] = [
    BuiltinContract {
        builtin: BuiltinFunction::Print,
        name: "print",
        arity: BuiltinArity::Variadic,
        arguments: BuiltinArguments::AnyValue,
        return_type: Type::Void,
    },
    BuiltinContract {
        builtin: BuiltinFunction::Println,
        name: "println",
        arity: BuiltinArity::Variadic,
        arguments: BuiltinArguments::AnyValue,
        return_type: Type::Void,
    },
    BuiltinContract {
        builtin: BuiltinFunction::Len,
        name: "len",
        arity: BuiltinArity::Exact(1),
        arguments: BuiltinArguments::Exact(&LEN_ARGUMENTS),
        return_type: Type::Int,
    },
];

pub fn resolve_builtin(name: &str) -> Option<BuiltinFunction> {
    BUILTIN_CONTRACTS
        .iter()
        .find(|contract| contract.name == name)
        .map(|contract| contract.builtin)
}

pub fn validate_builtin_call(
    builtin: BuiltinFunction,
    argument_types: &[Type],
) -> Result<Type, String> {
    let contract = builtin_contract(builtin);
    validate_builtin_arity(contract, argument_types.len())?;

    match contract.arguments {
        BuiltinArguments::AnyValue => {
            for (index, argument) in argument_types.iter().enumerate() {
                if !argument.produces_value() {
                    return Err(format!(
                        "argument {} in call to builtin `{}` must produce a value",
                        index + 1,
                        contract.name
                    ));
                }
            }
        }
        BuiltinArguments::Exact(expected_types) => {
            for (index, (expected, actual)) in
                expected_types.iter().zip(argument_types.iter()).enumerate()
            {
                if expected != actual {
                    return Err(format!(
                        "argument {} in call to builtin `{}` requires `{}`, found `{}`",
                        index + 1,
                        contract.name,
                        expected.render(),
                        actual.render()
                    ));
                }
            }
        }
    }

    Ok(contract.return_type)
}

fn builtin_contract(builtin: BuiltinFunction) -> &'static BuiltinContract {
    BUILTIN_CONTRACTS
        .iter()
        .find(|contract| contract.builtin == builtin)
        .expect("all builtin functions must have a contract")
}

fn validate_builtin_arity(contract: &BuiltinContract, actual: usize) -> Result<(), String> {
    match contract.arity {
        BuiltinArity::Variadic => Ok(()),
        BuiltinArity::Exact(expected) if expected == actual => Ok(()),
        BuiltinArity::Exact(expected) => Err(format!(
            "builtin `{}` expects {} arguments, found {}",
            contract.name, expected, actual
        )),
    }
}
