use crate::builtin::BuiltinFunction;
use crate::semantic::model::Type;

#[derive(Clone, Copy)]
pub struct BuiltinContract {
    pub builtin: BuiltinFunction,
    pub name: &'static str,
    pub validator: fn(&[Type]) -> Result<Vec<Type>, String>,
}

const BUILTIN_CONTRACTS: [BuiltinContract; 9] = [
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
    BuiltinContract {
        builtin: BuiltinFunction::Make,
        name: "make",
        validator: validate_make_value_builtin,
    },
    BuiltinContract {
        builtin: BuiltinFunction::Delete,
        name: "delete",
        validator: validate_delete_builtin,
    },
    BuiltinContract {
        builtin: BuiltinFunction::Close,
        name: "close",
        validator: validate_close_builtin,
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
) -> Result<Vec<Type>, String> {
    let contract = builtin_contract(builtin);
    (contract.validator)(argument_types)
}

pub fn validate_append_spread_call(
    prefix_argument_types: &[Type],
    spread_type: &Type,
) -> Result<Vec<Type>, String> {
    if prefix_argument_types.len() != 1 {
        return Err(format!(
            "builtin `append` with `...` expects 2 arguments, found {}",
            prefix_argument_types.len() + 1
        ));
    }

    let slice_type = prefix_argument_types[0].clone();
    let element_type = slice_type.slice_element_type().cloned().ok_or_else(|| {
        format!(
            "argument 1 in call to builtin `append` requires `slice`, found `{}`",
            prefix_argument_types[0].render()
        )
    })?;

    let expected_spread_type = Type::Slice(Box::new(element_type.clone()));
    if spread_type == &expected_spread_type
        || (element_type == Type::Byte && spread_type == &Type::String)
    {
        return Ok(vec![slice_type]);
    }

    let expected = if element_type == Type::Byte {
        "`[]byte` or `string`".to_string()
    } else {
        format!("`{}`", expected_spread_type.render())
    };
    Err(format!(
        "spread argument in call to builtin `append` requires {expected}, found `{}`",
        spread_type.render()
    ))
}

fn builtin_contract(builtin: BuiltinFunction) -> &'static BuiltinContract {
    BUILTIN_CONTRACTS
        .iter()
        .find(|contract| contract.builtin == builtin)
        .expect("all builtin functions must have a contract")
}

fn validate_variadic_output_builtin(argument_types: &[Type]) -> Result<Vec<Type>, String> {
    for (index, argument) in argument_types.iter().enumerate() {
        if !argument.produces_value() {
            return Err(format!(
                "argument {} in call to builtin output must produce a value",
                index + 1
            ));
        }
    }

    Ok(Vec::new())
}

fn validate_len_builtin(argument_types: &[Type]) -> Result<Vec<Type>, String> {
    validate_exact_arity("len", 1, argument_types.len())?;
    let actual = &argument_types[0];
    if matches!(
        actual,
        Type::String | Type::Slice(_) | Type::Chan(_) | Type::Map { .. }
    ) {
        Ok(vec![Type::Int])
    } else {
        Err(format!(
            "argument 1 in call to builtin `len` requires `string`, `slice`, `chan`, or `map`, found `{}`",
            actual.render()
        ))
    }
}

fn validate_cap_builtin(argument_types: &[Type]) -> Result<Vec<Type>, String> {
    validate_exact_arity("cap", 1, argument_types.len())?;
    let actual = &argument_types[0];
    if matches!(actual, Type::Slice(_) | Type::Chan(_)) {
        Ok(vec![Type::Int])
    } else {
        Err(format!(
            "argument 1 in call to builtin `cap` requires `slice` or `chan`, found `{}`",
            actual.render()
        ))
    }
}

fn validate_copy_builtin(argument_types: &[Type]) -> Result<Vec<Type>, String> {
    validate_exact_arity("copy", 2, argument_types.len())?;
    let destination = argument_types[0].clone();
    let destination_element = destination.slice_element_type().cloned().ok_or_else(|| {
        format!(
            "argument 1 in call to builtin `copy` requires `slice`, found `{}`",
            argument_types[0].render()
        )
    })?;
    let source = &argument_types[1];
    if destination_element == Type::Byte && source == &Type::String {
        return Ok(vec![Type::Int]);
    }
    if source.slice_element_type() != Some(&destination_element) {
        return Err(format!(
            "argument 2 in call to builtin `copy` requires `{}`, found `{}`",
            destination.render(),
            source.render()
        ));
    }

    Ok(vec![Type::Int])
}

fn validate_append_builtin(argument_types: &[Type]) -> Result<Vec<Type>, String> {
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

    Ok(vec![slice_type])
}

pub fn validate_make_call(allocated_type: &Type, argument_types: &[Type]) -> Result<Type, String> {
    match allocated_type {
        Type::Slice(_) => validate_make_slice_call(allocated_type, argument_types),
        Type::Chan(_) => validate_make_chan_call(allocated_type, argument_types),
        Type::Map { .. } => validate_make_map_call(allocated_type, argument_types),
        _ => Err(format!(
            "argument 1 in call to builtin `make` requires `slice`, `chan`, or `map`, found `{}`",
            allocated_type.render()
        )),
    }
}

fn validate_make_value_builtin(_argument_types: &[Type]) -> Result<Vec<Type>, String> {
    Err("builtin `make` requires a type argument".to_string())
}

fn validate_delete_builtin(argument_types: &[Type]) -> Result<Vec<Type>, String> {
    validate_exact_arity("delete", 2, argument_types.len())?;
    let map_type = argument_types[0].clone();
    let (key_type, _) = map_type.map_parts().ok_or_else(|| {
        format!(
            "argument 1 in call to builtin `delete` requires `map`, found `{}`",
            argument_types[0].render()
        )
    })?;
    if &argument_types[1] != key_type {
        return Err(format!(
            "argument 2 in call to builtin `delete` requires `{}`, found `{}`",
            key_type.render(),
            argument_types[1].render()
        ));
    }

    Ok(Vec::new())
}

fn validate_close_builtin(argument_types: &[Type]) -> Result<Vec<Type>, String> {
    validate_exact_arity("close", 1, argument_types.len())?;
    let channel_type = &argument_types[0];
    if matches!(channel_type, Type::Chan(_)) {
        Ok(Vec::new())
    } else {
        Err(format!(
            "argument 1 in call to builtin `close` requires `chan`, found `{}`",
            channel_type.render()
        ))
    }
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

fn validate_make_arity(actual: usize) -> Result<(), String> {
    if matches!(actual, 1 | 2) {
        Ok(())
    } else {
        Err(format!(
            "builtin `make` expects 2 or 3 arguments including the type, found {}",
            actual + 1
        ))
    }
}

fn validate_make_slice_call(
    allocated_type: &Type,
    argument_types: &[Type],
) -> Result<Type, String> {
    validate_make_arity(argument_types.len())?;
    validate_make_integer_arguments(argument_types)?;
    Ok(allocated_type.clone())
}

fn validate_make_chan_call(allocated_type: &Type, argument_types: &[Type]) -> Result<Type, String> {
    if argument_types.len() > 1 {
        return Err(format!(
            "builtin `make` expects 1 or 2 arguments including the type for channels, found {}",
            argument_types.len() + 1
        ));
    }
    validate_make_integer_arguments(argument_types)?;
    Ok(allocated_type.clone())
}

fn validate_make_map_call(allocated_type: &Type, argument_types: &[Type]) -> Result<Type, String> {
    if argument_types.len() > 1 {
        return Err(format!(
            "builtin `make` expects 1 or 2 arguments including the type for maps, found {}",
            argument_types.len() + 1
        ));
    }
    validate_make_integer_arguments(argument_types)?;
    Ok(allocated_type.clone())
}

fn validate_make_integer_arguments(argument_types: &[Type]) -> Result<(), String> {
    for (index, argument) in argument_types.iter().enumerate() {
        if argument != &Type::Int {
            return Err(format!(
                "argument {} in call to builtin `make` requires `int`, found `{}`",
                index + 2,
                argument.render()
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{validate_append_spread_call, validate_builtin_call, validate_make_call};
    use crate::builtin::BuiltinFunction;
    use crate::semantic::model::Type;

    #[test]
    fn len_accepts_slice_arguments() {
        let result =
            validate_builtin_call(BuiltinFunction::Len, &[Type::Slice(Box::new(Type::Int))])
                .expect("len should accept slices");
        assert_eq!(result, vec![Type::Int]);
    }

    #[test]
    fn len_accepts_map_arguments() {
        let result = validate_builtin_call(
            BuiltinFunction::Len,
            &[Type::Map {
                key: Box::new(Type::String),
                value: Box::new(Type::Int),
            }],
        )
        .expect("len should accept maps");

        assert_eq!(result, vec![Type::Int]);
    }

    #[test]
    fn len_accepts_channel_arguments() {
        let result =
            validate_builtin_call(BuiltinFunction::Len, &[Type::Chan(Box::new(Type::Int))])
                .expect("len should accept channels");

        assert_eq!(result, vec![Type::Int]);
    }

    #[test]
    fn cap_rejects_non_slice_arguments() {
        let error = validate_builtin_call(BuiltinFunction::Cap, &[Type::String])
            .expect_err("cap should reject strings in the current subset");

        assert!(error.contains("requires `slice`"));
    }

    #[test]
    fn cap_accepts_channel_arguments() {
        let result =
            validate_builtin_call(BuiltinFunction::Cap, &[Type::Chan(Box::new(Type::Int))])
                .expect("cap should accept channels");

        assert_eq!(result, vec![Type::Int]);
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
    fn copy_accepts_byte_slice_and_string() {
        let result = validate_builtin_call(
            BuiltinFunction::Copy,
            &[Type::Slice(Box::new(Type::Byte)), Type::String],
        )
        .expect("copy should accept []byte <- string");

        assert_eq!(result, vec![Type::Int]);
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

    #[test]
    fn append_spread_accepts_matching_slice_type() {
        let result = validate_append_spread_call(
            &[Type::Slice(Box::new(Type::Int))],
            &Type::Slice(Box::new(Type::Int)),
        )
        .expect("append spread should accept matching slice type");

        assert_eq!(result, vec![Type::Slice(Box::new(Type::Int))]);
    }

    #[test]
    fn append_spread_accepts_byte_string_special_case() {
        let result =
            validate_append_spread_call(&[Type::Slice(Box::new(Type::Byte))], &Type::String)
                .expect("append spread should accept []byte plus string");

        assert_eq!(result, vec![Type::Slice(Box::new(Type::Byte))]);
    }

    #[test]
    fn append_spread_rejects_mismatched_slice_type() {
        let error = validate_append_spread_call(
            &[Type::Slice(Box::new(Type::Int))],
            &Type::Slice(Box::new(Type::String)),
        )
        .expect_err("append spread should reject mismatched slice type");

        assert!(error.contains("spread argument"));
        assert!(error.contains("[]int"));
    }

    #[test]
    fn make_requires_slice_type_argument() {
        let error = validate_make_call(&Type::Int, &[Type::Int])
            .expect_err("make should reject non-slice type arguments");

        assert!(error.contains("argument 1"));
        assert!(error.contains("requires `slice`, `chan`, or `map`"));
    }

    #[test]
    fn make_requires_integer_size_arguments() {
        let error = validate_make_call(&Type::Slice(Box::new(Type::Int)), &[Type::Bool])
            .expect_err("make should reject non-integer sizes");

        assert!(error.contains("argument 2"));
        assert!(error.contains("requires `int`"));
    }

    #[test]
    fn make_accepts_map_type_argument() {
        let result = validate_make_call(
            &Type::Map {
                key: Box::new(Type::String),
                value: Box::new(Type::Int),
            },
            &[Type::Int],
        )
        .expect("make should accept map types");

        assert_eq!(
            result,
            Type::Map {
                key: Box::new(Type::String),
                value: Box::new(Type::Int),
            }
        );
    }

    #[test]
    fn make_accepts_channel_type_argument() {
        let result = validate_make_call(&Type::Chan(Box::new(Type::Int)), &[Type::Int])
            .expect("make should accept channel types");

        assert_eq!(result, Type::Chan(Box::new(Type::Int)));
    }

    #[test]
    fn delete_accepts_map_and_matching_key() {
        let result = validate_builtin_call(
            BuiltinFunction::Delete,
            &[
                Type::Map {
                    key: Box::new(Type::String),
                    value: Box::new(Type::Int),
                },
                Type::String,
            ],
        )
        .expect("delete should accept maps");

        assert_eq!(result, Vec::<Type>::new());
    }

    #[test]
    fn delete_rejects_non_map_argument() {
        let error = validate_builtin_call(BuiltinFunction::Delete, &[Type::Int, Type::String])
            .expect_err("delete should reject non-map targets");

        assert!(error.contains("argument 1 in call to builtin `delete` requires `map`"));
    }

    #[test]
    fn close_accepts_channel_argument() {
        let result =
            validate_builtin_call(BuiltinFunction::Close, &[Type::Chan(Box::new(Type::Int))])
                .expect("close should accept channels");

        assert_eq!(result, Vec::<Type>::new());
    }

    #[test]
    fn close_rejects_non_channel_argument() {
        let error =
            validate_builtin_call(BuiltinFunction::Close, &[Type::Slice(Box::new(Type::Int))])
                .expect_err("close should reject non-channels");

        assert!(error.contains("requires `chan`"));
    }
}
