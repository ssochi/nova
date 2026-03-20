mod support;

use support::{cleanup_temp_source, run_cli, write_temp_source};

#[test]
fn check_rejects_type_switch_guard_on_non_interface_operand() {
    let path = write_temp_source(
        "type-switch-non-interface",
        "package main\n\nfunc main() {\n\tvar value int = 7\n\tswitch value.(type) {\n\tdefault:\n\t}\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("type switch guard requires interface operand"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_duplicate_type_switch_cases() {
    let path = write_temp_source(
        "type-switch-duplicate-case",
        "package main\n\nfunc main() {\n\tvar value any = 1\n\tswitch value.(type) {\n\tcase int:\n\tcase string:\n\tcase int:\n\t}\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("duplicate case int in type switch"));

    cleanup_temp_source(path);
}

#[test]
fn check_rejects_type_switch_blank_binding() {
    let path = write_temp_source(
        "type-switch-blank-binding",
        "package main\n\nfunc main() {\n\tvar value any = 1\n\tswitch _ := value.(type) {\n\tcase int:\n\t\tprintln(true)\n\t}\n}\n",
    );

    let error = run_cli(&["check", path.to_str().unwrap()]).expect_err("check should fail");
    assert!(error.contains("type switch guard requires a named identifier"));

    cleanup_temp_source(path);
}
