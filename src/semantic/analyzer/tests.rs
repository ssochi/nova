use super::analyze_package;
use crate::frontend::{lexer::lex, parser::parse_source_file};
use crate::semantic::model::Type;
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
fn analyze_range_loops_for_slices_and_maps() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tvar values = []int{1, 2}\n\tvar total int\n\tfor _, value := range values {\n\t\ttotal = total + value\n\t}\n\tvar counts = map[string]int{\"nova\": 3}\n\tvar seen string\n\tfor key := range counts {\n\t\tseen = seen + key\n\t}\n\tprintln(total, seen)\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let program = analyze_package(&ast).expect("analysis should succeed");

    assert_eq!(program.functions.len(), 1);
}

#[test]
fn analyze_map_lookup_statements() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tvar counts map[string]int\n\tvalue, ok := counts[\"nova\"]\n\tcounts = map[string]int{\"nova\": 3}\n\tvalue, ok = counts[\"nova\"]\n\tvalue, seen := counts[\"nova\"]\n\tprintln(value, ok, seen)\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let program = analyze_package(&ast).expect("analysis should succeed");

    assert_eq!(program.functions.len(), 1);
}

#[test]
fn analyze_if_initializers_and_else_if_chains() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tvar counts = map[string]int{\"go\": 2, \"fallback\": 1}\n\tif value, ok := counts[\"go\"]; ok {\n\t\tprintln(value, ok)\n\t} else {\n\t\tprintln(ok)\n\t}\n\tvar seen int\n\tif seen = counts[\"missing\"]; seen > 0 {\n\t\tprintln(seen)\n\t} else if var ready bool = false; ready {\n\t\tprintln(1)\n\t} else {\n\t\tprintln(seen)\n\t}\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let program = analyze_package(&ast).expect("analysis should succeed");

    assert_eq!(program.functions.len(), 1);
}

#[test]
fn analyze_switch_statements() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tvar counts = map[string]int{\"nova\": 3}\n\tswitch value, ok := counts[\"nova\"]; {\n\tcase ok:\n\t\tprintln(value)\n\tdefault:\n\t\tprintln(0)\n\t}\n\tvar score int = 2\n\tswitch score {\n\tcase 0, 1:\n\t\tprintln(0)\n\tcase 2:\n\t\tprintln(score)\n\tdefault:\n\t\tprintln(9)\n\t}\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let program = analyze_package(&ast).expect("analysis should succeed");

    assert_eq!(program.functions.len(), 1);
}

#[test]
fn analyze_short_declarations_and_inc_dec_statements() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\ttotal := 0\n\tvalues := []int{1, 2}\n\tfor i := 0; i < len(values); i++ {\n\t\ttotal = total + values[i]\n\t}\n\tcounts := map[string]int{\"go\": 1}\n\tcounts[\"go\"]++\n\tif count := len(values); count > 1 {\n\t\ttotal--\n\t}\n\tprintln(total, counts[\"go\"])\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let program = analyze_package(&ast).expect("analysis should succeed");

    assert_eq!(program.functions.len(), 1);
}

#[test]
fn analyze_compound_assignments() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\ttotal := 1\n\tvalues := []int{2, 3}\n\tfor i := 0; i < len(values); i += 1 {\n\t\ttotal += values[i]\n\t}\n\twords := map[string]string{\"lang\": \"go\"}\n\twords[\"lang\"] += \"pher\"\n\tif total -= 1; total > 0 {\n\t\tprintln(total, words[\"lang\"])\n\t}\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let program = analyze_package(&ast).expect("analysis should succeed");

    assert_eq!(program.functions.len(), 1);
}

#[test]
fn analyze_channels_first_slice() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tvar ready chan int\n\tprintln(ready == nil)\n\tready = make(chan int, 2)\n\tvar alias chan int = ready\n\tready <- 4\n\tvar first = <-ready\n\tclose(ready)\n\tprintln(len(ready), cap(ready), alias == ready, first, <-ready)\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let program = analyze_package(&ast).expect("analysis should succeed");

    assert_eq!(program.functions.len(), 1);
}

#[test]
fn reject_channel_send_type_mismatch() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tvar ready = make(chan int, 1)\n\tready <- \"oops\"\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let error = analyze_package(&ast).expect_err("analysis should reject channel send mismatch");

    assert!(
        error
            .to_string()
            .contains("send statement requires `int`, found `string`")
    );
}

#[test]
fn reject_short_declaration_in_same_scope() {
    let source = SourceFile {
        path: "test.go".into(),
        contents:
            "package main\n\nfunc main() {\n\tvalue := 1\n\tvalue := 2\n\tprintln(value)\n}\n"
                .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let error = analyze_package(&ast).expect_err("analysis should reject short redeclaration");

    assert!(
        error
            .to_string()
            .contains("short declaration `:=` requires at least one new named variable")
    );
}

#[test]
fn reject_inc_dec_on_non_numeric_target() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tvar label string = \"go\"\n\tlabel++\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let error = analyze_package(&ast).expect_err("analysis should reject non-numeric inc/dec");

    assert!(
        error
            .to_string()
            .contains("`++` requires `int` or `byte`, found `string`")
    );
}

#[test]
fn reject_compound_assignment_on_invalid_target_type() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tvar label bool = true\n\tlabel += false\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let error =
        analyze_package(&ast).expect_err("analysis should reject invalid compound assignment");

    assert!(
        error
            .to_string()
            .contains("`+=` requires `int`, `byte`, or `string`, found `bool`")
    );
}

#[test]
fn reject_switch_header_scope_leak() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tswitch var value int = 1; value {\n\tcase 1:\n\t\tprintln(value)\n\t}\n\tprintln(value)\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let error = analyze_package(&ast).expect_err("analysis should reject leaked switch binding");

    assert!(error.to_string().contains("unknown variable `value`"));
}

#[test]
fn reject_if_initializer_scope_leak() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tif value, ok := map[string]int{\"go\": 2}[\"go\"]; ok {\n\t\tprintln(value)\n\t}\n\tprintln(ok)\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let error = analyze_package(&ast).expect_err("analysis should reject leaked if binding");

    assert!(error.to_string().contains("unknown variable `ok`"));
}

#[test]
fn reject_range_loop_with_non_iterable_source() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tfor range 1 {\n\t\tprintln(1)\n\t}\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let error = analyze_package(&ast).expect_err("analysis should reject invalid range source");

    assert!(
        error
            .to_string()
            .contains("range loop requires `slice` or `map` source, found `int`")
    );
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
fn reject_duplicate_constant_map_literal_keys() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tvar counts = map[string]int{\"go\": 1, \"go\": 2}\n\tprintln(counts)\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let error = analyze_package(&ast).expect_err("analysis should reject duplicate keys");

    assert!(
        error
            .to_string()
            .contains("map literal contains duplicate constant key \"go\"")
    );
}

#[test]
fn reject_map_lookup_define_without_new_name() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tvar counts map[string]int\n\tvar value int = 1\n\tvar ok bool\n\tvalue, ok := counts[\"nova\"]\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let error = analyze_package(&ast).expect_err("analysis should reject stale short define");

    assert!(
        error
            .to_string()
            .contains("comma-ok lookup `:=` requires at least one new named variable")
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
fn analyze_grouped_import_aliases_and_bytes_package_calls() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nimport (\n\tb \"bytes\"\n\t\"fmt\"\n)\n\nfunc main() {\n\tvar joined = b.Join([][]byte{[]byte(\"go\"), []byte(\"vm\")}, []byte(\"-\"))\n\tfmt.Println(string(joined), b.Equal(nil, []byte{}), b.Contains(joined, []byte(\"vm\")), b.HasPrefix(joined, []byte(\"go\")))\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let program = analyze_package(&ast).expect("analysis should succeed");

    assert_eq!(program.functions.len(), 1);
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

#[test]
fn analyze_for_clauses_and_loop_control_statements() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tvar counts = map[string]int{\"go\": 2, \"nova\": 3}\n\tvar total int\n\tfor var i int = 0; i < 4; i = i + 1 {\n\t\tif i == 1 {\n\t\t\tcontinue\n\t\t}\n\t\tswitch i {\n\t\tcase 3:\n\t\t\tbreak\n\t\t}\n\t\ttotal = total + i\n\t}\n\tvar seen string\n\tfor key, value := range counts {\n\t\tif value < 3 {\n\t\t\tcontinue\n\t\t}\n\t\tseen = seen + key\n\t\tbreak\n\t}\n\tprintln(total, seen)\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let program = analyze_package(&ast).expect("analysis should succeed");

    assert_eq!(program.functions.len(), 1);
}

#[test]
fn analyze_multi_result_functions_and_cut_calls() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nimport (\n\t\"bytes\"\n\t\"strings\"\n)\n\nfunc split(value string) (string, string, bool) {\n\treturn strings.Cut(value, \"-\")\n}\n\nfunc main() {\n\thead, tail, found := split(\"nova-go\")\n\tbyteHead, byteTail, byteFound := bytes.Cut([]byte(\"vm-loop\"), []byte(\"-\"))\n\thead, tail, found = split(\"nova\")\n\tprintln(head, tail, found, string(byteHead), string(byteTail), byteFound)\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let program = analyze_package(&ast).expect("analysis should succeed");

    assert_eq!(
        program.functions[0].return_types,
        vec![Type::String, Type::String, Type::Bool]
    );
}

#[test]
fn analyze_call_argument_forwarding_and_cut_variants() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nimport (\n\t\"bytes\"\n\t\"strings\"\n)\n\nfunc pair() (string, string) {\n\treturn \"nova\", \"go\"\n}\n\nfunc joinPair(left string, right string) string {\n\treturn left + right\n}\n\nfunc describeStringFlag(value string, found bool) string {\n\tif found {\n\t\treturn value\n\t}\n\treturn \"missing\"\n}\n\nfunc describeBytesFlag(value []byte, found bool) string {\n\tif found {\n\t\treturn string(value)\n\t}\n\treturn \"missing\"\n}\n\nfunc main() {\n\tprintln(joinPair(pair()))\n\tprintln(strings.Cut(\"nova-go\", \"-\"))\n\tprintln(describeStringFlag(strings.CutPrefix(\"nova-go\", \"nova-\")))\n\tprintln(describeStringFlag(strings.CutSuffix(\"nova-go\", \"-go\")))\n\tprintln(describeBytesFlag(bytes.CutPrefix([]byte(\"nova-go\"), []byte(\"nova-\"))))\n\tprintln(describeBytesFlag(bytes.CutSuffix([]byte(\"nova-go\"), []byte(\"-go\"))))\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let program = analyze_package(&ast).expect("analysis should succeed");

    assert_eq!(program.functions.len(), 5);
}

#[test]
fn analyze_variadic_functions_and_append_spread() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc total(prefix int, values ...int) int {\n\tif values == nil {\n\t\treturn prefix\n\t}\n\tvar sum = prefix\n\tfor _, value := range values {\n\t\tsum = sum + value\n\t}\n\treturn sum\n}\n\nfunc main() {\n\tvar values = []int{2, 3}\n\tvar bytes = []byte(\"go\")\n\tprintln(total(1))\n\tprintln(total(1, 2, 3))\n\tprintln(total(1, values...))\n\tbytes = append(bytes, []byte(\"-nova\")...)\n\tbytes = append(bytes, \"!\"...)\n\tprintln(string(bytes))\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let program = analyze_package(&ast).expect("analysis should succeed");

    assert_eq!(program.functions.len(), 2);
    assert_eq!(program.functions[0].variadic_element_type, Some(Type::Int));
}

#[test]
fn analyze_grouped_parameter_names() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc describe(left, right string) string {\n\treturn left + right\n}\n\nfunc total(base, offset int, values ...int) int {\n\treturn base + offset + len(values)\n}\n\nfunc main() {\n\tprintln(describe(\"nova\", \"go\"))\n\tprintln(total(1, 2, 3, 4))\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let program = analyze_package(&ast).expect("analysis should succeed");

    assert_eq!(program.functions[0].parameter_count, 2);
    assert_eq!(program.functions[0].local_names, vec!["left", "right"]);
    assert_eq!(program.functions[1].parameter_count, 3);
    assert_eq!(program.functions[1].variadic_element_type, Some(Type::Int));
    assert_eq!(
        program.functions[1].local_names,
        vec!["base", "offset", "values"]
    );
}

#[test]
fn analyze_named_result_locals_and_bare_return() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc classify(value int) (sign string, abs int) {\n\tsign = \"non-negative\"\n\tabs = value\n\tif value < 0 {\n\t\tsign = \"negative\"\n\t\tabs = 0 - value\n\t\treturn\n\t}\n\treturn\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let program = analyze_package(&ast).expect("analysis should succeed");

    assert_eq!(
        program.functions[0].return_types,
        vec![Type::String, Type::Int]
    );
    assert_eq!(
        program.functions[0].local_names,
        vec!["value", "sign", "abs"]
    );
    assert_eq!(program.functions[0].result_locals.len(), 2);
}

#[test]
fn analyze_blank_named_result_slot() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc blankLabel(flag bool) (_ int, label string) {\n\tlabel = \"cold\"\n\treturn\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let program = analyze_package(&ast).expect("analysis should succeed");

    assert_eq!(
        program.functions[0].local_names,
        vec!["flag", "result$0", "label"]
    );
    assert_eq!(program.functions[0].result_locals.len(), 2);
}

#[test]
fn reject_duplicate_grouped_parameter_name() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc pair(left, right string, left int) {}\n".to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let error = analyze_package(&ast).expect_err("analysis should reject duplicate parameter");

    assert!(
        error
            .to_string()
            .contains("parameter `left` is already defined in function `pair`")
    );
}

#[test]
fn reject_bare_return_when_result_parameter_is_shadowed() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc shadow() (err string) {\n\tif true {\n\t\terr := \"inner\"\n\t\treturn\n\t}\n\treturn\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let error = analyze_package(&ast).expect_err("analysis should reject shadowed bare return");

    assert!(
        error
            .to_string()
            .contains("result parameter `err` not in scope at return")
    );
}

#[test]
fn reject_break_outside_breakable_statement() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tbreak\n}\n".to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let error = analyze_package(&ast).expect_err("analysis should reject top-level break");

    assert!(error.to_string().contains("`break` requires an enclosing"));
}

#[test]
fn reject_spread_call_to_non_variadic_function() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc fixed(values []int) {}\n\nfunc main() {\n\tvar values = []int{1, 2}\n\tfixed(values...)\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let error = analyze_package(&ast).expect_err("analysis should reject spread call");

    assert!(
        error
            .to_string()
            .contains("function `fixed` does not support explicit `...` arguments")
    );
}

#[test]
fn reject_spread_call_with_non_fixed_prefix_shape() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc total(values ...int) int {\n\treturn len(values)\n}\n\nfunc main() {\n\tvar values = []int{1, 2}\n\tprintln(total(1, values...))\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let error = analyze_package(&ast).expect_err("analysis should reject mixed prefix and spread");

    assert!(
        error
            .to_string()
            .contains("requires 0 fixed arguments before the spread value")
    );
}

#[test]
fn reject_continue_outside_loop() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc main() {\n\tswitch 1 {\n\tdefault:\n\t\tcontinue\n\t}\n}\n"
            .to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let error = analyze_package(&ast).expect_err("analysis should reject switch-only continue");

    assert!(
        error
            .to_string()
            .contains("`continue` requires an enclosing `for` or `range` loop")
    );
}

#[test]
fn reject_infinite_loop_with_break_as_missing_return() {
    let source = SourceFile {
        path: "test.go".into(),
        contents: "package main\n\nfunc helper() int {\n\tfor {\n\t\tbreak\n\t}\n}\n".to_string(),
    };

    let tokens = lex(&source).expect("lexing should succeed");
    let ast = parse_source_file(&tokens).expect("parsing should succeed");
    let error = analyze_package(&ast).expect_err("analysis should reject fallthrough via break");

    assert!(
        error
            .to_string()
            .contains("must return a `int` on every path")
    );
}
