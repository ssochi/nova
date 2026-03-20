mod support;

use support::{cleanup_temp_source, run_cli, write_temp_source};

#[test]
fn check_rejects_bad_strings_compare_argument_type() {
    let path = write_temp_source(
        "check-bad-strings-compare",
        "package main\n\nimport \"strings\"\n\nfunc main() {\n\tprintln(strings.Compare([]byte(\"nova\"), \"go\"))\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(
        error.contains("argument 1 in call to `strings.Compare` requires `string`, found `[]byte`")
    );

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_bad_bytes_compare_argument_type() {
    let path = write_temp_source(
        "check-bad-bytes-compare",
        "package main\n\nimport \"bytes\"\n\nfunc main() {\n\tprintln(bytes.Compare([]byte(\"nova\"), \"go\"))\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(
        error.contains("argument 2 in call to `bytes.Compare` requires `[]byte`, found `string`")
    );

    cleanup_temp_source(path);
}
