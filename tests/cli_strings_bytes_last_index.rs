mod support;

use support::run_cli;

#[test]
fn run_executes_strings_and_bytes_last_index_package_calls() {
    let output =
        run_cli(&["run", "examples/strings_bytes_last_index.go"]).expect("program should run");

    assert_eq!(output, "8\n10\n4\n9\ntrue 0\n8\n4\n9\n");
}

#[test]
fn dump_ast_renders_strings_and_bytes_last_index_calls() {
    let output = run_cli(&["dump-ast", "examples/strings_bytes_last_index.go"])
        .expect("ast should be rendered");

    assert!(output.contains("strings.LastIndex(text, \"go\")"));
    assert!(output.contains("strings.IndexByte(text, text[4])"));
    assert!(output.contains("bytes.LastIndex(raw, []byte(\"\"))"));
    assert!(output.contains("bytes.LastIndexByte(value, value[1])"));
}

#[test]
fn dump_bytecode_shows_strings_and_bytes_last_index_calls() {
    let output = run_cli(&["dump-bytecode", "examples/strings_bytes_last_index.go"])
        .expect("bytecode should be generated");

    assert!(output.contains("call-package strings.LastIndex 2"));
    assert!(output.contains("call-package strings.IndexByte 2"));
    assert!(output.contains("call-package strings.LastIndexByte 2"));
    assert!(output.contains("call-package bytes.LastIndex 2"));
    assert!(output.contains("call-package bytes.IndexByte 2"));
    assert!(output.contains("call-package bytes.LastIndexByte 2"));
}

#[test]
fn check_accepts_strings_and_bytes_last_index_example() {
    let output = run_cli(&["check", "examples/strings_bytes_last_index.go"])
        .expect("check should accept the example");

    assert!(output.contains("ok:"));
}
