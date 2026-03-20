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
    let slice = match value {
        Value::Slice(slice) => slice,
        other => {
            return Err(RuntimeError::new(format!(
                "argument {} in call to `{}` expected `[]byte`, found `{}`",
                position,
                function.render(),
                runtime_type_name(&other)
            )));
        }
    };

    slice.byte_elements().map_err(|_| {
        RuntimeError::new(format!(
            "argument {} in call to `{}` expected `[]byte`, found a non-byte slice",
            position,
            function.render()
        ))
    })
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
) -> Result<Value, RuntimeError> {
    match function {
        PackageFunction::BytesEqual => {
            let [left, right] = expect_exact_package_arguments(function, arguments, 2)?;
            let left = expect_byte_slice_package_argument(function, 1, left)?;
            let right = expect_byte_slice_package_argument(function, 2, right)?;
            Ok(Value::Boolean(left == right))
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
            Ok(Value::Boolean(found))
        }
        PackageFunction::BytesHasPrefix => {
            let [value, prefix] = expect_exact_package_arguments(function, arguments, 2)?;
            let value = expect_byte_slice_package_argument(function, 1, value)?;
            let prefix = expect_byte_slice_package_argument(function, 2, prefix)?;
            Ok(Value::Boolean(value.starts_with(prefix.as_slice())))
        }
        PackageFunction::BytesJoin => {
            let [elements, separator] = expect_exact_package_arguments(function, arguments, 2)?;
            let elements = expect_byte_slice_slice_package_argument(function, 1, elements)?;
            let separator = expect_byte_slice_package_argument(function, 2, separator)?;
            Ok(Value::Slice(SliceValue::from_bytes(&join_byte_slices(
                &elements, &separator,
            ))))
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
            Ok(Value::Slice(SliceValue::from_bytes(&repeated)))
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
