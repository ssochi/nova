use super::{RuntimeError, VirtualMachine};
use crate::bytecode::instruction::Program;
use crate::runtime::value::Value;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum PanicPayload {
    Value(Value),
    NilArgument,
    Message(String),
}

impl PanicPayload {
    pub(super) fn render_message(&self) -> String {
        match self {
            PanicPayload::Value(value) => value.to_string(),
            PanicPayload::NilArgument => "panic called with nil argument".to_string(),
            PanicPayload::Message(message) => message.clone(),
        }
    }
}

#[derive(Debug)]
pub(super) struct CallFrame {
    pub(super) function_index: usize,
    pub(super) pc: usize,
    pub(super) locals: Vec<Value>,
    pub(super) deferred_calls: Vec<DeferredCall>,
    pub(super) pending_return_values: Option<Vec<Value>>,
    pub(super) return_mode: ReturnMode,
}

impl CallFrame {
    pub(super) fn new(function_index: usize, local_count: usize, return_mode: ReturnMode) -> Self {
        Self {
            function_index,
            pc: 0,
            locals: vec![Value::default(); local_count],
            deferred_calls: Vec::new(),
            pending_return_values: None,
            return_mode,
        }
    }
}

#[derive(Clone, Debug)]
pub(super) enum DeferredCall {
    Builtin {
        builtin: crate::builtin::BuiltinFunction,
        arguments: Vec<Value>,
    },
    PanicValue(Value),
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
            DeferredCall::PanicValue(value) => Err(RuntimeError::panic_value(value)),
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
            ),
        }
    }

    pub(super) fn decorate_runtime_error(&self, error: RuntimeError) -> RuntimeError {
        if self.output.is_empty() {
            error
        } else {
            error.with_output(self.output.clone())
        }
    }
}
