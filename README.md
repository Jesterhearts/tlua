# tlua A lua interpreter written in Rust.

Final name tbd.

## Status
This is very much a work in progress. It supports basic Lua language constructs, including basic
table support, but is still missing many important features.
Nothing should be considered stable, especially the bytecode format.

## Goals
- Full Lua 5.4 compliance
- Good test coverage
- Reasonable performance

## Architecture
- Source -> [parse] -> AST -> [optimize] -> OptimizedAST -> [compile] -> Bytecode

## Milestones
- ✔ Full parser
- Full bytecode generator
- ✔ Support for Tables
- ✔ Safe GC Support
- Support for Userdata
- Benchmarks & performance at least in the neighborhood of PUC-Rio Lua
- Fuzz testing

### Not yet implemented
- Attributes (const, close)
- Meta tables
- Coroutines
- Lua standard lib
- Userdata

### Cleanup work
- Serde integration for lua AST/tables
- Actual good errors using something like ariadne