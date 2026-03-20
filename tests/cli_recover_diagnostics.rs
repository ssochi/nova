mod support;

use support::{cleanup_temp_source, run_cli, write_temp_source};

#[test]
fn check_rejects_recover_with_argument() {
    let path = write_temp_source(
        "check-bad-recover-arity",
        "package main\n\nfunc main() {\n\trecover(1)\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("builtin `recover` expects 0 arguments, found 1"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_recover_with_multiple_arguments() {
    let path = write_temp_source(
        "check-bad-recover-multi",
        "package main\n\nfunc main() {\n\trecover(1, 2)\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("builtin `recover` expects 0 arguments, found 2"));

    cleanup_temp_source(path);
}
