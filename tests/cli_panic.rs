mod support;

use support::{cleanup_temp_source, run_cli, write_temp_source};

#[test]
fn run_reports_panic_example_output_and_message() {
    let error = run_cli(&["run", "examples/panic.go"]).expect_err("panic example should fail");

    assert_eq!(error, "body\ninner defer\nouter defer\npanic: boom");
}

#[test]
fn dump_ast_renders_panic_call() {
    let output = run_cli(&["dump-ast", "examples/panic.go"]).expect("ast should render");

    assert!(output.contains("panic(\"boom\")"));
}

#[test]
fn dump_bytecode_shows_explicit_panic_instruction() {
    let output = run_cli(&["dump-bytecode", "examples/panic.go"]).expect("bytecode should render");

    assert!(output.contains("panic"));
}

#[test]
fn dump_bytecode_shows_panic_nil_and_deferred_panic_nil() {
    let path = write_temp_source(
        "panic-nil-bytecode",
        "package main\n\nfunc main() {\n\tdefer panic(nil)\n\tpanic(nil)\n}\n",
    );

    let output =
        run_cli(&["dump-bytecode", path.to_str().unwrap()]).expect("bytecode should render");
    assert!(output.contains("defer-panic-nil"));
    assert!(output.contains("panic-nil"));

    cleanup_temp_source(path);
}

#[test]
fn check_accepts_panic_example() {
    let output = run_cli(&["check", "examples/panic.go"]).expect("check should accept example");

    assert!(output.contains("ok:"));
}

#[test]
fn run_unwinds_deferred_calls_for_nil_map_runtime_panic() {
    let path = write_temp_source(
        "panic-nil-map",
        "package main\n\nfunc inner() {\n\tdefer println(\"inner\")\n\tvar counts map[string]int\n\tcounts[\"go\"] = 1\n}\n\nfunc main() {\n\tdefer println(\"outer\")\n\tinner()\n}\n",
    );

    let error = run_cli(&["run", path.to_str().unwrap()]).expect_err("runtime panic should fail");
    assert_eq!(error, "inner\nouter\npanic: assignment to entry in nil map");

    cleanup_temp_source(path);
}

#[test]
fn run_reports_panic_nil_runtime_error() {
    let path = write_temp_source(
        "panic-nil-run",
        "package main\n\nfunc main() {\n\tdefer println(\"tail\")\n\tpanic(nil)\n}\n",
    );

    let error = run_cli(&["run", path.to_str().unwrap()]).expect_err("panic(nil) should fail");
    assert_eq!(error, "tail\npanic: panic called with nil argument");

    cleanup_temp_source(path);
}
