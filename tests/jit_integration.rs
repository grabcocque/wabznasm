//! Integration tests for JIT functionality in wabznasm
use wabznasm::evaluator::evaluate_expression;

/// Test simple addition via JIT
#[test]
fn test_jit_addition() {
    assert_eq!(evaluate_expression("1+2").unwrap(), 3);
}

/// Test operator precedence and multiplicative
#[test]
fn test_jit_precedence_and_mul() {
    assert_eq!(evaluate_expression("2*3+4").unwrap(), 10);
    assert_eq!(evaluate_expression("2+3*4").unwrap(), 14);
}

/// Test parentheses handling
#[test]
fn test_jit_parentheses() {
    assert_eq!(evaluate_expression("2*(3+4)").unwrap(), 14);
}

/// Test unary operations
#[test]
fn test_jit_unary() {
    assert_eq!(evaluate_expression("-5").unwrap(), -5);
    assert_eq!(evaluate_expression("--5").unwrap(), 5);
}

/// Test factorial via interpreter fallback
#[test]
fn test_jit_factorial() {
    assert_eq!(evaluate_expression("3!").unwrap(), 6);
    assert_eq!(evaluate_expression("3!!").unwrap(), 720);
}

/// Test power operator via interpreter fallback
#[test]
fn test_jit_power() {
    assert_eq!(evaluate_expression("2^3").unwrap(), 8);
    assert_eq!(evaluate_expression("2^3^2").unwrap(), 512);
}

/// Test JIT error on division by zero
#[test]
fn test_jit_error_division_by_zero() {
    let err = evaluate_expression("1/0").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("Division by zero"));
}
// Test modulo operator
#[test]
fn test_jit_modulo() {
    assert_eq!(evaluate_expression("5%2").unwrap(), 1);
    assert_eq!(evaluate_expression("2+5%3").unwrap(), 2 + 5 % 3);
    assert_eq!(evaluate_expression("10%3*2").unwrap(), 2);
}

// Test JIT error on modulo by zero
#[test]
fn test_jit_error_modulo_by_zero() {
    let err = evaluate_expression("5%0").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("Division by zero"));
}

/// Test JIT error on syntax error
#[test]
fn test_jit_error_syntax() {
    // Empty input triggers syntax error
    let err = evaluate_expression("").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("Syntax error in expression"));
}

/// Test negative exponent error via interpreter fallback
#[test]
fn test_jit_negative_exponent() {
    let err = evaluate_expression("2^-1").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("Negative exponent"));
}

/// Test exponent too large error via interpreter fallback
#[test]
fn test_jit_exponent_too_large() {
    let err = evaluate_expression("2^64").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("Exponent too large"));
}

/// Test factorial too large error via interpreter fallback
#[test]
fn test_jit_factorial_too_large() {
    let err = evaluate_expression("21!").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("Factorial too large"));
}

/// Test missing operand syntax error
#[test]
fn test_jit_missing_operand() {
    let err = evaluate_expression("1+").unwrap_err();
    let msg = format!("{}", err);
    // Missing operand yields syntax error
    assert!(msg.contains("Syntax error in expression"));
}

/// Test invalid token syntax error
#[test]
fn test_jit_invalid_token() {
    let err = evaluate_expression("@#$").unwrap_err();
    let msg = format!("{}", err);
    // Invalid token yields syntax error
    assert!(msg.contains("Syntax error in expression"));
}

/// Test a more complex expression via JIT
#[test]
fn test_jit_complex_expression() {
    assert_eq!(evaluate_expression("2 * (3 + 4) + 5").unwrap(), 19);
}

/// Test whitespace handling in JIT path
#[test]
fn test_jit_whitespace() {
    assert_eq!(evaluate_expression("   1    +     2   ").unwrap(), 3);
}

/// Test binary operations through the table-driven implementation
#[test]
fn test_table_driven_binary_ops() {
    // Multiplication
    assert_eq!(evaluate_expression("4*5").unwrap(), 20);

    // Division
    assert_eq!(evaluate_expression("10/2").unwrap(), 5);

    // Modulo
    assert_eq!(evaluate_expression("10%3").unwrap(), 1);

    // Subtraction
    assert_eq!(evaluate_expression("7-2").unwrap(), 5);

    // Addition
    assert_eq!(evaluate_expression("7+8").unwrap(), 15);
}

/// Test division by zero error detection in table-driven implementation
#[test]
fn test_table_driven_div_by_zero() {
    // Test with constant zero
    let err = evaluate_expression("42/0").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("Division by zero"));

    // Test with computed zero
    let err = evaluate_expression("42/(3-3)").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("Division by zero"));
}

/// Test modulo by zero error detection in table-driven implementation
#[test]
fn test_table_driven_mod_by_zero() {
    // Test with constant zero
    let err = evaluate_expression("42%0").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("Division by zero"));

    // Test with computed zero
    let err = evaluate_expression("42%(2-2)").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("Division by zero"));
}
