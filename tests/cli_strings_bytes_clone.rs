mod support;

use support::run_cli;

#[test]
fn run_executes_strings_and_bytes_clone_package_calls() {
    let output = run_cli(&["run", "examples/strings_bytes_clone.go"]).expect("program should run");

    assert_eq!(output, "nova\ntrue\nfalse\n[103 111]\n");
}

#[test]
fn dump_ast_renders_strings_and_bytes_clone_calls() {
    let output =
        run_cli(&["dump-ast", "examples/strings_bytes_clone.go"]).expect("ast should render");

    assert!(output.contains("strings.Clone(\"nova\")"));
    assert!(output.contains("bytes.Clone(nil)"));
    assert!(output.contains("bytes.Clone(empty)"));
}

#[test]
fn dump_bytecode_shows_strings_and_bytes_clone_calls() {
    let output = run_cli(&["dump-bytecode", "examples/strings_bytes_clone.go"])
        .expect("bytecode should be generated");

    assert!(output.contains("call-package strings.Clone 1"));
    assert!(output.contains("call-package bytes.Clone 1"));
}

#[test]
fn check_accepts_strings_and_bytes_clone_example() {
    let output = run_cli(&["check", "examples/strings_bytes_clone.go"])
        .expect("check should accept the example");

    assert!(output.contains("ok:"));
}
