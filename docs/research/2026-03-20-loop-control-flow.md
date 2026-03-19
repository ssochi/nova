# Loop Control Flow Research

## Goal

Lock the official behavior baseline for the next staged loop-control slice: classic `for` clauses, omitted loop conditions, unlabeled `break`, unlabeled `continue`, and the nearest-enclosing-target rule.

## Sources Reviewed

- Go language specification: `For statements` (`https://go.dev/ref/spec#For_statements`)
- Go language specification: `Break statements` (`https://go.dev/ref/spec#Break_statements`)
- Go language specification: `Continue statements` (`https://go.dev/ref/spec#Continue_statements`)
- Go language specification: `Terminating statements` (`https://go.dev/ref/spec#Terminating_statements`)

## Confirmed Findings

- Go has three `for` forms: condition-only loops, `for` clauses with init / condition / post, and `range` loops.
- In a `for` clause, the init statement runs once before the first condition test, the condition is evaluated before each iteration when present, and the post statement runs after each completed loop body iteration.
- An omitted `for` condition is equivalent to `true`, so `for { ... }` is the canonical infinite loop surface.
- `break` terminates execution of the innermost `for`, `switch`, or `select` unless a label redirects it.
- `continue` resumes the next iteration of the innermost `for`; in clause-based loops the post statement executes before the next condition check.
- Labeled `break` / `continue` exist in Go, but they depend on label support and are separable from the unlabeled control-flow baseline.
- A syntactically infinite loop only guarantees non-fallthrough when a modeled `break` path cannot escape it.

## Implementation Implications

- Keep classic `for` clauses explicit in the AST and checked model instead of lowering them into synthetic pre-loop statements during parsing.
- Reuse the current staged header subset for init work where possible, but keep the post statement explicit so `continue` can target it without guesswork.
- Model `break` and `continue` as dedicated statements in the AST, checked layer, and bytecode lowering; do not encode them as disguised jumps before semantic validation.
- Keep unlabeled `break` target selection explicit in semantic analysis and conservative return-path analysis, especially for nested `switch` inside `for`.
- Defer labels, `fallthrough`, and `select` deliberately instead of implying partial support.

## Deferred Questions

- Whether later short variable declaration support should replace the current staged `var`-based header subset for `if` / `switch` / `for`.
- Whether loop termination analysis should grow beyond absent / literal-true conditions into broader constant-expression reasoning.
