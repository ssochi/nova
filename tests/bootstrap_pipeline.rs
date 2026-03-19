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
fn dump_bytecode_shows_stack_machine_instructions() {
    let output = run_cli(&["dump-bytecode", "examples/arithmetic.go"])
        .expect("bytecode should be generated");

    assert!(output.contains("call-builtin println 1"));
    assert!(output.contains("multiply"));
    assert!(output.contains("store-local 2"));
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
