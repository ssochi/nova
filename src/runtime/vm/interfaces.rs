use super::{RuntimeError, VirtualMachine};
use crate::bytecode::instruction::ValueType;
use crate::runtime::value::{InterfaceValue, Value};

impl VirtualMachine {
    pub(super) fn box_any(&mut self, value_type: ValueType) -> Result<(), RuntimeError> {
        let value = self.pop_value()?;
        self.stack.push(Value::boxed_any(value_type, value));
        Ok(())
    }

    pub(super) fn compare_values_for_equality(
        &self,
        left: Value,
        right: Value,
    ) -> Result<bool, RuntimeError> {
        match (left, right) {
            (Value::Interface(left), Value::Interface(right)) => {
                self.compare_interface_values(left, right)
            }
            (Value::Interface(interface), other) | (other, Value::Interface(interface)) => {
                self.compare_interface_to_value(interface, other)
            }
            (left, right) => Ok(left == right),
        }
    }

    fn compare_interface_values(
        &self,
        left: InterfaceValue,
        right: InterfaceValue,
    ) -> Result<bool, RuntimeError> {
        match (left.value_type(), right.value_type()) {
            (None, None) => Ok(true),
            (None, Some(_)) | (Some(_), None) => Ok(false),
            (Some(left_type), Some(right_type)) if left_type != right_type => Ok(false),
            (Some(_), Some(_)) => {
                let left_value = left.into_inner().ok_or_else(|| {
                    RuntimeError::new("boxed interface value was missing payload")
                })?;
                let right_value = right.into_inner().ok_or_else(|| {
                    RuntimeError::new("boxed interface value was missing payload")
                })?;
                if !left_value.is_runtime_comparable() {
                    return Err(RuntimeError::user_panic_message(
                        "runtime error: comparing uncomparable interface value",
                    ));
                }
                Ok(left_value == right_value)
            }
        }
    }

    fn compare_interface_to_value(
        &self,
        interface: InterfaceValue,
        other: Value,
    ) -> Result<bool, RuntimeError> {
        let Some(interface_type) = interface.value_type() else {
            return Ok(false);
        };
        let Some(other_type) = runtime_value_type(&other) else {
            return Ok(false);
        };
        if interface_type != &other_type {
            return Ok(false);
        }
        let value = interface
            .into_inner()
            .ok_or_else(|| RuntimeError::new("boxed interface value was missing payload"))?;
        Ok(value == other)
    }
}

fn runtime_value_type(value: &Value) -> Option<ValueType> {
    match value {
        Value::Integer(_) => Some(ValueType::Int),
        Value::Byte(_) => Some(ValueType::Byte),
        Value::Boolean(_) => Some(ValueType::Bool),
        Value::String(_) => Some(ValueType::String),
        Value::Interface(_) => None,
        Value::Slice(_) => None,
        Value::Chan(_) => None,
        Value::Map(_) => None,
    }
}
