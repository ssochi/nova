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
                    Instruction::Panic(ValueType::String),
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
                    Instruction::DeferPanic(ValueType::String),
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

#[test]
fn execute_recover_stops_panic_and_preserves_named_result_value() {
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
                    Instruction::CallFunction(1, 0),
                    Instruction::CallBuiltin(BuiltinFunction::Println, 1),
                    Instruction::Return,
                ],
            },
            CompiledFunction {
                name: "recovering".to_string(),
                parameter_count: 0,
                variadic_element_type: None,
                return_types: vec![ValueType::Int],
                local_names: vec!["result".to_string()],
                instructions: vec![
                    Instruction::PushInt(0),
                    Instruction::StoreLocal(0),
                    Instruction::DeferFunction(2, 0),
                    Instruction::PushInt(7),
                    Instruction::StoreLocal(0),
                    Instruction::PushString("boom".to_string()),
                    Instruction::Panic(ValueType::String),
                    Instruction::Return,
                ],
            },
            CompiledFunction {
                name: "handle".to_string(),
                parameter_count: 0,
                variadic_element_type: None,
                return_types: Vec::new(),
                local_names: vec![],
                instructions: vec![
                    Instruction::CallBuiltin(BuiltinFunction::Recover, 0),
                    Instruction::CallBuiltin(BuiltinFunction::Println, 1),
                    Instruction::Return,
                ],
            },
        ],
    };

    let result = VirtualMachine::new()
        .execute(&program)
        .expect("recover should stop the panic");

    assert_eq!(result.render_output(), "boom\n7\n");
}

#[test]
fn helper_function_called_by_deferred_recover_frame_cannot_recover() {
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
                instructions: vec![Instruction::CallFunction(1, 0), Instruction::Return],
            },
            CompiledFunction {
                name: "recovering".to_string(),
                parameter_count: 0,
                variadic_element_type: None,
                return_types: Vec::new(),
                local_names: vec![],
                instructions: vec![
                    Instruction::DeferFunction(2, 0),
                    Instruction::PushString("boom".to_string()),
                    Instruction::Panic(ValueType::String),
                    Instruction::Return,
                ],
            },
            CompiledFunction {
                name: "deferred".to_string(),
                parameter_count: 0,
                variadic_element_type: None,
                return_types: Vec::new(),
                local_names: vec![],
                instructions: vec![
                    Instruction::CallFunction(3, 0),
                    Instruction::CallBuiltin(BuiltinFunction::Println, 1),
                    Instruction::Return,
                ],
            },
            CompiledFunction {
                name: "helper".to_string(),
                parameter_count: 0,
                variadic_element_type: None,
                return_types: vec![ValueType::Any],
                local_names: vec![],
                instructions: vec![
                    Instruction::CallBuiltin(BuiltinFunction::Recover, 0),
                    Instruction::Return,
                ],
            },
        ],
    };

    let error = VirtualMachine::new()
        .execute(&program)
        .expect_err("helper recover should not stop the panic");

    assert_eq!(error.to_string(), "<nil>\npanic: boom");
}
