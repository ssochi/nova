mod support;

use support::{cleanup_temp_source, run_cli, write_temp_source};

#[test]
fn run_executes_recover_example() {
    let output = run_cli(&["run", "examples/recover.go"]).expect("program should run");

    assert_eq!(output, "<nil>\n<nil>\nboom\n0\n[103 111]\n7\n");
}

#[test]
fn dump_ast_renders_recover_calls() {
    let output = run_cli(&["dump-ast", "examples/recover.go"]).expect("ast should render");

    assert!(output.contains("fmt.Println(recover())"));
    assert!(output.contains("defer printRecover()"));
}

#[test]
fn dump_bytecode_shows_recover_calls_and_typed_panic_payloads() {
    let output =
        run_cli(&["dump-bytecode", "examples/recover.go"]).expect("bytecode should render");

    assert!(output.contains("call-builtin recover 0"));
    assert!(output.contains("panic string"));
    assert!(output.contains("panic []byte"));
}

#[test]
fn check_accepts_recover_example() {
    let output = run_cli(&["check", "examples/recover.go"]).expect("check should accept example");

    assert!(output.contains("ok:"));
}

#[test]
fn deferred_builtin_recover_does_not_stop_a_panic() {
    let path = write_temp_source(
        "recover-deferred-builtin",
        "package main\n\nfunc main() {\n\tdefer recover()\n\tpanic(\"boom\")\n}\n",
    );

    let error = run_cli(&["run", path.to_str().unwrap()])
        .expect_err("deferred builtin recover should fail");
    assert_eq!(error, "panic: boom");

    cleanup_temp_source(path);
}

#[test]
fn helper_called_by_deferred_function_does_not_recover() {
    let path = write_temp_source(
        "recover-helper-indirect",
        "package main\n\nimport \"fmt\"\n\nfunc helper() any {\n\treturn recover()\n}\n\nfunc deferred() {\n\tfmt.Println(helper())\n}\n\nfunc main() {\n\tdefer deferred()\n\tpanic(\"boom\")\n}\n",
    );

    let error = run_cli(&["run", path.to_str().unwrap()]).expect_err("helper recover should fail");
    assert_eq!(error, "<nil>\npanic: boom");

    cleanup_temp_source(path);
}
