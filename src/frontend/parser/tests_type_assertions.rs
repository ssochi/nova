use super::parse_source_file;
use crate::frontend::ast::Expression;
use crate::frontend::lexer::lex;
use crate::frontend::signature::TypeRef;
use crate::source::SourceFile;

#[test]
fn parse_type_assertions_with_rendering() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tvar value any = \"go\"\n\tvar word = value.(string)\n\tvar boxed = value.(interface{})\n\tprintln(word, boxed)\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let function = &ast.functions[0];

    let value = match &function.body.statements[1] {
        crate::frontend::ast::Statement::VarDecl {
            value: Some(value), ..
        } => value,
        _ => panic!("expected variable declaration"),
    };
    let Expression::TypeAssertion { asserted_type, .. } = value else {
        panic!("expected type assertion");
    };
    assert_eq!(asserted_type, &TypeRef::Named("string".to_string()));

    let rendered = ast.render();
    assert!(rendered.contains("var word = value.(string)"));
    assert!(rendered.contains("var boxed = value.(interface{})"));
}

#[test]
fn reject_type_switch_syntax() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tvar value any\n\t_ = value.(type)\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let error = parse_source_file(&tokens).expect_err("type switches should be rejected");

    assert!(
        error
            .to_string()
            .contains("type switches are not supported")
    );
}
