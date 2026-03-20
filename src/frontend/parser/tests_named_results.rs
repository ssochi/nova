use super::parse_source_file;
use crate::frontend::ast::{ResultDecl, TypeRef};
use crate::frontend::lexer::lex;
use crate::source::SourceFile;

#[test]
fn parse_grouped_named_result_declarations() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc split(value string) (head, tail string, ok bool) {\n\treturn value, \"\", false\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let function = &ast.functions[0];

    assert_eq!(
        function.results,
        vec![
            ResultDecl {
                names: vec!["head".to_string(), "tail".to_string()],
                type_ref: TypeRef::Named("string".to_string()),
            },
            ResultDecl {
                names: vec!["ok".to_string()],
                type_ref: TypeRef::Named("bool".to_string()),
            },
        ]
    );

    let rendered = ast.render();
    assert!(rendered.contains("func split(value string) (head, tail string, ok bool)"));
}

#[test]
fn parse_unnamed_result_list_still_works() {
    let source = SourceFile {
        path: "test.go".into(),
        contents:
            "package main\n\nfunc split() (string, string, bool) {\n\treturn \"\", \"\", false\n}\n"
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
            },
        ]
    );
}

#[test]
fn reject_mixed_named_and_unnamed_result_declarations() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc split() (head string, bool) {\n\treturn \"\", false\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let error = parse_source_file(&tokens).expect_err("parsing should fail");

    assert!(
        error
            .to_string()
            .contains("mixed named and unnamed parameters")
    );
}

#[test]
fn parse_blank_named_result_identifier() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc split() (_ int, ok bool) {\n\treturn 0, false\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");

    assert_eq!(
        ast.functions[0].results[0],
        ResultDecl {
            names: vec!["_".to_string()],
            type_ref: TypeRef::Named("int".to_string()),
        }
    );
}
