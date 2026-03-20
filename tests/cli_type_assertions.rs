mod support;

use support::{cleanup_temp_source, run_cli, write_temp_source};

#[test]
fn run_executes_type_assertions_example() {
    let output = run_cli(&["run", "examples/type_assertions.go"]).expect("program should run");

    assert_eq!(output, "go\ntrue 0\ntrue\n");
}

#[test]
fn dump_ast_renders_type_assertions() {
    let output = run_cli(&["dump-ast", "examples/type_assertions.go"]).expect("ast should render");

    assert!(output.contains("var word = text.(string)"));
    assert!(output.contains("var bytes = payload.([]byte)"));
    assert!(output.contains("count.(any)"));
}

#[test]
fn dump_bytecode_shows_type_assert_instructions() {
    let output =
        run_cli(&["dump-bytecode", "examples/type_assertions.go"]).expect("bytecode should render");

    assert!(output.contains("type-assert string"));
    assert!(output.contains("type-assert []byte"));
    assert!(output.contains("type-assert any"));
}

#[test]
fn check_accepts_type_assertions_example() {
    let output =
        run_cli(&["check", "examples/type_assertions.go"]).expect("check should accept example");

    assert!(output.contains("ok:"));
}

#[test]
fn run_reports_runtime_error_for_nil_interface_assertion() {
    let path = write_temp_source(
        "type-assertion-nil",
        "package main\n\nfunc main() {\n\tvar value any\n\tprintln(value.(string))\n}\n",
    );

    let error = run_cli(&["run", path.to_str().unwrap()]).expect_err("program should fail");
    assert_eq!(
        error,
        "panic: interface conversion: interface {} is nil, not string"
    );

    cleanup_temp_source(path);
}

#[test]
fn run_reports_runtime_error_for_mismatched_type_assertion() {
    let path = write_temp_source(
        "type-assertion-mismatch",
        "package main\n\nfunc main() {\n\tvar value any = \"go\"\n\tprintln(value.([]byte))\n}\n",
    );

    let error = run_cli(&["run", path.to_str().unwrap()]).expect_err("program should fail");
    assert_eq!(
        error,
        "panic: interface conversion: interface {} is string, not []byte"
    );

    cleanup_temp_source(path);
}
