mod support;

use support::{cleanup_temp_source, run_cli, write_temp_source};

#[test]
fn check_rejects_mixed_named_and_unnamed_results() {
    let path = write_temp_source(
        "check-bad-mixed-results",
        "package main\n\nfunc split() (head string, bool) {\n\treturn \"\", false\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("mixed named and unnamed parameters"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_bare_return_without_named_results() {
    let path = write_temp_source(
        "check-bad-bare-return",
        "package main\n\nfunc plain() int {\n\treturn\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("function `plain` must return a `int` value"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_shadowed_named_result_on_bare_return() {
    let path = write_temp_source(
        "check-bad-shadowed-result",
        "package main\n\nfunc shadow() (err string) {\n\tif true {\n\t\terr := \"inner\"\n\t\treturn\n\t}\n\treturn\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("result parameter `err` not in scope at return"));

    cleanup_temp_source(path);
}
