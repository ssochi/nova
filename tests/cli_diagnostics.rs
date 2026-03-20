mod support;

use support::{cleanup_temp_source, run_cli, write_temp_source};

#[test]
fn run_rejects_missing_entry_function() {
    let path = write_temp_source(
        "missing-main",
        "package main\n\nfunc helper() {\n\tprintln(1)\n}\n",
    );

    let error = run_cli(&["run", path.to_str().unwrap()]).expect_err("run should fail");
    assert!(error.contains("entry function `main` was not found"));

    cleanup_temp_source(path);
}

#[test]
fn check_accepts_a_valid_source_file() {
    let path = write_temp_source(
        "check-ok",
        "package util\n\nfunc helper() {\n\tvar value = 3 + 4\n\tprintln(value)\n}\n",
    );

    let output = run_cli(&["check", path.to_str().unwrap()]).expect("check should pass");
    assert!(output.contains("ok:"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_non_boolean_if_condition() {
    let path = write_temp_source(
        "check-bad-if",
        "package main\n\nfunc main() {\n\tif 1 {\n\t\tprintln(1)\n\t}\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("if condition requires `bool`, found `int`"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_non_boolean_for_condition() {
    let path = write_temp_source(
        "check-bad-for",
        "package main\n\nfunc main() {\n\tfor 1 {\n\t\tprintln(1)\n\t}\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("for condition requires `bool`, found `int`"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_break_outside_breakable_statement() {
    let path = write_temp_source(
        "check-bad-break",
        "package main\n\nfunc main() {\n\tbreak\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("`break` requires an enclosing `for`, `range`, or `switch`"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_continue_outside_loop() {
    let path = write_temp_source(
        "check-bad-continue",
        "package main\n\nfunc main() {\n\tswitch 1 {\n\tdefault:\n\t\tcontinue\n\t}\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("`continue` requires an enclosing `for` or `range` loop"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_range_with_non_iterable_source() {
    let path = write_temp_source(
        "check-bad-range-source",
        "package main\n\nfunc main() {\n\tfor range 1 {\n\t\tprintln(1)\n\t}\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("range loop requires `slice` or `map` source, found `int`"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_range_assignment_type_mismatch() {
    let path = write_temp_source(
        "check-bad-range-assign",
        "package main\n\nfunc main() {\n\tvar label string\n\tfor label = range []int{1} {\n\t\tprintln(label)\n\t}\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("range loop assignment to `label` requires `string`, found `int`"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_range_define_without_named_variable() {
    let path = write_temp_source(
        "check-bad-range-define",
        "package main\n\nfunc main() {\n\tfor _, _ := range []int{1} {\n\t\tprintln(1)\n\t}\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("range loop `:=` requires at least one named iteration variable"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_invalid_len_argument_type() {
    let path = write_temp_source(
        "check-bad-len",
        "package main\n\nfunc main() {\n\tprintln(len(1))\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains(
        "argument 1 in call to builtin `len` requires `string`, `slice`, `chan`, or `map`, found `int`"
    ));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_typed_var_initializer_type_mismatch() {
    let path = write_temp_source(
        "check-bad-typed-var",
        "package main\n\nfunc main() {\n\tvar ready bool = 1\n\tprintln(ready)\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("variable `ready` requires `bool`, found `int`"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_package_call_without_import() {
    let path = write_temp_source(
        "check-missing-import",
        "package main\n\nfunc main() {\n\tfmt.Println(1)\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("package `fmt` is not imported"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_unsupported_import_member() {
    let path = write_temp_source(
        "check-bad-import-member",
        "package main\n\nimport \"fmt\"\n\nfunc main() {\n\tfmt.Printf(1)\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("does not export supported member `Printf`"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_dot_imports() {
    let path = write_temp_source(
        "check-dot-import",
        "package main\n\nimport . \"fmt\"\n\nfunc main() {\n\tPrintln(1)\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("dot imports are not supported"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_blank_imports() {
    let path = write_temp_source(
        "check-blank-import",
        "package main\n\nimport _ \"fmt\"\n\nfunc main() {}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("blank imports are not supported"));

    cleanup_temp_source(path);
}

#[test]
fn run_rejects_missing_return_on_value_function() {
    let path = write_temp_source(
        "check-missing-return",
        "package main\n\nfunc helper() int {\n\tif true {\n\t\treturn 1\n\t}\n}\n\nfunc main() {\n\tprintln(helper())\n}\n",
    );

    let error = run_cli(&["run", path.to_str().unwrap()]).expect_err("run should fail");
    assert!(error.contains("must return a `int` on every path"));

    cleanup_temp_source(path);
}

#[test]
fn dump_tokens_rejects_unterminated_string_literal() {
    let path = write_temp_source(
        "unterminated-string",
        "package main\n\nfunc main() {\n\tprintln(\"oops)\n}\n",
    );

    let error = run_cli(&["dump-tokens", path.to_str().unwrap()]).expect_err("lexing should fail");
    assert!(error.contains("unterminated string literal"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_value_function_with_only_conditional_loop_return() {
    let path = write_temp_source(
        "check-loop-missing-return",
        "package main\n\nfunc helper(value int) int {\n\tfor value > 0 {\n\t\treturn value\n\t}\n}\n\nfunc main() {\n\tprintln(helper(1))\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("must return a `int` on every path"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_value_function_with_breaking_infinite_loop() {
    let path = write_temp_source(
        "check-loop-break-return",
        "package main\n\nfunc helper() int {\n\tfor {\n\t\tbreak\n\t}\n}\n\nfunc main() {\n\tprintln(helper())\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("must return a `int` on every path"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_non_integer_slice_index() {
    let path = write_temp_source(
        "check-bad-slice-index",
        "package main\n\nfunc main() {\n\tvar values = []int{1, 2}\n\tprintln(values[true])\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("index expression requires `int`, found `bool`"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_append_element_type_mismatch() {
    let path = write_temp_source(
        "check-bad-append",
        "package main\n\nfunc main() {\n\tvar values = []int{1, 2}\n\tvalues = append(values, \"oops\")\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(
        error.contains("argument 2 in call to builtin `append` requires `int`, found `string`")
    );

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_bad_strings_join_argument_type() {
    let path = write_temp_source(
        "check-bad-strings-join",
        "package main\n\nimport \"strings\"\n\nfunc main() {\n\tprintln(strings.Join(\"oops\", \",\"))\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(
        error.contains("argument 1 in call to `strings.Join` requires `[]string`, found `string`")
    );

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_unsupported_strings_member() {
    let path = write_temp_source(
        "check-bad-strings-member",
        "package main\n\nimport \"strings\"\n\nfunc main() {\n\tprintln(strings.ToUpper(\"nova\"))\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("does not export supported member `ToUpper`"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_bad_bytes_join_argument_type() {
    let path = write_temp_source(
        "check-bad-bytes-join",
        "package main\n\nimport b \"bytes\"\n\nfunc main() {\n\tprintln(b.Join([]byte(\"oops\"), []byte(\",\")))\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(
        error.contains("argument 1 in call to `bytes.Join` requires `[][]byte`, found `[]byte`")
    );

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_unsupported_bytes_member() {
    let path = write_temp_source(
        "check-bad-bytes-member",
        "package main\n\nimport b \"bytes\"\n\nfunc main() {\n\tprintln(b.ToUpper([]byte(\"nova\")))\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("does not export supported member `ToUpper`"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_bad_slice_upper_bound_type() {
    let path = write_temp_source(
        "check-bad-slice-upper",
        "package main\n\nfunc main() {\n\tvar values = []int{1, 2}\n\tprintln(values[0:true])\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("slice expression upper bound requires `int`, found `bool`"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_copy_string_into_non_byte_slice() {
    let path = write_temp_source(
        "check-bad-byte-copy",
        "package main\n\nfunc main() {\n\tvar values = make([]int, 2)\n\tprintln(copy(values, \"no\"))\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(
        error.contains("argument 2 in call to builtin `copy` requires `[]int`, found `string`")
    );

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_invalid_cap_argument_type() {
    let path = write_temp_source(
        "check-bad-cap",
        "package main\n\nfunc main() {\n\tprintln(cap(\"oops\"))\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains(
        "argument 1 in call to builtin `cap` requires `slice` or `chan`, found `string`"
    ));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_short_declaration_without_new_variable() {
    let path = write_temp_source(
        "check-bad-short-decl",
        "package main\n\nfunc main() {\n\tvalue := 1\n\tvalue := 2\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("short declaration `:=` requires at least one new named variable"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_inc_dec_on_string_target() {
    let path = write_temp_source(
        "check-bad-incdec",
        "package main\n\nfunc main() {\n\tvar label string = \"go\"\n\tlabel++\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("`++` requires `int` or `byte`, found `string`"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_compound_assignment_on_invalid_target_type() {
    let path = write_temp_source(
        "check-bad-compound-type",
        "package main\n\nfunc main() {\n\tvar label bool = true\n\tlabel += false\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("`+=` requires `int`, `byte`, or `string`, found `bool`"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_compound_assignment_on_non_assignable_left_side() {
    let path = write_temp_source(
        "check-bad-compound-target",
        "package main\n\nfunc main() {\n\tlen([]int{1}) += 1\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("assignment target must be a variable name or index expression"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_short_declaration_in_for_post_clause() {
    let path = write_temp_source(
        "check-bad-for-post-short",
        "package main\n\nfunc main() {\n\tfor i := 0; i < 3; i := 1 {\n\t\tprintln(i)\n\t}\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("for post statement does not support `:=`"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_untyped_nil_variable_inference() {
    let path = write_temp_source(
        "check-bad-nil-var",
        "package main\n\nfunc main() {\n\tvar values = nil\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("requires an explicit type when initialized with `nil`"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_nil_equals_nil() {
    let path = write_temp_source(
        "check-bad-nil-equality",
        "package main\n\nfunc main() {\n\tprintln(nil == nil)\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("does not support untyped `nil` operands"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_copy_slice_type_mismatch() {
    let path = write_temp_source(
        "check-bad-copy",
        "package main\n\nfunc main() {\n\tvar left = []int{1, 2}\n\tvar right = []string{\"x\"}\n\tprintln(copy(left, right))\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(
        error.contains("argument 2 in call to builtin `copy` requires `[]int`, found `[]string`")
    );

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_slice_assignment_type_mismatch() {
    let path = write_temp_source(
        "check-bad-slice-assign",
        "package main\n\nfunc main() {\n\tvar values = []int{1, 2}\n\tvalues[0] = \"oops\"\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("slice element assignment requires `int`, found `string`"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_full_slice_expression() {
    let path = write_temp_source(
        "check-full-slice",
        "package main\n\nfunc main() {\n\tvar values = []int{1, 2, 3}\n\tprintln(values[0:2:3])\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("full slice expressions are not supported"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_make_with_non_slice_type_argument() {
    let path = write_temp_source(
        "check-bad-make-type",
        "package main\n\nfunc main() {\n\tvar values = make(int, 2)\n\tprintln(values)\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains(
        "argument 1 in call to builtin `make` requires `slice`, `chan`, or `map`, found `int`"
    ));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_make_when_length_exceeds_capacity() {
    let path = write_temp_source(
        "check-bad-make-bounds",
        "package main\n\nfunc main() {\n\tvar values = make([]int, 3, 2)\n\tprintln(values)\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("builtin `make` length 3 exceeds capacity 2"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_string_conversion_from_non_byte_slice() {
    let path = write_temp_source(
        "check-bad-string-conversion",
        "package main\n\nfunc main() {\n\tvar values = []int{65}\n\tprintln(string(values))\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("conversion to `string` requires `[]byte`, found `[]int`"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_non_comparable_map_key_type() {
    let path = write_temp_source(
        "check-bad-map-key",
        "package main\n\nfunc main() {\n\tvar counts map[[]int]int\n\tprintln(counts)\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("comparable map key type"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_map_literal_value_type_mismatch() {
    let path = write_temp_source(
        "check-bad-map-literal-value",
        "package main\n\nfunc main() {\n\tvar counts = map[string]int{\"nova\": \"oops\"}\n\tprintln(counts)\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("map literal value 1 requires `int`, found `string`"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_duplicate_constant_map_literal_keys() {
    let path = write_temp_source(
        "check-bad-map-literal-duplicate",
        "package main\n\nfunc main() {\n\tvar counts = map[string]int{\"go\": 1, \"go\": 2}\n\tprintln(counts)\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("map literal contains duplicate constant key \"go\""));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_map_lookup_with_non_map_target() {
    let path = write_temp_source(
        "check-bad-map-lookup-target",
        "package main\n\nfunc main() {\n\tvar values = []int{1}\n\tvalue, ok := values[0]\n\tprintln(value, ok)\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("comma-ok lookup requires `map` target, found `[]int`"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_map_lookup_define_without_new_name() {
    let path = write_temp_source(
        "check-bad-map-lookup-define",
        "package main\n\nfunc main() {\n\tvar counts map[string]int\n\tvar value int = 1\n\tvar ok bool\n\tvalue, ok := counts[\"nova\"]\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("comma-ok lookup `:=` requires at least one new named variable"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_if_header_scope_leak() {
    let path = write_temp_source(
        "check-bad-if-scope",
        "package main\n\nfunc main() {\n\tif value, ok := map[string]int{\"go\": 2}[\"go\"]; ok {\n\t\tprintln(value)\n\t}\n\tprintln(ok)\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("unknown variable `ok`"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_missing_if_header_semicolon() {
    let path = write_temp_source(
        "check-bad-if-header-semicolon",
        "package main\n\nfunc main() {\n\tvar counts = map[string]int{\"go\": 2}\n\tif value, ok := counts[\"go\"] {\n\t\tprintln(value, ok)\n\t}\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("expected `;`"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_switch_header_scope_leak() {
    let path = write_temp_source(
        "check-bad-switch-scope",
        "package main\n\nfunc main() {\n\tswitch var value int = 1; value {\n\tcase 1:\n\t\tprintln(value)\n\t}\n\tprintln(value)\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("unknown variable `value`"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_duplicate_switch_default() {
    let path = write_temp_source(
        "check-bad-switch-default",
        "package main\n\nfunc main() {\n\tswitch 1 {\n\tdefault:\n\t\tprintln(0)\n\tcase 1:\n\t\tprintln(1)\n\tdefault:\n\t\tprintln(2)\n\t}\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("may only contain one `default` clause"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_duplicate_switch_literal_case() {
    let path = write_temp_source(
        "check-bad-switch-case",
        "package main\n\nfunc main() {\n\tswitch 1 {\n\tcase 1:\n\t\tprintln(1)\n\tcase 1:\n\t\tprintln(2)\n\t}\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("duplicate switch case literal 1"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_delete_with_wrong_key_type() {
    let path = write_temp_source(
        "check-bad-delete-key",
        "package main\n\nfunc main() {\n\tvar counts = make(map[string]int)\n\tdelete(counts, 1)\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(
        error.contains("argument 2 in call to builtin `delete` requires `string`, found `int`")
    );

    cleanup_temp_source(path);
}

#[test]
fn run_rejects_nil_map_assignment() {
    let path = write_temp_source(
        "run-nil-map-assign",
        "package main\n\nfunc main() {\n\tvar counts map[string]int\n\tcounts[\"nova\"] = 1\n}\n",
    );

    let error = run_cli(&["run", path.to_str().unwrap()]).expect_err("run should fail");
    assert!(error.contains("assignment to entry in nil map"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_channel_send_type_mismatch() {
    let path = write_temp_source(
        "check-bad-channel-send",
        "package main\n\nfunc main() {\n\tvar ready = make(chan int, 1)\n\tready <- \"oops\"\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("send statement requires `int`, found `string`"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_close_on_non_channel() {
    let path = write_temp_source(
        "check-bad-close-target",
        "package main\n\nfunc main() {\n\tclose([]int{1})\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("argument 1 in call to builtin `close` requires `chan`, found `[]int`"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_multi_result_call_in_single_value_context() {
    let path = write_temp_source(
        "check-bad-multi-result-single-context",
        "package main\n\nfunc pair() (int, int) {\n\treturn 1, 2\n}\n\nfunc main() {\n\tprintln(pair())\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("call to `pair` produces `(int, int)`"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_multi_result_binding_arity_mismatch() {
    let path = write_temp_source(
        "check-bad-multi-result-arity",
        "package main\n\nimport \"strings\"\n\nfunc main() {\n\thead, found := strings.Cut(\"nova-go\", \"-\")\n\tprintln(head, found)\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("short declaration expects 2 values, found 3"));

    cleanup_temp_source(path);
}
