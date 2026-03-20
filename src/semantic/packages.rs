use crate::package::{ImportedPackage, PackageFunction};
use crate::semantic::model::Type;

#[derive(Clone)]
pub struct PackageFunctionContract {
    pub function: PackageFunction,
    pub member_name: &'static str,
    pub validator: fn(&[Type]) -> Result<Type, String>,
}

const FMT_FUNCTIONS: [PackageFunctionContract; 3] = [
    PackageFunctionContract {
        function: PackageFunction::FmtPrint,
        member_name: "Print",
        validator: validate_variadic_any_value,
    },
    PackageFunctionContract {
        function: PackageFunction::FmtPrintln,
        member_name: "Println",
        validator: validate_variadic_any_value,
    },
    PackageFunctionContract {
        function: PackageFunction::FmtSprint,
        member_name: "Sprint",
        validator: validate_fmt_sprint,
    },
];

const STRINGS_FUNCTIONS: [PackageFunctionContract; 4] = [
    PackageFunctionContract {
        function: PackageFunction::StringsContains,
        member_name: "Contains",
        validator: validate_strings_contains,
    },
    PackageFunctionContract {
        function: PackageFunction::StringsHasPrefix,
        member_name: "HasPrefix",
        validator: validate_strings_has_prefix,
    },
    PackageFunctionContract {
        function: PackageFunction::StringsJoin,
        member_name: "Join",
        validator: validate_strings_join,
    },
    PackageFunctionContract {
        function: PackageFunction::StringsRepeat,
        member_name: "Repeat",
        validator: validate_strings_repeat,
    },
];

const BYTES_FUNCTIONS: [PackageFunctionContract; 5] = [
    PackageFunctionContract {
        function: PackageFunction::BytesEqual,
        member_name: "Equal",
        validator: validate_bytes_equal,
    },
    PackageFunctionContract {
        function: PackageFunction::BytesContains,
        member_name: "Contains",
        validator: validate_bytes_contains,
    },
    PackageFunctionContract {
        function: PackageFunction::BytesHasPrefix,
        member_name: "HasPrefix",
        validator: validate_bytes_has_prefix,
    },
    PackageFunctionContract {
        function: PackageFunction::BytesJoin,
        member_name: "Join",
        validator: validate_bytes_join,
    },
    PackageFunctionContract {
        function: PackageFunction::BytesRepeat,
        member_name: "Repeat",
        validator: validate_bytes_repeat,
    },
];

pub fn resolve_import_path(path: &str) -> Option<ImportedPackage> {
    match path {
        "fmt" => Some(ImportedPackage::Fmt),
        "strings" => Some(ImportedPackage::Strings),
        "bytes" => Some(ImportedPackage::Bytes),
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
    (contract.validator)(argument_types)
}

pub fn expected_argument_types(function: PackageFunction) -> Option<Vec<Type>> {
    match function {
        PackageFunction::FmtPrint | PackageFunction::FmtPrintln | PackageFunction::FmtSprint => {
            None
        }
        PackageFunction::StringsContains | PackageFunction::StringsHasPrefix => {
            Some(vec![Type::String, Type::String])
        }
        PackageFunction::StringsJoin => {
            Some(vec![Type::Slice(Box::new(Type::String)), Type::String])
        }
        PackageFunction::StringsRepeat => Some(vec![Type::String, Type::Int]),
        PackageFunction::BytesEqual
        | PackageFunction::BytesContains
        | PackageFunction::BytesHasPrefix => Some(vec![byte_slice_type(), byte_slice_type()]),
        PackageFunction::BytesJoin => Some(vec![byte_slice_slice_type(), byte_slice_type()]),
        PackageFunction::BytesRepeat => Some(vec![byte_slice_type(), Type::Int]),
    }
}

fn package_functions(package: ImportedPackage) -> &'static [PackageFunctionContract] {
    match package {
        ImportedPackage::Fmt => &FMT_FUNCTIONS,
        ImportedPackage::Strings => &STRINGS_FUNCTIONS,
        ImportedPackage::Bytes => &BYTES_FUNCTIONS,
    }
}

fn package_function_contract(function: PackageFunction) -> &'static PackageFunctionContract {
    FMT_FUNCTIONS
        .iter()
        .chain(STRINGS_FUNCTIONS.iter())
        .chain(BYTES_FUNCTIONS.iter())
        .find(|contract| contract.function == function)
        .expect("all package functions must have contracts")
}

fn validate_variadic_any_value(argument_types: &[Type]) -> Result<Type, String> {
    for (index, argument) in argument_types.iter().enumerate() {
        if !argument.produces_value() {
            return Err(format!(
                "argument {} in call to package function must produce a value",
                index + 1
            ));
        }
    }

    Ok(Type::Void)
}

fn validate_fmt_sprint(argument_types: &[Type]) -> Result<Type, String> {
    validate_variadic_any_value(argument_types)?;
    Ok(Type::String)
}

fn validate_strings_contains(argument_types: &[Type]) -> Result<Type, String> {
    validate_exact_package_arity(PackageFunction::StringsContains, 2, argument_types.len())?;
    expect_package_argument_type(
        PackageFunction::StringsContains,
        1,
        &Type::String,
        &argument_types[0],
    )?;
    expect_package_argument_type(
        PackageFunction::StringsContains,
        2,
        &Type::String,
        &argument_types[1],
    )?;
    Ok(Type::Bool)
}

fn validate_strings_has_prefix(argument_types: &[Type]) -> Result<Type, String> {
    validate_exact_package_arity(PackageFunction::StringsHasPrefix, 2, argument_types.len())?;
    expect_package_argument_type(
        PackageFunction::StringsHasPrefix,
        1,
        &Type::String,
        &argument_types[0],
    )?;
    expect_package_argument_type(
        PackageFunction::StringsHasPrefix,
        2,
        &Type::String,
        &argument_types[1],
    )?;
    Ok(Type::Bool)
}

fn validate_strings_join(argument_types: &[Type]) -> Result<Type, String> {
    validate_exact_package_arity(PackageFunction::StringsJoin, 2, argument_types.len())?;
    let string_slice = Type::Slice(Box::new(Type::String));
    expect_package_argument_type(
        PackageFunction::StringsJoin,
        1,
        &string_slice,
        &argument_types[0],
    )?;
    expect_package_argument_type(
        PackageFunction::StringsJoin,
        2,
        &Type::String,
        &argument_types[1],
    )?;
    Ok(Type::String)
}

fn validate_strings_repeat(argument_types: &[Type]) -> Result<Type, String> {
    validate_exact_package_arity(PackageFunction::StringsRepeat, 2, argument_types.len())?;
    expect_package_argument_type(
        PackageFunction::StringsRepeat,
        1,
        &Type::String,
        &argument_types[0],
    )?;
    expect_package_argument_type(
        PackageFunction::StringsRepeat,
        2,
        &Type::Int,
        &argument_types[1],
    )?;
    Ok(Type::String)
}

fn validate_bytes_equal(argument_types: &[Type]) -> Result<Type, String> {
    validate_exact_package_arity(PackageFunction::BytesEqual, 2, argument_types.len())?;
    let byte_slice = byte_slice_type();
    expect_package_argument_type(
        PackageFunction::BytesEqual,
        1,
        &byte_slice,
        &argument_types[0],
    )?;
    expect_package_argument_type(
        PackageFunction::BytesEqual,
        2,
        &byte_slice,
        &argument_types[1],
    )?;
    Ok(Type::Bool)
}

fn validate_bytes_contains(argument_types: &[Type]) -> Result<Type, String> {
    validate_exact_package_arity(PackageFunction::BytesContains, 2, argument_types.len())?;
    let byte_slice = byte_slice_type();
    expect_package_argument_type(
        PackageFunction::BytesContains,
        1,
        &byte_slice,
        &argument_types[0],
    )?;
    expect_package_argument_type(
        PackageFunction::BytesContains,
        2,
        &byte_slice,
        &argument_types[1],
    )?;
    Ok(Type::Bool)
}

fn validate_bytes_has_prefix(argument_types: &[Type]) -> Result<Type, String> {
    validate_exact_package_arity(PackageFunction::BytesHasPrefix, 2, argument_types.len())?;
    let byte_slice = byte_slice_type();
    expect_package_argument_type(
        PackageFunction::BytesHasPrefix,
        1,
        &byte_slice,
        &argument_types[0],
    )?;
    expect_package_argument_type(
        PackageFunction::BytesHasPrefix,
        2,
        &byte_slice,
        &argument_types[1],
    )?;
    Ok(Type::Bool)
}

fn validate_bytes_join(argument_types: &[Type]) -> Result<Type, String> {
    validate_exact_package_arity(PackageFunction::BytesJoin, 2, argument_types.len())?;
    let slices = byte_slice_slice_type();
    let separator = byte_slice_type();
    expect_package_argument_type(PackageFunction::BytesJoin, 1, &slices, &argument_types[0])?;
    expect_package_argument_type(
        PackageFunction::BytesJoin,
        2,
        &separator,
        &argument_types[1],
    )?;
    Ok(byte_slice_type())
}

fn validate_bytes_repeat(argument_types: &[Type]) -> Result<Type, String> {
    validate_exact_package_arity(PackageFunction::BytesRepeat, 2, argument_types.len())?;
    let byte_slice = byte_slice_type();
    expect_package_argument_type(
        PackageFunction::BytesRepeat,
        1,
        &byte_slice,
        &argument_types[0],
    )?;
    expect_package_argument_type(
        PackageFunction::BytesRepeat,
        2,
        &Type::Int,
        &argument_types[1],
    )?;
    Ok(byte_slice)
}

fn byte_slice_type() -> Type {
    Type::Slice(Box::new(Type::Byte))
}

fn byte_slice_slice_type() -> Type {
    Type::Slice(Box::new(byte_slice_type()))
}

fn validate_exact_package_arity(
    function: PackageFunction,
    expected: usize,
    actual: usize,
) -> Result<(), String> {
    if expected == actual {
        Ok(())
    } else {
        Err(format!(
            "package function `{}` expects {} arguments, found {}",
            function.render(),
            expected,
            actual
        ))
    }
}

fn expect_package_argument_type(
    function: PackageFunction,
    position: usize,
    expected: &Type,
    actual: &Type,
) -> Result<(), String> {
    if expected == actual {
        Ok(())
    } else {
        Err(format!(
            "argument {} in call to `{}` requires `{}`, found `{}`",
            position,
            function.render(),
            expected.render(),
            actual.render()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::validate_package_call;
    use crate::package::PackageFunction;
    use crate::semantic::model::Type;

    #[test]
    fn join_accepts_string_slices() {
        let result = validate_package_call(
            PackageFunction::StringsJoin,
            &[Type::Slice(Box::new(Type::String)), Type::String],
        )
        .expect("strings.Join should accept []string and string");

        assert_eq!(result, Type::String);
    }

    #[test]
    fn repeat_rejects_non_integer_count() {
        let error =
            validate_package_call(PackageFunction::StringsRepeat, &[Type::String, Type::Bool])
                .expect_err("strings.Repeat should reject non-int counts");

        assert!(error.contains("argument 2"));
        assert!(error.contains("requires `int`"));
    }

    #[test]
    fn bytes_join_accepts_nested_byte_slices() {
        let result = validate_package_call(
            PackageFunction::BytesJoin,
            &[
                Type::Slice(Box::new(Type::Slice(Box::new(Type::Byte)))),
                Type::Slice(Box::new(Type::Byte)),
            ],
        )
        .expect("bytes.Join should accept [][]byte and []byte");

        assert_eq!(result, Type::Slice(Box::new(Type::Byte)));
    }

    #[test]
    fn bytes_repeat_rejects_non_integer_count() {
        let error = validate_package_call(
            PackageFunction::BytesRepeat,
            &[Type::Slice(Box::new(Type::Byte)), Type::Bool],
        )
        .expect_err("bytes.Repeat should reject non-int counts");

        assert!(error.contains("argument 2"));
        assert!(error.contains("requires `int`"));
    }
}
