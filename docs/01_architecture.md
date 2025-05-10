# Chapter 1: Architecture and Language Foundation

## Overview

wabznasm is a high-performance array processing language implemented in Rust, inspired by Q/KDB+ but modernized with concepts from APL, J, and functional programming languages. This chapter explains the fundamental architecture and design principles that underpin the current implementation.

## System Architecture

### Core Components

The wabznasm system is structured around several key components that work together to provide a functional programming environment:

1. **Tree-sitter Grammar**: Provides robust, incremental parsing capabilities
2. **Environment System**: Manages lexical scoping and variable bindings
3. **Evaluator**: Implements the expression evaluation engine with environment support
4. **REPL**: Interactive read-eval-print loop with persistent state
5. **Storage Layer**: Columnar data storage using Arrow2 and memory mapping

### Language Paradigm

wabznasm follows several key design principles:

- **Functional-first**: Functions are first-class values with lexical closures
- **Right-to-left evaluation**: Expressions evaluate from right to left following Q conventions
- **Environment-aware evaluation**: All evaluation occurs within lexical environments
- **Type safety**: Strong typing with runtime type checking and conversion

## Core Value System

### Value Types

The language currently supports two primary value types:

```rust
pub enum Value {
    Integer(i64),
    Function {
        params: Vec<String>,
        body: String,
        closure: Option<Rc<Environment>>,
    },
}
```

#### Integer Values

- **Type**: 64-bit signed integers (`i64`)
- **Range**: -9,223,372,036,854,775,808 to 9,223,372,036,854,775,807
- **Operations**: Full arithmetic support with overflow checking
- **Literals**: Direct numeric literals: `42`, `-17`, `0`

#### Function Values

Functions in wabznasm are first-class values that capture their lexical environment:

- **Parameters**: Named parameter list with explicit binding
- **Body**: Source code stored as string for re-parsing and evaluation
- **Closure**: Optional captured environment for lexical scoping

### Type Introspection

Values provide runtime type information through methods:

```rust
impl Value {
    pub fn is_function(&self) -> bool
    pub fn arity(&self) -> Option<usize>
    pub fn as_integer(&self) -> Option<i64>
}
```

## Environment System

### Lexical Environments

The environment system implements lexical scoping with proper closure semantics:

```rust
pub struct Environment {
    bindings: HashMap<String, Value>,
    parent: Option<Rc<Environment>>,
}
```

#### Key Features

1. **Lexical Scoping**: Inner environments can access outer scope variables
2. **Variable Shadowing**: Local definitions override parent scope bindings
3. **Immutable Semantics**: Environments are logically immutable with controlled extension
4. **Reference Counting**: Shared ownership through `Rc<Environment>`

#### Environment Operations

- **`define(name, value)`**: Add new binding to current environment
- **`lookup(name)`**: Resolve variable through scope chain
- **`extend()`**: Create child environment inheriting current scope
- **`bind_parameters(params, args)`**: Create function call environment

### Closure Semantics

Functions capture their defining environment, enabling lexical closures:

```wabz
outer: 5
f: {outer + x}  // Captures 'outer' from defining environment
f[3]           // Returns 8 (5 + 3)
```

The closure is stored as `Option<Rc<Environment>>` and used as the base environment for function evaluation.

## Expression Evaluation

### Evaluation Strategy

The evaluator uses a visitor pattern to traverse the Abstract Syntax Tree (AST) produced by Tree-sitter:

```rust
impl Evaluator {
    pub fn eval_with_env<'a>(
        &self,
        node: Node<'a>,
        src: &str,
        env: &mut Environment
    ) -> Result<Value, EvalError>
}
```

#### Key Characteristics

1. **Environment-Aware**: All evaluation passes mutable environment reference
2. **AST-Walking**: Direct interpretation without intermediate representation
3. **Error Propagation**: Comprehensive error handling with source location
4. **Type-Flexible**: Returns `Value` enum supporting multiple types

### Node Type Dispatch

The evaluator dispatches on Tree-sitter node types:

- **`number`**: Parse integer literals
- **`identifier`**: Variable lookup in environment
- **`assignment`**: Variable binding and function definition
- **`function_call`**: Function application with parameter binding
- **`additive`/`multiplicative`**: Binary arithmetic operations
- **`unary`**: Unary negation
- **`power`**: Exponentiation
- **`postfix`**: Factorial operations

### Function Evaluation

Function calls follow a precise evaluation sequence:

1. **Function Resolution**: Look up function value by identifier
2. **Argument Evaluation**: Evaluate all arguments in current environment
3. **Parameter Binding**: Create call environment with parameter bindings
4. **Body Parsing**: Re-parse function body (stored as string)
5. **Body Evaluation**: Evaluate body in call environment with closure as parent

## Grammar and Parsing

### Tree-sitter Integration

wabznasm uses Tree-sitter for robust, incremental parsing:

- **Error Recovery**: Handles partial/invalid input gracefully
- **Performance**: Incremental parsing for interactive use
- **Flexibility**: Grammar modifications without parser regeneration
- **Stability**: Mature parsing technology with proven reliability

### Syntax Overview

The current grammar supports:

```
Expression Types:
- Numbers: 42, -17, 0
- Identifiers: x, foo, calculate
- Assignments: name: value
- Function Bodies: {expr} or {[params] expr}
- Function Calls: func[args]
- Arithmetic: +, -, *, /, %, ^, !
- Grouping: (expr)

Function Definitions:
- Simple: f: {x+1}
- With Parameters: add: {[x;y] x+y}

Function Calls:
- No Arguments: f[]
- With Arguments: add[2;3]
```

### Right-to-Left Evaluation

Following Q conventions, expressions evaluate right-to-left:

```wabz
2 + 3 * 4  // Evaluates as 2 + (3 * 4) = 14
```

Explicit grouping with parentheses overrides default precedence:

```wabz
(2 + 3) * 4  // Evaluates as (2 + 3) * 4 = 20
```

## Error Handling

### Error Types

The system defines comprehensive error categories:

```rust
pub enum EvalErrorKind {
    DivisionByZero,
    IntegerOverflow(String),
    UnknownOperator(String),
    InvalidNumber(String),
    MissingOperand,
    UndefinedVariable(String),
    Other(String),
}
```

### Error Context

All errors include source location information:

```rust
pub struct EvalError {
    kind: EvalErrorKind,
    node: Node,  // AST node for source location
}
```

This enables precise error reporting with line/column information for debugging.

### Error Propagation

The evaluation system uses Rust's `Result<T, E>` for error propagation:

- **Explicit Handling**: All error cases explicitly handled
- **Early Return**: `?` operator for propagation
- **Context Preservation**: Error location maintained through call stack

## Memory Management

### Ownership Model

The system follows Rust's ownership principles:

- **Move Semantics**: Values transferred by default
- **Reference Counting**: Shared ownership for environments (`Rc<Environment>`)
- **Cloning**: Explicit copying where needed
- **Lifetime Management**: Automatic cleanup without garbage collection

### Performance Considerations

- **Arena Allocation**: Future consideration for value allocation
- **String Interning**: Planned for identifier optimization
- **Copy-on-Write**: Potential for environment optimization
- **SIMD**: Planned for vector operations

## Testing Strategy

### Test Coverage

The current implementation includes comprehensive tests:

- **Environment Tests**: Scope resolution and closure behavior
- **Function Tests**: Definition, calling, and parameter binding
- **Evaluation Tests**: All arithmetic operations and edge cases
- **Parser Tests**: Grammar coverage and error recovery
- **Integration Tests**: End-to-end functionality

### Test Organization

Tests are organized by functional area:

- `src/environment.rs`: Environment behavior
- `tests/function_evaluation.rs`: Function system tests
- `tests/function_parsing.rs`: Grammar tests
- Integration with existing arithmetic test suites

## Future Directions

### Planned Extensions

1. **List/Vector Types**: Homogeneous collections with SIMD operations
2. **Dictionary Types**: Hash-based key-value mappings
3. **Table Types**: Columnar data structures
4. **Control Flow**: Conditional expressions and loops
5. **Pattern Matching**: Structured data matching
6. **Module System**: Namespace organization
7. **Performance Optimization**: JIT compilation and vectorization

### Compatibility Goals

- **Q/KDB+ Compatibility**: Syntax and semantic alignment where practical
- **Rust Integration**: Seamless interop with Rust ecosystem
- **Performance Parity**: Match or exceed Q performance characteristics
- **Modern Features**: Contemporary language features while maintaining terseness
