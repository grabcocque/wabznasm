# Chapter 2: Language Semantics and Evaluation Model

## Overview

This chapter provides a detailed examination of wabznasm's evaluation semantics, function system, and runtime behavior. Understanding these concepts is essential for both language users and contributors to the implementation.

## Evaluation Model

### Expression Evaluation Pipeline

Every expression in wabznasm follows a consistent evaluation pipeline:

1. **Parsing**: Source text → Tree-sitter AST
2. **Validation**: AST node type checking
3. **Environment Resolution**: Variable and function lookup
4. **Evaluation**: Computation with type checking
5. **Result**: Typed value or error

```rust
// Core evaluation method signature
pub fn eval_with_env<'a>(
    &self,
    node: Node<'a>,
    src: &str,
    env: &mut Environment
) -> Result<Value, EvalError>
```

### Right-to-Left Associativity

Following Q/KDB+ conventions, wabznasm uses right-to-left evaluation:

```wabz
// Mathematical expressions
2 + 3 * 4     // → 2 + (3 * 4) → 2 + 12 → 14
2 ^ 3 ^ 2     // → 2 ^ (3 ^ 2) → 2 ^ 9 → 512

// Function composition
f g h x       // → f(g(h(x)))
```

This differs from most programming languages but provides natural mathematical notation and powerful composition capabilities.

### Type Coercion and Checking

The current implementation enforces strict typing with explicit conversions:

- **No Implicit Coercion**: Operations require matching types
- **Runtime Type Checking**: Dynamic verification during evaluation
- **Error Propagation**: Type mismatches generate descriptive errors

```rust
// Type checking in binary operations
let left_val = match self.eval_with_env(lhs, src, env)? {
    Value::Integer(n) => n,
    _ => return Err(EvalError::new(
        EvalErrorKind::Other("Expected integer in arithmetic".into()),
        lhs,
    )),
};
```

## Function System

### Function Definition Semantics

Functions in wabznasm are defined using assignment syntax with curly braces:

```wabz
// Simple function (implicit parameter)
f: {x + 1}

// Function with explicit parameters
add: {[x;y] x + y}

// Function with multiple parameters
calc: {[a;b;c] a * b + c}
```

#### Parameter Binding Rules

1. **Explicit Parameters**: Listed in `[param1;param2;...]` syntax
2. **Implicit Parameters**: Single expressions can reference variables from closure
3. **Arity Checking**: Function calls must match declared parameter count
4. **Lexical Binding**: Parameters shadow outer scope variables

### Function Call Semantics

Function calls use bracket notation with semicolon-separated arguments:

```wabz
// No arguments
f[]

// Single argument
f[5]

// Multiple arguments
add[2; 3]

// Nested calls
outer[inner[x]; y]
```

#### Call Resolution Process

1. **Function Lookup**: Resolve function identifier in environment
2. **Type Verification**: Ensure callable value (Function type)
3. **Arity Validation**: Check argument count matches parameters
4. **Argument Evaluation**: Evaluate all arguments in caller environment
5. **Environment Creation**: Build call environment with parameter bindings
6. **Body Evaluation**: Parse and evaluate function body in call environment

### Closure Semantics

Functions capture their lexical environment, creating true closures:

```wabz
x: 10
f: {x + y}    // Captures x=10 from definition environment

x: 20         // Changing x doesn't affect f's closure
f[5]          // Returns 15 (10 + 5), not 25
```

#### Closure Implementation

```rust
Value::Function {
    params: Vec<String>,           // Parameter names
    body: String,                  // Source code for re-parsing
    closure: Option<Rc<Environment>>, // Captured environment
}
```

The closure field stores a reference to the environment active when the function was defined, enabling proper lexical scoping.

### Higher-Order Functions

Functions are first-class values and can be passed as arguments and returned from other functions:

```wabz
// Function taking another function as parameter
apply: {[f;x] f[x]}

// Function returning a function
makeAdder: {[n] {x + n}}

// Usage
double: {x * 2}
apply[double; 5]     // Returns 10

addTen: makeAdder[10]
addTen[5]            // Returns 15
```

## Environment and Scoping

### Environment Structure

The environment system implements lexical scoping through a chain of binding maps:

```rust
pub struct Environment {
    bindings: HashMap<String, Value>,    // Local bindings
    parent: Option<Rc<Environment>>,     // Parent scope
}
```

### Variable Resolution Algorithm

Variable lookup follows the lexical scope chain:

1. **Local Lookup**: Check current environment bindings
2. **Parent Traversal**: Recursively check parent environments
3. **Error Generation**: Undefined variable error if not found

```rust
pub fn get(&self, name: &str, node: Node) -> Result<&Value, EvalError> {
    if let Some(value) = self.bindings.get(name) {
        Ok(value)
    } else if let Some(parent) = &self.parent {
        parent.get(name, node)
    } else {
        Err(EvalError::new(
            EvalErrorKind::UndefinedVariable(name.to_string()),
            node,
        ))
    }
}
```

### Assignment Semantics

Variable assignment creates bindings in the current environment:

```wabz
x: 42           // Bind x to 42 in current environment
f: {x + 1}      // Bind f to function value
```

Assignment always targets the local environment, enabling variable shadowing:

```wabz
x: 10           // Global x
{
    x: 20       // Local x shadows global
    x           // Returns 20
}
x               // Still returns 10
```

### Environment Extension

Function calls create extended environments with parameter bindings:

```rust
pub fn bind_parameters(
    &self,
    params: &[String],
    args: &[Value],
    node: Node,
) -> Result<Environment, EvalError> {
    // Arity checking
    if params.len() != args.len() {
        return Err(EvalError::new(
            EvalErrorKind::Other(format!(
                "Arity mismatch: expected {} arguments, got {}",
                params.len(),
                args.len()
            )),
            node,
        ));
    }

    // Create child environment
    let mut child_env = self.extend();

    // Bind parameters to arguments
    for (param, arg) in params.iter().zip(args.iter()) {
        child_env.define(param.clone(), arg.clone());
    }

    Ok(child_env)
}
```

## Arithmetic System

### Supported Operations

The arithmetic system provides comprehensive numeric operations:

- **Addition**: `+` with overflow checking
- **Subtraction**: `-` with overflow checking
- **Multiplication**: `*` with overflow checking
- **Division**: `/` with zero-check
- **Modulo**: `%` with zero-check
- **Exponentiation**: `^` with range limits
- **Factorial**: `!` with domain restrictions
- **Unary Negation**: `-x` with overflow checking

### Overflow Handling

All arithmetic operations include overflow detection:

```rust
fn evaluate_binary_operation(
    left: i64,
    right: i64,
    op: &str,
    node: Node,
    op_node: Node,
) -> Result<i64, EvalError> {
    match op {
        "+" => left.checked_add(right).ok_or_else(|| {
            EvalError::new(
                EvalErrorKind::IntegerOverflow("addition".into()),
                node
            )
        }),
        // ... other operations
    }
}
```

This prevents silent wraparound and ensures predictable behavior with large numbers.

### Domain Restrictions

Certain operations have domain restrictions for safety and predictability:

- **Factorial**: Only non-negative integers ≤ 20
- **Exponentiation**: Non-negative exponents ≤ 63
- **Division/Modulo**: Non-zero divisors

These restrictions prevent infinite loops, stack overflows, and undefined behavior.

## Error Handling Model

### Error Categories

The error system categorizes errors by type and context:

```rust
pub enum EvalErrorKind {
    DivisionByZero,                    // Mathematical errors
    IntegerOverflow(String),           // Overflow in operations
    FactorialOfNegative,               // Domain violations
    FactorialTooLarge,
    NegativeExponent,
    ExponentTooLarge,
    UnknownOperator(String),           // Syntax errors
    InvalidNumber(String),
    MissingOperand,
    UndefinedVariable(String),         // Environment errors
    Other(String),                     // Generic errors
}
```

### Error Propagation

Errors propagate through the evaluation stack using Rust's `Result` type:

```rust
// Error propagation in nested evaluation
let result = self.eval_with_env(child_node, src, env)?;
//                                                  ^ Automatic propagation
```

This ensures errors are caught at the appropriate level with full context preservation.

### Source Location Tracking

All errors include source location information from the AST:

```rust
pub struct EvalError {
    kind: EvalErrorKind,
    node: Node,  // Tree-sitter node with source position
}
```

This enables precise error reporting in the REPL and development tools.

## Performance Characteristics

### Evaluation Complexity

Current implementation complexities:

- **Variable Lookup**: O(d) where d = environment depth
- **Function Call**: O(p + b) where p = parameter count, b = body complexity
- **Arithmetic**: O(1) for basic operations
- **Assignment**: O(1) for binding creation

### Memory Usage

Memory usage patterns:

- **Environment Chain**: Linear in scope depth
- **Function Storage**: Linear in body source length
- **Value Storage**: Constant for integers, linear for function parameter count
- **Closure Capture**: Shared reference counting, no copying

### Optimization Opportunities

Future optimization targets:

1. **String Interning**: Reduce memory for identifiers
2. **Bytecode Compilation**: Eliminate re-parsing function bodies
3. **Inline Caching**: Optimize variable lookups
4. **SIMD Operations**: Vectorized arithmetic for arrays
5. **JIT Compilation**: Runtime optimization for hot functions

## Compatibility and Extensions

### Q/KDB+ Alignment

Current compatibility with Q/KDB+ semantics:

- ✅ Right-to-left evaluation
- ✅ Function assignment syntax
- ✅ Bracket notation for function calls
- ✅ Lexical scoping behavior
- ⚠️ Limited type system (integers and functions only)
- ❌ Missing: lists, dictionaries, tables
- ❌ Missing: system functions and operators

### Planned Extensions

Near-term language extensions:

1. **List Types**: Homogeneous collections with vector operations
2. **Dictionary Types**: Key-value mappings with hash table implementation
3. **Table Types**: Columnar data structures with query operations
4. **Control Flow**: Conditional expressions and iteration constructs
5. **Pattern Matching**: Destructuring and conditional binding
6. **Module System**: Namespace organization and import/export

### Implementation Strategy

Extension implementation follows these principles:

1. **Backward Compatibility**: Existing code continues to work
2. **Performance**: New features don't impact existing performance
3. **Type Safety**: Strong typing with runtime verification
4. **Compositional**: Features work together naturally
5. **Incremental**: Each feature is independently useful
