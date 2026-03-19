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
