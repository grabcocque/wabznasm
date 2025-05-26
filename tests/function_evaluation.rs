use wabznasm::environment::{Environment, Value};
use wabznasm::evaluator::Evaluator;
use wabznasm::parser::parse_expression;

#[test]
fn test_simple_assignment() {
    let mut env = Environment::new();
    let mut evaluator = Evaluator::new();

    // Test: x: 42
    let tree = parse_expression("x: 42").unwrap();
    let result = evaluator
        .eval_with_env(tree.root_node(), "x: 42", &mut env)
        .unwrap();

    assert_eq!(result, Value::Integer(42));

    // Test that we can retrieve x
    let tree = parse_expression("x").unwrap();
    let result = evaluator
        .eval_with_env(tree.root_node(), "x", &mut env)
        .unwrap();
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_function_definition() {
    let mut env = Environment::new();
    let mut evaluator = Evaluator::new();

    // Test: f: {x+1}
    let tree = parse_expression("f: {x+1}").unwrap();
    let result = evaluator
        .eval_with_env(tree.root_node(), "f: {x+1}", &mut env)
        .unwrap();

    match result {
        Value::Function { .. } => {
            // Function should be stored and callable
            assert!(result.is_function());
        }
        _ => panic!("Expected function value"),
    }
}

#[test]
fn test_function_definition_with_params() {
    let mut env = Environment::new();
    let mut evaluator = Evaluator::new();

    // Test: add: {[x;y] x+y}
    let tree = parse_expression("add: {[x;y] x+y}").unwrap();
    let result = evaluator
        .eval_with_env(tree.root_node(), "add: {[x;y] x+y}", &mut env)
        .unwrap();

    match result {
        Value::Function { .. } => {
            assert!(result.is_function());
        }
        _ => panic!("Expected function value"),
    }
}

#[test]
fn test_identifier_lookup() {
    let mut env = Environment::new();
    let mut evaluator = Evaluator::new();

    // Set up a variable by assignment
    let tree = parse_expression("x: 42").unwrap();
    evaluator
        .eval_with_env(tree.root_node(), "x: 42", &mut env)
        .unwrap();

    // Test: x
    let tree = parse_expression("x").unwrap();
    let result = evaluator
        .eval_with_env(tree.root_node(), "x", &mut env)
        .unwrap();

    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_simple_function_call() {
    let mut env = Environment::new();
    let mut evaluator = Evaluator::new();

    // Set up variable x in environment first
    let tree = parse_expression("x: 5").unwrap();
    evaluator
        .eval_with_env(tree.root_node(), "x: 5", &mut env)
        .unwrap();

    // Define function: f: {x+1} (captures x from closure)
    let tree = parse_expression("f: {x+1}").unwrap();
    evaluator
        .eval_with_env(tree.root_node(), "f: {x+1}", &mut env)
        .unwrap();

    // Call function: f[]
    let tree = parse_expression("f[]").unwrap();
    let result = evaluator
        .eval_with_env(tree.root_node(), "f[]", &mut env)
        .unwrap();

    assert_eq!(result, Value::Integer(6)); // x+1 = 5+1 = 6
}

#[test]
fn test_function_call_with_args() {
    let mut env = Environment::new();
    let mut evaluator = Evaluator::new();

    // Define function: add: {[x;y] x+y}
    let tree = parse_expression("add: {[x;y] x+y}").unwrap();
    evaluator
        .eval_with_env(tree.root_node(), "add: {[x;y] x+y}", &mut env)
        .unwrap();

    // Call function: add[2;3]
    let tree = parse_expression("add[2;3]").unwrap();
    let result = evaluator
        .eval_with_env(tree.root_node(), "add[2;3]", &mut env)
        .unwrap();

    assert_eq!(result, Value::Integer(5)); // 2+3 = 5
}
