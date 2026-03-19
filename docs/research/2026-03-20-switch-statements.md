# Switch Statements Research

## Goal

Define the compatibility baseline for the first staged `switch` implementation in `nova-go`, with emphasis on expression switches, tagless switches, shared header scope, clause evaluation order, and the diagnostics worth enforcing in the current subset.

## Sources Reviewed

- Official Go specification section `Switch statements` on `go.dev/ref/spec` (language version `go1.26`, published January 12, 2026).
- Official Go specification sections `Blocks`, `Declarations and scope`, and terminating-statement rules on `go.dev/ref/spec`.
- Local Go toolchain spot-check with `go version go1.21.5 darwin/arm64`.
- Local Go spot-checks for:
  - expression switch with header statement and multiple case expressions
  - duplicate `default`
  - duplicate constant cases

## Confirmed Findings

- Go has two switch families: expression switches and type switches. The first staged `nova-go` slice should target expression switches only.
- Expression-switch grammar is `switch [ SimpleStmt ";" ] [ Expression ] "{" { ExprCaseClause } "}"`.
- The switch statement itself is an implicit block, and each clause is also its own implicit block. A header simple statement is therefore visible to the tag expression and every clause body, but names declared inside one clause do not leak to sibling clauses or after the switch.
- The switch expression is evaluated exactly once.
- Case expressions are evaluated left-to-right and top-to-bottom until the first matching case is found.
- A missing switch expression is equivalent to the boolean value `true`.
- There may be at most one `default` clause, and it may appear anywhere in the clause list.
- The switch expression must be comparable. Each case expression must be comparable to the switch expression after the usual untyped-to-default / untyped-to-tag conversions.
- The predeclared untyped value `nil` cannot be used as a switch expression in real Go.
- Go allows `fallthrough` as the final non-empty statement of a non-final expression-switch clause, but it is optional syntax, not part of the default control-flow path.
- The Go spec only makes duplicate constant-case rejection an implementation restriction, but current Go compilers reject duplicate integer and string constants in expression switches. Local spot-checks confirm duplicate `default` and duplicate integer-case diagnostics.
- A switch with a `default` clause whose every clause ends in a terminating statement counts as terminating, provided no `break` statement targets the switch.

## Implementation Implications

- The current slice should support expression switches with:
  - optional shared header statement from the already supported simple-statement subset
  - optional tag expression
  - comma-separated case-expression lists
  - one optional `default`
- Tagless switches should stay explicit in the AST instead of being hidden during parsing, but the checked or lowered form may normalize them to a boolean `true` comparison strategy if that keeps lowering simple.
- The existing `if`-header abstraction should be generalized into a control-flow header abstraction shared by `if` and `switch`.
- Clause bodies should analyze as their own nested scopes, while still sharing the enclosing switch-header scope.
- The compiler should preserve “evaluate the tag once” semantics, most likely with an explicit hidden local or an equivalent single-evaluation lowering path.
- The staged semantic surface should reject:
  - multiple `default` clauses
  - non-comparable switch tags
  - case expressions whose types cannot be compared to the tag
  - explicit `nil` as the switch tag
- It is worth adding duplicate constant-case diagnostics now for the currently modeled scalar constants (`int`, `bool`, `string`) because that provides user-facing value without requiring broader constant folding.
- Return-path analysis can treat a switch as terminating when:
  - a `default` clause exists, and
  - every clause body is terminating in the currently modeled subset.

## Deferred Questions

- Whether later `switch` work should add `break` / `fallthrough` first or wait for a broader statement-control slice.
- Whether type switches should reuse the same checked statement shell or live as a distinct staged surface once interfaces exist.
- Whether duplicate constant-case diagnostics should later expand beyond scalar literals into a broader constant-expression system.
