mod support;

use support::{cleanup_temp_source, run_cli, write_temp_source};

#[test]
fn check_rejects_clear_on_string() {
    let path = write_temp_source(
        "check-bad-clear-string",
        "package main\n\nfunc main() {\n\tclear(\"nova\")\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains(
        "argument 1 in call to builtin `clear` requires `slice` or `map`, found `string`"
    ));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_clear_on_channel() {
    let path = write_temp_source(
        "check-bad-clear-chan",
        "package main\n\nfunc main() {\n\tvar ready = make(chan int, 1)\n\tclear(ready)\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains(
        "argument 1 in call to builtin `clear` requires `slice` or `map`, found `chan int`"
    ));

    cleanup_temp_source(path);
}
