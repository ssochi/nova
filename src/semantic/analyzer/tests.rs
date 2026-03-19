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
fn analyze_string_byte_conversions() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tvar bytes = []byte(\"nova\")\n\tbytes[0] = \"X\"[0]\n\tvar text = string(bytes)\n\tprintln(text)\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let program = analyze_package(&ast).expect("analysis should succeed");

    assert_eq!(program.functions.len(), 1);
}

#[test]
fn analyze_maps_with_make_len_index_and_assignment() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tvar counts map[string]int\n\tcounts = make(map[string]int, 2)\n\tcounts[\"nova\"] = 3\n\tprintln(len(counts), counts[\"nova\"], counts[\"missing\"])\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let program = analyze_package(&ast).expect("analysis should succeed");

    assert_eq!(program.functions.len(), 1);
}

#[test]
fn analyze_map_literals_and_delete() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tvar counts = map[string]int{\"nova\": 3, \"go\": 2}\n\tdelete(counts, \"go\")\n\tprintln(len(counts), counts[\"nova\"], counts[\"go\"])\n}\n"
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
fn reject_map_literal_value_type_mismatch() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tvar counts = map[string]int{\"nova\": \"oops\"}\n\tprintln(counts)\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let error =
        analyze_package(&ast).expect_err("analysis should reject invalid map literal values");

    assert!(
        error
            .to_string()
            .contains("map literal value 1 requires `int`, found `string`")
    );
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

#[test]
fn reject_string_conversion_from_non_byte_slice() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tvar values = []int{65}\n\tprintln(string(values))\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let error = analyze_package(&ast).expect_err("analysis should reject invalid conversion");

    assert!(
        error
            .to_string()
            .contains("conversion to `string` requires `[]byte`, found `[]int`")
    );
}

#[test]
fn reject_non_comparable_map_key_type() {
    let source = SourceFile {
        path: "test.go".into(),
        contents:
            "package main\n\nfunc main() {\n\tvar counts map[[]int]int\n\tprintln(counts)\n}\n"
                .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let error = analyze_package(&ast).expect_err("analysis should reject invalid map key types");

    assert!(error.to_string().contains("comparable map key type"));
}

#[test]
fn reject_delete_with_wrong_key_type() {
    let source = SourceFile {
        path: "test.go".into(),
        contents:
            "package main\n\nfunc main() {\n\tvar counts = make(map[string]int)\n\tdelete(counts, 1)\n}\n"
                .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let error = analyze_package(&ast).expect_err("analysis should reject mismatched delete keys");

    assert!(
        error
            .to_string()
            .contains("argument 2 in call to builtin `delete` requires `string`")
    );
}

#[test]
fn analyze_explicit_nil_for_slices_maps_and_typed_calls() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nimport \"strings\"\n\nfunc accept(values []string) bool {\n\treturn values == nil\n}\n\nfunc provide() map[string]int {\n\treturn nil\n}\n\nfunc main() {\n\tvar values []int = nil\n\tvar counts map[string]int = nil\n\tvalues = nil\n\tcounts = nil\n\tprintln(values == nil, counts == nil)\n\tprintln(accept(nil), provide() == nil, strings.Join(nil, \":\") == \"\")\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let program = analyze_package(&ast).expect("analysis should succeed");

    assert_eq!(program.functions.len(), 3);
}

#[test]
fn reject_untyped_nil_without_context() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tvar values = nil\n}\n".to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let error = analyze_package(&ast).expect_err("analysis should reject untyped nil vars");

    assert!(
        error
            .to_string()
            .contains("requires an explicit type when initialized with `nil`")
    );
}

#[test]
fn reject_nil_equals_nil() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tprintln(nil == nil)\n}\n".to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let error = analyze_package(&ast).expect_err("analysis should reject nil equality");

    assert!(
        error
            .to_string()
            .contains("does not support untyped `nil` operands")
    );
}
