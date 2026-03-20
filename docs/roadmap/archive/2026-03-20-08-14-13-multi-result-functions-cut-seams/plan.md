# Plan: Multi-Result Functions and Cut Package Seams

## Basic Information

- Plan ID: `2026-03-20-08-14-13-multi-result-functions-cut-seams`
- Milestone: `M3-standard-library-and-runtime-model`
- Status: `completed`
- Owner: primary project agent

## Goals

- Introduce the first staged multi-result model without turning tuples into ordinary runtime values.
- Expand user-defined and package-backed call compatibility through unnamed multi-result function signatures, returns, and binding forms.
- Use `strings.Cut` and `bytes.Cut` as real package seams that prove the new model across parsing, semantic analysis, bytecode lowering, VM execution, and CLI inspection.

## Scope

- Unnamed multi-result function signatures such as `func pair() (int, string)`
- Multi-expression `return` statements and direct `return pair()` style forwarding when the callee returns multiple values
- Multi-binding short declarations and identifier/blank assignments in ordinary statements plus staged header positions
- Shared call-contract plumbing so user-defined functions and package functions can report zero, one, or multiple results explicitly
- `strings.Cut(string, string) -> (string, string, bool)`
- `bytes.Cut([]byte, []byte) -> ([]byte, []byte, bool)`
- Examples, automated coverage, CLI validation, and documentation updates for the new staged surface

## Non-Goals

- Named result parameters or naked `return`
- General tuple expressions, tuple runtime values, or multi-result values flowing through arbitrary expression positions
- Multi-result builtin functions, comma-ok receive, or refactoring staged comma-ok `map` lookups into generic tuple expressions
- Multi-target assignments that require index-target single-evaluation guarantees
- Import graph expansion beyond the current metadata-backed package registry

## Phase Breakdown

1. Capture the official behavior baseline for multi-result declarations, assignment/return usage, and the chosen `Cut` package APIs.
2. Record the staged design for explicit multi-result call plumbing without first-class tuple values.
3. Extend AST, parser, semantic analysis, package contracts, and checked-model plumbing for the new function/result forms.
4. Extend bytecode lowering, VM call/return handling, and package runtime dispatch for multi-result execution plus `strings.Cut` / `bytes.Cut`.
5. Add examples, tests, formatting, serial CLI validation, reports, and roadmap synchronization; archive the plan if complete.

## Acceptance Criteria

- `check`, `dump-ast`, `dump-bytecode`, and `run` handle at least one example that combines user-defined multi-result functions with `strings.Cut` and `bytes.Cut`.
- Multi-result calls stay explicit and are rejected in unsupported expression-only contexts with targeted diagnostics.
- Function signatures, package contracts, bytecode metadata, and VM return handling all expose result arity explicitly instead of hiding it inside ad hoc runtime conventions.
- The new docs and plan context leave enough information for later work on broader multi-result surfaces such as comma-ok receive or more package APIs.

## Risks

- Generalizing short declarations and assignments can accidentally blur the current explicit simple-statement model if the staged scope is not kept narrow.
- Multi-result call handling touches parser, semantic, compiler, and VM boundaries at once; a partial update could leave `dump-ast` or `dump-bytecode` misleading.
- `bytes.Cut` must preserve the staged nil/empty slice model clearly enough that later slice and package work does not inherit accidental semantics.
