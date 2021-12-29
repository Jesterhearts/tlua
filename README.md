# tlua A lua interpreter written in Rust.

## Goals
- Full Lua 5.4 compliance
- Good test coverage
- Reasonable performance

## Architecture
- Source -> [parse] -> AST -> [optimize] -> OptimizedAST -> [compile] -> Bytecode

## Milestones
- ✔ Full parser
- Full bytecode generator
- Support for Tables
- ✔ Safe GC Support
- Support for Userdata
- Benchmarks & performance at least in the neighborhood of PUC-Rio Lua
- Fuzz testing

### Cleanup work
- Switch to lexical for parsing (currently always panics on hex parsing).
- Switch to Rowan for token tree representation 
- Serde integration for lua AST/tables
- Actual good errors using something like ariadne

## Lua Details
A collection of notes on specific corners of lua behavior.

### Variable Captures

Functions capture variables from the closest enclosing scope where the variable was declared.
When called, the value of that variable in the function is the most recent value of that variable
in the scope from which the variable was captured.

Because we use a register vm, we can accomplish this in the compiler by tracking variable mapped
registers - provided that all lua code within a single chunk context shares one set of registers.
We can substitute any variable reference with the appropriate register, respecting shadowing.