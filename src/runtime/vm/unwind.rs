use super::{RuntimeError, VirtualMachine};
use crate::bytecode::instruction::{Program, ValueType};
use crate::runtime::value::{StringValue, Value};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum PanicPayload {
    Value { value: Value, value_type: ValueType },
    NilArgument,
    Message(String),
}

impl PanicPayload {
    pub(super) fn render_message(&self) -> String {
        match self {
            PanicPayload::Value { value, .. } => value.to_string(),
            PanicPayload::NilArgument => "panic called with nil argument".to_string(),
            PanicPayload::Message(message) => message.clone(),
        }
    }

    pub(super) fn into_recovered_value(self) -> Value {
        match self {
            PanicPayload::Value { value, value_type } => {
                if matches!(value_type, ValueType::Any) {
                    value
                } else {
                    Value::boxed_any(value_type, value)
                }
            }
            PanicPayload::NilArgument => recover_message_value("panic called with nil argument"),
            PanicPayload::Message(message) => recover_message_value(&message),
        }
    }
}

#[derive(Debug)]
pub(super) struct CallFrame {
    pub(super) function_index: usize,
    pub(super) pc: usize,
    pub(super) locals: Vec<Value>,
    pub(super) parameter_count: usize,
    pub(super) return_types: Vec<ValueType>,
    pub(super) has_named_results: bool,
    pub(super) deferred_calls: Vec<DeferredCall>,
    pub(super) pending_return_values: Option<Vec<Value>>,
    pub(super) return_mode: ReturnMode,
    pub(super) returning: bool,
    pub(super) recover_target: Option<usize>,
}

impl CallFrame {
    pub(super) fn new(
        function_index: usize,
        local_count: usize,
        parameter_count: usize,
        return_mode: ReturnMode,
        return_types: Vec<ValueType>,
        has_named_results: bool,
    ) -> Self {
        Self {
            function_index,
            pc: 0,
            locals: vec![Value::default(); local_count],
            parameter_count,
            return_types,
            has_named_results,
            deferred_calls: Vec::new(),
            pending_return_values: None,
            return_mode,
            returning: false,
            recover_target: None,
        }
    }
}

#[derive(Clone, Debug)]
pub(super) enum DeferredCall {
    Builtin {
        builtin: crate::builtin::BuiltinFunction,
        arguments: Vec<Value>,
    },
    PanicValue {
        value: Value,
        value_type: ValueType,
    },
    PanicNil,
    Package {
        function: crate::package::PackageFunction,
        arguments: Vec<Value>,
    },
    UserDefined {
        function_index: usize,
        arguments: Vec<Value>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ReturnMode {
    Normal,
    Discard,
}

impl VirtualMachine {
    pub(super) fn begin_panic(&mut self, payload: PanicPayload) {
        self.pending_panic = Some(payload);
        self.panic_depth = self.frames.len();
    }

    pub(super) fn finish_pending_panic(&mut self) -> Option<RuntimeError> {
        let payload = self.pending_panic.take()?;
        self.panic_depth = 0;
        Some(RuntimeError::from_panic_payload(payload))
    }

    pub(super) fn panic_ready_to_unwind(&self) -> bool {
        self.pending_panic.is_some()
            && self.panic_depth > 0
            && self.frames.len() == self.panic_depth
    }

    pub(super) fn resume_panic_unwind(
        &mut self,
        program: &Program,
    ) -> Result<Option<RuntimeError>, RuntimeError> {
        if self.frames.is_empty() {
            return Ok(self.finish_pending_panic());
        }

        let frame_index = self.frames.len() - 1;
        let deferred_call = self.frames[frame_index].deferred_calls.pop();
        if let Some(deferred_call) = deferred_call {
            match self.execute_deferred_call(program, frame_index, deferred_call) {
                Ok(()) => return Ok(None),
                Err(error) => {
                    if let Some(payload) = error.clone().into_panic_payload() {
                        self.begin_panic(payload);
                        return Ok(None);
                    }
                    return Err(error);
                }
            }
        }

        self.frames
            .pop()
            .ok_or_else(|| RuntimeError::new("panic unwind with no active call frame"))?;
        self.panic_depth = self.frames.len();
        if self.frames.is_empty() {
            return Ok(self.finish_pending_panic());
        }

        Ok(None)
    }

    pub(super) fn execute_deferred_call(
        &mut self,
        program: &Program,
        frame_index: usize,
        deferred_call: DeferredCall,
    ) -> Result<(), RuntimeError> {
        match deferred_call {
            DeferredCall::Builtin { builtin, arguments } => {
                let _ = self.execute_builtin_call(builtin, arguments)?;
                Ok(())
            }
            DeferredCall::PanicValue { value, value_type } => {
                Err(RuntimeError::panic_value(value, value_type))
            }
            DeferredCall::PanicNil => Err(RuntimeError::panic_nil()),
            DeferredCall::Package {
                function,
                arguments,
            } => {
                let _ = self.execute_package_function(function, arguments)?;
                Ok(())
            }
            DeferredCall::UserDefined {
                function_index,
                arguments,
            } => self.call_user_function(
                program,
                frame_index,
                function_index,
                arguments,
                ReturnMode::Discard,
                false,
                self.pending_panic.as_ref().map(|_| frame_index),
            ),
        }
    }

    pub(super) fn recover_builtin_value(&mut self) -> Result<Value, RuntimeError> {
        let Some(frame_index) = self.frames.len().checked_sub(1) else {
            return Ok(Value::nil_any());
        };
        let Some(target_frame_index) = self.frames[frame_index].recover_target else {
            return Ok(Value::nil_any());
        };
        let Some(payload) = self.pending_panic.take() else {
            return Ok(Value::nil_any());
        };
        self.panic_depth = 0;
        self.prepare_frame_for_return(target_frame_index)?;
        Ok(payload.into_recovered_value())
    }

    pub(super) fn decorate_runtime_error(&self, error: RuntimeError) -> RuntimeError {
        if self.output.is_empty() {
            error
        } else {
            error.with_output(self.output.clone())
        }
    }
}

fn recover_message_value(message: &str) -> Value {
    Value::boxed_any(
        ValueType::String,
        Value::String(StringValue::from(message.to_string())),
    )
}
