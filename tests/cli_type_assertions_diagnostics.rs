mod support;

use support::{cleanup_temp_source, run_cli, write_temp_source};

#[test]
fn check_rejects_type_assertion_on_non_interface_operand() {
    let path = write_temp_source(
        "type-assertion-non-interface",
        "package main\n\nfunc main() {\n\tvar value int = 7\n\tprintln(value.(int))\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("type assertion requires interface operand"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_type_switch_syntax() {
    let path = write_temp_source(
        "type-assertion-switch-syntax",
        "package main\n\nfunc main() {\n\tvar value any\n\t_ = value.(type)\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("type switches are not supported"));

    cleanup_temp_source(path);
}
