mod support;

use support::run_cli;

#[test]
fn run_executes_strings_and_bytes_compare_package_calls() {
    let output =
        run_cli(&["run", "examples/strings_bytes_compare.go"]).expect("program should run");

    assert_eq!(output, "0\n-1\n1\n0\n-1\n1\n");
}

#[test]
fn dump_ast_renders_strings_and_bytes_compare_calls() {
    let output =
        run_cli(&["dump-ast", "examples/strings_bytes_compare.go"]).expect("ast should render");

    assert!(output.contains("strings.Compare(\"go\", \"go\")"));
    assert!(output.contains("bytes.Compare(nil, empty)"));
    assert!(output.contains("bytes.Compare([]byte(\"vm\"), nil)"));
}

#[test]
fn dump_bytecode_shows_strings_and_bytes_compare_calls() {
    let output = run_cli(&["dump-bytecode", "examples/strings_bytes_compare.go"])
        .expect("bytecode should be generated");

    assert!(output.contains("call-package strings.Compare 2"));
    assert!(output.contains("call-package bytes.Compare 2"));
}

#[test]
fn check_accepts_strings_and_bytes_compare_example() {
    let output = run_cli(&["check", "examples/strings_bytes_compare.go"])
        .expect("check should accept the example");

    assert!(output.contains("ok:"));
}
