mod support;

use support::{cleanup_temp_source, run_cli, write_temp_source};

#[test]
fn check_rejects_panic_without_argument() {
    let path = write_temp_source(
        "check-bad-panic-arity",
        "package main\n\nfunc main() {\n\tpanic()\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("builtin `panic` expects 1 arguments, found 0"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_panic_with_multiple_arguments() {
    let path = write_temp_source(
        "check-bad-panic-multi",
        "package main\n\nfunc main() {\n\tpanic(1, 2)\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("builtin `panic` expects 1 arguments, found 2"));

    cleanup_temp_source(path);
}
