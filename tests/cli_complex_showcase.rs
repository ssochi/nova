mod support;

use support::run_cli;

#[test]
fn run_executes_complex_showcase_example() {
    let output = run_cli(&["run", "examples/complex_showcase.go"]).expect("program should run");

    assert_eq!(
        output,
        "nova go true\nvm loop true\nnova:go\nroot/nova/go/go 3\nfalse true\nboom true\n1 0 0 4 0 11\ngo true\nnova go true 3 root/nova/go/go\n2 2 5 7 0\n[103 111]\npanic-safe 2\n"
    );
}

#[test]
fn check_accepts_complex_showcase_example() {
    let output =
        run_cli(&["check", "examples/complex_showcase.go"]).expect("check should accept example");

    assert!(output.contains("ok:"));
}

#[test]
fn dump_bytecode_shows_complex_showcase_feature_mix() {
    let output = run_cli(&["dump-bytecode", "examples/complex_showcase.go"])
        .expect("bytecode should render");

    assert!(output.contains("call-package-spread fmt.Println 0"));
    assert!(output.contains("function 1: collect (params=1 + ...string"));
    assert!(output.contains("type-assert []byte"));
    assert!(output.contains("call-builtin clear 1"));
    assert!(output.contains("map-keys string"));
    assert!(output.contains("panic []byte"));
    assert!(output.contains("defer-function"));
    assert!(output.contains("box-any string"));
}
