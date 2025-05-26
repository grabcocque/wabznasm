//! Tests for the wabznasm expression evaluator.
// EvalError is not directly asserted by type in these tests, relying on string messages.
// If specific error kinds need to be asserted, import wabznasm::errors::EvalErrorKind;
use wabznasm::evaluator::evaluate_expression; // Use the standalone function

/// Test simple addition
#[test]
fn test_simple_addition() {
    assert_eq!(evaluate_expression("1+2").unwrap(), 3);
    assert_eq!(evaluate_expression("10+5").unwrap(), 15);
    assert_eq!(evaluate_expression("100+200").unwrap(), 300);
}

/// Test subtraction
#[test]
fn test_simple_subtraction() {
    assert_eq!(evaluate_expression("5-2").unwrap(), 3);
    assert_eq!(evaluate_expression("10-7").unwrap(), 3);
    assert_eq!(evaluate_expression("0-5").unwrap(), -5);
}

/// Test multiplication
#[test]
fn test_multiplication() {
    assert_eq!(evaluate_expression("3*4").unwrap(), 12);
    assert_eq!(evaluate_expression("7*8").unwrap(), 56);
    assert_eq!(evaluate_expression("0*100").unwrap(), 0);
}

/// Test division
#[test]
fn test_division() {
    assert_eq!(evaluate_expression("10/2").unwrap(), 5);
    assert_eq!(evaluate_expression("15/3").unwrap(), 5);
    assert_eq!(evaluate_expression("20/4").unwrap(), 5);
}

/// Test modulo
#[test]
fn test_modulo() {
    assert_eq!(evaluate_expression("5%2").unwrap(), 1);
    assert_eq!(evaluate_expression("10%3").unwrap(), 1);
    assert_eq!(evaluate_expression("7%4").unwrap(), 3);
}

/// Test modulo operations combined with other operators
#[test]
fn test_modulo_with_other_ops() {
    assert_eq!(evaluate_expression("2+5%3").unwrap(), 4); // 2 + (5 % 3) = 2 + 2 = 4
    assert_eq!(evaluate_expression("10%3*2").unwrap(), 2); // (10 % 3) * 2 = 1 * 2 = 2
}

/// Test unary negation
#[test]
fn test_unary_negation() {
    assert_eq!(evaluate_expression("-5").unwrap(), -5);
    assert_eq!(evaluate_expression("-(-3)").unwrap(), 3);
    assert_eq!(evaluate_expression("--5").unwrap(), 5);
}

/// Test operator precedence
#[test]
fn test_precedence() {
    assert_eq!(evaluate_expression("2*3+4").unwrap(), 10);
    assert_eq!(evaluate_expression("2+3*4").unwrap(), 14);
    assert_eq!(evaluate_expression("10-2*3").unwrap(), 4);
    assert_eq!(evaluate_expression("20/4+2").unwrap(), 7);
}

/// Test parentheses
#[test]
fn test_parentheses() {
    assert_eq!(evaluate_expression("2*(3+4)").unwrap(), 14);
    assert_eq!(evaluate_expression("(5-2)*3").unwrap(), 9);
    assert_eq!(evaluate_expression("10/(2+3)").unwrap(), 2);
}

/// Test power operations
#[test]
fn test_power() {
    assert_eq!(evaluate_expression("2^3").unwrap(), 8);
    assert_eq!(evaluate_expression("3^2").unwrap(), 9);
    assert_eq!(evaluate_expression("5^0").unwrap(), 1);
    assert_eq!(evaluate_expression("1^10").unwrap(), 1);
}

/// Test power operator precedence
#[test]
fn test_power_precedence() {
    assert_eq!(evaluate_expression("2^3^2").unwrap(), 512); // Right associative: 2^(3^2) = 2^9 = 512
    assert_eq!(evaluate_expression("2*3^2").unwrap(), 18); // Power higher precedence: 2*(3^2) = 2*9 = 18
}

/// Test factorial operations
#[test]
fn test_factorial() {
    assert_eq!(evaluate_expression("0!").unwrap(), 1);
    assert_eq!(evaluate_expression("1!").unwrap(), 1);
    assert_eq!(evaluate_expression("3!").unwrap(), 6);
    assert_eq!(evaluate_expression("4!").unwrap(), 24);
    assert_eq!(evaluate_expression("5!").unwrap(), 120);
}

/// Test multiple factorial operations
#[test]
fn test_multiple_factorials() {
    assert_eq!(evaluate_expression("3!!").unwrap(), 720); // (3!)! = 6! = 720
    assert_eq!(evaluate_expression("2!!!").unwrap(), 2); // ((2!)!)! = (2!)! = 2! = 2 (left associative)
}

/// Test complex expressions combining multiple operations
#[test]
fn test_complex_expressions() {
    assert_eq!(evaluate_expression("2 * (3 + 4) + 5").unwrap(), 19);
    assert_eq!(evaluate_expression("(10 - 5) * 2 + 3").unwrap(), 13);
    assert_eq!(evaluate_expression("2^3 + 3!").unwrap(), 14); // 8 + 6 = 14
    assert_eq!(evaluate_expression("4! / 2^3").unwrap(), 3); // 24 / 8 = 3
}

/// Test whitespace handling
#[test]
fn test_whitespace() {
    assert_eq!(evaluate_expression("   1    +     2   ").unwrap(), 3);
    assert_eq!(evaluate_expression("\t5\n*\r3\t").unwrap(), 15);
}

/// Test constants and literals
#[test]
fn test_constants() {
    assert_eq!(evaluate_expression("42").unwrap(), 42);
    assert_eq!(evaluate_expression("0").unwrap(), 0);
    assert_eq!(evaluate_expression("999").unwrap(), 999);
}

// Error handling tests

/// Test division by zero error detection
#[test]
fn test_division_by_zero() {
    let err = evaluate_expression("1/0").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("Division by zero"));

    // Test with computed zero divisor
    let err = evaluate_expression("42/(3-3)").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("Division by zero"));
}

/// Test modulo by zero error detection
#[test]
fn test_modulo_by_zero() {
    let err = evaluate_expression("5%0").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("Division by zero")); // Modulo by zero uses same error as division by zero

    // Test with computed zero divisor
    let err = evaluate_expression("42%(2-2)").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("Division by zero")); // Runtime detection also uses same error message
}

/// Test negative exponent error
#[test]
fn test_negative_exponent() {
    let err = evaluate_expression("2^-1").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("Negative exponent"));

    let err = evaluate_expression("5^(-2)").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("Negative exponent"));
}

/// Test exponent too large error
#[test]
fn test_exponent_too_large() {
    let err = evaluate_expression("2^64").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("Exponent too large"));

    let err = evaluate_expression("2^100").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("Exponent too large"));
}

/// Test factorial of negative number error
#[test]
fn test_factorial_negative() {
    let err = evaluate_expression("(-1)!").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("Factorial of negative"));

    let err = evaluate_expression("(-5)!").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("Factorial of negative"));
}

/// Test factorial too large error
#[test]
fn test_factorial_too_large() {
    let err = evaluate_expression("21!").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("Factorial too large"));

    let err = evaluate_expression("25!").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("Factorial too large"));
}

/// Test syntax errors
#[test]
fn test_syntax_errors() {
    // Empty input
    let err = evaluate_expression("").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("Syntax error in expression"));

    // Missing operand
    let err = evaluate_expression("1+").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("Syntax error in expression"));

    // Invalid token
    let err = evaluate_expression("@#$").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("Syntax error in expression"));
}

// Edge cases and regression tests

/// Test edge cases for power operations
#[test]
fn test_power_edge_cases() {
    assert_eq!(evaluate_expression("0^0").unwrap(), 1); // Mathematical convention: 0^0 = 1
    assert_eq!(evaluate_expression("0^5").unwrap(), 0);
    assert_eq!(evaluate_expression("(-2)^3").unwrap(), -8);
    assert_eq!(evaluate_expression("(-2)^2").unwrap(), 4);
}

/// Test edge cases for factorial operations
#[test]
fn test_factorial_edge_cases() {
    assert_eq!(evaluate_expression("0!").unwrap(), 1); // 0! = 1 by definition
    assert_eq!(evaluate_expression("1!").unwrap(), 1);
    assert_eq!(evaluate_expression("20!").unwrap(), 2432902008176640000); // 20! is within bounds
}

/// Test deeply nested expressions
#[test]
fn test_nested_expressions() {
    assert_eq!(evaluate_expression("((((1+2)*3)+4)*5)").unwrap(), 65);
    assert_eq!(evaluate_expression("2^(3^(2))").unwrap(), 512); // 2^(3^2) = 2^9 = 512
}

/// Test mixed operation types
#[test]
fn test_mixed_operations() {
    assert_eq!(evaluate_expression("2^3 * 4! - 5").unwrap(), 187); // 8 * 24 - 5 = 192 - 5 = 187
    assert_eq!(evaluate_expression("3! + 2^2 - 1").unwrap(), 9); // 6 + 4 - 1 = 9
    assert_eq!(evaluate_expression("(2+3)! / 5^2").unwrap(), 4); // 5! / 25 = 120 / 25 = 4
}

/// Test associativity rules
#[test]
fn test_associativity() {
    // Left associativity for binary operators
    assert_eq!(evaluate_expression("10-5-2").unwrap(), 3); // (10-5)-2 = 5-2 = 3
    assert_eq!(evaluate_expression("20/4/2").unwrap(), 2); // (20/4)/2 = 5/2 = 2

    // Right associativity for power operator
    assert_eq!(evaluate_expression("2^3^2").unwrap(), 512); // 2^(3^2) = 2^9 = 512
}

// Tests from numeric.rs (originally table_driven)
// These might represent a different evaluation path or specific aspect.
// Renaming to test_evaluator_table_driven_* to keep that context if significant.

/// Test binary operations through the table-driven implementation
#[test]
fn test_evaluator_table_driven_binary_ops() {
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
fn test_evaluator_table_driven_div_by_zero() {
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
fn test_evaluator_table_driven_mod_by_zero() {
    // Test with constant zero
    let err = evaluate_expression("42%0").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("Division by zero"));

    // Test with computed zero
    let err = evaluate_expression("42%(2-2)").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("Division by zero"));
}
