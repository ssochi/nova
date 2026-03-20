use super::analyze_package;
use crate::frontend::{lexer::lex, parser::parse_source_file};
use crate::semantic::model::{CheckedExpressionKind, CheckedStatement, Type};
use crate::source::SourceFile;

#[test]
fn analyze_type_assertions_from_any_values() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tvar value any = \"go\"\n\tvar word = value.(string)\n\tvar boxed = value.(any)\n\tprintln(word, boxed)\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let program = analyze_package(&ast).expect("analysis should succeed");

    match &program.functions[0].body.statements[1] {
        CheckedStatement::VarDecl {
            value: Some(value), ..
        } => match &value.kind {
            CheckedExpressionKind::TypeAssertion { asserted_type, .. } => {
                assert_eq!(asserted_type, &Type::String)
            }
            _ => panic!("expected type assertion"),
        },
        _ => panic!("expected variable declaration"),
    }

    match &program.functions[0].body.statements[2] {
        CheckedStatement::VarDecl {
            value: Some(value), ..
        } => {
            assert_eq!(value.ty, Type::Any);
            assert!(matches!(
                value.kind,
                CheckedExpressionKind::TypeAssertion {
                    asserted_type: Type::Any,
                    ..
                }
            ));
        }
        _ => panic!("expected variable declaration"),
    }
}

#[test]
fn reject_type_assertions_on_non_interface_operands() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tvar value int = 7\n\tprintln(value.(int))\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let error = analyze_package(&ast).expect_err("type assertion should fail");

    assert!(
        error
            .to_string()
            .contains("type assertion requires interface operand")
    );
}
