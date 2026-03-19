use super::analyze_package;
use crate::frontend::{lexer::lex, parser::parse_source_file};
use crate::source::SourceFile;

#[test]
fn analyze_slice_index_and_append() {
    let source = SourceFile {
        path: "test.go".into(),
        contents:
            "package main\n\nfunc main() {\n\tvar values = []int{1, 2}\n\tvalues = append(values, 3)\n\tprintln(values[1])\n}\n"
                .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let program = analyze_package(&ast).expect("analysis should succeed");

    assert_eq!(program.functions.len(), 1);
}

#[test]
fn reject_slice_equality() {
    let source = SourceFile {
        path: "test.go".into(),
        contents:
            "package main\n\nfunc main() {\n\tvar left = []int{1}\n\tvar right = []int{1}\n\tprintln(left == right)\n}\n"
                .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let error = analyze_package(&ast).expect_err("slice equality should fail");

    assert!(
        error
            .to_string()
            .contains("does not support `[]int` operands")
    );
}

#[test]
fn analyze_slice_expression_and_index_assignment() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tvar values = []int{1, 2, 3}\n\tvar middle = values[1:3]\n\tmiddle[0] = 9\n\tprintln(values[1], middle[0])\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let program = analyze_package(&ast).expect("analysis should succeed");

    assert_eq!(program.functions.len(), 1);
}

#[test]
fn analyze_typed_var_declarations_without_initializers() {
    let source = SourceFile {
        path: "test.go".into(),
        contents:
            "package main\n\nfunc main() {\n\tvar total int\n\tvar ready bool\n\tvar label string\n\tvar values []int\n\tprintln(total, ready, len(label), len(values), cap(values))\n}\n"
                .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let program = analyze_package(&ast).expect("analysis should succeed");

    assert_eq!(program.functions.len(), 1);
}

#[test]
fn analyze_make_slice_expression() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tvar values = make([]int, 2, 4)\n\tvalues[1] = 9\n\tprintln(len(values), cap(values), values[1])\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let program = analyze_package(&ast).expect("analysis should succeed");

    assert_eq!(program.functions.len(), 1);
}

#[test]
fn analyze_string_index_slice_and_byte_copy() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tvar word = \"nova\"\n\tvar letter byte = word[1]\n\tvar window = word[1:3]\n\tvar buf = make([]byte, len(word))\n\tvar copied = copy(buf, word)\n\tprintln(letter, window, copied, buf[0])\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let program = analyze_package(&ast).expect("analysis should succeed");

    assert_eq!(program.functions.len(), 1);
}

#[test]
fn reject_make_with_constant_length_exceeding_capacity() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tvar values = make([]int, 3, 2)\n\tprintln(values)\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let error = analyze_package(&ast).expect_err("analysis should reject invalid make bounds");

    assert!(error.to_string().contains("length 3 exceeds capacity 2"));
}

#[test]
fn reject_copy_from_string_into_non_byte_slice() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tvar values = make([]int, 2)\n\tprintln(copy(values, \"no\"))\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let error = analyze_package(&ast).expect_err("analysis should reject int slice <- string");

    assert!(
        error
            .to_string()
            .contains("argument 2 in call to builtin `copy` requires `[]int`")
    );
}
