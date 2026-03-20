mod support;

use support::run_cli;

#[test]
fn run_executes_builtin_clear_example() {
    let output = run_cli(&["run", "examples/builtin_clear.go"]).expect("program should run");

    assert_eq!(output, "1 0 0 4 2 3\ntrue 0 0\ntrue true\n0 0 false\n");
}

#[test]
fn dump_ast_renders_clear_calls() {
    let output = run_cli(&["dump-ast", "examples/builtin_clear.go"]).expect("ast should render");

    assert!(output.contains("clear(window)"));
    assert!(output.contains("clear(missing)"));
    assert!(output.contains("clear(alias)"));
}

#[test]
fn dump_bytecode_shows_clear_builtin_calls() {
    let output =
        run_cli(&["dump-bytecode", "examples/builtin_clear.go"]).expect("bytecode should render");

    assert!(output.contains("call-builtin clear 1"));
}

#[test]
fn check_accepts_builtin_clear_example() {
    let output =
        run_cli(&["check", "examples/builtin_clear.go"]).expect("check should accept the example");

    assert!(output.contains("ok:"));
}
