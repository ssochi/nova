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

    assert_eq!(result, vec![Type::String]);
}

#[test]
fn repeat_rejects_non_integer_count() {
    let error = validate_package_call(PackageFunction::StringsRepeat, &[Type::String, Type::Bool])
        .expect_err("strings.Repeat should reject non-int counts");

    assert!(error.contains("argument 2"));
    assert!(error.contains("requires `int`"));
}

#[test]
fn cut_reports_multi_result_contract() {
    let result = validate_package_call(PackageFunction::StringsCut, &[Type::String, Type::String])
        .expect("strings.Cut should accept two strings");

    assert_eq!(result, vec![Type::String, Type::String, Type::Bool]);
}

#[test]
fn cut_prefix_reports_multi_result_contract() {
    let result = validate_package_call(
        PackageFunction::StringsCutPrefix,
        &[Type::String, Type::String],
    )
    .expect("strings.CutPrefix should accept two strings");

    assert_eq!(result, vec![Type::String, Type::Bool]);
}

#[test]
fn strings_index_reports_integer_contract() {
    let result =
        validate_package_call(PackageFunction::StringsIndex, &[Type::String, Type::String])
            .expect("strings.Index should accept two strings");

    assert_eq!(result, vec![Type::Int]);
}

#[test]
fn strings_last_index_byte_reports_integer_contract() {
    let result = validate_package_call(
        PackageFunction::StringsLastIndexByte,
        &[Type::String, Type::Byte],
    )
    .expect("strings.LastIndexByte should accept string and byte");

    assert_eq!(result, vec![Type::Int]);
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

    assert_eq!(result, vec![Type::Slice(Box::new(Type::Byte))]);
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

#[test]
fn bytes_cut_reports_multi_result_contract() {
    let result = validate_package_call(
        PackageFunction::BytesCut,
        &[
            Type::Slice(Box::new(Type::Byte)),
            Type::Slice(Box::new(Type::Byte)),
        ],
    )
    .expect("bytes.Cut should accept two []byte values");

    assert_eq!(
        result,
        vec![
            Type::Slice(Box::new(Type::Byte)),
            Type::Slice(Box::new(Type::Byte)),
            Type::Bool
        ]
    );
}

#[test]
fn bytes_cut_suffix_reports_multi_result_contract() {
    let result = validate_package_call(
        PackageFunction::BytesCutSuffix,
        &[
            Type::Slice(Box::new(Type::Byte)),
            Type::Slice(Box::new(Type::Byte)),
        ],
    )
    .expect("bytes.CutSuffix should accept two []byte values");

    assert_eq!(result, vec![Type::Slice(Box::new(Type::Byte)), Type::Bool]);
}

#[test]
fn bytes_trim_suffix_reports_slice_contract() {
    let result = validate_package_call(
        PackageFunction::BytesTrimSuffix,
        &[
            Type::Slice(Box::new(Type::Byte)),
            Type::Slice(Box::new(Type::Byte)),
        ],
    )
    .expect("bytes.TrimSuffix should accept two []byte values");

    assert_eq!(result, vec![Type::Slice(Box::new(Type::Byte))]);
}

#[test]
fn bytes_last_index_reports_integer_contract() {
    let result = validate_package_call(
        PackageFunction::BytesLastIndex,
        &[
            Type::Slice(Box::new(Type::Byte)),
            Type::Slice(Box::new(Type::Byte)),
        ],
    )
    .expect("bytes.LastIndex should accept two []byte values");

    assert_eq!(result, vec![Type::Int]);
}
