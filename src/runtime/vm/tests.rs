use super::VirtualMachine;
use crate::builtin::BuiltinFunction;
use crate::bytecode::instruction::{
    CompiledFunction, Instruction, Program, SequenceKind, ValueType,
};
use crate::conversion::ConversionKind;
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
            variadic_element_type: None,
            return_types: Vec::new(),
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
fn execute_compare_package_functions() {
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
                Instruction::PushString("go".to_string()),
                Instruction::CallPackage(PackageFunction::StringsCompare, 2),
                Instruction::CallBuiltin(BuiltinFunction::Println, 1),
                Instruction::PushNilSlice,
                Instruction::PushByte(b'g'),
                Instruction::PushByte(b'o'),
                Instruction::BuildSlice(2),
                Instruction::CallPackage(PackageFunction::BytesCompare, 2),
                Instruction::CallBuiltin(BuiltinFunction::Println, 1),
                Instruction::Return,
            ],
        }],
    };

    let output = VirtualMachine::new()
        .execute(&program)
        .expect("compare package functions should execute")
        .render_output();

    assert_eq!(output, "0\n-1\n");
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
            variadic_element_type: None,
            return_types: Vec::new(),
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
fn execute_cut_prefix_and_suffix_package_functions() {
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
                Instruction::PushString("nova-go".to_string()),
                Instruction::PushString("nova-".to_string()),
                Instruction::CallPackage(PackageFunction::StringsCutPrefix, 2),
                Instruction::CallBuiltin(BuiltinFunction::Println, 2),
                Instruction::PushString("nova-go".to_string()),
                Instruction::PushString("-go".to_string()),
                Instruction::CallPackage(PackageFunction::StringsCutSuffix, 2),
                Instruction::CallBuiltin(BuiltinFunction::Println, 2),
                Instruction::PushByte(b'n'),
                Instruction::PushByte(b'o'),
                Instruction::PushByte(b'v'),
                Instruction::PushByte(b'a'),
                Instruction::BuildSlice(4),
                Instruction::PushByte(b'g'),
                Instruction::PushByte(b'o'),
                Instruction::BuildSlice(2),
                Instruction::CallPackage(PackageFunction::BytesCutPrefix, 2),
                Instruction::CallBuiltin(BuiltinFunction::Println, 2),
                Instruction::Return,
            ],
        }],
    };

    let output = VirtualMachine::new()
        .execute(&program)
        .expect("cut prefix/suffix functions should execute")
        .render_output();

    assert_eq!(output, "go true\nnova true\n[110 111 118 97] false\n");
}

#[test]
fn execute_index_suffix_and_trim_package_functions() {
    let program = Program {
        package_name: "main".to_string(),
        entry_function: "main".to_string(),
        entry_function_index: 0,
        functions: vec![CompiledFunction {
            name: "main".to_string(),
            parameter_count: 0,
            variadic_element_type: None,
            return_types: Vec::new(),
            local_names: vec!["raw".to_string()],
            instructions: vec![
                Instruction::PushString("nova-go-go".to_string()),
                Instruction::PushString("go".to_string()),
                Instruction::CallPackage(PackageFunction::StringsIndex, 2),
                Instruction::CallBuiltin(BuiltinFunction::Println, 1),
                Instruction::PushString("nova-go-go".to_string()),
                Instruction::PushString("go".to_string()),
                Instruction::CallPackage(PackageFunction::StringsHasSuffix, 2),
                Instruction::CallBuiltin(BuiltinFunction::Println, 1),
                Instruction::PushString("nova-go-go".to_string()),
                Instruction::PushString("nova-".to_string()),
                Instruction::CallPackage(PackageFunction::StringsTrimPrefix, 2),
                Instruction::CallBuiltin(BuiltinFunction::Println, 1),
                Instruction::PushNilSlice,
                Instruction::StoreLocal(0),
                Instruction::LoadLocal(0),
                Instruction::PushNilSlice,
                Instruction::CallPackage(PackageFunction::BytesTrimPrefix, 2),
                Instruction::PushNilSlice,
                Instruction::Equal,
                Instruction::CallBuiltin(BuiltinFunction::Println, 1),
                Instruction::PushByte(b'n'),
                Instruction::PushByte(b'o'),
                Instruction::PushByte(b'v'),
                Instruction::PushByte(b'a'),
                Instruction::PushByte(b'-'),
                Instruction::PushByte(b'g'),
                Instruction::PushByte(b'o'),
                Instruction::BuildSlice(7),
                Instruction::PushByte(b'g'),
                Instruction::PushByte(b'o'),
                Instruction::BuildSlice(2),
                Instruction::CallPackage(PackageFunction::BytesIndex, 2),
                Instruction::CallBuiltin(BuiltinFunction::Println, 1),
                Instruction::Return,
            ],
        }],
    };

    let output = VirtualMachine::new()
        .execute(&program)
        .expect("index/suffix/trim package functions should execute")
        .render_output();

    assert_eq!(output, "5\ntrue\ngo-go\ntrue\n5\n");
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
            variadic_element_type: None,
            return_types: Vec::new(),
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
            variadic_element_type: None,
            return_types: Vec::new(),
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
            variadic_element_type: None,
            return_types: Vec::new(),
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

#[test]
fn execute_byte_multiply_and_divide() {
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
                Instruction::PushByte(6),
                Instruction::PushByte(2),
                Instruction::Multiply,
                Instruction::CallBuiltin(BuiltinFunction::Println, 1),
                Instruction::PushByte(6),
                Instruction::PushByte(2),
                Instruction::Divide,
                Instruction::CallBuiltin(BuiltinFunction::Println, 1),
                Instruction::Return,
            ],
        }],
    };

    let output = VirtualMachine::new()
        .execute(&program)
        .expect("byte arithmetic program should execute")
        .render_output();

    assert_eq!(output, "12\n3\n");
}

#[test]
fn execute_string_byte_conversions() {
    let program = Program {
        package_name: "main".to_string(),
        entry_function: "main".to_string(),
        entry_function_index: 0,
        functions: vec![CompiledFunction {
            name: "main".to_string(),
            parameter_count: 0,
            variadic_element_type: None,
            return_types: Vec::new(),
            local_names: vec!["bytes".to_string()],
            instructions: vec![
                Instruction::PushString("nova".to_string()),
                Instruction::Convert(ConversionKind::StringToBytes),
                Instruction::StoreLocal(0),
                Instruction::LoadLocal(0),
                Instruction::PushInt(0),
                Instruction::PushByte(88),
                Instruction::SetIndex,
                Instruction::LoadLocal(0),
                Instruction::Convert(ConversionKind::BytesToString),
                Instruction::CallBuiltin(BuiltinFunction::Println, 1),
                Instruction::Return,
            ],
        }],
    };

    let output = VirtualMachine::new()
        .execute(&program)
        .expect("conversion program should execute")
        .render_output();

    assert_eq!(output, "Xova\n");
}

#[test]
fn execute_maps_with_nil_reads_and_updates() {
    let map_type = ValueType::Map {
        key: Box::new(ValueType::String),
        value: Box::new(ValueType::Int),
    };
    let program = Program {
        package_name: "main".to_string(),
        entry_function: "main".to_string(),
        entry_function_index: 0,
        functions: vec![CompiledFunction {
            name: "main".to_string(),
            parameter_count: 0,
            variadic_element_type: None,
            return_types: Vec::new(),
            local_names: vec!["counts".to_string()],
            instructions: vec![
                Instruction::PushNilMap,
                Instruction::StoreLocal(0),
                Instruction::LoadLocal(0),
                Instruction::CallBuiltin(BuiltinFunction::Len, 1),
                Instruction::LoadLocal(0),
                Instruction::PushString("nova".to_string()),
                Instruction::IndexMap(map_type.clone()),
                Instruction::CallBuiltin(BuiltinFunction::Println, 2),
                Instruction::PushInt(2),
                Instruction::MakeMap {
                    map_type: map_type.clone(),
                    has_hint: true,
                },
                Instruction::StoreLocal(0),
                Instruction::LoadLocal(0),
                Instruction::PushString("nova".to_string()),
                Instruction::PushInt(3),
                Instruction::SetMapIndex,
                Instruction::LoadLocal(0),
                Instruction::PushString("nova".to_string()),
                Instruction::IndexMap(map_type),
                Instruction::CallBuiltin(BuiltinFunction::Println, 1),
                Instruction::Return,
            ],
        }],
    };

    let output = VirtualMachine::new()
        .execute(&program)
        .expect("map program should execute")
        .render_output();

    assert_eq!(output, "0 0\n3\n");
}

#[test]
fn execute_map_literals_and_delete() {
    let map_type = ValueType::Map {
        key: Box::new(ValueType::String),
        value: Box::new(ValueType::Int),
    };
    let program = Program {
        package_name: "main".to_string(),
        entry_function: "main".to_string(),
        entry_function_index: 0,
        functions: vec![CompiledFunction {
            name: "main".to_string(),
            parameter_count: 0,
            variadic_element_type: None,
            return_types: Vec::new(),
            local_names: vec!["counts".to_string(), "nil_counts".to_string()],
            instructions: vec![
                Instruction::PushString("nova".to_string()),
                Instruction::PushInt(3),
                Instruction::PushString("go".to_string()),
                Instruction::PushInt(2),
                Instruction::BuildMap {
                    map_type: map_type.clone(),
                    entry_count: 2,
                },
                Instruction::StoreLocal(0),
                Instruction::LoadLocal(0),
                Instruction::PushString("go".to_string()),
                Instruction::CallBuiltin(BuiltinFunction::Delete, 2),
                Instruction::LoadLocal(0),
                Instruction::CallBuiltin(BuiltinFunction::Len, 1),
                Instruction::LoadLocal(0),
                Instruction::PushString("nova".to_string()),
                Instruction::IndexMap(map_type.clone()),
                Instruction::LoadLocal(0),
                Instruction::PushString("go".to_string()),
                Instruction::IndexMap(map_type.clone()),
                Instruction::CallBuiltin(BuiltinFunction::Println, 3),
                Instruction::PushNilMap,
                Instruction::StoreLocal(1),
                Instruction::LoadLocal(1),
                Instruction::PushString("ghost".to_string()),
                Instruction::CallBuiltin(BuiltinFunction::Delete, 2),
                Instruction::LoadLocal(1),
                Instruction::CallBuiltin(BuiltinFunction::Len, 1),
                Instruction::CallBuiltin(BuiltinFunction::Println, 1),
                Instruction::Return,
            ],
        }],
    };

    let output = VirtualMachine::new()
        .execute(&program)
        .expect("map literal program should execute")
        .render_output();

    assert_eq!(output, "1 3 0\n0\n");
}

#[test]
fn execute_map_literals_keep_last_duplicate_key() {
    let map_type = ValueType::Map {
        key: Box::new(ValueType::String),
        value: Box::new(ValueType::Int),
    };
    let program = Program {
        package_name: "main".to_string(),
        entry_function: "main".to_string(),
        entry_function_index: 0,
        functions: vec![CompiledFunction {
            name: "main".to_string(),
            parameter_count: 0,
            variadic_element_type: None,
            return_types: Vec::new(),
            local_names: vec!["counts".to_string()],
            instructions: vec![
                Instruction::PushString("nova".to_string()),
                Instruction::PushInt(1),
                Instruction::PushString("nova".to_string()),
                Instruction::PushInt(5),
                Instruction::BuildMap {
                    map_type: map_type.clone(),
                    entry_count: 2,
                },
                Instruction::StoreLocal(0),
                Instruction::LoadLocal(0),
                Instruction::CallBuiltin(BuiltinFunction::Len, 1),
                Instruction::LoadLocal(0),
                Instruction::PushString("nova".to_string()),
                Instruction::IndexMap(map_type),
                Instruction::CallBuiltin(BuiltinFunction::Println, 2),
                Instruction::Return,
            ],
        }],
    };

    let output = VirtualMachine::new()
        .execute(&program)
        .expect("duplicate-key map literal should execute")
        .render_output();

    assert_eq!(output, "1 5\n");
}

#[test]
fn execute_nil_map_assignment_fails() {
    let program = Program {
        package_name: "main".to_string(),
        entry_function: "main".to_string(),
        entry_function_index: 0,
        functions: vec![CompiledFunction {
            name: "main".to_string(),
            parameter_count: 0,
            variadic_element_type: None,
            return_types: Vec::new(),
            local_names: vec!["counts".to_string()],
            instructions: vec![
                Instruction::PushNilMap,
                Instruction::StoreLocal(0),
                Instruction::LoadLocal(0),
                Instruction::PushString("nova".to_string()),
                Instruction::PushInt(1),
                Instruction::SetMapIndex,
                Instruction::Return,
            ],
        }],
    };

    let error = VirtualMachine::new()
        .execute(&program)
        .expect_err("nil map assignment should fail");

    assert!(error.to_string().contains("assignment to entry in nil map"));
}

#[test]
fn execute_map_keys_returns_deterministic_key_slice() {
    let map_type = ValueType::Map {
        key: Box::new(ValueType::String),
        value: Box::new(ValueType::Int),
    };
    let program = Program {
        package_name: "main".to_string(),
        entry_function: "main".to_string(),
        entry_function_index: 0,
        functions: vec![CompiledFunction {
            name: "main".to_string(),
            parameter_count: 0,
            variadic_element_type: None,
            return_types: Vec::new(),
            local_names: vec!["keys".to_string()],
            instructions: vec![
                Instruction::PushString("nova".to_string()),
                Instruction::PushInt(3),
                Instruction::PushString("go".to_string()),
                Instruction::PushInt(2),
                Instruction::BuildMap {
                    map_type: map_type.clone(),
                    entry_count: 2,
                },
                Instruction::MapKeys(ValueType::String),
                Instruction::StoreLocal(0),
                Instruction::LoadLocal(0),
                Instruction::PushInt(0),
                Instruction::Index(SequenceKind::Slice),
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
        .expect("map-keys program should execute")
        .render_output();

    assert_eq!(output, "go nova\n");
}

#[test]
fn execute_lookup_map_reports_value_and_presence() {
    let map_type = ValueType::Map {
        key: Box::new(ValueType::String),
        value: Box::new(ValueType::Int),
    };
    let program = Program {
        package_name: "main".to_string(),
        entry_function: "main".to_string(),
        entry_function_index: 0,
        functions: vec![CompiledFunction {
            name: "main".to_string(),
            parameter_count: 0,
            variadic_element_type: None,
            return_types: Vec::new(),
            local_names: vec!["counts".to_string()],
            instructions: vec![
                Instruction::PushNilMap,
                Instruction::StoreLocal(0),
                Instruction::LoadLocal(0),
                Instruction::PushString("nova".to_string()),
                Instruction::LookupMap(map_type.clone()),
                Instruction::CallBuiltin(BuiltinFunction::Println, 2),
                Instruction::PushInt(1),
                Instruction::MakeMap {
                    map_type: map_type.clone(),
                    has_hint: true,
                },
                Instruction::StoreLocal(0),
                Instruction::LoadLocal(0),
                Instruction::PushString("nova".to_string()),
                Instruction::PushInt(3),
                Instruction::SetMapIndex,
                Instruction::LoadLocal(0),
                Instruction::PushString("nova".to_string()),
                Instruction::LookupMap(map_type),
                Instruction::CallBuiltin(BuiltinFunction::Println, 2),
                Instruction::Return,
            ],
        }],
    };

    let output = VirtualMachine::new()
        .execute(&program)
        .expect("lookup-map program should execute")
        .render_output();

    assert_eq!(output, "0 false\n3 true\n");
}

#[test]
fn execute_channels_send_receive_and_close() {
    let program = Program {
        package_name: "main".to_string(),
        entry_function: "main".to_string(),
        entry_function_index: 0,
        functions: vec![CompiledFunction {
            name: "main".to_string(),
            parameter_count: 0,
            variadic_element_type: None,
            return_types: Vec::new(),
            local_names: vec!["ready".to_string()],
            instructions: vec![
                Instruction::PushNilChan,
                Instruction::StoreLocal(0),
                Instruction::LoadLocal(0),
                Instruction::CallBuiltin(BuiltinFunction::Len, 1),
                Instruction::LoadLocal(0),
                Instruction::CallBuiltin(BuiltinFunction::Cap, 1),
                Instruction::CallBuiltin(BuiltinFunction::Println, 2),
                Instruction::PushInt(2),
                Instruction::MakeChan {
                    element_type: ValueType::Int,
                    has_buffer: true,
                },
                Instruction::StoreLocal(0),
                Instruction::LoadLocal(0),
                Instruction::PushInt(4),
                Instruction::Send,
                Instruction::LoadLocal(0),
                Instruction::PushInt(7),
                Instruction::Send,
                Instruction::LoadLocal(0),
                Instruction::Receive(ValueType::Int),
                Instruction::LoadLocal(0),
                Instruction::CallBuiltin(BuiltinFunction::Close, 1),
                Instruction::LoadLocal(0),
                Instruction::Receive(ValueType::Int),
                Instruction::LoadLocal(0),
                Instruction::Receive(ValueType::Int),
                Instruction::CallBuiltin(BuiltinFunction::Println, 3),
                Instruction::Return,
            ],
        }],
    };

    let output = VirtualMachine::new()
        .execute(&program)
        .expect("channel program should execute")
        .render_output();

    assert_eq!(output, "0 0\n4 7 0\n");
}

#[test]
fn execute_channel_send_reports_blocking_error() {
    let program = Program {
        package_name: "main".to_string(),
        entry_function: "main".to_string(),
        entry_function_index: 0,
        functions: vec![CompiledFunction {
            name: "main".to_string(),
            parameter_count: 0,
            variadic_element_type: None,
            return_types: Vec::new(),
            local_names: vec!["ready".to_string()],
            instructions: vec![
                Instruction::PushInt(1),
                Instruction::MakeChan {
                    element_type: ValueType::Int,
                    has_buffer: true,
                },
                Instruction::StoreLocal(0),
                Instruction::LoadLocal(0),
                Instruction::PushInt(4),
                Instruction::Send,
                Instruction::LoadLocal(0),
                Instruction::PushInt(7),
                Instruction::Send,
                Instruction::Return,
            ],
        }],
    };

    let error = VirtualMachine::new()
        .execute(&program)
        .expect_err("full buffered channel should report a blocking error");

    assert!(
        error
            .to_string()
            .contains("send would block in the current single-threaded VM")
    );
}

#[test]
fn execute_variadic_user_function_calls_and_spread() {
    let program = Program {
        package_name: "main".to_string(),
        entry_function: "main".to_string(),
        entry_function_index: 1,
        functions: vec![
            CompiledFunction {
                name: "count".to_string(),
                parameter_count: 1,
                variadic_element_type: Some(ValueType::Int),
                return_types: vec![ValueType::Int],
                local_names: vec!["values".to_string()],
                instructions: vec![
                    Instruction::LoadLocal(0),
                    Instruction::CallBuiltin(BuiltinFunction::Len, 1),
                    Instruction::Return,
                ],
            },
            CompiledFunction {
                name: "main".to_string(),
                parameter_count: 0,
                variadic_element_type: None,
                return_types: Vec::new(),
                local_names: vec!["values".to_string()],
                instructions: vec![
                    Instruction::PushInt(1),
                    Instruction::PushInt(2),
                    Instruction::CallFunction(0, 2),
                    Instruction::CallBuiltin(BuiltinFunction::Println, 1),
                    Instruction::PushInt(4),
                    Instruction::PushInt(5),
                    Instruction::BuildSlice(2),
                    Instruction::StoreLocal(0),
                    Instruction::LoadLocal(0),
                    Instruction::CallFunctionSpread(0, 0),
                    Instruction::CallBuiltin(BuiltinFunction::Println, 1),
                    Instruction::PushNilSlice,
                    Instruction::CallFunctionSpread(0, 0),
                    Instruction::CallBuiltin(BuiltinFunction::Println, 1),
                    Instruction::Return,
                ],
            },
        ],
    };

    let output = VirtualMachine::new()
        .execute(&program)
        .expect("variadic calls should execute")
        .render_output();

    assert_eq!(output, "2\n2\n0\n");
}

#[test]
fn execute_append_spread_for_slice_and_string_sources() {
    let program = Program {
        package_name: "main".to_string(),
        entry_function: "main".to_string(),
        entry_function_index: 0,
        functions: vec![CompiledFunction {
            name: "main".to_string(),
            parameter_count: 0,
            variadic_element_type: None,
            return_types: Vec::new(),
            local_names: vec!["bytes".to_string()],
            instructions: vec![
                Instruction::PushByte(b'g'),
                Instruction::PushByte(b'o'),
                Instruction::BuildSlice(2),
                Instruction::StoreLocal(0),
                Instruction::LoadLocal(0),
                Instruction::PushByte(b'-'),
                Instruction::PushByte(b'n'),
                Instruction::PushByte(b'o'),
                Instruction::PushByte(b'v'),
                Instruction::PushByte(b'a'),
                Instruction::BuildSlice(5),
                Instruction::CallBuiltinSpread(BuiltinFunction::Append, 1),
                Instruction::StoreLocal(0),
                Instruction::LoadLocal(0),
                Instruction::PushString("!".to_string()),
                Instruction::CallBuiltinSpread(BuiltinFunction::Append, 1),
                Instruction::Convert(ConversionKind::BytesToString),
                Instruction::CallBuiltin(BuiltinFunction::Println, 1),
                Instruction::Return,
            ],
        }],
    };

    let output = VirtualMachine::new()
        .execute(&program)
        .expect("append spread should execute")
        .render_output();

    assert_eq!(output, "go-nova!\n");
}
