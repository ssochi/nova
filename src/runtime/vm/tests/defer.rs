use super::super::VirtualMachine;
use crate::builtin::BuiltinFunction;
use crate::bytecode::instruction::{CompiledFunction, Instruction, Program, ValueType};

#[test]
fn execute_deferred_calls_in_lifo_order_and_discard_user_results() {
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
                    Instruction::PushString("builtin".to_string()),
                    Instruction::DeferBuiltin(BuiltinFunction::Println, 1),
                    Instruction::DeferFunction(1, 0),
                    Instruction::PushString("body".to_string()),
                    Instruction::CallBuiltin(BuiltinFunction::Println, 1),
                    Instruction::Return,
                ],
            },
            CompiledFunction {
                name: "pair".to_string(),
                parameter_count: 0,
                variadic_element_type: None,
                return_types: vec![ValueType::Int, ValueType::Int],
                local_names: vec![],
                instructions: vec![
                    Instruction::PushString("pair".to_string()),
                    Instruction::CallBuiltin(BuiltinFunction::Println, 1),
                    Instruction::PushInt(1),
                    Instruction::PushInt(2),
                    Instruction::Return,
                ],
            },
        ],
    };

    let output = VirtualMachine::new()
        .execute(&program)
        .expect("program should execute")
        .render_output();

    assert_eq!(output, "body\npair\nbuiltin\n");
}
