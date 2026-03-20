use super::analyze_package;
use crate::frontend::{lexer::lex, parser::parse_source_file};
use crate::semantic::model::{
    CheckedStatement, CheckedTypeSwitchBindingSource, CheckedTypeSwitchCase, Type,
};
use crate::source::SourceFile;

#[test]
fn analyze_comma_ok_type_assertions_and_type_switch_bindings() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tvar boxed any = []byte(\"go\")\n\tvalue, ok := boxed.([]byte)\n\tswitch current := boxed.(type) {\n\tcase []byte:\n\t\tprintln(ok, string(value), string(current))\n\tcase string, bool:\n\t\tprintln(current == true)\n\tdefault:\n\t\tprintln(current == nil)\n\t}\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let program = analyze_package(&ast).expect("analysis should succeed");

    match &program.functions[0].body.statements[1] {
        CheckedStatement::TypeAssert {
            asserted_type,
            value_binding,
            ok_binding,
            ..
        } => {
            assert_eq!(asserted_type, &Type::Slice(Box::new(Type::Byte)));
            assert!(matches!(
                value_binding,
                crate::semantic::model::CheckedBinding::Local { .. }
            ));
            assert!(matches!(
                ok_binding,
                crate::semantic::model::CheckedBinding::Local { .. }
            ));
        }
        _ => panic!("expected comma-ok type assertion"),
    }

    match &program.functions[0].body.statements[2] {
        CheckedStatement::TypeSwitch(type_switch) => {
            match &type_switch.clauses[0] {
                crate::semantic::model::CheckedTypeSwitchClause::Case {
                    cases, binding, ..
                } => {
                    assert_eq!(
                        cases,
                        &vec![CheckedTypeSwitchCase::Type(Type::Slice(Box::new(
                            Type::Byte
                        )))]
                    );
                    let binding = binding.as_ref().expect("single-type clause should bind");
                    assert_eq!(
                        binding.source,
                        CheckedTypeSwitchBindingSource::Asserted(Type::Slice(Box::new(Type::Byte)))
                    );
                }
                _ => panic!("expected first clause"),
            }

            match &type_switch.clauses[1] {
                crate::semantic::model::CheckedTypeSwitchClause::Case { binding, .. } => {
                    let binding = binding.as_ref().expect("multi-type clause should bind");
                    assert_eq!(binding.source, CheckedTypeSwitchBindingSource::Interface);
                }
                _ => panic!("expected second clause"),
            }

            match &type_switch.clauses[2] {
                crate::semantic::model::CheckedTypeSwitchClause::Default { binding, .. } => {
                    let binding = binding.as_ref().expect("default clause should bind");
                    assert_eq!(binding.source, CheckedTypeSwitchBindingSource::Interface);
                }
                _ => panic!("expected default clause"),
            }
        }
        _ => panic!("expected type switch"),
    }
}

#[test]
fn reject_type_switch_guard_on_non_interface_operand() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tvar value int = 7\n\tswitch value.(type) {\n\tdefault:\n\t}\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let error = analyze_package(&ast).expect_err("type switch should fail");

    assert!(
        error
            .to_string()
            .contains("type switch guard requires interface operand")
    );
}
