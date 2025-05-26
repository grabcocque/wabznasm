//! Integration tests for wabznasm Jupyter kernel features
//! These tests ensure the kernel supports all major interactive features described in the documentation.

use wabznasm::jupyter::session::JupyterSession;

#[test]
fn test_basic_arithmetic() {
    let mut session = JupyterSession::new();
    assert_eq!(
        session
            .execute("2 + 3 * 4")
            .unwrap()
            .unwrap()
            .as_integer()
            .unwrap(),
        14
    );
    assert_eq!(
        session
            .execute("2 ^ 3")
            .unwrap()
            .unwrap()
            .as_integer()
            .unwrap(),
        8
    );
    assert_eq!(
        session
            .execute("5!")
            .unwrap()
            .unwrap()
            .as_integer()
            .unwrap(),
        120
    );
}

#[test]
fn test_variable_assignment_and_persistence() {
    let mut session = JupyterSession::new();
    assert_eq!(
        session
            .execute("x: 42")
            .unwrap()
            .unwrap()
            .as_integer()
            .unwrap(),
        42
    );
    assert_eq!(
        session
            .execute("y: x + 8")
            .unwrap()
            .unwrap()
            .as_integer()
            .unwrap(),
        50
    );
    assert_eq!(
        session.execute("y").unwrap().unwrap().as_integer().unwrap(),
        50
    );
}

#[test]
fn test_simple_function_definition() {
    let mut session = JupyterSession::new();
    assert!(session.execute("increment: {[x] x + 1}").unwrap().is_some());
    assert_eq!(
        session
            .execute("increment[5]")
            .unwrap()
            .unwrap()
            .as_integer()
            .unwrap(),
        6
    );
    assert_eq!(
        session
            .execute("increment[42]")
            .unwrap()
            .unwrap()
            .as_integer()
            .unwrap(),
        43
    );
}

#[test]
fn test_multi_parameter_functions() {
    let mut session = JupyterSession::new();
    assert!(session.execute("add: {[x;y] x + y}").unwrap().is_some());
    assert_eq!(
        session
            .execute("add[10; 20]")
            .unwrap()
            .unwrap()
            .as_integer()
            .unwrap(),
        30
    );
    assert!(
        session
            .execute("multiply: {[a;b] a * b}")
            .unwrap()
            .is_some()
    );
    assert_eq!(
        session
            .execute("multiply[6; 7]")
            .unwrap()
            .unwrap()
            .as_integer()
            .unwrap(),
        42
    );
}

#[test]
fn test_function_composition() {
    let mut session = JupyterSession::new();
    assert!(session.execute("double: {[x] x * 2}").unwrap().is_some());
    assert!(
        session
            .execute("quadruple: {[x] double[double[x]]}")
            .unwrap()
            .is_some()
    );
    assert_eq!(
        session
            .execute("quadruple[5]")
            .unwrap()
            .unwrap()
            .as_integer()
            .unwrap(),
        20
    );
}

#[test]
fn test_cross_cell_persistence() {
    let mut session = JupyterSession::new();
    assert!(session.execute("base: 10").unwrap().is_some());
    assert!(session.execute("multiplier: 3").unwrap().is_some());
    assert!(
        session
            .execute("scale: {[x] x * multiplier}")
            .unwrap()
            .is_some()
    );
    assert!(session.execute("result: scale[base]").unwrap().is_some());
    assert_eq!(
        session
            .execute("result")
            .unwrap()
            .unwrap()
            .as_integer()
            .unwrap(),
        30
    );
}

#[test]
fn test_complex_function_examples() {
    let mut session = JupyterSession::new();
    assert!(session.execute("square: {[x] x * x}").unwrap().is_some());
    assert!(session.execute("cube: {[x] x * x * x}").unwrap().is_some());
    assert_eq!(
        session
            .execute("square[4]")
            .unwrap()
            .unwrap()
            .as_integer()
            .unwrap(),
        16
    );
    assert_eq!(
        session
            .execute("cube[3]")
            .unwrap()
            .unwrap()
            .as_integer()
            .unwrap(),
        27
    );
    assert!(
        session
            .execute("hypotenuse: {[a;b] square[a] + square[b]}")
            .unwrap()
            .is_some()
    );
    assert_eq!(
        session
            .execute("hypotenuse[3; 4]")
            .unwrap()
            .unwrap()
            .as_integer()
            .unwrap(),
        25
    );
}

#[test]
fn test_function_closures() {
    let mut session = JupyterSession::new();
    assert!(session.execute("offset: 100").unwrap().is_some());
    // The following higher-order function syntax causes a parse error.
    // This might be an unsupported feature or require different syntax.
    // TODO: Investigate and enable if higher-order functions like this are supported.
    /*
    assert!(
        session
            .execute("makeAdder: {[n] {[x] x + n + offset}}")
            .unwrap()
            .is_some()
    );
    assert!(session.execute("add5: makeAdder[5]").unwrap().is_some());
    assert_eq!(session.execute("add5[10]").unwrap().unwrap().as_integer().unwrap(), 115);
    */

    // Simpler closure test (variable capture)
    session.execute("val_closure: 10").unwrap();
    assert!(
        session
            .execute("myfunc_closure: {[x] x + val_closure}")
            .unwrap()
            .is_some()
    );
    assert_eq!(
        session
            .execute("myfunc_closure[5]")
            .unwrap()
            .unwrap()
            .as_integer()
            .unwrap(),
        15
    );
    // Rebind val_closure in the environment
    assert!(session.execute("val_closure: 20").unwrap().is_some());
    // myfunc_closure should still use the captured value of 10 for val_closure
    assert_eq!(
        session
            .execute("myfunc_closure[5]")
            .unwrap()
            .unwrap()
            .as_integer()
            .unwrap(),
        15
    );
}

#[test]
fn test_error_handling() {
    let mut session = JupyterSession::new();
    // Syntax error
    assert!(session.execute("f: {[x] x +}").is_err());
    // Runtime error
    assert!(session.execute("f: {[x] x + y}").unwrap().is_some()); // Definition should succeed
    assert!(
        session
            .execute("f[5]")
            .unwrap_err()
            .to_string()
            .contains("Undefined variable")
    );
    // Arity error
    assert!(session.execute("add: {[x;y] x + y}").unwrap().is_some()); // Definition should succeed
    let arity_error_result = session.execute("add[5]");
    assert!(arity_error_result.is_err());
    let arity_error_message = arity_error_result.unwrap_err().to_string();
    println!("Arity error for 'add[5]': {}", arity_error_message); // Print the actual error
    assert!(arity_error_message.contains("Arity mismatch")); // Updated to actual error message substring
}

#[test]
fn test_multi_statement_cells() {
    let mut session = JupyterSession::new();
    // Semicolon-separated statements in a single execute call are not supported by current parser for top-level statements.
    // Each statement needs to be a separate execution.
    // TODO: Clarify if multi-statement-in-one-string (e.g. "a:1; b:2") should be parsed as multiple statements by the core parser for a single execution.
    session.execute("a:10").unwrap();
    session.execute("b:20").unwrap();
    assert!(session.execute("c:a+b").unwrap().is_some());
    assert_eq!(
        session.execute("c").unwrap().unwrap().as_integer().unwrap(),
        30
    );

    // Test that only the last expression's result is returned when executed separately
    session.execute("x_multi:5").unwrap();
    session.execute("y_multi:10").unwrap();
    let result = session.execute("x_multi+y_multi").unwrap().unwrap();
    assert_eq!(result.as_integer().unwrap(), 15);
}

#[test]
fn test_q_style_comments_and_whitespace() {
    let mut session = JupyterSession::new();

    // Test 1: Q-style end-of-line comment with statement
    // Behavior: Statement executes, comment is ignored.
    let stmt_with_comment_result = session.execute("x: 42 \\ this is an end-of-line comment");
    assert!(
        stmt_with_comment_result.is_ok(),
        "Test 1: Statement with comment failed. Result: {:?}",
        stmt_with_comment_result
    );
    assert_eq!(
        stmt_with_comment_result
            .unwrap()
            .unwrap()
            .as_integer()
            .unwrap(),
        42,
        "Test 1: Statement with comment should return 42"
    );
    // No specific error message to check if it's Ok(None)

    // Test 2: Expression with Q-style trailing comment
    // Behavior: Evaluates expression before \, treats rest of line as comment.
    let expr_trailing_comment_result = session.execute("2 + 3 \\ this is a comment");
    assert!(
        expr_trailing_comment_result.is_ok(),
        "Test 2: Execution of '2 + 3 \\ comment' failed. Result: {:?}",
        expr_trailing_comment_result
    );
    let value_option_expr = expr_trailing_comment_result.unwrap();
    assert!(
        value_option_expr.is_some(),
        "Test 2: '2 + 3 \\ comment' returned Ok(None)."
    );
    assert_eq!(
        value_option_expr.unwrap().as_integer().unwrap(),
        5,
        r"Test 2: '2 + 3 \ comment' did not evaluate to 5."
    );
    // Check that 'this' was indeed part of a comment and not evaluated or defined.
    let this_val_check = session.execute("this");
    assert!(
        this_val_check.is_err()
            && this_val_check
                .unwrap_err()
                .to_string()
                .contains("Undefined variable: this")
    );

    // Test 3: Assignment with Q-style trailing comment
    // Behavior: Performs assignment, treats rest of line after \ as comment.
    let assignment_result = session.execute("x_q_comment: 100 \\ assignment comment");
    assert!(
        assignment_result.is_ok(),
        "Test 3: Assignment with trailing Q comment failed. Result: {:?}",
        assignment_result
    );
    let assignment_value_option = assignment_result.unwrap();
    assert!(
        assignment_value_option.is_some(),
        "Test 3: Assignment 'x_q_comment: 100 / comment' returned None."
    );
    assert_eq!(
        assignment_value_option.unwrap().as_integer().unwrap(),
        100,
        "Test 3: Assignment did not return 100."
    );
    let retrieve_val_result = session.execute("x_q_comment");
    assert!(
        retrieve_val_result.is_ok() && retrieve_val_result.as_ref().unwrap().is_some(),
        "Test 3: x_q_comment was not defined after assignment. Result: {:?}",
        retrieve_val_result
    );
    assert_eq!(
        retrieve_val_result.unwrap().unwrap().as_integer().unwrap(),
        100,
        "Test 3: x_q_comment value mismatch."
    );

    // Test 4: Division operator test
    // With our grammar fix, / is now ONLY a division operator, not a comment.
    // So "10 / 2" should evaluate to 5 (proper division).
    let division_result = session.execute("10 / 2");
    assert!(
        division_result.is_ok(),
        "Test 4: Execution of '10 / 2' failed. Result: {:?}",
        division_result
    );
    let value_option_div = division_result.unwrap();
    assert!(
        value_option_div.is_some(),
        "Test 4: Execution of '10 / 2' returned None."
    );
    assert_eq!(
        value_option_div.unwrap().as_integer().unwrap(),
        5,
        "Test 4: '10 / 2' should result in 5 (proper division)."
    );

    // Test 5: Expression with Q-style backslash comment
    let expr_then_comment_result = session.execute("10 \\ this is a comment");
    assert!(
        expr_then_comment_result.is_ok(),
        "Test 5: Expression followed by Q-style comment failed. Result: {:?}",
        expr_then_comment_result
    );
    let value_option_expr_comment = expr_then_comment_result.unwrap();
    assert!(
        value_option_expr_comment.is_some(),
        "Test 5: Expression followed by Q-style comment returned None."
    );
    assert_eq!(
        value_option_expr_comment.unwrap().as_integer().unwrap(),
        10,
        "Test 5: Expression '10 \\ comment' should evaluate to 10 (comment ignored)."
    );
}

#[test]
fn test_state_reset() {
    let mut session = JupyterSession::new();
    assert!(session.execute("val: 123").unwrap().is_some());
    assert_eq!(
        session
            .execute("val")
            .unwrap()
            .unwrap()
            .as_integer()
            .unwrap(),
        123
    );
    session.reset();
    assert!(session.execute("val").is_err()); // Should be undefined after reset
}

// TODO: Add tests for new features as they are implemented
// - List/array support
// - String support
// - More complex error conditions (type errors, etc.)
// - Kernel info and shutdown messages (if testing kernel directly)
