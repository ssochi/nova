mod support;

use support::{cleanup_temp_source, run_cli, write_temp_source};

#[test]
fn check_rejects_parenthesized_defer_expression() {
    let path = write_temp_source(
        "check-bad-defer-paren",
        "package main\n\nfunc main() {\n\tdefer (println(\"tail\"))\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("expression in defer must not be parenthesized"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_builtin_not_allowed_in_defer_context() {
    let path = write_temp_source(
        "check-bad-defer-builtin",
        "package main\n\nfunc main() {\n\tdefer len(\"go\")\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("builtin `len` is not permitted in defer statement context"));

    cleanup_temp_source(path);
}
