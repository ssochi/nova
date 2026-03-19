use super::VirtualMachine;
use crate::builtin::BuiltinFunction;
use crate::bytecode::instruction::{
    CompiledFunction, Instruction, Program, SequenceKind, ValueType,
};
use crate::package::PackageFunction;

#[test]
fn execute_builds_and_indexes_slices() {
    let program = Program {
        package_name: "main".to_string(),
        entry_function: "main".to_string(),
        entry_function_index: 0,
        functions: vec![CompiledFunction {
            name: "main".to_string(),
            parameter_count: 0,
            returns_value: false,
            local_names: vec!["values".to_string()],
            instructions: vec![
                Instruction::PushInt(1),
                Instruction::PushInt(2),
                Instruction::BuildSlice(2),
                Instruction::StoreLocal(0),
                Instruction::LoadLocal(0),
                Instruction::CallBuiltin(BuiltinFunction::Len, 1),
                Instruction::LoadLocal(0),
                Instruction::PushInt(1),
                Instruction::Index(SequenceKind::Slice),
                Instruction::CallBuiltin(BuiltinFunction::Println, 2),
                Instruction::Return,
            ],
        }],
    };

    let output = VirtualMachine::new()
        .execute(&program)
        .expect("program should execute")
        .render_output();
    assert_eq!(output, "2 2\n");
}

#[test]
fn execute_strings_package_functions() {
    let program = Program {
        package_name: "main".to_string(),
        entry_function: "main".to_string(),
        entry_function_index: 0,
        functions: vec![CompiledFunction {
            name: "main".to_string(),
            parameter_count: 0,
            returns_value: false,
            local_names: vec![],
            instructions: vec![
                Instruction::PushString("nova".to_string()),
                Instruction::PushString("go".to_string()),
                Instruction::PushString("go".to_string()),
                Instruction::BuildSlice(3),
                Instruction::PushString("-".to_string()),
                Instruction::CallPackage(PackageFunction::StringsJoin, 2),
                Instruction::PushString("gogo".to_string()),
                Instruction::CallPackage(PackageFunction::StringsContains, 2),
                Instruction::CallBuiltin(BuiltinFunction::Println, 1),
                Instruction::PushString("vm".to_string()),
                Instruction::PushInt(2),
                Instruction::CallPackage(PackageFunction::StringsRepeat, 2),
                Instruction::PushString("vmvm".to_string()),
                Instruction::CallPackage(PackageFunction::StringsHasPrefix, 2),
                Instruction::CallBuiltin(BuiltinFunction::Println, 1),
                Instruction::Return,
            ],
        }],
    };

    let output = VirtualMachine::new()
        .execute(&program)
        .expect("strings package functions should execute")
        .render_output();

    assert_eq!(output, "false\ntrue\n");
}

#[test]
fn execute_slice_windows_and_index_assignment() {
    let program = Program {
        package_name: "main".to_string(),
        entry_function: "main".to_string(),
        entry_function_index: 0,
        functions: vec![CompiledFunction {
            name: "main".to_string(),
            parameter_count: 0,
            returns_value: false,
            local_names: vec!["values".to_string(), "window".to_string()],
            instructions: vec![
                Instruction::PushInt(1),
                Instruction::PushInt(2),
                Instruction::PushInt(3),
                Instruction::BuildSlice(3),
                Instruction::StoreLocal(0),
                Instruction::LoadLocal(0),
                Instruction::PushInt(1),
                Instruction::Slice {
                    target_kind: SequenceKind::Slice,
                    has_low: true,
                    has_high: false,
                },
                Instruction::StoreLocal(1),
                Instruction::LoadLocal(1),
                Instruction::PushInt(0),
                Instruction::PushInt(9),
                Instruction::SetIndex,
                Instruction::LoadLocal(0),
                Instruction::PushInt(1),
                Instruction::Index(SequenceKind::Slice),
                Instruction::CallBuiltin(BuiltinFunction::Println, 1),
                Instruction::Return,
            ],
        }],
    };

    let output = VirtualMachine::new()
        .execute(&program)
        .expect("slice window program should execute")
        .render_output();

    assert_eq!(output, "9\n");
}

#[test]
fn execute_make_slice_allocation() {
    let program = Program {
        package_name: "main".to_string(),
        entry_function: "main".to_string(),
        entry_function_index: 0,
        functions: vec![CompiledFunction {
            name: "main".to_string(),
            parameter_count: 0,
            returns_value: false,
            local_names: vec!["values".to_string(), "grown".to_string()],
            instructions: vec![
                Instruction::PushInt(2),
                Instruction::PushInt(4),
                Instruction::MakeSlice {
                    element_type: ValueType::Int,
                    has_capacity: true,
                },
                Instruction::StoreLocal(0),
                Instruction::LoadLocal(0),
                Instruction::PushInt(3),
                Instruction::Slice {
                    target_kind: SequenceKind::Slice,
                    has_low: false,
                    has_high: true,
                },
                Instruction::PushInt(2),
                Instruction::Index(SequenceKind::Slice),
                Instruction::CallBuiltin(BuiltinFunction::Println, 1),
                Instruction::LoadLocal(0),
                Instruction::PushInt(9),
                Instruction::CallBuiltin(BuiltinFunction::Append, 2),
                Instruction::StoreLocal(1),
                Instruction::LoadLocal(1),
                Instruction::CallBuiltin(BuiltinFunction::Len, 1),
                Instruction::LoadLocal(1),
                Instruction::CallBuiltin(BuiltinFunction::Cap, 1),
                Instruction::CallBuiltin(BuiltinFunction::Println, 2),
                Instruction::Return,
            ],
        }],
    };

    let output = VirtualMachine::new()
        .execute(&program)
        .expect("make-allocated slice program should execute")
        .render_output();

    assert_eq!(output, "0\n3 4\n");
}

#[test]
fn execute_string_indexing_slicing_and_byte_copy() {
    let program = Program {
        package_name: "main".to_string(),
        entry_function: "main".to_string(),
        entry_function_index: 0,
        functions: vec![CompiledFunction {
            name: "main".to_string(),
            parameter_count: 0,
            returns_value: false,
            local_names: vec!["buf".to_string()],
            instructions: vec![
                Instruction::PushString("nova".to_string()),
                Instruction::PushInt(1),
                Instruction::Index(SequenceKind::String),
                Instruction::CallBuiltin(BuiltinFunction::Println, 1),
                Instruction::PushString("nova".to_string()),
                Instruction::PushInt(1),
                Instruction::PushInt(3),
                Instruction::Slice {
                    target_kind: SequenceKind::String,
                    has_low: true,
                    has_high: true,
                },
                Instruction::CallBuiltin(BuiltinFunction::Println, 1),
                Instruction::PushInt(4),
                Instruction::MakeSlice {
                    element_type: ValueType::Byte,
                    has_capacity: false,
                },
                Instruction::StoreLocal(0),
                Instruction::LoadLocal(0),
                Instruction::PushString("gopher".to_string()),
                Instruction::CallBuiltin(BuiltinFunction::Copy, 2),
                Instruction::LoadLocal(0),
                Instruction::PushInt(0),
                Instruction::Index(SequenceKind::Slice),
                Instruction::CallBuiltin(BuiltinFunction::Println, 2),
                Instruction::Return,
            ],
        }],
    };

    let output = VirtualMachine::new()
        .execute(&program)
        .expect("string byte program should execute")
        .render_output();

    assert_eq!(output, "111\nov\n4 103\n");
}
