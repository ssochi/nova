# If Statement Headers and Else-If Chains

## Goal

Capture the Go behavior baseline needed for the next `nova-go` control-flow slice, with emphasis on `if` statement initializers, scope visibility, and `else if` chaining.

## Sources Reviewed

- Official Go language specification section `If statements`
- Official Go language specification section `Blocks`
- Official Go language specification section `Short variable declarations`
- Local Go 1.21.5 spot checks for header binding scope and outer-name shadowing

## Confirmed Findings

- Go `if` syntax permits an optional simple statement before the condition, separated by a semicolon.
- The simple statement executes before condition evaluation and shares one statement scope with both the `then` and `else` branches.
- Names introduced by an `if` header are not visible after the entire `if` statement ends.
- A short declaration in an `if` header may shadow an outer variable name instead of reusing it as a same-block redeclaration, because the header statement lives in the `if` statement's own implicit block.
- `else if` is part of the `if` grammar rather than a separate standalone construct; the `else` arm may be either a block or another `if` statement.
- The existing staged comma-ok `map` lookup statement fits naturally into an `if` header even without introducing general multi-result expressions.

## Implementation Implications

- `nova-go` should model `if` initializers as explicit statement-header data instead of lowering them into preceding statements, so scope and debug output remain correct.
- Semantic analysis should create one dedicated `if`-header scope that encloses the condition and both branches, while keeping outer-scope leakage impossible.
- The current implementation slice can stay narrow by reusing the already modeled simple-statement subset rather than opening full Go statement coverage.
- `else if` should remain source-visible in the AST rendering and checked model so `dump-ast` does not degrade into synthetic nested blocks.

## Deferred Questions

- Whether `switch` and `for` header initializers should reuse the same header-statement abstraction later.
- Whether broader short variable declarations should land before or alongside wider statement-header support.
