# Chapter 5: Language Reference and Syntax Guide

## Overview

This chapter provides a comprehensive reference for the wabznasm language syntax, covering all currently implemented features with detailed examples and usage patterns. This serves as both a learning resource and a quick reference for developers.

## Basic Syntax

### Literals

#### Integer Literals

Integer literals support standard decimal notation:

```wabz
42          // Positive integer
-17         // Negative integer
0           // Zero
9223372036854775807   // Maximum i64 value
-9223372036854775808  // Minimum i64 value
```

**Range**: 64-bit signed integers (-2^63 to 2^63-1)

**Error Cases**:
- Overflow: Numbers outside i64 range
- Invalid format: Non-numeric characters

```wabz
9999999999999999999999999999999  // Error: Integer overflow
abc                              // Error: Invalid number (now treated as identifier)
12.34                           // Error: Decimal not supported yet
```

### Identifiers

Identifiers follow standard programming language conventions:

```wabz
x           // Simple identifier
foo         // Multi-character identifier
my_var      // Underscore allowed
calculate   // Descriptive names
f1          // Numbers allowed after first character
```

**Rules**:
- Must start with letter or underscore
- Can contain letters, digits, underscores
- Case-sensitive
- No length limit
- Cannot be reserved words (when implemented)

### Comments

End-of-line comments are supported using Q/KDB+ style backslash syntax:

```wabz
x: 42 \ This is an end-of-line comment
add: {[a;b] a + b} \ Function definition with comment
result: add[10; 20] \ Should return 30

\ Comments can also start at the beginning of a line
y: result * 2 \ Multiply by 2
```

**Important**: The `/` character is reserved for division operations only. To avoid ambiguity between division (`1/2`) and comments, only backslash (`\`) comments are supported.

**Rules**:
- Comments start with `\` and continue to end of line
- Can appear at end of any line after code
- Can appear on their own line
- Everything after `\` is ignored by the parser
- Comments do not nest

## Expressions

### Arithmetic Operations

#### Binary Operations

All arithmetic operations use infix notation with right-to-left associativity:

```wabz
// Addition
2 + 3               // → 5
10 + 20 + 30        // → 10 + (20 + 30) → 60

// Subtraction
10 - 3              // → 7
100 - 20 - 5        // → 100 - (20 - 5) → 85

// Multiplication
4 * 5               // → 20
2 * 3 * 4           // → 2 * (3 * 4) → 24

// Division
20 / 4              // → 5
24 / 6 / 2          // → 24 / (6 / 2) → 8

// Modulo
17 % 5              // → 2
100 % 7 % 3         // → 100 % (7 % 3) → 3

// Exponentiation
2 ^ 3               // → 8
2 ^ 3 ^ 2           // → 2 ^ (3 ^ 2) → 512
```

#### Unary Operations

```wabz
// Negation
-5                  // → -5
--10                // → 10 (double negation)
-(2 + 3)            // → -5

// Factorial
5!                  // → 120
3!!                 // → (3!)! → 6! → 720
(2 + 3)!            // → 5! → 120
```

#### Operator Precedence

From highest to lowest precedence (right-to-left within same level):

1. **Factorial** (`!`) - postfix, highest precedence
2. **Exponentiation** (`^`) - right-associative
3. **Unary minus** (`-`) - prefix negation
4. **Multiplication/Division/Modulo** (`*`, `/`, `%`) - same level
5. **Addition/Subtraction** (`+`, `-`) - lowest precedence

```wabz
// Precedence examples
2 + 3 * 4           // → 2 + (3 * 4) → 14
2 * 3 ^ 2           // → 2 * (3 ^ 2) → 18
-2 ^ 2              // → -(2 ^ 2) → -4
5!^2                // → (5!)^2 → 120^2 → 14400
```

#### Grouping with Parentheses

Use parentheses to override default precedence:

```wabz
(2 + 3) * 4         // → 5 * 4 → 20
(10 - 2) / (3 + 1)  // → 8 / 4 → 2
-(2 + 3)            // → -5
(2 + 3)!            // → 5! → 120
```

### Error Handling

Arithmetic operations include comprehensive error checking:

#### Division by Zero

```wabz
5 / 0               // Error: Division by zero
10 % 0              // Error: Division by zero (modulo)
```

#### Integer Overflow

```wabz
9223372036854775807 + 1         // Error: Integer overflow
9223372036854775807 * 2         // Error: Integer overflow
```

#### Domain Violations

```wabz
(-5)!               // Error: Factorial of negative number
25!                 // Error: Factorial too large (>20)
2 ^ (-3)            // Error: Negative exponent
2 ^ 100             // Error: Exponent too large (>63)
```

## Variable Assignment

### Basic Assignment

Variables are assigned using colon notation:

```wabz
x: 42               // Assign integer to x
y: 2 + 3            // Assign expression result to y
result: x * y       // Use previously defined variables
```

**Semantics**:
- Assignment creates binding in current environment
- Variables shadow outer scope definitions
- Assignment returns the assigned value
- Variables must be defined before use

### Variable Lookup

Variables are resolved through lexical scoping:

```wabz
x: 10               // Define x
y: x + 5            // Use x → 15

outer: 100
{
    inner: 50
    total: outer + inner    // Access both outer and inner → 150
}
```

**Lookup Rules**:
- Search current environment first
- Traverse parent environments if not found
- Error if variable not found in any scope

### Environment Scoping

Variables follow lexical scoping rules:

```wabz
x: 10               // Global scope
{
    x: 20           // Local scope, shadows global
    x               // Returns 20 (local value)
}
x                   // Returns 10 (global value unchanged)
```

## Function System

### Function Definition

Functions are defined using assignment with curly brace syntax:

#### Simple Functions

Functions without explicit parameters:

```wabz
// Simple function accessing closure variable
x: 5
f: {x + 1}          // Function capturing x from closure

// Function with computation
compute: {2 * 3 + 4}

// Function accessing multiple closure variables
a: 10
b: 20
sum: {a + b}
```

#### Functions with Parameters

Functions with explicit parameter lists:

```wabz
// Single parameter
double: {[x] x * 2}

// Multiple parameters
add: {[x; y] x + y}

// Many parameters
calc: {[a; b; c; d] a * b + c * d}

// Descriptive parameter names
calculate_area: {[width; height] width * height}
```

**Parameter Syntax**:
- Parameters enclosed in `[` and `]`
- Multiple parameters separated by `;`
- Parameter names follow identifier rules
- Parameters shadow closure variables

### Function Calls

Functions are called using bracket notation:

#### No Arguments

```wabz
f: {42}
f[]                 // Returns 42

compute: {2 + 3}
compute[]           // Returns 5
```

#### Single Argument

```wabz
double: {[x] x * 2}
double[5]           // Returns 10

square: {[n] n * n}
square[7]           // Returns 49
```

#### Multiple Arguments

```wabz
add: {[x; y] x + y}
add[3; 4]           // Returns 7

max: {[a; b] if a > b then a else b}  // (when conditionals implemented)
max[10; 15]         // Returns 15

// Complex expressions as arguments
add[2 * 3; 4 + 5]   // Returns add[6; 9] → 15
```

### Closure Semantics

Functions capture their defining environment:

```wabz
// Example 1: Basic closure capture
x: 10
f: {x + 5}          // Captures x=10
x: 20               // Change x after function definition
f[]                 // Returns 15 (original x value)

// Example 2: Multiple closure variables
a: 100
b: 200
combine: {a + b}    // Captures both a and b
combine[]           // Returns 300

// Example 3: Nested closure access
outer: 50
make_adder: {[n] {outer + n}}    // Inner function captures both outer and n
add_to_outer: make_adder[25]
add_to_outer[]      // Returns 75 (50 + 25)
```

### Function Composition

Functions are first-class values enabling composition:

```wabz
// Higher-order function example
apply_twice: {[f; x] f[f[x]]}

double: {[x] x * 2}
apply_twice[double; 3]          // Returns double[double[3]] → double[6] → 12

// Function returning function
make_multiplier: {[factor] {[x] x * factor}}
triple: make_multiplier[3]
triple[4]                       // Returns 12
```

### Recursive Functions

Functions can call themselves (when variable is in scope):

```wabz
// Factorial function (alternative to ! operator)
factorial: {[n] if n <= 1 then 1 else n * factorial[n-1]}  // (when conditionals implemented)

// Fibonacci function
fib: {[n] if n <= 1 then n else fib[n-1] + fib[n-2]}      // (when conditionals implemented)
```

*Note: Conditional expressions are planned but not yet implemented*

## Type System

### Current Types

The language currently supports two value types:

#### Integer Type

- **Range**: 64-bit signed integers (-2^63 to 2^63-1)
- **Operations**: All arithmetic operations
- **Literals**: Decimal notation
- **Display**: Direct numeric output

```wabz
x: 42               // Integer value
type: "Integer"     // Runtime type (conceptual)
```

#### Function Type

- **Parameters**: List of parameter names
- **Body**: Source code as string
- **Closure**: Optional captured environment
- **Callable**: Invokable with bracket notation

```wabz
f: {[x; y] x + y}   // Function value
// Internal representation:
// Function {
//   params: ["x", "y"],
//   body: "x + y",
//   closure: Some(env)
// }
```

### Type Checking

Type checking occurs at runtime during evaluation:

```wabz
// Valid operations
x: 42
y: x + 10           // Integer arithmetic: OK

f: {[a] a * 2}
result: f[5]        // Function call: OK

// Type errors (conceptual - not all implemented)
x + f               // Error: Cannot add integer and function
f[x; y; z]          // Error: Arity mismatch (f expects 1 argument)
unknown_var         // Error: Undefined variable
```

### Type Coercion

No implicit type coercion is performed:

```wabz
// No automatic conversions
x: 42
f: {[a] a}
x + f               // Error: Type mismatch

// Explicit conversion (when implemented)
string_x: string[x] // Convert integer to string (planned)
```

## Error Handling

### Error Categories

The language provides detailed error reporting:

#### Mathematical Errors

```wabz
// Division by zero
10 / 0              // Error: Division by zero
15 % 0              // Error: Division by zero

// Overflow errors
9223372036854775807 + 1     // Error: Integer overflow in addition
9223372036854775807 * 2     // Error: Integer overflow in multiplication

// Domain errors
(-5)!               // Error: Factorial of negative number
25!                 // Error: Factorial too large
2 ^ (-1)            // Error: Negative exponent
2 ^ 100             // Error: Exponent too large
```

#### Language Errors

```wabz
// Undefined variables
undefined_var       // Error: Undefined variable 'undefined_var'

// Arity mismatches
f: {[x; y] x + y}
f[1]                // Error: Arity mismatch: expected 2 arguments, got 1
f[1; 2; 3]          // Error: Arity mismatch: expected 2 arguments, got 3

// Invalid function calls
x: 42
x[1]                // Error: Cannot call non-function value
```

#### Syntax Errors

```wabz
// Parse errors
1 +                 // Error: Missing operand
@#$                 // Error: Invalid token
{[x;y] x +}         // Error: Incomplete expression
```

### Error Context

All errors include source location information for debugging:

```
Error: Division by zero
  at line 1, column 5
    10 / 0
        ^
```

## REPL Features

### Interactive Usage

The REPL provides an interactive environment with persistent state:

```bash
$ cargo run
wabznasm REPL: enter expressions, assignments, or function definitions. Type 'exit' to quit
Examples: 1+2, f: {x+1}, add: {[x;y] x+y}, f[5], add[2;3]

wabz> 2 + 3
= 5

wabz> x: 42
= 42

wabz> f: {x + 10}
= {expr}

wabz> f[]
= 52

wabz> add: {[a; b] a + b}
= {[a;b] expr}

wabz> add[10; 20]
= 30

wabz> exit
```

### REPL Commands

#### Expression Evaluation

```wabz
wabz> 1 + 2 * 3     // Arithmetic expressions
= 7

wabz> (1 + 2) * 3   // Grouping
= 9
```

#### Variable Assignment

```wabz
wabz> x: 100        // Variable assignment
= 100

wabz> y: x / 2      // Using previous variables
= 50
```

#### Function Definition

```wabz
wabz> square: {[n] n * n}    // Function definition
= {[n] expr}

wabz> square[8]              // Function call
= 64
```

#### Error Display

```wabz
wabz> 1 / 0
Error: Division by zero

wabz> undefined_variable
Error: Undefined variable 'undefined_variable'
```

### State Persistence

The REPL maintains persistent state across commands:

```wabz
wabz> x: 10
= 10

wabz> f: {x * 2}     // Function captures current x
= {expr}

wabz> x: 20          // Change x value

wabz> f[]            // Function still uses original x
= 20                 // (10 * 2)

wabz> x              // Current x value
= 20
```

## Grammar Reference

### Formal Syntax

The complete grammar in EBNF-like notation:

```ebnf
source_file := expression

expression := assignment
           | additive

assignment := identifier ":" (expression | function_body)

function_body := "{" [parameter_list] expression "}"

parameter_list := "[" identifier (";" identifier)* "]"

function_call := identifier "[" [argument_list] "]"

argument_list := expression (";" expression)*

additive := multiplicative (("+" | "-") multiplicative)*

multiplicative := power (("*" | "/" | "%") power)*

power := unary ("^" unary)*

unary := ("-" unary) | postfix

postfix := primary ("!")*

primary := number
        | identifier
        | function_call
        | "(" expression ")"

identifier := [a-zA-Z_][a-zA-Z0-9_]*

number := "-"? [0-9]+
```

### Node Types

Tree-sitter AST node types used by the evaluator:

- `source_file` - Root node
- `expression` - Expression wrapper
- `statement` - Statement wrapper
- `assignment` - Variable assignment
- `function_body` - Function definition body
- `function_call` - Function invocation
- `parameter_list` - Function parameter list
- `argument_list` - Function argument list
- `additive` - Addition/subtraction
- `multiplicative` - Multiplication/division/modulo
- `power` - Exponentiation
- `unary` - Unary negation
- `postfix` - Factorial operation
- `primary` - Parenthesized expressions
- `number` - Integer literals
- `identifier` - Variable names

## Examples and Patterns

### Basic Calculations

```wabz
// Simple arithmetic
2 + 3 * 4           // = 14
(2 + 3) * 4         // = 20
10 - 2^3            // = 2
5! / 2^3            // = 15

// Using variables
price: 100
tax_rate: 8
tax: price * tax_rate / 100     // = 8
total: price + tax              // = 108
```

### Function Patterns

```wabz
// Mathematical functions
square: {[x] x * x}
cube: {[x] x * x * x}
circle_area: {[r] 3 * r * r}    // Approximation

// Utility functions
max: {[a; b] if a > b then a else b}     // (when conditionals implemented)
min: {[a; b] if a < b then a else b}     // (when conditionals implemented)
abs: {[x] if x < 0 then -x else x}       // (when conditionals implemented)

// Function composition
apply: {[f; x] f[x]}
compose: {[f; g; x] f[g[x]]}             // (when higher-order functions work)
```

### Closure Examples

```wabz
// Closure capture
base: 100
add_to_base: {[x] base + x}
add_to_base[23]     // = 123

// Counter pattern (when mutable state implemented)
make_counter: {[start] {start: start + 1}}    // (conceptual)

// Configuration pattern
config_multiplier: 5
process: {[data] data * config_multiplier}
process[20]         // = 100
```

### Error Handling Examples

```wabz
// Graceful error handling (conceptual)
safe_divide: {[a; b] if b = 0 then 0 else a / b}     // (when conditionals implemented)
validate_input: {[x] if x < 0 then 0 else x}         // (when conditionals implemented)

// Input validation patterns
factorial_safe: {[n]
    if n < 0 then 0
    else if n > 20 then 0
    else n!
}  // (when conditionals implemented)
```

## Future Language Features

### Planned Extensions

The following features are planned for future implementation:

#### List Types
```wabz
// Homogeneous lists (planned)
numbers: [1; 2; 3; 4; 5]
names: ["alice"; "bob"; "charlie"]

// List operations
first[numbers]      // = 1
last[numbers]       // = 5
count[numbers]      // = 5
sum[numbers]        // = 15
```

#### Conditional Expressions
```wabz
// If-then-else (planned)
max: {[a; b] if a > b then a else b}
sign: {[x] if x > 0 then 1 else if x < 0 then -1 else 0}
```

#### String Types
```wabz
// String literals and operations (planned)
name: "wabznasm"
greeting: "Hello, " + name + "!"
length[greeting]    // String length
```

#### Dictionary Types
```wabz
// Key-value mappings (planned)
person: `name`age!("Alice"; 30)
person[`name]       // = "Alice"
keys[person]        // = [`name; `age]
```

#### Table Types
```wabz
// Columnar tables (planned)
people: ([] name: ["Alice"; "Bob"]; age: [30; 25])
select from people where age > 25
```

These features will maintain compatibility with the current implementation while extending the language's capabilities.
