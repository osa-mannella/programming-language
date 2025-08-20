# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Mirrow is a web-first functional programming language built in Rust. It's currently in early development, focusing on immutability, safety, and readable code with features like async/await, pattern matching, and bytecode compilation.

## Architecture

### Core Components

- **Lexer** (`src/library/lexer.rs`) - Tokenizes Mirrow source code
- **Parser** (`src/library/parser.rs`) - Builds AST from tokens using recursive descent parsing
- **AST** (`src/library/ast.rs`) - Abstract syntax tree node definitions for all language constructs
- **Compiler** (`src/library/compiler.rs`) - Translates AST to custom bytecode format
- **Debug** (`src/library/debug.rs`) - Bytecode debugging and pretty-printing utilities
- **CLI** (`src/library/cli.rs`) - Command-line interface and output formatting

### Pipeline Flow
1. **Lexing**: Source code → Tokens
2. **Parsing**: Tokens → AST  
3. **Compilation**: AST → Bytecode
4. **Debug Output**: Bytecode analysis and visualization

### Language Features (Current Implementation)
- Functions with automatic currying
- Enums with algebraic data types (Result, Maybe, Unit)
- Pattern matching with match expressions
- Error propagation with `let!` operator
- **Pipeline operators `|>` (FULLY IMPLEMENTED)**
  - Simple function piping: `value |> func`
  - Function calls with additional args: `value |> func(arg2, arg3)`
  - Lambda functions: `value |> fn(x) -> x * 2`
  - Chained pipelines: `value |> func1 |> func2`
  - Module function calls: `value |> Module.func(args)`
- Struct operations and updates
- Async/await constructs
- Module imports
- Built-in I/O operations under `IO` namespace

## Development Commands

### Building
```bash
cargo build           # Build debug version
cargo build --release # Build optimized version
```

### Testing
```bash
cargo test                    # Run all tests
cargo test lexer             # Run specific test module
cargo test test_name         # Run specific test
cargo test -- --nocapture   # Show println! output in tests
```

### Running
```bash
cargo run main.mir           # Compile and run main.mir file
cargo run -- --help         # Show Mirrow CLI help
cargo run -- --debug file.mir # Run with debug bytecode output
```

### Testing Individual Components
- **Lexer tests**: `tests/lexer_tests.rs`
- **Compiler tests**: `tests/compiler_tests.rs` 
- **Debug tests**: `tests/debug_tests.rs`
- **Integration tests**: `src/lib.rs` (comprehensive pipeline tests)

## Key Files for Development

- `main.mir` - Example Mirrow program for testing
- `src/static/lib.mir` - Standard library definitions
- `docs/SYNTAX.md` - Complete language syntax reference
- `docs/BYTECODE.md` - Bytecode format specification

## Working with the Codebase

### Adding Language Features
1. Update lexer for new tokens in `lexer.rs`
2. Extend AST nodes in `ast.rs`
3. Add parsing logic in `parser.rs`
4. Implement compilation in `compiler.rs`
5. Add comprehensive tests in `tests/`

### Debugging Issues
- Use `cargo run -- --debug file.mir` to see bytecode output
- Integration tests in `src/lib.rs` test the full compilation pipeline
- Check parser error handling with `parser.had_error`

### Code Standards
- All data structures are immutable by design
- Error handling uses Result types extensively
- Comprehensive test coverage expected for new features
- Follow existing patterns for AST node definitions and compilation