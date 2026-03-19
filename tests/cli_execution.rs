mod support;

use support::run_cli;

#[test]
fn run_executes_entrypoint_program() {
    let output = run_cli(&["run", "examples/hello.go"]).expect("program should run");
    assert_eq!(output, "42\n");
}

#[test]
fn run_executes_multi_function_branches() {
    let output =
        run_cli(&["run", "examples/functions_and_branches.go"]).expect("program should run");
    assert_eq!(output, "false 11\n");
}

#[test]
fn run_executes_loops() {
    let output = run_cli(&["run", "examples/loops.go"]).expect("program should run");
    assert_eq!(output, "10 4\n");
}

#[test]
fn run_executes_strings_and_builtins() {
    let output = run_cli(&["run", "examples/strings.go"]).expect("program should run");
    assert_eq!(output, "hello, nova! 11\ntrue\n");
}

#[test]
fn run_executes_imported_fmt_package_calls() {
    let output = run_cli(&["run", "examples/imports_fmt.go"]).expect("program should run");
    assert_eq!(output, "hello, nova\nbytes=11");
}

#[test]
fn run_executes_slices() {
    let output = run_cli(&["run", "examples/slices.go"]).expect("program should run");
    assert_eq!(output, "4 1 5\n[1 2 3 5]\n");
}

#[test]
fn run_executes_strings_package_calls() {
    let output = run_cli(&["run", "examples/strings_package.go"]).expect("program should run");
    assert_eq!(output, "nova-gogo-vm\ntrue\ntrue\n");
}

#[test]
fn run_executes_slice_windows_and_index_assignment() {
    let output = run_cli(&["run", "examples/slice_windows.go"]).expect("program should run");
    assert_eq!(output, "3 9 7\n1 9 3 7\n");
}

#[test]
fn run_executes_slice_builtins_and_capacity_aware_append() {
    let output = run_cli(&["run", "examples/slice_builtins.go"]).expect("program should run");
    assert_eq!(output, "2 4\n9 3 4\n3 2 9 4 4\n");
}

#[test]
fn dump_bytecode_shows_stack_machine_instructions() {
    let output = run_cli(&["dump-bytecode", "examples/arithmetic.go"])
        .expect("bytecode should be generated");

    assert!(output.contains("call-builtin println 1"));
    assert!(output.contains("multiply"));
    assert!(output.contains("store-local 2"));
}

#[test]
fn dump_bytecode_shows_function_calls_and_branch_jumps() {
    let output = run_cli(&["dump-bytecode", "examples/functions_and_branches.go"])
        .expect("bytecode should be generated");

    assert!(output.contains("call-function 0 2"));
    assert!(output.contains("jump-if-false"));
    assert!(output.contains("greater"));
    assert_eq!(output.matches("jump-if-false").count(), 2);
}

#[test]
fn dump_ast_renders_loops() {
    let output = run_cli(&["dump-ast", "examples/loops.go"]).expect("ast should be rendered");

    assert!(output.contains("for (current > 0) {"));
    assert!(output.contains("for true {"));
}

#[test]
fn dump_tokens_show_string_literals() {
    let output =
        run_cli(&["dump-tokens", "examples/strings.go"]).expect("tokens should be rendered");

    assert!(output.contains("string(\"hello, \")"));
    assert!(output.contains("string(\"nova\")"));
}

#[test]
fn dump_tokens_show_imports_and_selector_calls() {
    let output =
        run_cli(&["dump-tokens", "examples/imports_fmt.go"]).expect("tokens should be rendered");

    assert!(output.contains("import"));
    assert!(output.contains("string(\"fmt\")"));
    assert!(output.contains("identifier(fmt)"));
    assert!(output.contains("."));
}

#[test]
fn dump_tokens_show_slice_syntax() {
    let output =
        run_cli(&["dump-tokens", "examples/slices.go"]).expect("tokens should be rendered");

    assert!(output.contains("identifier(append)"));
    assert!(output.matches('[').count() >= 2);
    assert!(output.matches(']').count() >= 2);
}

#[test]
fn dump_tokens_show_slice_window_syntax() {
    let output =
        run_cli(&["dump-tokens", "examples/slice_windows.go"]).expect("tokens should be rendered");

    assert!(output.contains(":"));
    assert!(output.contains("identifier(reopen)"));
}

#[test]
fn dump_ast_renders_strings_and_builtins() {
    let output = run_cli(&["dump-ast", "examples/strings.go"]).expect("ast should be rendered");

    assert!(output.contains("return (\"hello, \" + name)"));
    assert!(output.contains("println(\"!\", len(greeting))"));
}

#[test]
fn dump_ast_renders_imports_and_package_calls() {
    let output = run_cli(&["dump-ast", "examples/imports_fmt.go"]).expect("ast should be rendered");

    assert!(output.contains("import \"fmt\""));
    assert!(output.contains("return fmt.Sprint(\"hello, \", name)"));
    assert!(output.contains("fmt.Print(fmt.Sprint(\"bytes=\", len(message)))"));
}

#[test]
fn dump_ast_renders_slices() {
    let output = run_cli(&["dump-ast", "examples/slices.go"]).expect("ast should be rendered");

    assert!(output.contains("var values = []int{1, 2}"));
    assert!(output.contains("values = append(values, 3, 5)"));
    assert!(output.contains("println(len(values), values[0], values[3])"));
}

#[test]
fn dump_ast_renders_strings_package_calls() {
    let output =
        run_cli(&["dump-ast", "examples/strings_package.go"]).expect("ast should be rendered");

    assert!(output.contains("import \"strings\""));
    assert!(output.contains("return strings.Join(parts, \"-\")"));
    assert!(output.contains("strings.HasPrefix(joined, \"nova\")"));
}

#[test]
fn dump_ast_renders_slice_windows_and_index_assignment() {
    let output =
        run_cli(&["dump-ast", "examples/slice_windows.go"]).expect("ast should be rendered");

    assert!(output.contains("var head = values[:2]"));
    assert!(output.contains("var reopen = head[1:4]"));
    assert!(output.contains("reopen[2] = 7"));
}

#[test]
fn dump_ast_renders_slice_builtins() {
    let output =
        run_cli(&["dump-ast", "examples/slice_builtins.go"]).expect("ast should be rendered");

    assert!(output.contains("println(len(head), cap(head))"));
    assert!(output.contains("var copied = copy(values, values[1:])"));
    assert!(output.contains("var grown = append(head, 9)"));
}

#[test]
fn dump_bytecode_shows_loop_jumps() {
    let output =
        run_cli(&["dump-bytecode", "examples/loops.go"]).expect("bytecode should be generated");

    assert!(output.contains("function 0: sumDown"));
    assert!(output.contains("function 1: climbPast"));
    assert!(output.matches("jump-if-false").count() >= 2);
    assert!(output.contains("jump 2"));
}

#[test]
fn dump_bytecode_shows_string_instructions_and_builtins() {
    let output =
        run_cli(&["dump-bytecode", "examples/strings.go"]).expect("bytecode should be generated");

    assert!(output.contains("push-string \"hello, \""));
    assert!(output.contains("concat"));
    assert!(output.contains("call-builtin print 1"));
    assert!(output.contains("call-builtin len 1"));
}

#[test]
fn dump_bytecode_shows_package_calls() {
    let output = run_cli(&["dump-bytecode", "examples/imports_fmt.go"])
        .expect("bytecode should be generated");

    assert!(output.contains("call-package fmt.Sprint 2"));
    assert!(output.contains("call-package fmt.Println 1"));
    assert!(output.contains("call-package fmt.Print 1"));
}

#[test]
fn dump_bytecode_shows_slice_instructions() {
    let output =
        run_cli(&["dump-bytecode", "examples/slices.go"]).expect("bytecode should be generated");

    assert!(output.contains("build-slice 2"));
    assert!(output.contains("call-builtin append 3"));
    assert!(output.contains("index"));
}

#[test]
fn dump_bytecode_shows_strings_package_calls() {
    let output = run_cli(&["dump-bytecode", "examples/strings_package.go"])
        .expect("bytecode should be generated");

    assert!(output.contains("call-package strings.Repeat 2"));
    assert!(output.contains("call-package strings.Join 2"));
    assert!(output.contains("call-package strings.Contains 2"));
    assert!(output.contains("call-package strings.HasPrefix 2"));
}

#[test]
fn dump_bytecode_shows_slice_builtins() {
    let output = run_cli(&["dump-bytecode", "examples/slice_builtins.go"])
        .expect("bytecode should be generated");

    assert!(output.contains("call-builtin cap 1"));
    assert!(output.contains("call-builtin copy 2"));
    assert!(output.contains("call-builtin append 2"));
}

#[test]
fn dump_bytecode_shows_slice_window_instructions() {
    let output = run_cli(&["dump-bytecode", "examples/slice_windows.go"])
        .expect("bytecode should be generated");

    assert!(output.contains("slice low=false high=true"));
    assert!(output.contains("slice low=true high=true"));
    assert!(output.contains("set-index"));
}
