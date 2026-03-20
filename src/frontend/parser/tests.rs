use super::parse_source_file;
use crate::frontend::ast::{
    AssignmentTarget, Binding, BindingMode, CallArgument, ElseBranch, Expression, ForPostStatement,
    HeaderStatement, ImportDecl, ResultDecl, Statement, SwitchClause, TypeRef,
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
            assert!(matches!(
                arguments[0],
                CallArgument::Expression(Expression::Index { .. })
            ));
        }
        _ => panic!("expected call expression"),
    }
}

#[test]
fn parse_variadic_function_and_spread_call() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc collect(prefix int, values ...int) {\n\tprintln(prefix, values...)\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let function = &ast.functions[0];

    assert_eq!(function.parameters[0].names, vec!["prefix".to_string()]);
    assert!(!function.parameters[0].variadic);
    assert_eq!(function.parameters[1].names, vec!["values".to_string()]);
    assert!(function.parameters[1].variadic);
    assert_eq!(
        function.parameters[1].type_ref,
        TypeRef::Named("int".into())
    );

    match &function.body.statements[0] {
        Statement::Expr(Expression::Call { arguments, .. }) => {
            assert!(matches!(
                arguments.as_slice(),
                [
                    CallArgument::Expression(Expression::Identifier(prefix)),
                    CallArgument::Spread(Expression::Identifier(values))
                ] if prefix == "prefix" && values == "values"
            ));
        }
        _ => panic!("expected call expression"),
    }
}

#[test]
fn parse_grouped_parameter_names() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc pair(a, b int, prefix, suffix string, values ...int) {\n\tprintln(a, b, prefix, suffix, values)\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let function = &ast.functions[0];

    assert_eq!(
        function.parameters[0].names,
        vec!["a".to_string(), "b".to_string()]
    );
    assert_eq!(
        function.parameters[1].names,
        vec!["prefix".to_string(), "suffix".to_string()]
    );
    assert_eq!(function.parameters[2].names, vec!["values".to_string()]);
    assert!(function.parameters[2].variadic);

    let rendered = ast.render();
    assert!(rendered.contains("func pair(a, b int, prefix, suffix string, values ...int)"));
}

#[test]
fn reject_non_final_variadic_parameter() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc collect(values ...int, suffix int) {}\n".to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let error = parse_source_file(&tokens).expect_err("parsing should fail");

    assert!(
        error
            .to_string()
            .contains("variadic parameter must be the final parameter")
    );
}

#[test]
fn reject_grouped_variadic_parameter_names() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc collect(values, more ...int) {}\n".to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let error = parse_source_file(&tokens).expect_err("parsing should fail");

    assert!(
        error
            .to_string()
            .contains("can only use `...` with one final parameter")
    );
}

#[test]
fn reject_non_final_spread_argument() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tprintln(values..., 1)\n}\n".to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let error = parse_source_file(&tokens).expect_err("parsing should fail");

    assert!(
        error
            .to_string()
            .contains("spread argument must be the final call argument")
    );
}

#[test]
fn parse_grouped_imports_and_aliases() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nimport (\n\tb \"bytes\"\n\t\"fmt\"\n)\n\nfunc main() {}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");

    match &ast.imports[0] {
        ImportDecl::Group(specs) => {
            assert_eq!(specs.len(), 2);
            assert_eq!(specs[0].binding.as_deref(), Some("b"));
            assert_eq!(specs[0].path, "bytes");
            assert_eq!(specs[1].binding, None);
            assert_eq!(specs[1].path, "fmt");
        }
        _ => panic!("expected grouped import"),
    }

    let rendered = ast.render();
    assert!(rendered.contains("import ("));
    assert!(rendered.contains("    b \"bytes\""));
    assert!(rendered.contains("    \"fmt\""));
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
            assert_eq!(bindings, &vec![Binding::Identifier("index".into())]);
            assert_eq!(binding_mode, &Some(BindingMode::Define));
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
                &vec![Binding::Blank, Binding::Identifier("value".into())]
            );
            assert_eq!(binding_mode, &Some(BindingMode::Assign));
        }
        _ => panic!("expected assign-style range loop"),
    }
}

#[test]
fn parse_map_lookup_statement_forms() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tvar counts = map[string]int{\"go\": 2}\n\tvalue, ok := counts[\"go\"]\n\t_, present = counts[\"nova\"]\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let function = &ast.functions[0];

    match &function.body.statements[1] {
        Statement::MapLookup {
            bindings,
            binding_mode,
            target,
            key,
        } => {
            assert_eq!(
                bindings,
                &vec![
                    Binding::Identifier("value".into()),
                    Binding::Identifier("ok".into())
                ]
            );
            assert_eq!(binding_mode, &BindingMode::Define);
            assert_eq!(target, &Expression::Identifier("counts".into()));
            assert_eq!(key, &Expression::String("go".into()));
        }
        _ => panic!("expected define-style map lookup"),
    }

    match &function.body.statements[2] {
        Statement::MapLookup {
            bindings,
            binding_mode,
            ..
        } => {
            assert_eq!(
                bindings,
                &vec![Binding::Blank, Binding::Identifier("present".into())]
            );
            assert_eq!(binding_mode, &BindingMode::Assign);
        }
        _ => panic!("expected assign-style map lookup"),
    }
}

#[test]
fn parse_if_initializer_and_else_if_chain() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tvar counts = map[string]int{\"go\": 2}\n\tif value, ok := counts[\"go\"]; ok {\n\t\tprintln(value)\n\t} else if var ready bool = false; ready {\n\t\tprintln(0)\n\t} else {\n\t\tprintln(2)\n\t}\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let function = &ast.functions[0];

    match &function.body.statements[1] {
        Statement::If(if_statement) => {
            match if_statement.header.as_ref() {
                Some(HeaderStatement::MapLookup {
                    bindings,
                    binding_mode,
                    target,
                    key,
                }) => {
                    assert_eq!(
                        bindings,
                        &vec![
                            Binding::Identifier("value".into()),
                            Binding::Identifier("ok".into())
                        ]
                    );
                    assert_eq!(binding_mode, &BindingMode::Define);
                    assert_eq!(target, &Expression::Identifier("counts".into()));
                    assert_eq!(key, &Expression::String("go".into()));
                }
                _ => panic!("expected map lookup if initializer"),
            }
            assert_eq!(if_statement.condition, Expression::Identifier("ok".into()));
            match if_statement.else_branch.as_ref() {
                Some(ElseBranch::If(else_if)) => match else_if.header.as_ref() {
                    Some(HeaderStatement::VarDecl {
                        name,
                        type_ref,
                        value,
                    }) => {
                        assert_eq!(name, "ready");
                        assert_eq!(type_ref, &Some(TypeRef::Named("bool".into())));
                        assert_eq!(value.as_ref(), Some(&Expression::Bool(false)));
                    }
                    _ => panic!("expected var-decl else-if initializer"),
                },
                _ => panic!("expected else-if branch"),
            }
        }
        _ => panic!("expected if statement"),
    }
}

#[test]
fn parse_switch_statement_forms() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tvar counts = map[string]int{\"nova\": 3}\n\tswitch value, ok := counts[\"nova\"]; {\n\tcase ok:\n\t\tprintln(value)\n\tdefault:\n\t\tprintln(0)\n\t}\n\tswitch value {\n\tcase 1, 2:\n\t\tprintln(1)\n\tdefault:\n\t\tprintln(2)\n\t}\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let function = &ast.functions[0];

    match &function.body.statements[1] {
        Statement::Switch(switch_statement) => {
            assert!(switch_statement.expression.is_none());
            match switch_statement.header.as_ref() {
                Some(HeaderStatement::MapLookup {
                    bindings,
                    binding_mode,
                    ..
                }) => {
                    assert_eq!(
                        bindings,
                        &vec![
                            Binding::Identifier("value".into()),
                            Binding::Identifier("ok".into())
                        ]
                    );
                    assert_eq!(binding_mode, &BindingMode::Define);
                }
                _ => panic!("expected map lookup switch header"),
            }
            assert!(matches!(
                &switch_statement.clauses[0],
                SwitchClause::Case { expressions, .. }
                    if expressions == &vec![Expression::Identifier("ok".into())]
            ));
            assert!(matches!(
                &switch_statement.clauses[1],
                SwitchClause::Default(_)
            ));
        }
        _ => panic!("expected switch statement"),
    }

    match &function.body.statements[2] {
        Statement::Switch(switch_statement) => {
            assert_eq!(
                switch_statement.expression,
                Some(Expression::Identifier("value".into()))
            );
            assert!(switch_statement.header.is_none());
            assert!(matches!(
                &switch_statement.clauses[0],
                SwitchClause::Case { expressions, .. }
                    if expressions
                        == &vec![Expression::Integer(1), Expression::Integer(2)]
            ));
        }
        _ => panic!("expected expression switch"),
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
                CallArgument::Expression(Expression::Binary {
                    left,
                    operator: _,
                    right,
                }) if **left == Expression::Identifier("values".into())
                    && **right == Expression::Nil
            ));
        }
        _ => panic!("expected nil comparison call"),
    }
}

#[test]
fn parse_for_clauses_and_loop_control_statements() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tfor var i int = 0; i < 3; i = i + 1 {\n\t\tcontinue\n\t}\n\tfor ; ; value, ok = counts[\"go\"] {\n\t\tbreak\n\t}\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let function = &ast.functions[0];

    match &function.body.statements[0] {
        Statement::For(for_statement) => {
            assert!(matches!(
                &for_statement.init,
                Some(HeaderStatement::VarDecl { name, .. }) if name == "i"
            ));
            assert!(matches!(
                &for_statement.condition,
                Some(Expression::Binary { .. })
            ));
            assert!(matches!(
                &for_statement.post,
                Some(ForPostStatement::Assign {
                    target: AssignmentTarget::Identifier(name),
                    ..
                }) if name == "i"
            ));
            assert!(matches!(
                for_statement.body.statements[0],
                Statement::Continue
            ));
        }
        _ => panic!("expected classic for statement"),
    }

    match &function.body.statements[1] {
        Statement::For(for_statement) => {
            assert!(for_statement.init.is_none());
            assert!(for_statement.condition.is_none());
            assert!(matches!(
                &for_statement.post,
                Some(ForPostStatement::MapLookup { bindings, .. })
                    if bindings
                        == &vec![
                            Binding::Identifier("value".into()),
                            Binding::Identifier("ok".into())
                        ]
            ));
            assert!(matches!(for_statement.body.statements[0], Statement::Break));
        }
        _ => panic!("expected infinite for clause statement"),
    }
}

#[test]
fn parse_short_declarations_and_inc_dec_statements() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\ttotal := 0\n\ttotal++\n\tif count := 2; count > 1 {\n\t\tprintln(count)\n\t}\n\tswitch probe := total; {\n\tcase probe > 0:\n\t\tprintln(probe)\n\t}\n\tfor i := 0; i < 3; i++ {\n\t\tvalues[i]--\n\t}\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let function = &ast.functions[0];

    assert!(matches!(
        &function.body.statements[0],
        Statement::ShortVarDecl {
            bindings,
            values,
        } if bindings == &vec![Binding::Identifier("total".to_string())]
            && values == &vec![Expression::Integer(0)]
    ));
    assert!(matches!(
        &function.body.statements[1],
        Statement::IncDec {
            target: AssignmentTarget::Identifier(name),
            operator: crate::frontend::ast::IncDecOperator::Increment,
        } if name == "total"
    ));

    match &function.body.statements[2] {
        Statement::If(if_statement) => {
            assert!(matches!(
                &if_statement.header,
                Some(HeaderStatement::ShortVarDecl {
                    bindings,
                    values,
                }) if bindings == &vec![Binding::Identifier("count".to_string())]
                    && values == &vec![Expression::Integer(2)]
            ));
        }
        _ => panic!("expected if statement"),
    }

    match &function.body.statements[3] {
        Statement::Switch(switch_statement) => {
            assert!(matches!(
                &switch_statement.header,
                Some(HeaderStatement::ShortVarDecl {
                    bindings,
                    values,
                }) if bindings == &vec![Binding::Identifier("probe".to_string())]
                    && values == &vec![Expression::Identifier("total".to_string())]
            ));
        }
        _ => panic!("expected switch statement"),
    }

    match &function.body.statements[4] {
        Statement::For(for_statement) => {
            assert!(matches!(
                &for_statement.init,
                Some(HeaderStatement::ShortVarDecl {
                    bindings,
                    values,
                }) if bindings == &vec![Binding::Identifier("i".to_string())]
                    && values == &vec![Expression::Integer(0)]
            ));
            assert!(matches!(
                &for_statement.post,
                Some(ForPostStatement::IncDec {
                    target: AssignmentTarget::Identifier(name),
                    operator: crate::frontend::ast::IncDecOperator::Increment,
                }) if name == "i"
            ));
            assert!(matches!(
                &for_statement.body.statements[0],
                Statement::IncDec {
                    target: AssignmentTarget::Index { .. },
                    operator: crate::frontend::ast::IncDecOperator::Decrement,
                }
            ));
        }
        _ => panic!("expected for statement"),
    }
}

#[test]
fn parse_multi_result_functions_and_bindings() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc split(value string) (string, string, bool) {\n\treturn value, \"\", false\n}\n\nfunc main() {\n\thead, tail, found := split(\"nova\")\n\thead, tail, found = split(\"go\")\n\tprintln(head, tail, found)\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");

    assert_eq!(
        ast.functions[0].results,
        vec![
            ResultDecl {
                names: Vec::new(),
                type_ref: TypeRef::Named("string".to_string()),
            },
            ResultDecl {
                names: Vec::new(),
                type_ref: TypeRef::Named("string".to_string()),
            },
            ResultDecl {
                names: Vec::new(),
                type_ref: TypeRef::Named("bool".to_string()),
            }
        ]
    );
    assert!(matches!(
        &ast.functions[0].body.statements[0],
        Statement::Return(values) if values.len() == 3
    ));
    assert!(matches!(
        &ast.functions[1].body.statements[0],
        Statement::ShortVarDecl { bindings, values }
            if bindings.len() == 3 && values.len() == 1
    ));
    assert!(matches!(
        &ast.functions[1].body.statements[1],
        Statement::MultiAssign { bindings, values }
            if bindings.len() == 3 && values.len() == 1
    ));
}

#[test]
fn parse_compound_assignments() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\ttotal := 1\n\ttotal += 2\n\tif total -= 1; total > 0 {\n\t\tprintln(total)\n\t}\n\tswitch total += 3; {\n\tcase total > 1:\n\t\tprintln(total)\n\t}\n\tfor i := 0; i < 3; i += 1 {\n\t\tvalues[i] *= factor\n\t}\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let function = &ast.functions[0];

    assert!(matches!(
        &function.body.statements[1],
        Statement::CompoundAssign {
            target: AssignmentTarget::Identifier(name),
            operator: crate::frontend::ast::CompoundAssignOperator::Add,
            value: Expression::Integer(2),
        } if name == "total"
    ));

    match &function.body.statements[2] {
        Statement::If(if_statement) => {
            assert!(matches!(
                &if_statement.header,
                Some(HeaderStatement::CompoundAssign {
                    target: AssignmentTarget::Identifier(name),
                    operator: crate::frontend::ast::CompoundAssignOperator::Subtract,
                    value: Expression::Integer(1),
                }) if name == "total"
            ));
        }
        _ => panic!("expected if statement"),
    }

    match &function.body.statements[3] {
        Statement::Switch(switch_statement) => {
            assert!(matches!(
                &switch_statement.header,
                Some(HeaderStatement::CompoundAssign {
                    target: AssignmentTarget::Identifier(name),
                    operator: crate::frontend::ast::CompoundAssignOperator::Add,
                    value: Expression::Integer(3),
                }) if name == "total"
            ));
        }
        _ => panic!("expected switch statement"),
    }

    match &function.body.statements[4] {
        Statement::For(for_statement) => {
            assert!(matches!(
                &for_statement.post,
                Some(ForPostStatement::CompoundAssign {
                    target: AssignmentTarget::Identifier(name),
                    operator: crate::frontend::ast::CompoundAssignOperator::Add,
                    value: Expression::Integer(1),
                }) if name == "i"
            ));
            assert!(matches!(
                &for_statement.body.statements[0],
                Statement::CompoundAssign {
                    target: AssignmentTarget::Index { .. },
                    operator: crate::frontend::ast::CompoundAssignOperator::Multiply,
                    ..
                }
            ));
        }
        _ => panic!("expected for statement"),
    }
}

#[test]
fn parse_channel_types_and_send_receive() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tvar ready chan int\n\tready = make(chan int, 2)\n\tready <- 4\n\tvar first = <-ready\n\tclose(ready)\n}\n"
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
                &Some(TypeRef::Chan(Box::new(TypeRef::Named("int".into()))))
            );
            assert!(value.is_none());
        }
        _ => panic!("expected typed channel declaration"),
    }

    match &function.body.statements[1] {
        Statement::Assign { value, .. } => match value {
            Expression::Make {
                type_ref,
                arguments,
            } => {
                assert_eq!(
                    type_ref,
                    &TypeRef::Chan(Box::new(TypeRef::Named("int".into())))
                );
                assert_eq!(arguments, &vec![Expression::Integer(2)]);
            }
            _ => panic!("expected channel make expression"),
        },
        _ => panic!("expected channel assignment"),
    }

    assert!(matches!(
        &function.body.statements[2],
        Statement::Send {
            channel: Expression::Identifier(name),
            value: Expression::Integer(4),
        } if name == "ready"
    ));

    match &function.body.statements[3] {
        Statement::VarDecl { value, .. } => match value.as_ref() {
            Some(Expression::Receive { channel }) => {
                assert_eq!(channel.as_ref(), &Expression::Identifier("ready".into()));
            }
            _ => panic!("expected receive expression"),
        },
        _ => panic!("expected receive declaration"),
    }

    assert!(matches!(
        &function.body.statements[4],
        Statement::Expr(Expression::Call { callee, arguments })
            if callee.as_ref() == &Expression::Identifier("close".into())
                && arguments
                    == &vec![CallArgument::Expression(Expression::Identifier("ready".into()))]
    ));
}
