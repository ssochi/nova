use super::parse_source_file;
use crate::frontend::ast::{CallArgument, Expression, Statement};
use crate::frontend::lexer::lex;
use crate::source::SourceFile;

#[test]
fn parse_defer_call_statement() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tdefer println(\"tail\")\n}\n".to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");

    match &ast.functions[0].body.statements[0] {
        Statement::Defer(Expression::Call { callee, arguments }) => {
            assert_eq!(**callee, Expression::Identifier("println".to_string()));
            assert!(matches!(
                arguments.as_slice(),
                [CallArgument::Expression(Expression::String(value))] if value == "tail"
            ));
        }
        _ => panic!("expected defer call statement"),
    }

    let rendered = ast.render();
    assert!(rendered.contains("defer println(\"tail\")"));
}

#[test]
fn reject_parenthesized_defer_expression() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tdefer (println(\"tail\"))\n}\n".to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let error = parse_source_file(&tokens).expect_err("parsing should fail");

    assert!(
        error
            .to_string()
            .contains("expression in defer must not be parenthesized")
    );
}

#[test]
fn reject_non_call_defer_expression() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tdefer value\n}\n".to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let error = parse_source_file(&tokens).expect_err("parsing should fail");

    assert!(error.to_string().contains("defer requires a function call"));
}
