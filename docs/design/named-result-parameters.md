# Named Result Parameters

## Goal

Add staged support for grouped named result declarations plus bare `return` while preserving source-facing inspection and the current explicit multi-result pipeline.

## Constraints

- Rust standard library only
- Keep result declarations explicit in the AST, checked layer, and validation docs
- Preserve the existing explicit multi-result model instead of introducing tuple runtime values
- Keep `dump-ast` and `dump-bytecode` useful without reading implementation details
- Stay within the repository file-size limit by splitting near-limit files in the same round when needed

## Current Scope

- Frontend AST support for explicit result declarations, including grouped named results such as `(left, right string, ok bool)`
- Parser support for unnamed results, grouped named results, and parser-time rejection of mixed named/unnamed result lists
- Semantic support for named result slots that are zero-initialized at function entry and share scope with ordinary parameters
- Bare `return` support for named-result functions, including shadowing diagnostics when a result name is no longer the active binding
- Focused examples and CLI coverage that show the feature through `run`, `dump-ast`, `dump-bytecode`, and `check`

## Deferred Scope

- Named return values for methods, receivers, or broader function-type syntax
- Labeled control flow, deferred calls, or panic/recover interactions with named results
- General tuple expressions or tuple runtime values
- Richer result-slot metadata in bytecode beyond readable locals and return-type lists

## Interfaces and Extension Hooks

- `src/frontend/ast.rs` and adjacent frontend modules should keep grouped result declarations visible enough that `dump-ast` shows the source-level signature accurately
- `src/frontend/parser.rs` should parse result declarations explicitly instead of overloading ordinary type-list parsing
- `src/semantic/support.rs` should own result-declaration flattening so registry, analyzer, and future function-type work can reuse one shape
- `src/semantic/analyzer.rs` should treat named result slots as function-entry locals while keeping bare-return shadowing checks explicit in semantic analysis
- `tests/` and `examples/` should keep this slice in focused files instead of inflating the already-large broad integration suites
