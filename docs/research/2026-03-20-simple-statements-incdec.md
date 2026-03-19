# Short Declarations and Inc/Dec Research

## Goal

Lock the Go compatibility baseline for staged short variable declarations and `++` / `--` statements before extending the current simple-statement model.

## Sources Reviewed

- Go language specification section `Short variable declarations`
- Go language specification section `IncDec statements`
- Go language specification section `If statements`
- Go language specification section `Switch statements`
- Go language specification section `For statements`

## Confirmed Findings

- Short variable declarations use the form `IdentifierList := ExpressionList`, are valid only inside functions, and are part of Go's simple-statement surface.
- A short declaration may redeclare existing variables only when those names were declared earlier in the same block with the same type and at least one non-blank identifier on the left side is new.
- `++` and `--` are statements, not expressions. They cannot appear where a value is required.
- `if` and `switch` headers allow a simple statement before the optional semicolon-separated condition or tag expression, so short declarations belong naturally in those headers.
- In a classic `for` clause, the init statement may be a short declaration, but the post statement must not be one. `++` / `--` are valid post statements.
- The current `nova-go` frontend does not yet model multi-result expressions, so general multi-binding short declarations would be misleading unless the staged subset stays explicit.

## Implementation Implications

- Model short declarations explicitly in the AST and checked layer instead of lowering them into plain assignment, because redeclaration and new-binding rules differ from `=`.
- Keep the staged scope narrow to single-expression short declarations for now, while continuing to use the existing explicit comma-ok map lookup statement for the broader two-binding lookup surface.
- Extend the current header-statement abstraction so `if`, `switch`, and classic `for` init can share one simple-statement parser and semantic path.
- Model `++` / `--` explicitly as statements and for-post statements, with assignment-target validation restricted to incrementable / decrementable integer-like values already present in the runtime model (`int` and `byte`).
- Reject short declarations in classic `for` post clauses with a targeted parser diagnostic instead of silently treating them like init statements.

## Deferred Questions

- Whether a later broader multi-result model should generalize short declarations beyond the current single-expression staged surface.
- Whether compound assignment operators should share code with `++` / `--` once the simple-statement surface grows again.
