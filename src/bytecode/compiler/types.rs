use crate::bytecode::compiler::CompileError;
use crate::bytecode::instruction::{SequenceKind, ValueType};
use crate::semantic::model::Type;

pub(super) fn lower_value_type(ty: &Type) -> Result<ValueType, CompileError> {
    match ty {
        Type::Int => Ok(ValueType::Int),
        Type::Byte => Ok(ValueType::Byte),
        Type::Bool => Ok(ValueType::Bool),
        Type::String => Ok(ValueType::String),
        Type::Any => Ok(ValueType::Any),
        Type::UntypedNil => Err(CompileError::new(
            "runtime value types do not support untyped `nil`",
        )),
        Type::Slice(element) => Ok(ValueType::Slice(Box::new(lower_value_type(element)?))),
        Type::Chan(element) => Ok(ValueType::Chan(Box::new(lower_value_type(element)?))),
        Type::Map { key, value } => Ok(ValueType::Map {
            key: Box::new(lower_value_type(key)?),
            value: Box::new(lower_value_type(value)?),
        }),
        Type::Void => Err(CompileError::new(
            "runtime value types do not support `void`",
        )),
    }
}

pub(super) fn lower_result_types(types: &[Type]) -> Result<Vec<ValueType>, CompileError> {
    types.iter().map(lower_value_type).collect()
}

pub(super) fn lower_sequence_kind(ty: &Type) -> Result<SequenceKind, CompileError> {
    match ty {
        Type::Slice(_) => Ok(SequenceKind::Slice),
        Type::String => Ok(SequenceKind::String),
        _ => Err(CompileError::new(format!(
            "sequence operations do not support `{}`",
            ty.render()
        ))),
    }
}
