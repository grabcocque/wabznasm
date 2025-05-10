# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

wabznasm is a high-performance array processing language and columnar database system inspired by Q/KDB+. It combines a Q-style expression language with modern Rust performance and safety. The project is currently in Phase 1 development, with basic function definitions and calls working.

## Architecture

### Workspace Structure
- **Main crate (`/`)**: Core language implementation with parser, evaluator, and REPL
- **Storage crate (`/storage`)**: Memory-mapped columnar storage engine using Arrow2

### Core Components

**Parser Pipeline:**
- Tree-sitter grammar in `grammar/grammar.js` defines Q-style syntax
- Generated parser in `grammar/src/` (auto-generated, don't edit)
- `build.rs` handles Tree-sitter code generation and C compilation

**Evaluation Engine:**
- `src/environment.rs`: Lexical scoping with `Environment` and `Value` types
- `src/evaluator.rs`: AST visitor with `eval_with_env()` for environment-aware evaluation
- `src/errors.rs`: Structured error handling with source location tracking

**Language Features (Current):**
- Arithmetic expressions: `2+3*4`, `2^3`, `5!`, `-(2+3)`
- Function definitions: `f: {x+1}`, `add: {[x;y] x+y}`
- Function calls: `f[]`, `add[2;3]`
- Variable assignment: `x: 42`
- Lexical scoping and closures

## Development Commands

### Building and Testing
```bash
# Build entire workspace
cargo build

# Run all tests
cargo test

# Run specific test file
cargo test --test function_evaluation
cargo test environment

# Run single test
cargo test test_basic_arithmetic

# Run with output
cargo test test_name -- --nocapture
```

### Grammar Development
```bash
# Grammar changes auto-trigger rebuild via build.rs
# No manual steps needed - just run cargo build

# Test grammar parsing specifically
cargo test function_parsing
```

### REPL Development
```bash
# Run interactive REPL
cargo run

# Test REPL functionality
cargo test repl
```

### Storage Development
```bash
# Test storage layer
cargo test --package storage

# Integration tests
cargo test --test integration_tests
```

## Current Implementation Status

**✅ Completed:**
- Tree-sitter parser for Q-style syntax
- Lexical environments with closure support
- Function definition and call evaluation
- Basic arithmetic with operator precedence
- Memory-mapped columnar storage foundation

**🚧 In Progress:**
- REPL integration with persistent environments
- Vector/list operations
- Error handling improvements

**📋 Planned:**
- Control flow (`if`, `do`, `while`)
- Symbol interning for performance
- APL-inspired array primitives
- Advanced storage optimizations

## Key Technical Details

### Function Evaluation Flow
1. Parse with Tree-sitter → AST nodes
2. `evaluator.eval_with_env()` dispatches on node type
3. Assignments create `Value::Function` with captured closure
4. Function calls bind parameters and evaluate body in new environment

### Value System
- `Value::Integer(i64)` for numbers
- `Value::Function { params, body, closure }` for functions
- Environment chain traversal for lexical scoping

### Storage Architecture
- Arrow2-based columnar layout
- MessagePack serialization for values
- Memory-mapped files for zero-copy access
- One file per column in splayed tables

### Grammar Syntax
- Assignments: `name: value` or `name: {body}`
- Function bodies: `{expr}` or `{[params] expr}`
- Function calls: `name[args]` with `;`-separated arguments
- Standard arithmetic with Q-style precedence

## Testing Strategy

- Unit tests in each module (`src/*/mod.rs`)
- Integration tests in `tests/` directory
- Grammar parsing tests in `tests/function_parsing.rs`
- Function evaluation tests in `tests/function_evaluation.rs`
- Storage tests in `storage/tests/`

All existing arithmetic functionality must continue to work as new features are added.

## Development Workflow Guidance

- regularly ensure everything compiles, all tests pass, `cargo check` and `cargo clippy` are clean, and then create checkpoint commits.
