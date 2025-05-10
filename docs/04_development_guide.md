# Chapter 4: Development Guide and Contributing

## Overview

This chapter provides a comprehensive guide for developers working on wabznasm, including development workflows, testing procedures, debugging techniques, and contribution guidelines. Whether you're fixing bugs, adding features, or optimizing performance, this guide will help you navigate the codebase effectively.

## Development Environment Setup

### Prerequisites

Required tools and dependencies:

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable

# Development tools
cargo install cargo-edit          # Enhanced cargo commands
cargo install cargo-watch        # File watching for development
cargo install cargo-expand       # Macro expansion debugging
cargo install cargo-criterion    # Performance benchmarking

# Tree-sitter CLI (for grammar development)
npm install -g tree-sitter-cli
```

### Project Structure

Understanding the codebase organization:

```
wabznasm/
├── Cargo.toml                 # Workspace configuration
├── Cargo.lock                 # Dependency lock file
├── build.rs                   # Build script for Tree-sitter
├── src/                       # Core language implementation
│   ├── lib.rs                 # Library root and public API
│   ├── main.rs                # CLI entry point
│   ├── environment.rs         # Environment and scoping system
│   ├── evaluator.rs           # Expression evaluation engine
│   ├── parser.rs              # Tree-sitter integration
│   ├── repl.rs                # Interactive REPL implementation
│   └── errors.rs              # Error types and handling
├── grammar/                   # Tree-sitter grammar definition
│   ├── grammar.js             # Grammar specification
│   └── src/                   # Generated parser code
├── tests/                     # Integration tests
│   ├── function_evaluation.rs # Function system tests
│   ├── function_parsing.rs    # Grammar tests
│   └── snapshots/             # Test snapshots
├── docs/                      # Documentation
│   ├── 01_architecture.md     # System architecture
│   ├── 02_language_semantics.md
│   ├── 03_implementation_details.md
│   └── 04_development_guide.md
└── fuzz/                      # Fuzzing infrastructure
    ├── Cargo.toml
    └── fuzz_targets/
```

### Development Workflow

#### Initial Setup

Clone and set up the development environment:

```bash
git clone https://github.com/your-username/wabznasm.git
cd wabznasm

# Build project and run tests
cargo build
cargo test

# Start development with file watching
cargo watch -x "check" -x "test"
```

#### Grammar Development

When modifying the Tree-sitter grammar:

```bash
cd grammar

# Generate parser from grammar.js
tree-sitter generate

# Test grammar with sample expressions
tree-sitter parse ../test_cases.wabz

# Return to project root and rebuild
cd ..
cargo build
```

#### Testing Workflow

Comprehensive testing during development:

```bash
# Run all tests
cargo test

# Run specific test module
cargo test function_evaluation

# Run tests with output
cargo test -- --nocapture

# Run tests in watch mode
cargo watch -x "test"

# Update test snapshots when needed
cargo test -- --update-snapshots
```

## Code Organization and Architecture

### Module Responsibilities

Each module has specific responsibilities:

#### `src/environment.rs`
- Environment and scoping implementation
- Variable binding and lookup
- Closure capture semantics
- Parameter binding for function calls

```rust
// Key types and functions
pub struct Environment { ... }
impl Environment {
    pub fn new() -> Self
    pub fn define(&mut self, name: String, value: Value)
    pub fn lookup(&self, name: &str) -> Option<&Value>
    pub fn extend(&self) -> Environment
    pub fn bind_parameters(&self, params: &[String], args: &[Value], node: Node) -> Result<Environment, EvalError>
}

pub enum Value {
    Integer(i64),
    Function { params: Vec<String>, body: String, closure: Option<Rc<Environment>> },
}
```

#### `src/evaluator.rs`
- AST evaluation engine
- Visitor pattern implementation
- Type checking and conversion
- Arithmetic operations

```rust
// Key types and functions
pub struct Evaluator;
impl Evaluator {
    pub fn eval_with_env<'a>(&self, node: Node<'a>, src: &str, env: &mut Environment) -> Result<Value, EvalError>
    pub fn eval<'a>(&self, node: Node<'a>, src: &str) -> Result<i64, EvalError>  // Legacy compatibility
}

pub fn evaluate_expression(input: &str) -> Result<i64, Report>  // Public API
```

#### `src/parser.rs`
- Tree-sitter integration
- AST generation and validation
- Error reporting for syntax errors

```rust
// Key functions
pub fn parse_expression(input: &str) -> Result<Tree, Report>
pub fn query_expression<T>(tree: &Tree, source: &str, evaluator: impl Fn(Node, &str) -> Result<T, EvalError>) -> Result<T, Report>
pub fn language() -> tree_sitter::Language
```

#### `src/repl.rs`
- Interactive command loop
- Persistent environment management
- User interface and error display

```rust
// Key function
pub fn run() -> Result<(), eyre::Report>
```

### Design Patterns

#### Visitor Pattern

The evaluator uses the visitor pattern to traverse AST nodes:

```rust
impl Evaluator {
    // Main dispatcher
    pub fn eval_with_env<'a>(&self, node: Node<'a>, src: &str, env: &mut Environment) -> Result<Value, EvalError> {
        match node.kind() {
            "number" => Ok(Value::Integer(self.visit_number_raw(node, src)?)),
            "identifier" => self.visit_identifier(node, src, env),
            "assignment" => self.visit_assignment(node, src, env),
            "function_call" => self.visit_function_call(node, src, env),
            // ... other node types
        }
    }

    // Specialized visitor methods
    fn visit_assignment(&self, node: Node, src: &str, env: &mut Environment) -> Result<Value, EvalError> { ... }
    fn visit_function_call(&self, node: Node, src: &str, env: &mut Environment) -> Result<Value, EvalError> { ... }
}
```

#### Error Propagation

Consistent error handling using `Result<T, E>`:

```rust
// Error propagation example
pub fn complex_operation(&self, node: Node, src: &str, env: &mut Environment) -> Result<Value, EvalError> {
    let child = self.named_child(node)?;  // Propagate parse errors
    let value = self.eval_with_env(child, src, env)?;  // Propagate evaluation errors

    match value {
        Value::Integer(n) if n > 0 => Ok(value),
        _ => Err(EvalError::new(EvalErrorKind::Other("Invalid value".into()), node)),
    }
}
```

## Testing and Quality Assurance

### Test Categories

The project maintains several categories of tests:

#### Unit Tests

Located within source modules using `#[cfg(test)]`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_basic_operations() {
        let mut env = Environment::new();
        env.define("x".to_string(), Value::Integer(42));
        assert_eq!(env.lookup("x"), Some(&Value::Integer(42)));
        assert_eq!(env.lookup("y"), None);
    }
}
```

#### Integration Tests

Located in `tests/` directory for end-to-end functionality:

```rust
// tests/function_evaluation.rs
use wabznasm::environment::{Environment, Value};
use wabznasm::evaluator::Evaluator;
use wabznasm::parser::parse_expression;

#[test]
fn test_function_call_with_args() {
    let mut env = Environment::new();
    let evaluator = Evaluator::new();

    // Define function
    let tree = parse_expression("add: {[x;y] x+y}").unwrap();
    evaluator.eval_with_env(tree.root_node(), "add: {[x;y] x+y}", &mut env).unwrap();

    // Call function
    let tree = parse_expression("add[2;3]").unwrap();
    let result = evaluator.eval_with_env(tree.root_node(), "add[2;3]", &mut env).unwrap();

    assert_eq!(result, Value::Integer(5));
}
```

#### Property-Based Tests

For grammar and evaluation robustness:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_arithmetic_associativity(a in -1000i64..1000, b in -1000i64..1000, c in -1000i64..1000) {
        let expr1 = format!("({} + {}) + {}", a, b, c);
        let expr2 = format!("{} + ({} + {})", a, b, c);

        let result1 = evaluate_expression(&expr1);
        let result2 = evaluate_expression(&expr2);

        // Both should succeed or both should fail
        prop_assert_eq!(result1.is_ok(), result2.is_ok());

        if let (Ok(r1), Ok(r2)) = (result1, result2) {
            prop_assert_eq!(r1, r2);
        }
    }
}
```

### Test Data Management

#### Snapshot Testing

For complex outputs that need verification:

```rust
#[test]
fn test_complex_expression_output() {
    let result = evaluate_complex_expression("(1+2)*3+4");
    insta::assert_debug_snapshot!(result);
}
```

#### Test Fixtures

Shared test data and utilities:

```rust
// tests/common/mod.rs
pub fn setup_test_environment() -> Environment {
    let mut env = Environment::new();
    env.define("x".to_string(), Value::Integer(10));
    env.define("y".to_string(), Value::Integer(20));
    env
}

pub fn parse_and_eval(expr: &str, env: &mut Environment) -> Result<Value, EvalError> {
    let tree = parse_expression(expr).unwrap();
    let evaluator = Evaluator::new();
    evaluator.eval_with_env(tree.root_node(), expr, env)
}
```

### Performance Testing

#### Benchmark Setup

Using Criterion for performance measurement:

```rust
// benches/evaluation.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use wabznasm::evaluator::evaluate_expression;

fn benchmark_arithmetic(c: &mut Criterion) {
    c.bench_function("simple addition", |b| {
        b.iter(|| evaluate_expression(black_box("1 + 2")))
    });

    c.bench_function("complex expression", |b| {
        b.iter(|| evaluate_expression(black_box("(1 + 2) * (3 + 4) + 5")))
    });
}

criterion_group!(benches, benchmark_arithmetic);
criterion_main!(benches);
```

#### Performance Regression Detection

CI integration for performance monitoring:

```yaml
# .github/workflows/performance.yml
name: Performance Tests
on: [push, pull_request]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v2
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
    - name: Run benchmarks
      run: cargo bench -- --output-format json | tee output.json
    - name: Store benchmark result
      uses: benchmark-action/github-action-benchmark@v1
      with:
        tool: 'cargo'
        output-file-path: output.json
```

## Debugging and Development Tools

### Debug Configuration

Development debugging setup:

```rust
// Enable debug logging
env_logger::init();

// Debug environment state
debug!("Environment bindings: {:?}", env.bindings);

// Trace function calls
trace!("Calling function {} with args {:?}", func_name, args);
```

### REPL Debugging

Interactive debugging in the REPL:

```bash
# Start REPL with debug logging
RUST_LOG=debug cargo run

# In REPL - inspect environment
\env     # Show current bindings (planned feature)
\debug   # Toggle debug mode (planned feature)
```

### AST Inspection

Debug Tree-sitter AST generation:

```rust
// Print AST structure
let tree = parse_expression("f: {[x;y] x+y}").unwrap();
println!("AST: {}", tree.root_node().to_sexp());

// Inspect specific nodes
let mut cursor = tree.walk();
for child in tree.root_node().named_children(&mut cursor) {
    println!("Node: {} at {}..{}", child.kind(), child.start_byte(), child.end_byte());
}
```

### Memory Profiling

Track memory usage during development:

```bash
# Install valgrind (Linux)
sudo apt-get install valgrind

# Profile memory usage
valgrind --tool=massif cargo run

# Analyze results
ms_print massif.out.*
```

## Code Style and Best Practices

### Rust Idioms

Follow Rust best practices throughout the codebase:

#### Error Handling

```rust
// Use ? for error propagation
fn complex_operation() -> Result<Value, EvalError> {
    let parsed = parse_expression(input)?;
    let evaluated = evaluator.eval(parsed.root_node(), input)?;
    Ok(evaluated)
}

// Provide context for errors
.map_err(|e| EvalError::new(
    EvalErrorKind::Other(format!("Parse failed: {}", e)),
    node
))?
```

#### Memory Management

```rust
// Prefer borrowing over cloning
fn process_value(value: &Value) -> bool {
    match value {
        Value::Integer(n) => *n > 0,
        Value::Function { .. } => true,
    }
}

// Use Rc for shared ownership
let shared_env = Rc::new(environment);
let closure = Some(shared_env.clone());
```

#### Pattern Matching

```rust
// Exhaustive matching
match node.kind() {
    "number" => handle_number(node),
    "identifier" => handle_identifier(node),
    "assignment" => handle_assignment(node),
    other => Err(EvalError::new(
        EvalErrorKind::Other(format!("Unexpected node: {}", other)),
        node
    )),
}
```

### Documentation Standards

#### Code Documentation

```rust
/// Evaluates a function call with the given arguments.
///
/// This method handles the complete function call process:
/// 1. Resolves the function identifier in the current environment
/// 2. Evaluates all arguments in the caller's environment
/// 3. Creates a new environment with parameter bindings
/// 4. Parses and evaluates the function body
///
/// # Arguments
/// * `node` - The function call AST node
/// * `src` - The source code string
/// * `env` - The current evaluation environment
///
/// # Returns
/// * `Ok(Value)` - The result of function evaluation
/// * `Err(EvalError)` - If function resolution, arity checking, or evaluation fails
///
/// # Examples
/// ```
/// let result = evaluator.visit_function_call(call_node, "add[2;3]", &mut env)?;
/// ```
fn visit_function_call(&self, node: Node, src: &str, env: &mut Environment) -> Result<Value, EvalError> {
    // Implementation...
}
```

#### Module Documentation

```rust
//! Environment and Scoping System
//!
//! This module implements lexical scoping for the wabznasm language.
//! It provides the core `Environment` type for variable binding and lookup,
//! along with support for closures and function call environments.
//!
//! # Key Concepts
//!
//! - **Lexical Scoping**: Inner scopes can access outer scope variables
//! - **Variable Shadowing**: Local definitions override parent scope bindings
//! - **Closures**: Functions capture their defining environment
//! - **Parameter Binding**: Function calls create new environments with argument bindings
```

### Performance Guidelines

#### Allocation Minimization

```rust
// Avoid unnecessary allocations
fn get_identifier_name<'a>(node: Node<'a>, src: &'a str) -> Result<&'a str, EvalError> {
    node.utf8_text(src.as_bytes())
        .map_err(|e| EvalError::new(EvalErrorKind::Other(e.to_string()), node))
}

// Reuse containers when possible
let mut args = Vec::with_capacity(arg_count);
for arg_node in argument_nodes {
    args.push(evaluate_argument(arg_node)?);
}
```

#### Early Returns

```rust
// Exit early for common cases
fn lookup_variable(&self, name: &str) -> Option<&Value> {
    // Check local bindings first (most common case)
    if let Some(value) = self.bindings.get(name) {
        return Some(value);
    }

    // Fall back to parent chain
    self.parent.as_ref()?.lookup(name)
}
```

## Contributing Guidelines

### Contribution Workflow

1. **Fork the Repository**: Create your own fork on GitHub
2. **Create Feature Branch**: `git checkout -b feature/new-functionality`
3. **Implement Changes**: Follow coding standards and add tests
4. **Run Test Suite**: Ensure all tests pass
5. **Submit Pull Request**: Include clear description and test evidence

### Pull Request Requirements

#### Code Quality Checklist

- [ ] All tests pass (`cargo test`)
- [ ] No compiler warnings (`cargo clippy`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] Documentation is updated
- [ ] New features include tests
- [ ] Performance impact is considered

#### Commit Message Format

```
type(scope): brief description

Detailed explanation of changes, motivation, and impact.
Include any breaking changes or migration notes.

Fixes #123
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`

### Feature Development Process

#### Adding New Language Features

1. **Grammar Updates**: Modify `grammar/grammar.js` if needed
2. **AST Handling**: Add new node type processing in evaluator
3. **Value Types**: Extend `Value` enum if new types are needed
4. **Error Handling**: Add appropriate error cases
5. **Testing**: Comprehensive test coverage
6. **Documentation**: Update language documentation

#### Example: Adding String Type

```rust
// 1. Extend Value enum
pub enum Value {
    Integer(i64),
    String(String),  // New type
    Function { params: Vec<String>, body: String, closure: Option<Rc<Environment>> },
}

// 2. Add evaluator support
match node.kind() {
    "string_literal" => Ok(Value::String(self.visit_string(node, src)?)),
    // ... other cases
}

// 3. Add operations
fn visit_binary_raw(&self, node: Node, src: &str, env: &mut Environment) -> Result<Value, EvalError> {
    match (left_val, right_val, op) {
        (Value::String(s1), Value::String(s2), "+") => Ok(Value::String(format!("{}{}", s1, s2))),
        // ... other cases
    }
}

// 4. Add tests
#[test]
fn test_string_concatenation() {
    let result = evaluate_expression(r#""hello" + "world""#).unwrap();
    assert_eq!(result, Value::String("helloworld".to_string()));
}
```

### Documentation Contributions

#### Adding Examples

```rust
/// # Examples
///
/// Basic arithmetic:
/// ```
/// # use wabznasm::evaluator::evaluate_expression;
/// let result = evaluate_expression("2 + 3").unwrap();
/// assert_eq!(result, 5);
/// ```
///
/// Function definition and call:
/// ```
/// # use wabznasm::{Environment, Evaluator, parse_expression};
/// let mut env = Environment::new();
/// let evaluator = Evaluator::new();
///
/// // Define function
/// let tree = parse_expression("double: {x * 2}").unwrap();
/// evaluator.eval_with_env(tree.root_node(), "double: {x * 2}", &mut env).unwrap();
///
/// // Call function
/// let tree = parse_expression("double[5]").unwrap();
/// let result = evaluator.eval_with_env(tree.root_node(), "double[5]", &mut env).unwrap();
/// assert_eq!(result, Value::Integer(10));
/// ```
```

#### Updating Reference Documentation

- Keep examples current with API changes
- Add new features to reference card
- Update architecture documentation for major changes
- Include performance characteristics for new features

## Release and Deployment

### Version Management

Follow semantic versioning (semver):

- **Major** (1.0.0): Breaking changes to public API
- **Minor** (0.1.0): New features, backward compatible
- **Patch** (0.0.1): Bug fixes, no feature changes

### Release Checklist

1. **Update Version**: Modify `Cargo.toml` version numbers
2. **Update Changelog**: Document all changes since last release
3. **Run Full Test Suite**: Ensure all tests pass
4. **Performance Validation**: Run benchmarks to check for regressions
5. **Documentation Review**: Verify all documentation is current
6. **Create Release Tag**: `git tag v0.1.0 && git push origin v0.1.0`
7. **Publish Crate**: `cargo publish` (when appropriate)

### Continuous Integration

The project uses GitHub Actions for CI:

```yaml
name: CI
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v2
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        components: rustfmt, clippy
    - name: Format check
      run: cargo fmt -- --check
    - name: Lint check
      run: cargo clippy -- -D warnings
    - name: Run tests
      run: cargo test
    - name: Run benchmarks
      run: cargo bench
```

This ensures code quality and prevents regressions across all contributions.
