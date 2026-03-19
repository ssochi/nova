use super::parse_source_file;
use crate::frontend::ast::{
    AssignmentTarget, Expression, RangeBinding, RangeBindingMode, Statement, TypeRef,
};
use crate::frontend::lexer::lex;
use crate::source::SourceFile;

#[test]
fn parse_slice_literal_and_index_expression() {
    let source = SourceFile {
        path: "test.go".into(),
        contents:
            "package main\n\nfunc main() {\n\tvar values = []int{1, 2}\n\tprintln(values[1])\n}\n"
                .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let function = &ast.functions[0];

    match &function.body.statements[0] {
        Statement::VarDecl { value, .. } => match value.as_ref() {
            Some(Expression::SliceLiteral { element_type, .. }) => {
                assert_eq!(
                    element_type,
                    &TypeRef::Slice(Box::new(TypeRef::Named("int".into())))
                );
            }
            _ => panic!("expected slice literal"),
        },
        _ => panic!("expected variable declaration"),
    }

    match &function.body.statements[1] {
        Statement::Expr(Expression::Call { arguments, .. }) => {
            assert!(matches!(arguments[0], Expression::Index { .. }));
        }
        _ => panic!("expected call expression"),
    }
}

#[test]
fn parse_slice_expression_and_index_assignment() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tvar values = []int{1, 2, 3}\n\tvar middle = values[1:3]\n\tvalues[:2][1] = 9\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let function = &ast.functions[0];

    match &function.body.statements[1] {
        Statement::VarDecl { value, .. } => match value.as_ref() {
            Some(Expression::Slice { low, high, .. }) => {
                assert!(matches!(low, Some(value) if **value == Expression::Integer(1)));
                assert!(matches!(high, Some(value) if **value == Expression::Integer(3)));
            }
            _ => panic!("expected slice expression"),
        },
        _ => panic!("expected variable declaration"),
    }

    match &function.body.statements[2] {
        Statement::Assign { target, .. } => match target {
            AssignmentTarget::Index { target, index } => {
                assert_eq!(*index, Expression::Integer(1));
                assert!(matches!(target, Expression::Slice { .. }));
            }
            _ => panic!("expected index assignment target"),
        },
        _ => panic!("expected assignment statement"),
    }
}

#[test]
fn parse_typed_var_declarations_with_and_without_initializers() {
    let source = SourceFile {
        path: "test.go".into(),
        contents:
            "package main\n\nfunc main() {\n\tvar total int\n\tvar values []int = []int{1, 2}\n}\n"
                .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let function = &ast.functions[0];

    match &function.body.statements[0] {
        Statement::VarDecl {
            type_ref, value, ..
        } => {
            assert_eq!(type_ref, &Some(TypeRef::Named("int".into())));
            assert!(value.is_none());
        }
        _ => panic!("expected typed variable declaration"),
    }

    match &function.body.statements[1] {
        Statement::VarDecl {
            type_ref, value, ..
        } => {
            assert_eq!(
                type_ref,
                &Some(TypeRef::Slice(Box::new(TypeRef::Named("int".into()))))
            );
            assert!(matches!(value, Some(Expression::SliceLiteral { .. })));
        }
        _ => panic!("expected typed variable declaration with initializer"),
    }
}

#[test]
fn parse_make_slice_expression() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tvar values = make([]int, 2, 4)\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let function = &ast.functions[0];

    match &function.body.statements[0] {
        Statement::VarDecl { value, .. } => match value.as_ref() {
            Some(Expression::Make {
                type_ref,
                arguments,
            }) => {
                assert_eq!(
                    type_ref,
                    &TypeRef::Slice(Box::new(TypeRef::Named("int".into())))
                );
                assert_eq!(
                    arguments,
                    &vec![Expression::Integer(2), Expression::Integer(4)]
                );
            }
            _ => panic!("expected make expression"),
        },
        _ => panic!("expected variable declaration"),
    }
}

#[test]
fn parse_map_types_and_make_expression() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tvar counts map[string]int\n\tvar ready = make(map[bool]string)\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let function = &ast.functions[0];

    match &function.body.statements[0] {
        Statement::VarDecl {
            type_ref, value, ..
        } => {
            assert_eq!(
                type_ref,
                &Some(TypeRef::Map {
                    key: Box::new(TypeRef::Named("string".into())),
                    value: Box::new(TypeRef::Named("int".into())),
                })
            );
            assert!(value.is_none());
        }
        _ => panic!("expected typed map declaration"),
    }

    match &function.body.statements[1] {
        Statement::VarDecl { value, .. } => match value.as_ref() {
            Some(Expression::Make {
                type_ref,
                arguments,
            }) => {
                assert_eq!(
                    type_ref,
                    &TypeRef::Map {
                        key: Box::new(TypeRef::Named("bool".into())),
                        value: Box::new(TypeRef::Named("string".into())),
                    }
                );
                assert!(arguments.is_empty());
            }
            _ => panic!("expected map make expression"),
        },
        _ => panic!("expected variable declaration"),
    }
}

#[test]
fn parse_map_literal_with_entries_and_trailing_comma() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tvar counts = map[string]int{\"nova\": 1, \"go\": 2,}\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let function = &ast.functions[0];

    match &function.body.statements[0] {
        Statement::VarDecl { value, .. } => match value.as_ref() {
            Some(Expression::MapLiteral { map_type, entries }) => {
                assert_eq!(
                    map_type,
                    &TypeRef::Map {
                        key: Box::new(TypeRef::Named("string".into())),
                        value: Box::new(TypeRef::Named("int".into())),
                    }
                );
                assert_eq!(entries.len(), 2);
                assert_eq!(entries[0].key, Expression::String("nova".into()));
                assert_eq!(entries[0].value, Expression::Integer(1));
            }
            _ => panic!("expected map literal"),
        },
        _ => panic!("expected variable declaration"),
    }
}

#[test]
fn parse_range_loop_forms() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tvar values = []int{1, 2}\n\tfor range values {\n\t\tprintln(1)\n\t}\n\tfor index := range values {\n\t\tprintln(index)\n\t}\n\tfor _, value = range values {\n\t\tprintln(value)\n\t}\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let function = &ast.functions[0];

    match &function.body.statements[1] {
        Statement::RangeFor {
            bindings,
            binding_mode,
            target,
            ..
        } => {
            assert!(bindings.is_empty());
            assert_eq!(binding_mode, &None);
            assert_eq!(target, &Expression::Identifier("values".into()));
        }
        _ => panic!("expected range loop without bindings"),
    }

    match &function.body.statements[2] {
        Statement::RangeFor {
            bindings,
            binding_mode,
            ..
        } => {
            assert_eq!(bindings, &vec![RangeBinding::Identifier("index".into())]);
            assert_eq!(binding_mode, &Some(RangeBindingMode::Define));
        }
        _ => panic!("expected define-style range loop"),
    }

    match &function.body.statements[3] {
        Statement::RangeFor {
            bindings,
            binding_mode,
            ..
        } => {
            assert_eq!(
                bindings,
                &vec![
                    RangeBinding::Blank,
                    RangeBinding::Identifier("value".into())
                ]
            );
            assert_eq!(binding_mode, &Some(RangeBindingMode::Assign));
        }
        _ => panic!("expected assign-style range loop"),
    }
}

#[test]
fn parse_string_and_byte_conversions() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tvar bytes = []byte(\"go\",)\n\tvar text = string(bytes)\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let function = &ast.functions[0];

    match &function.body.statements[0] {
        Statement::VarDecl { value, .. } => match value.as_ref() {
            Some(Expression::Conversion { type_ref, value }) => {
                assert_eq!(
                    type_ref,
                    &TypeRef::Slice(Box::new(TypeRef::Named("byte".into())))
                );
                assert_eq!(value.as_ref(), &Expression::String("go".into()));
            }
            _ => panic!("expected byte conversion"),
        },
        _ => panic!("expected variable declaration"),
    }

    match &function.body.statements[1] {
        Statement::VarDecl { value, .. } => match value.as_ref() {
            Some(Expression::Conversion { type_ref, value }) => {
                assert_eq!(type_ref, &TypeRef::Named("string".into()));
                assert_eq!(value.as_ref(), &Expression::Identifier("bytes".into()));
            }
            _ => panic!("expected string conversion"),
        },
        _ => panic!("expected variable declaration"),
    }
}

#[test]
fn parse_nil_expression_and_nil_comparison() {
    let source = SourceFile {
        path: "test.go".into(),
        contents:
            "package main\n\nfunc main() {\n\tvar values []int = nil\n\tprintln(values == nil)\n}\n"
                .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let function = &ast.functions[0];

    match &function.body.statements[0] {
        Statement::VarDecl { value, .. } => {
            assert_eq!(value.as_ref(), Some(&Expression::Nil));
        }
        _ => panic!("expected variable declaration"),
    }

    match &function.body.statements[1] {
        Statement::Expr(Expression::Call { arguments, .. }) => {
            assert!(matches!(
                &arguments[0],
                Expression::Binary {
                    left,
                    operator: _,
                    right,
                } if **left == Expression::Identifier("values".into())
                    && **right == Expression::Nil
            ));
        }
        _ => panic!("expected nil comparison call"),
    }
}
