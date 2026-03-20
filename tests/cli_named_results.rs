mod support;

use support::run_cli;

#[test]
fn run_executes_named_result_example() {
    let output = run_cli(&["run", "examples/named_results.go"]).expect("program should run");

    assert_eq!(output, "negative 3\nnon-negative 5\nnova go true cold\n");
}

#[test]
fn dump_ast_renders_named_result_signatures() {
    let output = run_cli(&["dump-ast", "examples/named_results.go"]).expect("ast should render");

    assert!(output.contains("func classify(value int) (sign string, abs int)"));
    assert!(output.contains("func pair() (head, tail string, ok bool)"));
    assert!(output.contains("func blankLabel(flag bool) (_ int, label string)"));
}

#[test]
fn dump_bytecode_shows_result_locals_and_zero_init() {
    let output =
        run_cli(&["dump-bytecode", "examples/named_results.go"]).expect("bytecode should render");

    assert!(output.contains("function 0: classify (params=1, returns=string, int"));
    assert!(output.contains("locals=value, sign, abs"));
    assert!(output.contains("locals=flag, result$0, label"));
    assert!(output.contains("store-local 1"));
    assert!(output.contains("store-local 2"));
}

#[test]
fn check_accepts_named_result_example() {
    let output =
        run_cli(&["check", "examples/named_results.go"]).expect("check should accept example");

    assert!(output.contains("ok:"));
}
