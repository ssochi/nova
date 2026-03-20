# Recover First Slice

## Goal

Add the first staged `recover()` model on top of the existing panic-aware unwind path while keeping the VM-first architecture explicit and file-size-safe.

## Constraints

- Rust standard library only
- Reuse the current direct-call `defer` surface instead of introducing closures or function values
- Keep recover eligibility explicit in frame metadata instead of inferring it from ad hoc stack shape
- Preserve readable runtime inspection surfaces and stay under the repository file-size ceiling

## Current Scope

- A builtin `recover()` registered in the shared builtin table and validated centrally with zero arguments and `any` result typing
- Statement-context support for `recover()` so ordinary `recover()` and deferred builtin `recover()` remain expressible even though only directly deferred user functions can actually recover
- Frame metadata that marks directly deferred user-function calls as recovery-eligible while they execute during panic unwinding
- Runtime conversion of the pending panic payload into a staged `any` value when recovery succeeds
- VM return/unwind behavior that clears the active panic and resumes ordinary function return after a successful direct recovery

## Deferred Scope

- Closures, function literals, and outer-local mutation patterns that real Go often combines with `recover`
- Exact Go runtime payload types such as `*runtime.PanicNilError` or runtime error interfaces
- Goroutine-aware panic/recover semantics, scheduler-aware blocking, or stack traces
- Broader runtime-trap promotion beyond the traps already wired into the staged panic path

## Interfaces and Extension Hooks

- `src/semantic/builtins.rs` should keep `recover` validation and statement-context handling centralized instead of adding recover-specific checks inside statement analysis
- `src/runtime/vm/unwind.rs` should own the panic-state and recover-eligibility metadata so normal return and panic return keep sharing one mechanism
- `src/runtime/vm/builtins.rs` should expose `recover()` through ordinary builtin execution, with a dedicated helper that reads and clears pending panic state only when the active frame is eligible
- If `src/runtime/vm.rs` grows further, extract call/defer/return orchestration helpers into submodules before adding more control-flow features
- Future closure-aware recover work should build on the same per-frame eligibility flag rather than weakening the direct-call rule into a generic "any deferred descendant can recover" approximation
