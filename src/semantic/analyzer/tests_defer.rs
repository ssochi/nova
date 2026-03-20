use super::analyze_package;
use crate::frontend::{lexer::lex, parser::parse_source_file};
use crate::semantic::model::{CallTarget, CheckedStatement};
use crate::source::SourceFile;

#[test]
fn analyze_defer_user_and_builtin_calls() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc pair() (int, int) {\n\treturn 1, 2\n}\n\nfunc main() {\n\tdefer println(\"tail\")\n\tdefer pair()\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let program = analyze_package(&ast).expect("analysis should succeed");
    let statements = &program.functions[1].body.statements;

    assert!(matches!(
        &statements[0],
        CheckedStatement::Defer(call) if matches!(call.target, CallTarget::Builtin(_))
    ));
    assert!(matches!(
        &statements[1],
        CheckedStatement::Defer(call) if matches!(call.target, CallTarget::UserDefined { .. })
    ));
}

#[test]
fn reject_defer_of_builtin_outside_statement_context() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tdefer len(\"go\")\n}\n".to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let error = analyze_package(&ast).expect_err("analysis should fail");

    assert!(
        error
            .to_string()
            .contains("builtin `len` is not permitted in defer statement context")
    );
}

#[test]
fn recoverable_panic_counts_as_terminating_path_for_value_function() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc report() {\n\tprintln(recover())\n}\n\nfunc value() int {\n\tdefer report()\n\tpanic(\"boom\")\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    analyze_package(&ast).expect("analysis should accept panic as terminating");
}
