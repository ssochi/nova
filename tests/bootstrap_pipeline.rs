use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn run_cli(arguments: &[&str]) -> Result<String, String> {
    let mut args = vec!["nova-go".to_string()];
    args.extend(arguments.iter().map(|value| value.to_string()));
    nova_go::run_cli(args).map_err(|error| error.to_string())
}

fn write_temp_source(name: &str, contents: &str) -> PathBuf {
    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("nova-go-{name}-{unique_suffix}.go"));
    fs::write(&path, contents).expect("temporary source file should be written");
    path
}

#[test]
fn run_executes_entrypoint_program() {
    let output = run_cli(&["run", "examples/hello.go"]).expect("program should run");
    assert_eq!(output, "42\n");
}

#[test]
fn run_executes_multi_function_branches() {
    let output =
        run_cli(&["run", "examples/functions_and_branches.go"]).expect("program should run");
    assert_eq!(output, "false 11\n");
}

#[test]
fn run_executes_loops() {
    let output = run_cli(&["run", "examples/loops.go"]).expect("program should run");
    assert_eq!(output, "10 4\n");
}

#[test]
fn run_executes_strings_and_builtins() {
    let output = run_cli(&["run", "examples/strings.go"]).expect("program should run");
    assert_eq!(output, "hello, nova! 11\ntrue\n");
}

#[test]
fn run_executes_imported_fmt_package_calls() {
    let output = run_cli(&["run", "examples/imports_fmt.go"]).expect("program should run");
    assert_eq!(output, "hello, nova\nbytes=11");
}

#[test]
fn dump_bytecode_shows_stack_machine_instructions() {
    let output = run_cli(&["dump-bytecode", "examples/arithmetic.go"])
        .expect("bytecode should be generated");

    assert!(output.contains("call-builtin println 1"));
    assert!(output.contains("multiply"));
    assert!(output.contains("store-local 2"));
}

#[test]
fn dump_bytecode_shows_function_calls_and_branch_jumps() {
    let output = run_cli(&["dump-bytecode", "examples/functions_and_branches.go"])
        .expect("bytecode should be generated");

    assert!(output.contains("call-function 0 2"));
    assert!(output.contains("jump-if-false"));
    assert!(output.contains("greater"));
    assert_eq!(output.matches("jump-if-false").count(), 2);
}

#[test]
fn dump_ast_renders_loops() {
    let output = run_cli(&["dump-ast", "examples/loops.go"]).expect("ast should be rendered");

    assert!(output.contains("for (current > 0) {"));
    assert!(output.contains("for true {"));
}

#[test]
fn dump_tokens_show_string_literals() {
    let output =
        run_cli(&["dump-tokens", "examples/strings.go"]).expect("tokens should be rendered");

    assert!(output.contains("string(\"hello, \")"));
    assert!(output.contains("string(\"nova\")"));
}

#[test]
fn dump_tokens_show_imports_and_selector_calls() {
    let output =
        run_cli(&["dump-tokens", "examples/imports_fmt.go"]).expect("tokens should be rendered");

    assert!(output.contains("import"));
    assert!(output.contains("string(\"fmt\")"));
    assert!(output.contains("identifier(fmt)"));
    assert!(output.contains("."));
}

#[test]
fn dump_ast_renders_strings_and_builtins() {
    let output = run_cli(&["dump-ast", "examples/strings.go"]).expect("ast should be rendered");

    assert!(output.contains("return (\"hello, \" + name)"));
    assert!(output.contains("println(\"!\", len(greeting))"));
}

#[test]
fn dump_ast_renders_imports_and_package_calls() {
    let output = run_cli(&["dump-ast", "examples/imports_fmt.go"]).expect("ast should be rendered");

    assert!(output.contains("import \"fmt\""));
    assert!(output.contains("return fmt.Sprint(\"hello, \", name)"));
    assert!(output.contains("fmt.Print(fmt.Sprint(\"bytes=\", len(message)))"));
}

#[test]
fn dump_bytecode_shows_loop_jumps() {
    let output =
        run_cli(&["dump-bytecode", "examples/loops.go"]).expect("bytecode should be generated");

    assert!(output.contains("function 0: sumDown"));
    assert!(output.contains("function 1: climbPast"));
    assert!(output.matches("jump-if-false").count() >= 2);
    assert!(output.contains("jump 2"));
}

#[test]
fn dump_bytecode_shows_string_instructions_and_builtins() {
    let output =
        run_cli(&["dump-bytecode", "examples/strings.go"]).expect("bytecode should be generated");

    assert!(output.contains("push-string \"hello, \""));
    assert!(output.contains("concat"));
    assert!(output.contains("call-builtin print 1"));
    assert!(output.contains("call-builtin len 1"));
}

#[test]
fn dump_bytecode_shows_package_calls() {
    let output = run_cli(&["dump-bytecode", "examples/imports_fmt.go"])
        .expect("bytecode should be generated");

    assert!(output.contains("call-package fmt.Sprint 2"));
    assert!(output.contains("call-package fmt.Println 1"));
    assert!(output.contains("call-package fmt.Print 1"));
}

#[test]
fn run_rejects_missing_entry_function() {
    let path = write_temp_source(
        "missing-main",
        "package main\n\nfunc helper() {\n\tprintln(1)\n}\n",
    );

    let error = run_cli(&["run", path.to_str().unwrap()]).expect_err("run should fail");
    assert!(error.contains("entry function `main` was not found"));

    let _ = fs::remove_file(path);
}

#[test]
fn check_accepts_a_valid_source_file() {
    let path = write_temp_source(
        "check-ok",
        "package util\n\nfunc helper() {\n\tvar value = 3 + 4\n\tprintln(value)\n}\n",
    );

    let output = run_cli(&["check", path.to_str().unwrap()]).expect("check should pass");
    assert!(output.contains("ok:"));

    let _ = fs::remove_file(path);
}

#[test]
fn check_rejects_non_boolean_if_condition() {
    let path = write_temp_source(
        "check-bad-if",
        "package main\n\nfunc main() {\n\tif 1 {\n\t\tprintln(1)\n\t}\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("if condition requires `bool`, found `int`"));

    let _ = fs::remove_file(path);
}

#[test]
fn check_rejects_non_boolean_for_condition() {
    let path = write_temp_source(
        "check-bad-for",
        "package main\n\nfunc main() {\n\tfor 1 {\n\t\tprintln(1)\n\t}\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("for condition requires `bool`, found `int`"));

    let _ = fs::remove_file(path);
}

#[test]
fn check_rejects_invalid_len_argument_type() {
    let path = write_temp_source(
        "check-bad-len",
        "package main\n\nfunc main() {\n\tprintln(len(1))\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("argument 1 in call to builtin `len` requires `string`, found `int`"));

    let _ = fs::remove_file(path);
}

#[test]
fn check_rejects_package_call_without_import() {
    let path = write_temp_source(
        "check-missing-import",
        "package main\n\nfunc main() {\n\tfmt.Println(1)\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("package `fmt` is not imported"));

    let _ = fs::remove_file(path);
}

#[test]
fn check_rejects_unsupported_import_member() {
    let path = write_temp_source(
        "check-bad-import-member",
        "package main\n\nimport \"fmt\"\n\nfunc main() {\n\tfmt.Printf(1)\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("does not export supported member `Printf`"));

    let _ = fs::remove_file(path);
}

#[test]
fn run_rejects_missing_return_on_value_function() {
    let path = write_temp_source(
        "check-missing-return",
        "package main\n\nfunc helper() int {\n\tif true {\n\t\treturn 1\n\t}\n}\n\nfunc main() {\n\tprintln(helper())\n}\n",
    );

    let error = run_cli(&["run", path.to_str().unwrap()]).expect_err("run should fail");
    assert!(error.contains("must return a `int` on every path"));

    let _ = fs::remove_file(path);
}

#[test]
fn dump_tokens_rejects_unterminated_string_literal() {
    let path = write_temp_source(
        "unterminated-string",
        "package main\n\nfunc main() {\n\tprintln(\"oops)\n}\n",
    );

    let error = run_cli(&["dump-tokens", path.to_str().unwrap()]).expect_err("lexing should fail");
    assert!(error.contains("unterminated string literal"));

    let _ = fs::remove_file(path);
}

#[test]
fn check_rejects_value_function_with_only_conditional_loop_return() {
    let path = write_temp_source(
        "check-loop-missing-return",
        "package main\n\nfunc helper(value int) int {\n\tfor value > 0 {\n\t\treturn value\n\t}\n}\n\nfunc main() {\n\tprintln(helper(1))\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("must return a `int` on every path"));

    let _ = fs::remove_file(path);
}
