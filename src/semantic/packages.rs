use crate::package::{ImportedPackage, PackageFunction};
use crate::semantic::model::Type;

#[derive(Clone)]
pub struct PackageFunctionContract {
    pub function: PackageFunction,
    pub member_name: &'static str,
    pub validator: fn(&[Type]) -> Result<Vec<Type>, String>,
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

const STRINGS_FUNCTIONS: [PackageFunctionContract; 15] = [
    PackageFunctionContract {
        function: PackageFunction::StringsCompare,
        member_name: "Compare",
        validator: validate_strings_compare,
    },
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
        function: PackageFunction::StringsHasSuffix,
        member_name: "HasSuffix",
        validator: validate_strings_has_suffix,
    },
    PackageFunctionContract {
        function: PackageFunction::StringsIndex,
        member_name: "Index",
        validator: validate_strings_index,
    },
    PackageFunctionContract {
        function: PackageFunction::StringsLastIndex,
        member_name: "LastIndex",
        validator: validate_strings_last_index,
    },
    PackageFunctionContract {
        function: PackageFunction::StringsIndexByte,
        member_name: "IndexByte",
        validator: validate_strings_index_byte,
    },
    PackageFunctionContract {
        function: PackageFunction::StringsLastIndexByte,
        member_name: "LastIndexByte",
        validator: validate_strings_last_index_byte,
    },
    PackageFunctionContract {
        function: PackageFunction::StringsCut,
        member_name: "Cut",
        validator: validate_strings_cut,
    },
    PackageFunctionContract {
        function: PackageFunction::StringsCutPrefix,
        member_name: "CutPrefix",
        validator: validate_strings_cut_prefix,
    },
    PackageFunctionContract {
        function: PackageFunction::StringsCutSuffix,
        member_name: "CutSuffix",
        validator: validate_strings_cut_suffix,
    },
    PackageFunctionContract {
        function: PackageFunction::StringsTrimPrefix,
        member_name: "TrimPrefix",
        validator: validate_strings_trim_prefix,
    },
    PackageFunctionContract {
        function: PackageFunction::StringsTrimSuffix,
        member_name: "TrimSuffix",
        validator: validate_strings_trim_suffix,
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

const BYTES_FUNCTIONS: [PackageFunctionContract; 16] = [
    PackageFunctionContract {
        function: PackageFunction::BytesCompare,
        member_name: "Compare",
        validator: validate_bytes_compare,
    },
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
        function: PackageFunction::BytesHasSuffix,
        member_name: "HasSuffix",
        validator: validate_bytes_has_suffix,
    },
    PackageFunctionContract {
        function: PackageFunction::BytesIndex,
        member_name: "Index",
        validator: validate_bytes_index,
    },
    PackageFunctionContract {
        function: PackageFunction::BytesLastIndex,
        member_name: "LastIndex",
        validator: validate_bytes_last_index,
    },
    PackageFunctionContract {
        function: PackageFunction::BytesIndexByte,
        member_name: "IndexByte",
        validator: validate_bytes_index_byte,
    },
    PackageFunctionContract {
        function: PackageFunction::BytesLastIndexByte,
        member_name: "LastIndexByte",
        validator: validate_bytes_last_index_byte,
    },
    PackageFunctionContract {
        function: PackageFunction::BytesCut,
        member_name: "Cut",
        validator: validate_bytes_cut,
    },
    PackageFunctionContract {
        function: PackageFunction::BytesCutPrefix,
        member_name: "CutPrefix",
        validator: validate_bytes_cut_prefix,
    },
    PackageFunctionContract {
        function: PackageFunction::BytesCutSuffix,
        member_name: "CutSuffix",
        validator: validate_bytes_cut_suffix,
    },
    PackageFunctionContract {
        function: PackageFunction::BytesTrimPrefix,
        member_name: "TrimPrefix",
        validator: validate_bytes_trim_prefix,
    },
    PackageFunctionContract {
        function: PackageFunction::BytesTrimSuffix,
        member_name: "TrimSuffix",
        validator: validate_bytes_trim_suffix,
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
) -> Result<Vec<Type>, String> {
    let contract = package_function_contract(function);
    (contract.validator)(argument_types)
}

pub fn expected_argument_types(function: PackageFunction) -> Option<Vec<Type>> {
    match function {
        PackageFunction::FmtPrint | PackageFunction::FmtPrintln | PackageFunction::FmtSprint => {
            None
        }
        PackageFunction::StringsCompare
        | PackageFunction::StringsContains
        | PackageFunction::StringsHasPrefix
        | PackageFunction::StringsHasSuffix
        | PackageFunction::StringsCut
        | PackageFunction::StringsCutPrefix
        | PackageFunction::StringsCutSuffix => Some(vec![Type::String, Type::String]),
        PackageFunction::StringsIndex | PackageFunction::StringsLastIndex => {
            Some(vec![Type::String, Type::String])
        }
        PackageFunction::StringsIndexByte | PackageFunction::StringsLastIndexByte => {
            Some(vec![Type::String, Type::Byte])
        }
        PackageFunction::StringsTrimPrefix | PackageFunction::StringsTrimSuffix => {
            Some(vec![Type::String, Type::String])
        }
        PackageFunction::StringsJoin => {
            Some(vec![Type::Slice(Box::new(Type::String)), Type::String])
        }
        PackageFunction::StringsRepeat => Some(vec![Type::String, Type::Int]),
        PackageFunction::BytesCompare
        | PackageFunction::BytesEqual
        | PackageFunction::BytesContains
        | PackageFunction::BytesHasPrefix
        | PackageFunction::BytesHasSuffix
        | PackageFunction::BytesCut
        | PackageFunction::BytesCutPrefix
        | PackageFunction::BytesCutSuffix => Some(vec![byte_slice_type(), byte_slice_type()]),
        PackageFunction::BytesIndex | PackageFunction::BytesLastIndex => {
            Some(vec![byte_slice_type(), byte_slice_type()])
        }
        PackageFunction::BytesIndexByte | PackageFunction::BytesLastIndexByte => {
            Some(vec![byte_slice_type(), Type::Byte])
        }
        PackageFunction::BytesTrimPrefix | PackageFunction::BytesTrimSuffix => {
            Some(vec![byte_slice_type(), byte_slice_type()])
        }
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

fn validate_variadic_any_value(argument_types: &[Type]) -> Result<Vec<Type>, String> {
    for (index, argument) in argument_types.iter().enumerate() {
        if !argument.produces_value() {
            return Err(format!(
                "argument {} in call to package function must produce a value",
                index + 1
            ));
        }
    }

    Ok(Vec::new())
}

fn validate_fmt_sprint(argument_types: &[Type]) -> Result<Vec<Type>, String> {
    validate_variadic_any_value(argument_types)?;
    Ok(vec![Type::String])
}

fn validate_strings_compare(argument_types: &[Type]) -> Result<Vec<Type>, String> {
    validate_exact_package_arity(PackageFunction::StringsCompare, 2, argument_types.len())?;
    expect_package_argument_type(
        PackageFunction::StringsCompare,
        1,
        &Type::String,
        &argument_types[0],
    )?;
    expect_package_argument_type(
        PackageFunction::StringsCompare,
        2,
        &Type::String,
        &argument_types[1],
    )?;
    Ok(vec![Type::Int])
}

fn validate_strings_contains(argument_types: &[Type]) -> Result<Vec<Type>, String> {
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
    Ok(vec![Type::Bool])
}

fn validate_strings_has_prefix(argument_types: &[Type]) -> Result<Vec<Type>, String> {
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
    Ok(vec![Type::Bool])
}

fn validate_strings_has_suffix(argument_types: &[Type]) -> Result<Vec<Type>, String> {
    validate_exact_package_arity(PackageFunction::StringsHasSuffix, 2, argument_types.len())?;
    expect_package_argument_type(
        PackageFunction::StringsHasSuffix,
        1,
        &Type::String,
        &argument_types[0],
    )?;
    expect_package_argument_type(
        PackageFunction::StringsHasSuffix,
        2,
        &Type::String,
        &argument_types[1],
    )?;
    Ok(vec![Type::Bool])
}

fn validate_strings_index(argument_types: &[Type]) -> Result<Vec<Type>, String> {
    validate_exact_package_arity(PackageFunction::StringsIndex, 2, argument_types.len())?;
    expect_package_argument_type(
        PackageFunction::StringsIndex,
        1,
        &Type::String,
        &argument_types[0],
    )?;
    expect_package_argument_type(
        PackageFunction::StringsIndex,
        2,
        &Type::String,
        &argument_types[1],
    )?;
    Ok(vec![Type::Int])
}

fn validate_strings_last_index(argument_types: &[Type]) -> Result<Vec<Type>, String> {
    validate_exact_package_arity(PackageFunction::StringsLastIndex, 2, argument_types.len())?;
    expect_package_argument_type(
        PackageFunction::StringsLastIndex,
        1,
        &Type::String,
        &argument_types[0],
    )?;
    expect_package_argument_type(
        PackageFunction::StringsLastIndex,
        2,
        &Type::String,
        &argument_types[1],
    )?;
    Ok(vec![Type::Int])
}

fn validate_strings_index_byte(argument_types: &[Type]) -> Result<Vec<Type>, String> {
    validate_exact_package_arity(PackageFunction::StringsIndexByte, 2, argument_types.len())?;
    expect_package_argument_type(
        PackageFunction::StringsIndexByte,
        1,
        &Type::String,
        &argument_types[0],
    )?;
    expect_package_argument_type(
        PackageFunction::StringsIndexByte,
        2,
        &Type::Byte,
        &argument_types[1],
    )?;
    Ok(vec![Type::Int])
}

fn validate_strings_last_index_byte(argument_types: &[Type]) -> Result<Vec<Type>, String> {
    validate_exact_package_arity(
        PackageFunction::StringsLastIndexByte,
        2,
        argument_types.len(),
    )?;
    expect_package_argument_type(
        PackageFunction::StringsLastIndexByte,
        1,
        &Type::String,
        &argument_types[0],
    )?;
    expect_package_argument_type(
        PackageFunction::StringsLastIndexByte,
        2,
        &Type::Byte,
        &argument_types[1],
    )?;
    Ok(vec![Type::Int])
}

fn validate_strings_cut(argument_types: &[Type]) -> Result<Vec<Type>, String> {
    validate_exact_package_arity(PackageFunction::StringsCut, 2, argument_types.len())?;
    expect_package_argument_type(
        PackageFunction::StringsCut,
        1,
        &Type::String,
        &argument_types[0],
    )?;
    expect_package_argument_type(
        PackageFunction::StringsCut,
        2,
        &Type::String,
        &argument_types[1],
    )?;
    Ok(vec![Type::String, Type::String, Type::Bool])
}

fn validate_strings_cut_prefix(argument_types: &[Type]) -> Result<Vec<Type>, String> {
    validate_exact_package_arity(PackageFunction::StringsCutPrefix, 2, argument_types.len())?;
    expect_package_argument_type(
        PackageFunction::StringsCutPrefix,
        1,
        &Type::String,
        &argument_types[0],
    )?;
    expect_package_argument_type(
        PackageFunction::StringsCutPrefix,
        2,
        &Type::String,
        &argument_types[1],
    )?;
    Ok(vec![Type::String, Type::Bool])
}

fn validate_strings_cut_suffix(argument_types: &[Type]) -> Result<Vec<Type>, String> {
    validate_exact_package_arity(PackageFunction::StringsCutSuffix, 2, argument_types.len())?;
    expect_package_argument_type(
        PackageFunction::StringsCutSuffix,
        1,
        &Type::String,
        &argument_types[0],
    )?;
    expect_package_argument_type(
        PackageFunction::StringsCutSuffix,
        2,
        &Type::String,
        &argument_types[1],
    )?;
    Ok(vec![Type::String, Type::Bool])
}

fn validate_strings_trim_prefix(argument_types: &[Type]) -> Result<Vec<Type>, String> {
    validate_exact_package_arity(PackageFunction::StringsTrimPrefix, 2, argument_types.len())?;
    expect_package_argument_type(
        PackageFunction::StringsTrimPrefix,
        1,
        &Type::String,
        &argument_types[0],
    )?;
    expect_package_argument_type(
        PackageFunction::StringsTrimPrefix,
        2,
        &Type::String,
        &argument_types[1],
    )?;
    Ok(vec![Type::String])
}

fn validate_strings_trim_suffix(argument_types: &[Type]) -> Result<Vec<Type>, String> {
    validate_exact_package_arity(PackageFunction::StringsTrimSuffix, 2, argument_types.len())?;
    expect_package_argument_type(
        PackageFunction::StringsTrimSuffix,
        1,
        &Type::String,
        &argument_types[0],
    )?;
    expect_package_argument_type(
        PackageFunction::StringsTrimSuffix,
        2,
        &Type::String,
        &argument_types[1],
    )?;
    Ok(vec![Type::String])
}

fn validate_strings_join(argument_types: &[Type]) -> Result<Vec<Type>, String> {
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
    Ok(vec![Type::String])
}

fn validate_strings_repeat(argument_types: &[Type]) -> Result<Vec<Type>, String> {
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
    Ok(vec![Type::String])
}

fn validate_bytes_compare(argument_types: &[Type]) -> Result<Vec<Type>, String> {
    validate_exact_package_arity(PackageFunction::BytesCompare, 2, argument_types.len())?;
    let byte_slice = byte_slice_type();
    expect_package_argument_type(
        PackageFunction::BytesCompare,
        1,
        &byte_slice,
        &argument_types[0],
    )?;
    expect_package_argument_type(
        PackageFunction::BytesCompare,
        2,
        &byte_slice,
        &argument_types[1],
    )?;
    Ok(vec![Type::Int])
}

fn validate_bytes_equal(argument_types: &[Type]) -> Result<Vec<Type>, String> {
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
    Ok(vec![Type::Bool])
}

fn validate_bytes_contains(argument_types: &[Type]) -> Result<Vec<Type>, String> {
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
    Ok(vec![Type::Bool])
}

fn validate_bytes_has_prefix(argument_types: &[Type]) -> Result<Vec<Type>, String> {
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
    Ok(vec![Type::Bool])
}

fn validate_bytes_has_suffix(argument_types: &[Type]) -> Result<Vec<Type>, String> {
    validate_exact_package_arity(PackageFunction::BytesHasSuffix, 2, argument_types.len())?;
    let byte_slice = byte_slice_type();
    expect_package_argument_type(
        PackageFunction::BytesHasSuffix,
        1,
        &byte_slice,
        &argument_types[0],
    )?;
    expect_package_argument_type(
        PackageFunction::BytesHasSuffix,
        2,
        &byte_slice,
        &argument_types[1],
    )?;
    Ok(vec![Type::Bool])
}

fn validate_bytes_index(argument_types: &[Type]) -> Result<Vec<Type>, String> {
    validate_exact_package_arity(PackageFunction::BytesIndex, 2, argument_types.len())?;
    let byte_slice = byte_slice_type();
    expect_package_argument_type(
        PackageFunction::BytesIndex,
        1,
        &byte_slice,
        &argument_types[0],
    )?;
    expect_package_argument_type(
        PackageFunction::BytesIndex,
        2,
        &byte_slice,
        &argument_types[1],
    )?;
    Ok(vec![Type::Int])
}

fn validate_bytes_last_index(argument_types: &[Type]) -> Result<Vec<Type>, String> {
    validate_exact_package_arity(PackageFunction::BytesLastIndex, 2, argument_types.len())?;
    let byte_slice = byte_slice_type();
    expect_package_argument_type(
        PackageFunction::BytesLastIndex,
        1,
        &byte_slice,
        &argument_types[0],
    )?;
    expect_package_argument_type(
        PackageFunction::BytesLastIndex,
        2,
        &byte_slice,
        &argument_types[1],
    )?;
    Ok(vec![Type::Int])
}

fn validate_bytes_index_byte(argument_types: &[Type]) -> Result<Vec<Type>, String> {
    validate_exact_package_arity(PackageFunction::BytesIndexByte, 2, argument_types.len())?;
    let byte_slice = byte_slice_type();
    expect_package_argument_type(
        PackageFunction::BytesIndexByte,
        1,
        &byte_slice,
        &argument_types[0],
    )?;
    expect_package_argument_type(
        PackageFunction::BytesIndexByte,
        2,
        &Type::Byte,
        &argument_types[1],
    )?;
    Ok(vec![Type::Int])
}

fn validate_bytes_last_index_byte(argument_types: &[Type]) -> Result<Vec<Type>, String> {
    validate_exact_package_arity(PackageFunction::BytesLastIndexByte, 2, argument_types.len())?;
    let byte_slice = byte_slice_type();
    expect_package_argument_type(
        PackageFunction::BytesLastIndexByte,
        1,
        &byte_slice,
        &argument_types[0],
    )?;
    expect_package_argument_type(
        PackageFunction::BytesLastIndexByte,
        2,
        &Type::Byte,
        &argument_types[1],
    )?;
    Ok(vec![Type::Int])
}

fn validate_bytes_cut(argument_types: &[Type]) -> Result<Vec<Type>, String> {
    validate_exact_package_arity(PackageFunction::BytesCut, 2, argument_types.len())?;
    let byte_slice = byte_slice_type();
    expect_package_argument_type(
        PackageFunction::BytesCut,
        1,
        &byte_slice,
        &argument_types[0],
    )?;
    expect_package_argument_type(
        PackageFunction::BytesCut,
        2,
        &byte_slice,
        &argument_types[1],
    )?;
    Ok(vec![byte_slice.clone(), byte_slice, Type::Bool])
}

fn validate_bytes_cut_prefix(argument_types: &[Type]) -> Result<Vec<Type>, String> {
    validate_exact_package_arity(PackageFunction::BytesCutPrefix, 2, argument_types.len())?;
    let byte_slice = byte_slice_type();
    expect_package_argument_type(
        PackageFunction::BytesCutPrefix,
        1,
        &byte_slice,
        &argument_types[0],
    )?;
    expect_package_argument_type(
        PackageFunction::BytesCutPrefix,
        2,
        &byte_slice,
        &argument_types[1],
    )?;
    Ok(vec![byte_slice, Type::Bool])
}

fn validate_bytes_cut_suffix(argument_types: &[Type]) -> Result<Vec<Type>, String> {
    validate_exact_package_arity(PackageFunction::BytesCutSuffix, 2, argument_types.len())?;
    let byte_slice = byte_slice_type();
    expect_package_argument_type(
        PackageFunction::BytesCutSuffix,
        1,
        &byte_slice,
        &argument_types[0],
    )?;
    expect_package_argument_type(
        PackageFunction::BytesCutSuffix,
        2,
        &byte_slice,
        &argument_types[1],
    )?;
    Ok(vec![byte_slice, Type::Bool])
}

fn validate_bytes_trim_prefix(argument_types: &[Type]) -> Result<Vec<Type>, String> {
    validate_exact_package_arity(PackageFunction::BytesTrimPrefix, 2, argument_types.len())?;
    let byte_slice = byte_slice_type();
    expect_package_argument_type(
        PackageFunction::BytesTrimPrefix,
        1,
        &byte_slice,
        &argument_types[0],
    )?;
    expect_package_argument_type(
        PackageFunction::BytesTrimPrefix,
        2,
        &byte_slice,
        &argument_types[1],
    )?;
    Ok(vec![byte_slice])
}

fn validate_bytes_trim_suffix(argument_types: &[Type]) -> Result<Vec<Type>, String> {
    validate_exact_package_arity(PackageFunction::BytesTrimSuffix, 2, argument_types.len())?;
    let byte_slice = byte_slice_type();
    expect_package_argument_type(
        PackageFunction::BytesTrimSuffix,
        1,
        &byte_slice,
        &argument_types[0],
    )?;
    expect_package_argument_type(
        PackageFunction::BytesTrimSuffix,
        2,
        &byte_slice,
        &argument_types[1],
    )?;
    Ok(vec![byte_slice])
}

fn validate_bytes_join(argument_types: &[Type]) -> Result<Vec<Type>, String> {
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
    Ok(vec![byte_slice_type()])
}

fn validate_bytes_repeat(argument_types: &[Type]) -> Result<Vec<Type>, String> {
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
    Ok(vec![byte_slice])
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
mod tests;
