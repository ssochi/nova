use super::parse_source_file;
use crate::frontend::ast::{Statement, TypeSwitchCase, TypeSwitchClause};
use crate::frontend::lexer::lex;
use crate::frontend::signature::TypeRef;
use crate::source::SourceFile;

#[test]
fn parse_comma_ok_type_assertions_and_type_switches() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tvar boxed any = []byte(\"go\")\n\tvalue, ok := boxed.([]byte)\n\tswitch current := boxed.(type) {\n\tcase []byte:\n\t\tprintln(ok, string(value), string(current))\n\tcase nil:\n\t\tprintln(false)\n\tdefault:\n\t\tprintln(true)\n\t}\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let function = &ast.functions[0];

    match &function.body.statements[1] {
        Statement::TypeAssert {
            bindings,
            binding_mode,
            asserted_type,
            ..
        } => {
            assert_eq!(bindings.len(), 2);
            assert_eq!(*binding_mode, crate::frontend::ast::BindingMode::Define);
            assert_eq!(
                asserted_type,
                &TypeRef::Slice(Box::new(TypeRef::Named("byte".to_string())))
            );
        }
        _ => panic!("expected comma-ok type assertion statement"),
    }

    match &function.body.statements[2] {
        Statement::TypeSwitch(type_switch) => {
            assert_eq!(type_switch.guard.binding.as_deref(), Some("current"));
            match &type_switch.clauses[0] {
                TypeSwitchClause::Case { cases, .. } => assert_eq!(
                    cases,
                    &vec![TypeSwitchCase::Type(TypeRef::Slice(Box::new(
                        TypeRef::Named("byte".to_string())
                    )))]
                ),
                _ => panic!("expected first type-switch clause"),
            }
            assert!(matches!(
                type_switch.clauses[1],
                TypeSwitchClause::Case { .. }
            ));
            assert!(matches!(
                type_switch.clauses[2],
                TypeSwitchClause::Default(_)
            ));
        }
        _ => panic!("expected type switch"),
    }

    let rendered = ast.render();
    assert!(rendered.contains("value, ok := boxed.([]byte)"));
    assert!(rendered.contains("switch current := boxed.(type) {"));
    assert!(rendered.contains("case []byte:"));
    assert!(rendered.contains("case nil:"));
}
