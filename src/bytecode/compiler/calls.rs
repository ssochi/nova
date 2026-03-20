use crate::bytecode::compiler::{CompileError, FunctionCompiler};
use crate::bytecode::instruction::Instruction;
use crate::semantic::model::{CheckedCall, CheckedCallArguments, Type};

impl<'a> FunctionCompiler<'a> {
    pub(super) fn compile_panic_call(
        &mut self,
        call: &CheckedCall,
        context: &str,
        deferred: bool,
    ) -> Result<(), CompileError> {
        match &call.arguments {
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
            }
            CheckedCallArguments::ExpandedCall(expanded_call) => {
                self.compile_call(expanded_call, context)?
            }
            CheckedCallArguments::Spread { .. } => {
                return Err(CompileError::new(
                    "builtin `panic` cannot be lowered with explicit `...` arguments",
                ));
            }
        }

        self.instructions.push(if deferred {
            Instruction::DeferPanic
        } else {
            Instruction::Panic
        });
        Ok(())
    }
}
