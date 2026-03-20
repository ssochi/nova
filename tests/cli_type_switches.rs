mod support;

use support::{cleanup_temp_source, run_cli, write_temp_source};

#[test]
fn run_executes_type_switches_and_comma_ok_example() {
    let output =
        run_cli(&["run", "examples/type_switches_and_comma_ok.go"]).expect("program should run");

    assert_eq!(
        output,
        "false\ntrue nova\nbytes nova\nfalse\nnil true\nmulti true\n"
    );
}

#[test]
fn dump_ast_renders_type_switches_and_comma_ok_assertions() {
    let output = run_cli(&["dump-ast", "examples/type_switches_and_comma_ok.go"])
        .expect("ast should render");

    assert!(output.contains("word, ok := boxed.(string)"));
    assert!(output.contains("bytes, ok := boxed.([]byte)"));
    assert!(output.contains("switch current := boxed.(type) {"));
    assert!(output.contains("case []byte:"));
    assert!(output.contains("case nil:"));
}

#[test]
fn dump_bytecode_shows_type_assert_ok_instructions() {
    let output = run_cli(&["dump-bytecode", "examples/type_switches_and_comma_ok.go"])
        .expect("bytecode should render");

    assert!(output.contains("type-assert-ok string"));
    assert!(output.contains("type-assert-ok []byte"));
    assert!(output.contains("type-assert-ok bool"));
}

#[test]
fn check_accepts_type_switches_and_comma_ok_example() {
    let output = run_cli(&["check", "examples/type_switches_and_comma_ok.go"])
        .expect("check should accept example");

    assert!(output.contains("ok:"));
}

#[test]
fn run_executes_type_switch_header_form() {
    let path = write_temp_source(
        "type-switch-header",
        "package main\n\nfunc main() {\n\tvar value any = true\n\tswitch seen, ok := value.(bool); value.(type) {\n\tcase bool:\n\t\tprintln(ok, seen)\n\tdefault:\n\t\tprintln(false)\n\t}\n}\n",
    );

    let output = run_cli(&["run", path.to_str().unwrap()]).expect("program should run");
    assert_eq!(output, "true true\n");

    cleanup_temp_source(path);
}
