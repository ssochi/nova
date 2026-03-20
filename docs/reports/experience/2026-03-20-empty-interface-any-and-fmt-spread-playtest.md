# Empty Interface Any and Fmt Spread Playtest

## Basic Context

- Date: `2026-03-20`
- Related plan: `2026-03-20-12-55-53-empty-interface-any-and-fmt-spread`
- Milestone: `M3-standard-library-and-runtime-model`

## Experience Path

- Ran the real CLI against `examples/empty_interface_any.go` through `run`, `dump-ast`, `dump-bytecode`, and `check`.
- Probed one temporary invalid `fmt.Println(1, args...)` program through `check` to confirm the staged spread-shape diagnostic.
- Probed one temporary invalid `fmt.Println(args...)` program where `args` is `[]string` to confirm the staged `[]any` requirement.
- Probed one temporary runtime program that compares an interface holding `[]int` against itself to confirm the runtime panic path.
- Scope boundary: this is a focused empty-interface slice playtest, not a milestone-closeout full CLI blackbox pass.

## Positive Experience

- The `run` path is readable and concrete: nil-interface checks, boxed typed-nil behavior, boxed scalar equality, interface printing, and `fmt.Println(args...)` all show up in one small example.
- `dump-ast` keeps the source surface recognizable because both `any` and `interface{}` remain visible instead of being normalized away.
- `dump-bytecode` is useful for debugging the new path quickly because `push-nil-interface`, `box-any <type>`, and `call-package-spread` expose the staged lowering directly.
- Package-only validation through `check` stays separate from runtime assumptions and surfaces the new spread diagnostics cleanly.

## Issues and Severity

- Medium: direct interface-vs-concrete equality is intentionally narrower than full Go today; the staged slice covers scalar cases but not every comparable runtime category.
- Medium: `recover` is still absent even though the project now has the payload-carrier type it needed.
- Low: `fmt` spread support is still limited to `[]any` with no extra prefix arguments before the spread value.

## Conclusion and Next Recommended Steps

The empty-interface slice is usable and materially advances the current Go surface because `any` / `interface{}` values now exist end to end and make `fmt` variadic APIs more realistic. The strongest next continuation is either `recover` on top of the new interface carrier and panic state, or a broader interface/runtime-type metadata pass that can safely widen the equality and printing surface.
