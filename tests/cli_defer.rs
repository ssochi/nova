mod support;

use support::run_cli;

#[test]
fn run_executes_defer_example() {
    let output = run_cli(&["run", "examples/defer.go"]).expect("program should run");

    assert_eq!(output, "body 9\npackage 1\npair\nbuiltin\n");
}

#[test]
fn dump_tokens_show_defer_keyword() {
    let output = run_cli(&["dump-tokens", "examples/defer.go"]).expect("tokens should render");

    assert!(output.contains("defer"));
}

#[test]
fn dump_ast_renders_defer_statements() {
    let output = run_cli(&["dump-ast", "examples/defer.go"]).expect("ast should render");

    assert!(output.contains("defer println(\"builtin\")"));
    assert!(output.contains("defer pair()"));
    assert!(output.contains("defer fmt.Println(\"package\", value)"));
}

#[test]
fn dump_bytecode_shows_explicit_defer_instructions() {
    let output = run_cli(&["dump-bytecode", "examples/defer.go"]).expect("bytecode should render");

    assert!(output.contains("defer-builtin println 1"));
    assert!(output.contains("defer-function"));
    assert!(output.contains("defer-package fmt.Println 2"));
}

#[test]
fn check_accepts_defer_example() {
    let output = run_cli(&["check", "examples/defer.go"]).expect("check should accept example");

    assert!(output.contains("ok:"));
}
