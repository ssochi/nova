mod support;

use support::{cleanup_temp_source, run_cli, write_temp_source};

#[test]
fn check_rejects_bad_strings_clone_argument_type() {
    let path = write_temp_source(
        "check-bad-strings-clone",
        "package main\n\nimport \"strings\"\n\nfunc main() {\n\tprintln(strings.Clone([]byte(\"nova\")))\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(
        error.contains("argument 1 in call to `strings.Clone` requires `string`, found `[]byte`")
    );

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_bad_bytes_clone_argument_type() {
    let path = write_temp_source(
        "check-bad-bytes-clone",
        "package main\n\nimport \"bytes\"\n\nfunc main() {\n\tprintln(bytes.Clone(\"nova\"))\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(
        error.contains("argument 1 in call to `bytes.Clone` requires `[]byte`, found `string`")
    );

    cleanup_temp_source(path);
}
