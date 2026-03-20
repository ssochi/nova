use super::super::VirtualMachine;
use crate::builtin::BuiltinFunction;
use crate::bytecode::instruction::{CompiledFunction, Instruction, Program, ValueType};

#[test]
fn execute_panic_unwinds_deferred_calls_across_frames() {
    let program = Program {
        package_name: "main".to_string(),
        entry_function: "main".to_string(),
        entry_function_index: 0,
        functions: vec![
            CompiledFunction {
                name: "main".to_string(),
                parameter_count: 0,
                variadic_element_type: None,
                return_types: Vec::new(),
                local_names: vec![],
                instructions: vec![
                    Instruction::PushString("outer".to_string()),
                    Instruction::DeferBuiltin(BuiltinFunction::Println, 1),
                    Instruction::CallFunction(1, 0),
                    Instruction::Return,
                ],
            },
            CompiledFunction {
                name: "inner".to_string(),
                parameter_count: 0,
                variadic_element_type: None,
                return_types: Vec::new(),
                local_names: vec![],
                instructions: vec![
                    Instruction::PushString("inner".to_string()),
                    Instruction::DeferBuiltin(BuiltinFunction::Println, 1),
                    Instruction::PushString("boom".to_string()),
                    Instruction::Panic,
                    Instruction::Return,
                ],
            },
        ],
    };

    let error = VirtualMachine::new()
        .execute(&program)
        .expect_err("panic should fail execution");

    assert_eq!(error.to_string(), "inner\nouter\npanic: boom");
}

#[test]
fn execute_deferred_panic_overrides_pending_return_values() {
    let program = Program {
        package_name: "main".to_string(),
        entry_function: "main".to_string(),
        entry_function_index: 0,
        functions: vec![
            CompiledFunction {
                name: "main".to_string(),
                parameter_count: 0,
                variadic_element_type: None,
                return_types: Vec::new(),
                local_names: vec![],
                instructions: vec![
                    Instruction::PushString("outer".to_string()),
                    Instruction::DeferBuiltin(BuiltinFunction::Println, 1),
                    Instruction::CallFunction(1, 0),
                    Instruction::Return,
                ],
            },
            CompiledFunction {
                name: "tail".to_string(),
                parameter_count: 0,
                variadic_element_type: None,
                return_types: vec![ValueType::Int],
                local_names: vec![],
                instructions: vec![
                    Instruction::PushString("later".to_string()),
                    Instruction::DeferPanic,
                    Instruction::PushString("inner".to_string()),
                    Instruction::DeferBuiltin(BuiltinFunction::Println, 1),
                    Instruction::PushInt(7),
                    Instruction::Return,
                ],
            },
        ],
    };

    let error = VirtualMachine::new()
        .execute(&program)
        .expect_err("deferred panic should fail execution");

    assert_eq!(error.to_string(), "inner\nouter\npanic: later");
}
