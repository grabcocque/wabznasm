# Chapter 3: Implementation Details and Internal Architecture

## Overview

This chapter provides an in-depth technical analysis of wabznasm's implementation, focusing on the internal data structures, algorithms, and design decisions that enable the language's functionality and performance characteristics.

## Tree-sitter Integration

### Parser Architecture

wabznasm leverages Tree-sitter for robust, incremental parsing with excellent error recovery:

```rust
// Core parsing interface
pub fn parse_expression(input: &str) -> Result<Tree, Report> {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&language())?;

    let tree = parser.parse(input, None)
        .ok_or_else(|| miette::miette!("Failed to parse expression"))?;

    if tree.root_node().has_error() {
        return Err(create_syntax_error(&tree, input));
    }

    Ok(tree)
}
```

### Grammar Definition

The Tree-sitter grammar defines the language syntax in `grammar/grammar.js`:

```javascript
// Key grammar rules
assignment: $ => seq(
    field('name', $.identifier),
    ':',
    field('value', choice($.expression, $.function_body))
),

function_call: $ => seq(
    field('function', $.identifier),
    '[',
    optional(field('args', $.argument_list)),
    ']'
),

function_body: $ => seq(
    '{',
    optional(field('params', $.parameter_list)),
    field('body', $.expression),
    '}'
)
```

### AST Node Processing

The evaluator dispatches on node types using Tree-sitter's built-in node classification:

```rust
pub fn eval_with_env<'a>(
    &self,
    node: Node<'a>,
    src: &str,
    env: &mut Environment
) -> Result<Value, EvalError> {
    match node.kind() {
        // Wrapper nodes - delegate to child
        "source_file" | "expression" | "statement" => {
            let child = self.named_child(node)?;
            self.eval_with_env(child, src, env)
        }

        // Leaf nodes - direct evaluation
        "number" => Ok(Value::Integer(self.visit_number_raw(node, src)?)),
        "identifier" => self.visit_identifier(node, src, env),

        // Complex nodes - specialized handlers
        "assignment" => self.visit_assignment(node, src, env),
        "function_call" => self.visit_function_call(node, src, env),

        // Arithmetic operations
        "additive" | "multiplicative" => {
            Ok(Value::Integer(self.visit_binary_raw(node, src, env)?))
        }

        other => Err(EvalError::new(
            EvalErrorKind::Other(format!("Unexpected node type: {}", other)),
            node,
        )),
    }
}
```

## Value System Implementation

### Value Type Design

The `Value` enum provides a tagged union for runtime type safety:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Function {
        params: Vec<String>,
        body: String,
        closure: Option<Rc<Environment>>,
    },
}
```

### Value Methods

Values expose type-safe accessors and utilities:

```rust
impl Value {
    /// Check if value is a function
    pub fn is_function(&self) -> bool {
        matches!(self, Value::Function { .. })
    }

    /// Get function arity (parameter count)
    pub fn arity(&self) -> Option<usize> {
        match self {
            Value::Function { params, .. } => Some(params.len()),
            _ => None,
        }
    }

    /// Extract integer value
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Value::Integer(n) => Some(*n),
            _ => None,
        }
    }
}
```

### Memory Management

Values use Rust's ownership system with strategic reference counting:

- **Integers**: Direct ownership, no allocation
- **Function Parameters**: Owned `Vec<String>` for parameter names
- **Function Body**: Owned `String` for source code storage
- **Closures**: `Rc<Environment>` for shared ownership of captured environments

## Environment Implementation

### Data Structure Design

Environments implement lexical scoping through parent chains:

```rust
#[derive(Debug, Clone)]
pub struct Environment {
    bindings: HashMap<String, Value>,     // Local variable bindings
    parent: Option<Rc<Environment>>,      // Parent scope reference
}
```

### Core Operations

#### Variable Lookup Algorithm

Variable resolution traverses the scope chain with early termination:

```rust
pub fn lookup(&self, name: &str) -> Option<&Value> {
    // Check local bindings first
    if let Some(value) = self.bindings.get(name) {
        return Some(value);
    }

    // Recursively check parent scopes
    if let Some(parent) = &self.parent {
        return parent.lookup(name);
    }

    // Variable not found in any scope
    None
}
```

**Complexity**: O(d) where d is the maximum nesting depth of scopes.

#### Environment Extension

Creating child environments preserves the parent chain:

```rust
pub fn extend(&self) -> Environment {
    Environment {
        bindings: HashMap::new(),
        parent: Some(Rc::new(self.clone())),
    }
}
```

**Memory Impact**: Each extension creates a new HashMap but shares parent references through `Rc`.

#### Parameter Binding

Function calls create specialized environments with parameter bindings:

```rust
pub fn bind_parameters(
    &self,
    params: &[String],
    args: &[Value],
    node: Node,
) -> Result<Environment, EvalError> {
    // Validate arity
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

    // Bind each parameter to its corresponding argument
    for (param, arg) in params.iter().zip(args.iter()) {
        child_env.define(param.clone(), arg.clone());
    }

    Ok(child_env)
}
```

**Performance**: O(p) where p is the parameter count, due to HashMap insertions.

## Function System Implementation

### Function Definition Processing

Function definitions parse parameter lists and capture environments:

```rust
fn visit_assignment(&self, node: Node, src: &str, env: &mut Environment) -> Result<Value, EvalError> {
    // Extract assignment components
    let name_node = node.child_by_field_name("name")
        .ok_or_else(|| EvalError::new(EvalErrorKind::MissingOperand, node))?;
    let value_node = node.child_by_field_name("value")
        .ok_or_else(|| EvalError::new(EvalErrorKind::MissingOperand, node))?;

    // Get variable name
    let name = get_node_text(name_node, src).map_err(|e| {
        EvalError::new(EvalErrorKind::Other(e), name_node)
    })?;

    // Handle function body vs. regular expression
    let value = match value_node.kind() {
        "function_body" => self.visit_function_body(value_node, src, env)?,
        _ => self.eval_with_env(value_node, src, env)?,
    };

    // Store binding in environment
    env.define(name.to_string(), value.clone());
    Ok(value)
}
```

### Function Body Analysis

Function body parsing extracts parameters and body expressions:

```rust
fn visit_function_body(&self, node: Node, src: &str, env: &Environment) -> Result<Value, EvalError> {
    // Extract body expression
    let body_node = node.child_by_field_name("body")
        .ok_or_else(|| EvalError::new(EvalErrorKind::MissingOperand, node))?;
    let body_text = get_node_text(body_node, src).map_err(|e| {
        EvalError::new(EvalErrorKind::Other(e), body_node)
    })?;

    // Extract optional parameter list
    let params = if let Some(params_node) = node.child_by_field_name("params") {
        self.extract_parameter_list(params_node, src)?
    } else {
        vec![]
    };

    // Create function value with closure
    Ok(Value::Function {
        params,
        body: body_text.to_string(),
        closure: Some(Rc::new(env.clone())),  // Capture current environment
    })
}
```

### Function Call Execution

Function calls involve complex environment management:

```rust
fn visit_function_call(&self, node: Node, src: &str, env: &mut Environment) -> Result<Value, EvalError> {
    // 1. Resolve function
    let func_node = node.child_by_field_name("function")
        .ok_or_else(|| EvalError::new(EvalErrorKind::MissingOperand, node))?;
    let func_value = self.visit_identifier(func_node, src, env)?;

    // 2. Extract function components
    let (params, body, closure) = match func_value {
        Value::Function { params, body, closure } => (params, body, closure),
        _ => return Err(EvalError::new(
            EvalErrorKind::Other("Cannot call non-function value".into()),
            func_node,
        )),
    };

    // 3. Evaluate arguments
    let args = if let Some(args_node) = node.child_by_field_name("args") {
        self.extract_argument_list(args_node, src, env)?
    } else {
        vec![]
    };

    // 4. Create call environment
    let base_env = closure.as_ref().map(|c| c.as_ref()).unwrap_or(env);
    let mut call_env = base_env.bind_parameters(&params, &args, node)?;

    // 5. Parse and evaluate function body
    let tree = parse_expression(&body).map_err(|e| {
        EvalError::new(EvalErrorKind::Other(format!("Function body parse error: {}", e)), node)
    })?;

    self.eval_with_env(tree.root_node(), &body, &mut call_env)
}
```

**Performance Analysis**:
- Function resolution: O(d) for environment depth
- Argument evaluation: O(a×e) where a = argument count, e = expression complexity
- Environment creation: O(p) for parameter count
- Body parsing: O(b) where b = body source length
- Body evaluation: O(b×c) where c = body complexity

## Arithmetic Implementation

### Binary Operation Framework

Binary operations follow a consistent pattern with comprehensive error handling:

```rust
fn visit_binary_raw(&self, node: Node, src: &str, env: &mut Environment) -> Result<i64, EvalError> {
    if let Some(opn) = node.child_by_field_name("operator") {
        // Extract operands
        let lhs = self.child(node, "left")?;
        let rhs = self.child(node, "right")?;

        // Evaluate operands to integers
        let left_val = match self.eval_with_env(lhs, src, env)? {
            Value::Integer(n) => n,
            _ => return Err(EvalError::new(
                EvalErrorKind::Other("Expected integer in arithmetic".into()),
                lhs,
            )),
        };

        let right_val = match self.eval_with_env(rhs, src, env)? {
            Value::Integer(n) => n,
            _ => return Err(EvalError::new(
                EvalErrorKind::Other("Expected integer in arithmetic".into()),
                rhs,
            )),
        };

        // Perform operation with overflow checking
        let op = self.op_text(opn, src)?;
        evaluate_binary_operation(left_val, right_val, op, node, opn)
    } else {
        // Single child fallback
        let child = self.named_child(node)?;
        match self.eval_with_env(child, src, env)? {
            Value::Integer(n) => Ok(n),
            _ => Err(EvalError::new(
                EvalErrorKind::Other("Expected integer".into()),
                child,
            )),
        }
    }
}
```

### Overflow Detection

All arithmetic operations use checked arithmetic to prevent silent overflow:

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
            EvalError::new(EvalErrorKind::IntegerOverflow("addition".into()), node)
        }),
        "-" => left.checked_sub(right).ok_or_else(|| {
            EvalError::new(EvalErrorKind::IntegerOverflow("subtraction".into()), node)
        }),
        "*" => left.checked_mul(right).ok_or_else(|| {
            EvalError::new(EvalErrorKind::IntegerOverflow("multiplication".into()), node)
        }),
        "/" => {
            if right == 0 {
                Err(EvalError::new(EvalErrorKind::DivisionByZero, node))
            } else {
                Ok(left / right)
            }
        }
        "%" => {
            if right == 0 {
                Err(EvalError::new(EvalErrorKind::DivisionByZero, node))
            } else {
                Ok(left % right)
            }
        }
        _ => Err(EvalError::new(
            EvalErrorKind::UnknownOperator(op.into()),
            op_node,
        )),
    }
}
```

## REPL Implementation

### Interactive Loop Architecture

The REPL provides persistent environment state across interactions:

```rust
pub fn run() -> Result<(), eyre::Report> {
    let mut rl: Editor<(), DefaultHistory> = Editor::new()?;
    let mut env = Environment::new();        // Persistent environment
    let evaluator = Evaluator::new();

    println!("wabznasm REPL: enter expressions, assignments, or function definitions. Type 'exit' to quit");
    println!("Examples: 1+2, f: {{x+1}}, add: {{[x;y] x+y}}, f[5], add[2;3]");

    loop {
        match rl.readline("wabz> ") {
            Ok(line) => {
                let input = line.trim();
                if input.is_empty() { continue; }

                let _ = rl.add_history_entry(input);
                if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
                    break;
                }

                // Parse and evaluate with persistent environment
                match parse_expression(input) {
                    Ok(tree) => {
                        match evaluator.eval_with_env(tree.root_node(), input, &mut env) {
                            Ok(Value::Integer(val)) => println!("= {}", val),
                            Ok(Value::Function { params, .. }) => {
                                if params.is_empty() {
                                    println!("= {{expr}}");
                                } else {
                                    println!("= {{[{}] expr}}", params.join(";"));
                                }
                            }
                            Err(e) => eprintln!("Error: {:?}", e),
                        }
                    }
                    Err(e) => eprintln!("Parse error: {:?}", e),
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(err) => {
                eprintln!("Error: {}", err);
                break;
            }
        }
    }
    Ok(())
}
```

### State Persistence

The REPL maintains state through:

1. **Environment Persistence**: Single `Environment` instance across all interactions
2. **History Management**: `rustyline` provides command history and editing
3. **Error Recovery**: Individual expression errors don't terminate the session

### User Experience Features

- **Descriptive Prompts**: Clear examples and usage instructions
- **Function Display**: Readable representation of function values
- **Error Reporting**: Detailed error messages with context
- **Command History**: Arrow key navigation and command recall

## Error System Implementation

### Error Type Hierarchy

The error system provides comprehensive error categorization:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum EvalErrorKind {
    // Mathematical errors
    DivisionByZero,
    IntegerOverflow(String),
    FactorialOfNegative,
    FactorialTooLarge,
    NegativeExponent,
    ExponentTooLarge,

    // Language errors
    UnknownOperator(String),
    InvalidNumber(String),
    MissingOperand,
    UndefinedVariable(String),

    // Generic error category
    Other(String),
}

#[derive(Debug, Clone)]
pub struct EvalError {
    kind: EvalErrorKind,
    node: Node,  // Source location information
}
```

### Error Construction and Propagation

Errors include source location for precise debugging:

```rust
impl EvalError {
    pub fn new(kind: EvalErrorKind, node: Node) -> Self {
        Self { kind, node }
    }
}

// Usage pattern throughout evaluator
let value = some_operation().map_err(|_| {
    EvalError::new(
        EvalErrorKind::InvalidNumber("parse failed".into()),
        node
    )
})?;
```

### Integration with Reporting Systems

Error types integrate with external reporting through conversion traits:

```rust
// Integration with miette for enhanced error reporting
impl From<EvalError> for miette::Report {
    fn from(err: EvalError) -> Self {
        miette::miette!("Evaluation error: {:?}", err.kind)
    }
}
```

## Testing Infrastructure

### Test Organization

Tests are organized by functional area with comprehensive coverage:

```rust
// Environment tests in src/environment.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_basic_operations() {
        let mut env = Environment::new();
        env.define("x".to_string(), Value::Integer(42));
        assert_eq!(env.lookup("x"), Some(&Value::Integer(42)));
    }

    #[test]
    fn test_environment_lexical_scoping() {
        let mut global = Environment::new();
        global.define("global_var".to_string(), Value::Integer(1));

        let mut local = Environment::with_parent(Rc::new(global));
        local.define("local_var".to_string(), Value::Integer(2));

        assert_eq!(local.lookup("local_var"), Some(&Value::Integer(2)));
        assert_eq!(local.lookup("global_var"), Some(&Value::Integer(1)));
    }
}
```

### Integration Tests

Comprehensive end-to-end testing in `tests/` directory:

```rust
// tests/function_evaluation.rs
#[test]
fn test_function_call_with_args() {
    let mut env = Environment::new();
    let evaluator = Evaluator::new();

    // Define function: add: {[x;y] x+y}
    let tree = parse_expression("add: {[x;y] x+y}").unwrap();
    evaluator.eval_with_env(tree.root_node(), "add: {[x;y] x+y}", &mut env).unwrap();

    // Call function: add[2;3]
    let tree = parse_expression("add[2;3]").unwrap();
    let result = evaluator.eval_with_env(tree.root_node(), "add[2;3]", &mut env).unwrap();

    assert_eq!(result, Value::Integer(5)); // 2+3 = 5
}
```

### Performance Testing

Future performance testing framework:

1. **Benchmark Suite**: Criterion-based performance measurement
2. **Regression Detection**: CI integration for performance monitoring
3. **Memory Profiling**: Heap usage analysis for optimization targets
4. **Scaling Tests**: Behavior with large inputs and deep environments

## Future Implementation Considerations

### Optimization Opportunities

1. **String Interning**: Reduce memory usage for identifiers
2. **Bytecode Compilation**: Eliminate function body re-parsing
3. **Inline Caching**: Cache variable lookup results
4. **Environment Flattening**: Optimize deep scope chains
5. **SIMD Vectorization**: Parallel arithmetic for future list types

### Architectural Extensions

1. **List Types**: Homogeneous collections with efficient layouts
2. **Type System**: Static type inference and checking
3. **Module System**: Namespace organization and imports
4. **Garbage Collection**: Cycle detection for circular references
5. **JIT Compilation**: Runtime optimization for hot code paths

### Compatibility Considerations

1. **API Stability**: Maintain backward compatibility for core interfaces
2. **Performance Guarantees**: Ensure optimizations don't change semantics
3. **Error Behavior**: Consistent error handling across language features
4. **Memory Model**: Clear ownership and lifetime semantics
