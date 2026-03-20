use super::parse_source_file;
use crate::frontend::lexer::lex;
use crate::frontend::signature::TypeRef;
use crate::source::SourceFile;

#[test]
fn parse_any_and_interface_types() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc wrap(value any) interface{} {\n\tvar args = []any{\"go\", nil}\n\tfmt.Println(args...)\n\treturn interface{}(value)\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let function = &ast.functions[0];

    assert_eq!(
        function.parameters[0].type_ref,
        TypeRef::Named("any".to_string())
    );
    assert_eq!(function.results[0].type_ref, TypeRef::Interface);
    let rendered = ast.render();
    assert!(rendered.contains("func wrap(value any) interface{}"));
    assert!(rendered.contains("var args = []any{\"go\", nil}"));
    assert!(rendered.contains("return interface{}(value)"));
}
