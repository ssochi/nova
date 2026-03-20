use super::super::VirtualMachine;
use crate::builtin::BuiltinFunction;
use crate::bytecode::instruction::{CompiledFunction, Instruction, Program, ValueType};

#[test]
fn execute_type_assertions_for_concrete_and_any_targets() {
    let program = Program {
        package_name: "main".to_string(),
        entry_function: "main".to_string(),
        entry_function_index: 0,
        functions: vec![CompiledFunction {
            name: "main".to_string(),
            parameter_count: 0,
            variadic_element_type: None,
            return_types: Vec::new(),
            local_names: vec![],
            instructions: vec![
                Instruction::PushString("go".to_string()),
                Instruction::BoxAny(ValueType::String),
                Instruction::TypeAssert(ValueType::String),
                Instruction::CallBuiltin(BuiltinFunction::Println, 1),
                Instruction::PushInt(7),
                Instruction::BoxAny(ValueType::Int),
                Instruction::TypeAssert(ValueType::Any),
                Instruction::PushInt(7),
                Instruction::Equal,
                Instruction::CallBuiltin(BuiltinFunction::Println, 1),
                Instruction::Return,
            ],
        }],
    };

    let output = VirtualMachine::new()
        .execute(&program)
        .expect("program should execute")
        .render_output();

    assert_eq!(output, "go\ntrue\n");
}

#[test]
fn execute_type_assertion_preserves_typed_nil_slice_payload() {
    let program = Program {
        package_name: "main".to_string(),
        entry_function: "main".to_string(),
        entry_function_index: 0,
        functions: vec![CompiledFunction {
            name: "main".to_string(),
            parameter_count: 0,
            variadic_element_type: None,
            return_types: Vec::new(),
            local_names: vec![],
            instructions: vec![
                Instruction::PushNilSlice,
                Instruction::BoxAny(ValueType::Slice(Box::new(ValueType::Byte))),
                Instruction::TypeAssert(ValueType::Slice(Box::new(ValueType::Byte))),
                Instruction::PushNilSlice,
                Instruction::Equal,
                Instruction::CallBuiltin(BuiltinFunction::Println, 1),
                Instruction::Return,
            ],
        }],
    };

    let output = VirtualMachine::new()
        .execute(&program)
        .expect("program should execute")
        .render_output();

    assert_eq!(output, "true\n");
}

#[test]
fn execute_type_assertion_panics_for_nil_interface() {
    let program = Program {
        package_name: "main".to_string(),
        entry_function: "main".to_string(),
        entry_function_index: 0,
        functions: vec![CompiledFunction {
            name: "main".to_string(),
            parameter_count: 0,
            variadic_element_type: None,
            return_types: Vec::new(),
            local_names: vec![],
            instructions: vec![
                Instruction::PushNilInterface,
                Instruction::TypeAssert(ValueType::String),
                Instruction::Return,
            ],
        }],
    };

    let error = VirtualMachine::new()
        .execute(&program)
        .expect_err("type assertion should fail");

    assert_eq!(
        error.to_string(),
        "panic: interface conversion: interface {} is nil, not string"
    );
}

#[test]
fn execute_type_assertion_panics_for_mismatched_dynamic_type() {
    let program = Program {
        package_name: "main".to_string(),
        entry_function: "main".to_string(),
        entry_function_index: 0,
        functions: vec![CompiledFunction {
            name: "main".to_string(),
            parameter_count: 0,
            variadic_element_type: None,
            return_types: Vec::new(),
            local_names: vec![],
            instructions: vec![
                Instruction::PushString("go".to_string()),
                Instruction::BoxAny(ValueType::String),
                Instruction::TypeAssert(ValueType::Slice(Box::new(ValueType::Byte))),
                Instruction::Return,
            ],
        }],
    };

    let error = VirtualMachine::new()
        .execute(&program)
        .expect_err("type assertion should fail");

    assert_eq!(
        error.to_string(),
        "panic: interface conversion: interface {} is string, not []byte"
    );
}

#[test]
fn execute_comma_ok_type_assertion_returns_value_and_true_for_match() {
    let program = Program {
        package_name: "main".to_string(),
        entry_function: "main".to_string(),
        entry_function_index: 0,
        functions: vec![CompiledFunction {
            name: "main".to_string(),
            parameter_count: 0,
            variadic_element_type: None,
            return_types: Vec::new(),
            local_names: vec![],
            instructions: vec![
                Instruction::PushString("go".to_string()),
                Instruction::BoxAny(ValueType::String),
                Instruction::TypeAssertOk(ValueType::String),
                Instruction::CallBuiltin(BuiltinFunction::Println, 2),
                Instruction::Return,
            ],
        }],
    };

    let output = VirtualMachine::new()
        .execute(&program)
        .expect("program should execute")
        .render_output();

    assert_eq!(output, "go true\n");
}

#[test]
fn execute_comma_ok_type_assertion_returns_zero_value_and_false_for_mismatch() {
    let program = Program {
        package_name: "main".to_string(),
        entry_function: "main".to_string(),
        entry_function_index: 0,
        functions: vec![CompiledFunction {
            name: "main".to_string(),
            parameter_count: 0,
            variadic_element_type: None,
            return_types: Vec::new(),
            local_names: vec![],
            instructions: vec![
                Instruction::PushNilInterface,
                Instruction::TypeAssertOk(ValueType::String),
                Instruction::CallBuiltin(BuiltinFunction::Println, 2),
                Instruction::Return,
            ],
        }],
    };

    let output = VirtualMachine::new()
        .execute(&program)
        .expect("program should execute")
        .render_output();

    assert_eq!(output, " false\n");
}
