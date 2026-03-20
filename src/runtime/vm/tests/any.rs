use super::super::VirtualMachine;
use crate::builtin::BuiltinFunction;
use crate::bytecode::instruction::{CompiledFunction, Instruction, Program, ValueType};
use crate::package::PackageFunction;

#[test]
fn execute_any_boxing_comparison_and_fmt_spread() {
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
                Instruction::PushNilInterface,
                Instruction::Equal,
                Instruction::CallBuiltin(BuiltinFunction::Println, 1),
                Instruction::PushString("boom".to_string()),
                Instruction::BoxAny(ValueType::String),
                Instruction::PushString("boom".to_string()),
                Instruction::Equal,
                Instruction::CallBuiltin(BuiltinFunction::Println, 1),
                Instruction::PushString("go".to_string()),
                Instruction::BoxAny(ValueType::String),
                Instruction::PushInt(7),
                Instruction::BoxAny(ValueType::Int),
                Instruction::PushNilInterface,
                Instruction::BuildSlice(3),
                Instruction::CallPackageSpread(PackageFunction::FmtPrintln, 0),
                Instruction::Return,
            ],
        }],
    };

    let output = VirtualMachine::new()
        .execute(&program)
        .expect("program should execute")
        .render_output();

    assert_eq!(output, "true\ntrue\ngo 7 <nil>\n");
}

#[test]
fn execute_any_equality_panics_for_uncomparable_interface_payload() {
    let program = Program {
        package_name: "main".to_string(),
        entry_function: "main".to_string(),
        entry_function_index: 0,
        functions: vec![CompiledFunction {
            name: "main".to_string(),
            parameter_count: 0,
            variadic_element_type: None,
            return_types: Vec::new(),
            local_names: vec!["boxed".to_string()],
            instructions: vec![
                Instruction::PushInt(1),
                Instruction::BuildSlice(1),
                Instruction::BoxAny(ValueType::Slice(Box::new(ValueType::Int))),
                Instruction::StoreLocal(0),
                Instruction::LoadLocal(0),
                Instruction::LoadLocal(0),
                Instruction::Equal,
                Instruction::Return,
            ],
        }],
    };

    let error = VirtualMachine::new()
        .execute(&program)
        .expect_err("uncomparable interface equality should fail");

    assert_eq!(
        error.to_string(),
        "panic: runtime error: comparing uncomparable interface value"
    );
}
