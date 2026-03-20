mod support;

use support::{cleanup_temp_source, run_cli, write_temp_source};

#[test]
fn run_executes_any_interface_example() {
    let output = run_cli(&["run", "examples/empty_interface_any.go"]).expect("program should run");

    assert_eq!(output, "true\nfalse\ntrue\nboom\ngo 7 <nil>\n");
}

#[test]
fn dump_ast_renders_any_and_interface_types() {
    let output =
        run_cli(&["dump-ast", "examples/empty_interface_any.go"]).expect("ast should render");

    assert!(output.contains("func wrap(value string) any"));
    assert!(output.contains("var boxed interface{} = []byte(\"go\")"));
    assert!(output.contains("fmt.Println(args...)"));
}

#[test]
fn dump_bytecode_shows_boxing_and_package_spread() {
    let output = run_cli(&["dump-bytecode", "examples/empty_interface_any.go"])
        .expect("bytecode should render");

    assert!(output.contains("push-nil-interface"));
    assert!(output.contains("box-any string"));
    assert!(output.contains("box-any []byte"));
    assert!(output.contains("call-package-spread fmt.Println 0"));
}

#[test]
fn check_accepts_any_interface_example() {
    let output = run_cli(&["check", "examples/empty_interface_any.go"])
        .expect("check should accept the example");

    assert!(output.contains("ok:"));
}

#[test]
fn run_reports_runtime_error_for_uncomparable_interface_equality() {
    let path = write_temp_source(
        "any-interface-uncomparable",
        "package main\n\nfunc main() {\n\tvar value any = []int{1}\n\tprintln(value == value)\n}\n",
    );

    let error = run_cli(&["run", path.to_str().unwrap()]).expect_err("program should fail");
    assert_eq!(
        error,
        "panic: runtime error: comparing uncomparable interface value"
    );

    cleanup_temp_source(path);
}
