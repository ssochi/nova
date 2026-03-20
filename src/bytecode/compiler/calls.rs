use crate::bytecode::compiler::{CompileError, FunctionCompiler};
use crate::bytecode::instruction::Instruction;
use crate::semantic::model::{CheckedCall, CheckedCallArguments, Type};

use super::types::lower_value_type;

impl<'a> FunctionCompiler<'a> {
    pub(super) fn compile_panic_call(
        &mut self,
        call: &CheckedCall,
        context: &str,
        deferred: bool,
    ) -> Result<(), CompileError> {
        let panic_type = match &call.arguments {
            CheckedCallArguments::Expressions(arguments) => {
                let [argument] = arguments.as_slice() else {
                    return Err(CompileError::new(
                        "builtin `panic` lowering expected exactly one argument",
                    ));
                };
                if argument.ty == Type::UntypedNil {
                    self.instructions.push(if deferred {
                        Instruction::DeferPanicNil
                    } else {
                        Instruction::PanicNil
                    });
                    return Ok(());
                }
                self.compile_expression(argument)?;
                self.expect_value(&argument.ty, context)?;
                lower_value_type(&argument.ty)?
            }
            CheckedCallArguments::ExpandedCall(expanded_call) => {
                self.compile_call(expanded_call, context)?;
                let [result_type] = expanded_call.result_types.as_slice() else {
                    return Err(CompileError::new(
                        "builtin `panic` lowering expected exactly one expanded result",
                    ));
                };
                lower_value_type(result_type)?
            }
            CheckedCallArguments::Spread { .. } => {
                return Err(CompileError::new(
                    "builtin `panic` cannot be lowered with explicit `...` arguments",
                ));
            }
        };

        self.instructions.push(if deferred {
            Instruction::DeferPanic(panic_type)
        } else {
            Instruction::Panic(panic_type)
        });

        Ok(())
    }
}
