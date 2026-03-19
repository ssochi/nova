# Compound Assignment Statements

## Goal

Capture the official Go behavior baseline needed for staged compound-assignment support in `nova-go`, with emphasis on `op=` evaluation rules, assignability, and the relationship to the existing explicit `++` / `--` model.

## Sources Reviewed

- Go specification, "Assignment statements": https://go.dev/ref/spec
- Go specification, "IncDec statements": https://go.dev/ref/spec

## Confirmed Findings

- Go defines `assign_op` as `[ add_op | mul_op ] "="`, so compound assignment is part of ordinary assignment syntax rather than a separate expression form.
- A compound assignment `x op= y` is defined as equivalent to `x = x op (y)` while evaluating `x` only once.
- In a compound assignment, both sides must contain exactly one single-valued expression, and the left side must not be `_`.
- Assignment targets must be addressable or map index expressions; this matches the operand rule already used by `nova-go` for `++` / `--`.
- The spec explicitly states that `x++` and `x--` are semantically equivalent to `x += 1` and `x -= 1`, which makes compound-assignment lowering a natural extension of the current explicit inc/dec seam.
- The full Go surface includes more operators than `nova-go` currently models, including `%`, shifts, and bitwise operators.

## Implementation Implications

- Model compound assignments explicitly in the AST, checked layer, and bytecode instead of erasing them into plain assignments during parsing.
- Reuse the existing assignable-target validation path so identifier and index targets stay aligned with plain assignment and inc/dec rules.
- Preserve single-evaluation behavior for indexed targets by lowering through hidden temporaries, the same way the current inc/dec lowering avoids re-evaluating map and slice indices.
- Stage the supported operator set to the arithmetic operators that already exist in the current runtime surface. For this round, that means `+=`, `-=`, `*=`, and `/=`, plus string `+=` through the existing concatenation path.
- Keep bitwise, shift, and modulo compound assignments deferred until the corresponding expression operators and runtime instructions exist; otherwise the frontend would imply language support the VM cannot execute.

## Deferred Questions

- Whether a later arithmetic-expansion round should add `%` together with modulo expressions and `%=` in one slice.
- Whether future broader numeric-constant modeling should make byte-target compound assignments accept untyped integer literals more like full Go.
