use crate::builtin::BuiltinFunction;
use crate::semantic::model::Type;

#[derive(Clone, Copy)]
pub struct BuiltinContract {
    pub builtin: BuiltinFunction,
    pub name: &'static str,
    pub validator: fn(&[Type]) -> Result<Type, String>,
}

const BUILTIN_CONTRACTS: [BuiltinContract; 6] = [
    BuiltinContract {
        builtin: BuiltinFunction::Print,
        name: "print",
        validator: validate_variadic_output_builtin,
    },
    BuiltinContract {
        builtin: BuiltinFunction::Println,
        name: "println",
        validator: validate_variadic_output_builtin,
    },
    BuiltinContract {
        builtin: BuiltinFunction::Len,
        name: "len",
        validator: validate_len_builtin,
    },
    BuiltinContract {
        builtin: BuiltinFunction::Cap,
        name: "cap",
        validator: validate_cap_builtin,
    },
    BuiltinContract {
        builtin: BuiltinFunction::Copy,
        name: "copy",
        validator: validate_copy_builtin,
    },
    BuiltinContract {
        builtin: BuiltinFunction::Append,
        name: "append",
        validator: validate_append_builtin,
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
    (contract.validator)(argument_types)
}

fn builtin_contract(builtin: BuiltinFunction) -> &'static BuiltinContract {
    BUILTIN_CONTRACTS
        .iter()
        .find(|contract| contract.builtin == builtin)
        .expect("all builtin functions must have a contract")
}

fn validate_variadic_output_builtin(argument_types: &[Type]) -> Result<Type, String> {
    for (index, argument) in argument_types.iter().enumerate() {
        if !argument.produces_value() {
            return Err(format!(
                "argument {} in call to builtin output must produce a value",
                index + 1
            ));
        }
    }

    Ok(Type::Void)
}

fn validate_len_builtin(argument_types: &[Type]) -> Result<Type, String> {
    validate_exact_arity("len", 1, argument_types.len())?;
    let actual = &argument_types[0];
    if matches!(actual, Type::String | Type::Slice(_)) {
        Ok(Type::Int)
    } else {
        Err(format!(
            "argument 1 in call to builtin `len` requires `string` or `slice`, found `{}`",
            actual.render()
        ))
    }
}

fn validate_cap_builtin(argument_types: &[Type]) -> Result<Type, String> {
    validate_exact_arity("cap", 1, argument_types.len())?;
    let actual = &argument_types[0];
    if matches!(actual, Type::Slice(_)) {
        Ok(Type::Int)
    } else {
        Err(format!(
            "argument 1 in call to builtin `cap` requires `slice`, found `{}`",
            actual.render()
        ))
    }
}

fn validate_copy_builtin(argument_types: &[Type]) -> Result<Type, String> {
    validate_exact_arity("copy", 2, argument_types.len())?;
    let destination = argument_types[0].clone();
    let destination_element = destination.slice_element_type().cloned().ok_or_else(|| {
        format!(
            "argument 1 in call to builtin `copy` requires `slice`, found `{}`",
            argument_types[0].render()
        )
    })?;
    let source = &argument_types[1];
    if source.slice_element_type() != Some(&destination_element) {
        return Err(format!(
            "argument 2 in call to builtin `copy` requires `{}`, found `{}`",
            destination.render(),
            source.render()
        ));
    }

    Ok(Type::Int)
}

fn validate_append_builtin(argument_types: &[Type]) -> Result<Type, String> {
    validate_min_arity("append", 1, argument_types.len())?;
    let slice_type = argument_types[0].clone();
    let element_type = slice_type.slice_element_type().cloned().ok_or_else(|| {
        format!(
            "argument 1 in call to builtin `append` requires `slice`, found `{}`",
            argument_types[0].render()
        )
    })?;

    for (index, argument) in argument_types.iter().enumerate().skip(1) {
        if argument != &element_type {
            return Err(format!(
                "argument {} in call to builtin `append` requires `{}`, found `{}`",
                index + 1,
                element_type.render(),
                argument.render()
            ));
        }
    }

    Ok(slice_type)
}

fn validate_exact_arity(name: &str, expected: usize, actual: usize) -> Result<(), String> {
    if expected == actual {
        Ok(())
    } else {
        Err(format!(
            "builtin `{name}` expects {expected} arguments, found {actual}"
        ))
    }
}

fn validate_min_arity(name: &str, minimum: usize, actual: usize) -> Result<(), String> {
    if actual >= minimum {
        Ok(())
    } else {
        Err(format!(
            "builtin `{name}` expects at least {minimum} arguments, found {actual}"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::validate_builtin_call;
    use crate::builtin::BuiltinFunction;
    use crate::semantic::model::Type;

    #[test]
    fn len_accepts_slice_arguments() {
        let result =
            validate_builtin_call(BuiltinFunction::Len, &[Type::Slice(Box::new(Type::Int))])
                .expect("len should accept slices");
        assert_eq!(result, Type::Int);
    }

    #[test]
    fn cap_rejects_non_slice_arguments() {
        let error = validate_builtin_call(BuiltinFunction::Cap, &[Type::String])
            .expect_err("cap should reject strings in the current subset");

        assert!(error.contains("requires `slice`"));
    }

    #[test]
    fn copy_requires_matching_slice_types() {
        let error = validate_builtin_call(
            BuiltinFunction::Copy,
            &[
                Type::Slice(Box::new(Type::Int)),
                Type::Slice(Box::new(Type::String)),
            ],
        )
        .expect_err("copy should reject mismatched slice types");

        assert!(error.contains("argument 2"));
        assert!(error.contains("requires `[]int`"));
    }

    #[test]
    fn append_requires_matching_element_types() {
        let error = validate_builtin_call(
            BuiltinFunction::Append,
            &[Type::Slice(Box::new(Type::Int)), Type::Int, Type::String],
        )
        .expect_err("append should reject mixed element types");

        assert!(error.contains("argument 3"));
        assert!(error.contains("requires `int`"));
    }
}
