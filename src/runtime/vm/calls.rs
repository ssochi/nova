use super::support::zero_value_for_type;
use super::unwind::{CallFrame, ReturnMode};
use super::{ExecutionResult, RuntimeError, VirtualMachine};
use crate::bytecode::instruction::{CompiledFunction, Instruction, Program, ValueType};
use crate::runtime::value::{SliceValue, Value};

impl VirtualMachine {
    pub(super) fn pop_arguments(&mut self, arity: usize) -> Result<Vec<Value>, RuntimeError> {
        let mut arguments = Vec::with_capacity(arity);
        for _ in 0..arity {
            arguments.push(self.pop_value()?);
        }
        arguments.reverse();
        Ok(arguments)
    }

    pub(super) fn call_user_function(
        &mut self,
        program: &Program,
        caller_frame_index: usize,
        function_index: usize,
        arguments: Vec<Value>,
        return_mode: ReturnMode,
        advance_caller_pc: bool,
        recover_target: Option<usize>,
    ) -> Result<(), RuntimeError> {
        let function = program
            .functions
            .get(function_index)
            .ok_or_else(|| RuntimeError::new(format!("invalid function index {function_index}")))?;
        let fixed_parameter_count = if function.variadic_element_type.is_some() {
            function.parameter_count.saturating_sub(1)
        } else {
            function.parameter_count
        };
        if function.variadic_element_type.is_some() {
            if arguments.len() < fixed_parameter_count {
                return Err(RuntimeError::new(format!(
                    "function `{}` expected at least {} arguments, received {}",
                    function.name,
                    fixed_parameter_count,
                    arguments.len()
                )));
            }
        } else if arguments.len() != function.parameter_count {
            return Err(RuntimeError::new(format!(
                "function `{}` expected {} arguments, received {}",
                function.name,
                function.parameter_count,
                arguments.len()
            )));
        }

        if advance_caller_pc {
            self.frames[caller_frame_index].pc += 1;
        }
        let mut frame = CallFrame::new(
            function_index,
            function.local_names.len(),
            function.parameter_count,
            return_mode,
            function.return_types.clone(),
            compiled_function_has_named_results(function),
        );
        frame.recover_target = recover_target;
        if function.variadic_element_type.is_some() {
            for (index, argument) in arguments.iter().take(fixed_parameter_count).enumerate() {
                frame.locals[index] = argument.clone();
            }
            frame.locals[fixed_parameter_count] = if arguments.len() == fixed_parameter_count {
                Value::Slice(SliceValue::nil())
            } else {
                Value::Slice(SliceValue::new(arguments[fixed_parameter_count..].to_vec()))
            };
        } else {
            for (index, argument) in arguments.into_iter().enumerate() {
                frame.locals[index] = argument;
            }
        }
        self.frames.push(frame);
        Ok(())
    }

    pub(super) fn resume_return(
        &mut self,
        program: &Program,
    ) -> Result<Option<ExecutionResult>, RuntimeError> {
        let frame_index = self
            .frames
            .len()
            .checked_sub(1)
            .ok_or_else(|| RuntimeError::new("return with no active call frame"))?;
        let deferred_call = self.frames[frame_index].deferred_calls.pop();

        if let Some(deferred_call) = deferred_call {
            self.execute_deferred_call(program, frame_index, deferred_call)?;
            return Ok(None);
        }

        let frame = self
            .frames
            .pop()
            .ok_or_else(|| RuntimeError::new("return with no active call frame"))?;
        let return_values = frame
            .pending_return_values
            .ok_or_else(|| RuntimeError::new("return frame missing values"))?;
        if matches!(frame.return_mode, ReturnMode::Normal) {
            for value in return_values {
                self.stack.push(value);
            }
        }
        if self.frames.is_empty() {
            return Ok(Some(ExecutionResult {
                output: self.output.clone(),
            }));
        }

        Ok(None)
    }

    pub(super) fn prepare_frame_for_return(
        &mut self,
        frame_index: usize,
    ) -> Result<(), RuntimeError> {
        let synthesized = {
            let frame = self
                .frames
                .get(frame_index)
                .ok_or_else(|| RuntimeError::new("return with no active call frame"))?;
            if frame.pending_return_values.is_none() {
                Some(self.synthesize_return_values(frame)?)
            } else {
                None
            }
        };
        let frame = self
            .frames
            .get_mut(frame_index)
            .ok_or_else(|| RuntimeError::new("return with no active call frame"))?;
        if let Some(values) = synthesized {
            frame.pending_return_values = Some(values);
        }
        frame.returning = true;
        Ok(())
    }

    fn synthesize_return_values(&self, frame: &CallFrame) -> Result<Vec<Value>, RuntimeError> {
        if frame.has_named_results {
            let mut values = Vec::with_capacity(frame.return_types.len());
            for offset in 0..frame.return_types.len() {
                let slot = frame.parameter_count + offset;
                let value = frame.locals.get(slot).cloned().ok_or_else(|| {
                    RuntimeError::new(format!("missing named result local slot {slot}"))
                })?;
                values.push(value);
            }
            Ok(values)
        } else {
            Ok(frame.return_types.iter().map(zero_value_for_type).collect())
        }
    }
}

fn compiled_function_has_named_results(function: &CompiledFunction) -> bool {
    if function.return_types.is_empty() {
        return false;
    }

    function
        .return_types
        .iter()
        .enumerate()
        .all(
            |(index, value_type)| match function.instructions.get(index * 2..index * 2 + 2) {
                Some([instruction, Instruction::StoreLocal(slot)])
                    if *slot == function.parameter_count + index =>
                {
                    instruction_matches_zero_value(instruction, value_type)
                }
                _ => false,
            },
        )
}

fn instruction_matches_zero_value(instruction: &Instruction, value_type: &ValueType) -> bool {
    matches!(
        (instruction, value_type),
        (Instruction::PushInt(0), ValueType::Int)
            | (Instruction::PushByte(0), ValueType::Byte)
            | (Instruction::PushBool(false), ValueType::Bool)
            | (Instruction::PushNilInterface, ValueType::Any)
            | (Instruction::PushNilSlice, ValueType::Slice(_))
            | (Instruction::PushNilChan, ValueType::Chan(_))
            | (Instruction::PushNilMap, ValueType::Map { .. })
    ) || matches!(
        (instruction, value_type),
        (Instruction::PushString(value), ValueType::String) if value.is_empty()
    )
}
