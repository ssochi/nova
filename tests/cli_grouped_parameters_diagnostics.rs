mod support;

use support::{cleanup_temp_source, run_cli, write_temp_source};

#[test]
fn check_rejects_grouped_variadic_parameter_names() {
    let path = write_temp_source(
        "check-bad-grouped-variadic-params",
        "package main\n\nfunc collect(values, more ...int) {}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("can only use `...` with one final parameter"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_duplicate_grouped_parameter_name() {
    let path = write_temp_source(
        "check-bad-duplicate-grouped-param",
        "package main\n\nfunc pair(left, right string, left int) {}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("parameter `left` is already defined in function `pair`"));

    cleanup_temp_source(path);
}
