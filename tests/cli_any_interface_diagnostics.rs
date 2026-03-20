mod support;

use support::{cleanup_temp_source, run_cli, write_temp_source};

#[test]
fn check_rejects_fmt_spread_with_prefix_arguments() {
    let path = write_temp_source(
        "any-interface-fmt-spread-prefix",
        "package main\n\nimport \"fmt\"\n\nfunc main() {\n\tvar args = []any{\"go\"}\n\tfmt.Println(1, args...)\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("requires 0 fixed arguments before the spread value"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_fmt_spread_from_non_any_slice() {
    let path = write_temp_source(
        "any-interface-fmt-spread-type",
        "package main\n\nimport \"fmt\"\n\nfunc main() {\n\tvar args = []string{\"go\"}\n\tfmt.Println(args...)\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("spread argument in call to `fmt.Println` requires `[]any`"));

    cleanup_temp_source(path);
}
