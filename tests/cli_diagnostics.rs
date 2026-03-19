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
fn check_rejects_invalid_len_argument_type() {
    let path = write_temp_source(
        "check-bad-len",
        "package main\n\nfunc main() {\n\tprintln(len(1))\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(
        error.contains(
            "argument 1 in call to builtin `len` requires `string` or `slice`, found `int`"
        )
    );

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
