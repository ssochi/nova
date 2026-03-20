mod support;

use support::run_cli;

#[test]
fn run_executes_grouped_parameter_example() {
    let output = run_cli(&["run", "examples/grouped_parameters.go"]).expect("program should run");

    assert_eq!(output, "nova-go\n3\n10\n");
}

#[test]
fn dump_ast_renders_grouped_parameter_signatures() {
    let output =
        run_cli(&["dump-ast", "examples/grouped_parameters.go"]).expect("ast should render");

    assert!(output.contains("func describe(left, right string) string"));
    assert!(output.contains("func total(base, offset int, values ...int) int"));
}

#[test]
fn dump_bytecode_preserves_flattened_parameter_metadata() {
    let output = run_cli(&["dump-bytecode", "examples/grouped_parameters.go"])
        .expect("bytecode should render");

    assert!(output.contains("params=2"));
    assert!(output.contains("function 1: total (params=2 + ...int"));
    assert!(output.contains("locals=base, offset, values, sum"));
    assert!(output.contains("range$source"));
}

#[test]
fn check_accepts_grouped_parameter_example() {
    let output = run_cli(&["check", "examples/grouped_parameters.go"])
        .expect("check should accept the example");

    assert!(output.contains("ok:"));
}
