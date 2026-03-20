use std::convert::TryFrom;

use crate::bytecode::instruction::ValueType;
use crate::package::PackageFunction;
use crate::runtime::value::{ChannelValue, MapValue, SliceValue, StringValue, Value};
use crate::runtime::vm::RuntimeError;

pub(super) fn render_builtin_arguments(arguments: &[Value]) -> String {
    arguments
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) fn render_package_arguments(arguments: &[Value], separator: &str) -> String {
    arguments
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(separator)
}

pub(super) fn expect_exact_package_arguments<const N: usize>(
    function: PackageFunction,
    arguments: Vec<Value>,
    expected: usize,
) -> Result<[Value; N], RuntimeError> {
    if arguments.len() != expected {
        return Err(RuntimeError::new(format!(
            "package function `{}` expected {} arguments, received {}",
            function.render(),
            expected,
            arguments.len()
        )));
    }

    arguments.try_into().map_err(|arguments: Vec<Value>| {
        RuntimeError::new(format!(
            "package function `{}` expected {} arguments, received {}",
            function.render(),
            expected,
            arguments.len()
        ))
    })
}

pub(super) fn expect_string_package_argument(
    function: PackageFunction,
    position: usize,
    value: Value,
) -> Result<StringValue, RuntimeError> {
    match value {
        Value::String(value) => Ok(value),
        other => Err(RuntimeError::new(format!(
            "argument {} in call to `{}` expected `string`, found `{}`",
            position,
            function.render(),
            runtime_type_name(&other)
        ))),
    }
}

pub(super) fn expect_integer_package_argument(
    function: PackageFunction,
    position: usize,
    value: Value,
) -> Result<i64, RuntimeError> {
    match value {
        Value::Integer(value) => Ok(value),
        other => Err(RuntimeError::new(format!(
            "argument {} in call to `{}` expected `int`, found `{}`",
            position,
            function.render(),
            runtime_type_name(&other)
        ))),
    }
}

pub(super) fn expect_byte_package_argument(
    function: PackageFunction,
    position: usize,
    value: Value,
) -> Result<u8, RuntimeError> {
    match value {
        Value::Byte(value) => Ok(value),
        other => Err(RuntimeError::new(format!(
            "argument {} in call to `{}` expected `byte`, found `{}`",
            position,
            function.render(),
            runtime_type_name(&other)
        ))),
    }
}

pub(super) fn expect_string_slice_package_argument(
    function: PackageFunction,
    position: usize,
    value: Value,
) -> Result<Vec<StringValue>, RuntimeError> {
    let elements = match value {
        Value::Slice(slice) => slice.visible_elements(),
        other => {
            return Err(RuntimeError::new(format!(
                "argument {} in call to `{}` expected `[]string`, found `{}`",
                position,
                function.render(),
                runtime_type_name(&other)
            )));
        }
    };

    let mut strings = Vec::with_capacity(elements.len());
    for element in elements {
        match element {
            Value::String(value) => strings.push(value),
            other => {
                return Err(RuntimeError::new(format!(
                    "argument {} in call to `{}` expected `[]string`, found `[]{}`",
                    position,
                    function.render(),
                    runtime_type_name(&other)
                )));
            }
        }
    }

    Ok(strings)
}

pub(super) fn expect_byte_slice_package_argument(
    function: PackageFunction,
    position: usize,
    value: Value,
) -> Result<Vec<u8>, RuntimeError> {
    let slice = expect_byte_slice_value(function, position, value)?;

    slice.byte_elements().map_err(|_| {
        RuntimeError::new(format!(
            "argument {} in call to `{}` expected `[]byte`, found a non-byte slice",
            position,
            function.render()
        ))
    })
}

pub(super) fn expect_byte_slice_value(
    function: PackageFunction,
    position: usize,
    value: Value,
) -> Result<SliceValue, RuntimeError> {
    match value {
        Value::Slice(slice) => Ok(slice),
        other => Err(RuntimeError::new(format!(
            "argument {} in call to `{}` expected `[]byte`, found `{}`",
            position,
            function.render(),
            runtime_type_name(&other)
        ))),
    }
}

pub(super) fn expect_byte_slice_slice_package_argument(
    function: PackageFunction,
    position: usize,
    value: Value,
) -> Result<Vec<Vec<u8>>, RuntimeError> {
    let elements = match value {
        Value::Slice(slice) => slice.visible_elements(),
        other => {
            return Err(RuntimeError::new(format!(
                "argument {} in call to `{}` expected `[][]byte`, found `{}`",
                position,
                function.render(),
                runtime_type_name(&other)
            )));
        }
    };

    let mut slices = Vec::with_capacity(elements.len());
    for element in elements {
        match element {
            Value::Slice(slice) => slices.push(slice.byte_elements().map_err(|_| {
                RuntimeError::new(format!(
                    "argument {} in call to `{}` expected `[][]byte`, found a slice with non-byte elements",
                    position,
                    function.render()
                ))
            })?),
            other => {
                return Err(RuntimeError::new(format!(
                    "argument {} in call to `{}` expected `[][]byte`, found `[]{}`",
                    position,
                    function.render(),
                    runtime_type_name(&other)
                )));
            }
        }
    }

    Ok(slices)
}

pub(super) fn execute_bytes_package_function(
    function: PackageFunction,
    arguments: Vec<Value>,
) -> Result<Vec<Value>, RuntimeError> {
    match function {
        PackageFunction::BytesEqual => {
            let [left, right] = expect_exact_package_arguments(function, arguments, 2)?;
            let left = expect_byte_slice_package_argument(function, 1, left)?;
            let right = expect_byte_slice_package_argument(function, 2, right)?;
            Ok(vec![Value::Boolean(left == right)])
        }
        PackageFunction::BytesContains => {
            let [haystack, needle] = expect_exact_package_arguments(function, arguments, 2)?;
            let haystack = expect_byte_slice_package_argument(function, 1, haystack)?;
            let needle = expect_byte_slice_package_argument(function, 2, needle)?;
            let found = if needle.is_empty() {
                true
            } else {
                haystack
                    .windows(needle.len())
                    .any(|window| window == needle.as_slice())
            };
            Ok(vec![Value::Boolean(found)])
        }
        PackageFunction::BytesHasPrefix => {
            let [value, prefix] = expect_exact_package_arguments(function, arguments, 2)?;
            let value = expect_byte_slice_package_argument(function, 1, value)?;
            let prefix = expect_byte_slice_package_argument(function, 2, prefix)?;
            Ok(vec![Value::Boolean(value.starts_with(prefix.as_slice()))])
        }
        PackageFunction::BytesHasSuffix => {
            let [value, suffix] = expect_exact_package_arguments(function, arguments, 2)?;
            let value = expect_byte_slice_package_argument(function, 1, value)?;
            let suffix = expect_byte_slice_package_argument(function, 2, suffix)?;
            Ok(vec![Value::Boolean(value.ends_with(suffix.as_slice()))])
        }
        PackageFunction::BytesIndex => {
            let [value, needle] = expect_exact_package_arguments(function, arguments, 2)?;
            let value = expect_byte_slice_value(function, 1, value)?;
            let needle = expect_byte_slice_package_argument(function, 2, needle)?;
            let index = value
                .byte_index_of(&needle)
                .map_err(|_| {
                    RuntimeError::new(format!(
                        "argument 1 in call to `{}` expected `[]byte`, found a non-byte slice",
                        function.render()
                    ))
                })?
                .map(|offset| offset as i64)
                .unwrap_or(-1);
            Ok(vec![Value::Integer(index)])
        }
        PackageFunction::BytesLastIndex => {
            let [value, needle] = expect_exact_package_arguments(function, arguments, 2)?;
            let value = expect_byte_slice_package_argument(function, 1, value)?;
            let needle = expect_byte_slice_package_argument(function, 2, needle)?;
            let index = last_subslice_index(&value, &needle)
                .map(|offset| offset as i64)
                .unwrap_or(-1);
            Ok(vec![Value::Integer(index)])
        }
        PackageFunction::BytesIndexByte => {
            let [value, needle] = expect_exact_package_arguments(function, arguments, 2)?;
            let value = expect_byte_slice_package_argument(function, 1, value)?;
            let needle = expect_byte_package_argument(function, 2, needle)?;
            let index = value
                .iter()
                .position(|value| *value == needle)
                .map(|offset| offset as i64)
                .unwrap_or(-1);
            Ok(vec![Value::Integer(index)])
        }
        PackageFunction::BytesLastIndexByte => {
            let [value, needle] = expect_exact_package_arguments(function, arguments, 2)?;
            let value = expect_byte_slice_package_argument(function, 1, value)?;
            let needle = expect_byte_package_argument(function, 2, needle)?;
            let index = value
                .iter()
                .rposition(|value| *value == needle)
                .map(|offset| offset as i64)
                .unwrap_or(-1);
            Ok(vec![Value::Integer(index)])
        }
        PackageFunction::BytesCut => {
            let [value, separator] = expect_exact_package_arguments(function, arguments, 2)?;
            let value = expect_byte_slice_value(function, 1, value)?;
            let separator = expect_byte_slice_package_argument(function, 2, separator)?;
            let found_index = value.byte_index_of(&separator).map_err(|_| {
                RuntimeError::new(format!(
                    "argument 1 in call to `{}` expected `[]byte`, found a non-byte slice",
                    function.render()
                ))
            })?;
            if let Some(index) = found_index {
                let before = value
                    .slice(0, index)
                    .map_err(|_| RuntimeError::new("bytes.Cut produced an invalid prefix slice"))?;
                let after = value
                    .slice(index + separator.len(), value.len())
                    .map_err(|_| RuntimeError::new("bytes.Cut produced an invalid suffix slice"))?;
                Ok(vec![
                    Value::Slice(before),
                    Value::Slice(after),
                    Value::Boolean(true),
                ])
            } else {
                Ok(vec![
                    Value::Slice(value),
                    Value::Slice(SliceValue::nil()),
                    Value::Boolean(false),
                ])
            }
        }
        PackageFunction::BytesCutPrefix => {
            let [value, prefix] = expect_exact_package_arguments(function, arguments, 2)?;
            let value = expect_byte_slice_value(function, 1, value)?;
            let prefix = expect_byte_slice_package_argument(function, 2, prefix)?;
            if prefix.is_empty()
                || value.has_byte_prefix(&prefix).map_err(|_| {
                    RuntimeError::new(format!(
                        "argument 1 in call to `{}` expected `[]byte`, found a non-byte slice",
                        function.render()
                    ))
                })?
            {
                let after = value.trim_byte_prefix(&prefix).map_err(|_| {
                    RuntimeError::new("bytes.CutPrefix produced an invalid suffix slice")
                })?;
                Ok(vec![Value::Slice(after), Value::Boolean(true)])
            } else {
                Ok(vec![Value::Slice(value), Value::Boolean(false)])
            }
        }
        PackageFunction::BytesCutSuffix => {
            let [value, suffix] = expect_exact_package_arguments(function, arguments, 2)?;
            let value = expect_byte_slice_value(function, 1, value)?;
            let suffix = expect_byte_slice_package_argument(function, 2, suffix)?;
            if suffix.is_empty()
                || value.has_byte_suffix(&suffix).map_err(|_| {
                    RuntimeError::new(format!(
                        "argument 1 in call to `{}` expected `[]byte`, found a non-byte slice",
                        function.render()
                    ))
                })?
            {
                let before = value.trim_byte_suffix(&suffix).map_err(|_| {
                    RuntimeError::new("bytes.CutSuffix produced an invalid prefix slice")
                })?;
                Ok(vec![Value::Slice(before), Value::Boolean(true)])
            } else {
                Ok(vec![Value::Slice(value), Value::Boolean(false)])
            }
        }
        PackageFunction::BytesTrimPrefix => {
            let [value, prefix] = expect_exact_package_arguments(function, arguments, 2)?;
            let value = expect_byte_slice_value(function, 1, value)?;
            let prefix = expect_byte_slice_package_argument(function, 2, prefix)?;
            let trimmed = value.trim_byte_prefix(&prefix).map_err(|_| {
                RuntimeError::new("bytes.TrimPrefix produced an invalid suffix slice")
            })?;
            Ok(vec![Value::Slice(trimmed)])
        }
        PackageFunction::BytesTrimSuffix => {
            let [value, suffix] = expect_exact_package_arguments(function, arguments, 2)?;
            let value = expect_byte_slice_value(function, 1, value)?;
            let suffix = expect_byte_slice_package_argument(function, 2, suffix)?;
            let trimmed = value.trim_byte_suffix(&suffix).map_err(|_| {
                RuntimeError::new("bytes.TrimSuffix produced an invalid prefix slice")
            })?;
            Ok(vec![Value::Slice(trimmed)])
        }
        PackageFunction::BytesJoin => {
            let [elements, separator] = expect_exact_package_arguments(function, arguments, 2)?;
            let elements = expect_byte_slice_slice_package_argument(function, 1, elements)?;
            let separator = expect_byte_slice_package_argument(function, 2, separator)?;
            Ok(vec![Value::Slice(SliceValue::from_bytes(
                &join_byte_slices(&elements, &separator),
            ))])
        }
        PackageFunction::BytesRepeat => {
            let [value, count] = expect_exact_package_arguments(function, arguments, 2)?;
            let value = expect_byte_slice_package_argument(function, 1, value)?;
            let count = expect_integer_package_argument(function, 2, count)?;
            if count < 0 {
                return Err(RuntimeError::new(format!(
                    "package function `{}` requires a non-negative repeat count",
                    function.render()
                )));
            }
            let repeat_count = usize::try_from(count).map_err(|_| {
                RuntimeError::new(format!(
                    "package function `{}` could not convert repeat count",
                    function.render()
                ))
            })?;
            value.len().checked_mul(repeat_count).ok_or_else(|| {
                RuntimeError::new(format!(
                    "package function `{}` overflowed the repeated byte slice size",
                    function.render()
                ))
            })?;
            let mut repeated = Vec::with_capacity(value.len() * repeat_count);
            for _ in 0..repeat_count {
                repeated.extend_from_slice(&value);
            }
            Ok(vec![Value::Slice(SliceValue::from_bytes(&repeated))])
        }
        _ => Err(RuntimeError::new(format!(
            "package function `{}` is not a staged bytes function",
            function.render()
        ))),
    }
}

pub(super) fn expect_exact_builtin_arguments<const N: usize>(
    arguments: Vec<Value>,
    expected: usize,
    builtin: &str,
) -> Result<[Value; N], RuntimeError> {
    if arguments.len() != expected {
        return Err(RuntimeError::new(format!(
            "builtin `{builtin}` expected {expected} arguments, received {}",
            arguments.len()
        )));
    }

    arguments.try_into().map_err(|arguments: Vec<Value>| {
        RuntimeError::new(format!(
            "builtin `{builtin}` expected {expected} arguments, received {}",
            arguments.len()
        ))
    })
}

pub(super) fn runtime_type_name(value: &Value) -> &'static str {
    match value {
        Value::Integer(_) => "int",
        Value::Byte(_) => "byte",
        Value::Boolean(_) => "bool",
        Value::String(_) => "string",
        Value::Slice(_) => "slice",
        Value::Chan(_) => "chan",
        Value::Map(_) => "map",
    }
}

pub(super) fn zero_value_for_type(value_type: &ValueType) -> Value {
    match value_type {
        ValueType::Int => Value::Integer(0),
        ValueType::Byte => Value::Byte(0),
        ValueType::Bool => Value::Boolean(false),
        ValueType::String => Value::String(StringValue::empty()),
        ValueType::Slice(_) => Value::Slice(SliceValue::nil()),
        ValueType::Chan(_) => Value::Chan(ChannelValue::nil()),
        ValueType::Map { .. } => Value::Map(MapValue::nil()),
    }
}

pub(super) fn normalize_slice_bound(
    bound: Option<i64>,
    default: i64,
    position: &str,
) -> Result<usize, RuntimeError> {
    let value = bound.unwrap_or(default);
    if value < 0 {
        return Err(RuntimeError::new(format!(
            "slice {position} bound {value} is out of bounds"
        )));
    }
    usize::try_from(value)
        .map_err(|_| RuntimeError::new(format!("slice {position} bound {value} is out of bounds")))
}

pub(super) fn slice_bounds_error_message(low: usize, high: usize) -> String {
    format!("slice bounds [{low}:{high}] are out of range")
}

fn join_byte_slices(elements: &[Vec<u8>], separator: &[u8]) -> Vec<u8> {
    if elements.is_empty() {
        return Vec::new();
    }

    let total_separator_bytes = separator.len() * elements.len().saturating_sub(1);
    let total_element_bytes = elements.iter().map(Vec::len).sum::<usize>();
    let mut joined = Vec::with_capacity(total_element_bytes + total_separator_bytes);
    for (index, element) in elements.iter().enumerate() {
        if index > 0 {
            joined.extend_from_slice(separator);
        }
        joined.extend_from_slice(element);
    }
    joined
}

fn last_subslice_index(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(haystack.len());
    }

    haystack
        .windows(needle.len())
        .rposition(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use super::last_subslice_index;

    #[test]
    fn last_subslice_index_matches_go_style_empty_and_missing_cases() {
        assert_eq!(last_subslice_index(b"nova-go-go", b"go"), Some(8));
        assert_eq!(last_subslice_index(b"nova", b""), Some(4));
        assert_eq!(last_subslice_index(b"nova", b"vm"), None);
    }

    #[test]
    fn last_subslice_index_handles_nil_equivalent_empty_slice_inputs() {
        assert_eq!(last_subslice_index(b"", b""), Some(0));
        assert_eq!(last_subslice_index(b"", b"x"), None);
    }
}
