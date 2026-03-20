use super::*;

#[test]
fn execute_clear_builtin_for_slices_and_maps() {
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
            local_names: vec![
                "values".to_string(),
                "window".to_string(),
                "missing".to_string(),
                "counts".to_string(),
                "alias".to_string(),
            ],
            instructions: vec![
                Instruction::PushInt(1),
                Instruction::PushInt(2),
                Instruction::PushInt(3),
                Instruction::PushInt(4),
                Instruction::BuildSlice(4),
                Instruction::StoreLocal(0),
                Instruction::LoadLocal(0),
                Instruction::PushInt(1),
                Instruction::PushInt(3),
                Instruction::Slice {
                    target_kind: SequenceKind::Slice,
                    has_low: true,
                    has_high: true,
                },
                Instruction::StoreLocal(1),
                Instruction::LoadLocal(1),
                Instruction::CallBuiltin(BuiltinFunction::Clear, 1),
                Instruction::LoadLocal(0),
                Instruction::PushInt(0),
                Instruction::Index(SequenceKind::Slice),
                Instruction::LoadLocal(0),
                Instruction::PushInt(1),
                Instruction::Index(SequenceKind::Slice),
                Instruction::LoadLocal(0),
                Instruction::PushInt(2),
                Instruction::Index(SequenceKind::Slice),
                Instruction::LoadLocal(0),
                Instruction::PushInt(3),
                Instruction::Index(SequenceKind::Slice),
                Instruction::LoadLocal(1),
                Instruction::CallBuiltin(BuiltinFunction::Len, 1),
                Instruction::LoadLocal(1),
                Instruction::CallBuiltin(BuiltinFunction::Cap, 1),
                Instruction::CallBuiltin(BuiltinFunction::Println, 6),
                Instruction::PushNilSlice,
                Instruction::StoreLocal(2),
                Instruction::LoadLocal(2),
                Instruction::CallBuiltin(BuiltinFunction::Clear, 1),
                Instruction::LoadLocal(2),
                Instruction::PushNilSlice,
                Instruction::Equal,
                Instruction::LoadLocal(2),
                Instruction::CallBuiltin(BuiltinFunction::Len, 1),
                Instruction::LoadLocal(2),
                Instruction::CallBuiltin(BuiltinFunction::Cap, 1),
                Instruction::CallBuiltin(BuiltinFunction::Println, 3),
                Instruction::PushString("nova".to_string()),
                Instruction::PushInt(3),
                Instruction::PushString("go".to_string()),
                Instruction::PushInt(2),
                Instruction::BuildMap {
                    map_type: map_type.clone(),
                    entry_count: 2,
                },
                Instruction::StoreLocal(3),
                Instruction::LoadLocal(3),
                Instruction::StoreLocal(4),
                Instruction::LoadLocal(4),
                Instruction::CallBuiltin(BuiltinFunction::Clear, 1),
                Instruction::LoadLocal(3),
                Instruction::CallBuiltin(BuiltinFunction::Len, 1),
                Instruction::LoadLocal(4),
                Instruction::CallBuiltin(BuiltinFunction::Len, 1),
                Instruction::LoadLocal(3),
                Instruction::PushNilMap,
                Instruction::Equal,
                Instruction::CallBuiltin(BuiltinFunction::Println, 3),
                Instruction::Return,
            ],
        }],
    };

    let output = VirtualMachine::new()
        .execute(&program)
        .expect("clear builtin program should execute")
        .render_output();

    assert_eq!(output, "1 0 0 4 2 3\ntrue 0 0\n0 0 false\n");
}
