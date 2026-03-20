mod support;

use support::{cleanup_temp_source, run_cli, write_temp_source};

#[test]
fn check_rejects_bad_strings_index_byte_argument_type() {
    let path = write_temp_source(
        "check-bad-strings-index-byte",
        "package main\n\nimport \"strings\"\n\nfunc main() {\n\tprintln(strings.IndexByte(\"nova\", \"x\"))\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(
        error.contains("argument 2 in call to `strings.IndexByte` requires `byte`, found `string`")
    );

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_bad_bytes_last_index_byte_argument_type() {
    let path = write_temp_source(
        "check-bad-bytes-last-index-byte",
        "package main\n\nimport \"bytes\"\n\nfunc main() {\n\tprintln(bytes.LastIndexByte([]byte(\"nova\"), 1))\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(
        error.contains("argument 2 in call to `bytes.LastIndexByte` requires `byte`, found `int`")
    );

    cleanup_temp_source(path);
}
