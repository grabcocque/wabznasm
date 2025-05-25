use wabznasm::environment::{Environment, Value};
use wabznasm::evaluator::Evaluator;
use wabznasm::parser::parse_expression;

#[test]
fn test_simple_assignment() {
    let mut env = Environment::new();
    let evaluator = Evaluator::new();

    // Test: x: 42
    let tree = parse_expression("x: 42").unwrap();
    let result = evaluator
        .eval_with_env(tree.root_node(), "x: 42", &mut env)
        .unwrap();

    assert_eq!(result, Value::Integer(42));
    assert_eq!(env.lookup("x"), Some(&Value::Integer(42)));
}

#[test]
fn test_function_definition() {
    let mut env = Environment::new();
    let evaluator = Evaluator::new();

    // Test: f: {x+1}
    let tree = parse_expression("f: {x+1}").unwrap();
    let result = evaluator
        .eval_with_env(tree.root_node(), "f: {x+1}", &mut env)
        .unwrap();

    match result {
        Value::Function { params, body, .. } => {
            assert_eq!(params, Vec::<String>::new()); // No explicit params
            assert_eq!(body, "x+1");
        }
        _ => panic!("Expected function value"),
    }

    // Function should be stored in environment
    assert!(env.lookup("f").is_some());
    assert!(env.lookup("f").unwrap().is_function());
}

#[test]
fn test_function_definition_with_params() {
    let mut env = Environment::new();
    let evaluator = Evaluator::new();

    // Test: add: {[x;y] x+y}
    let tree = parse_expression("add: {[x;y] x+y}").unwrap();
    let result = evaluator
        .eval_with_env(tree.root_node(), "add: {[x;y] x+y}", &mut env)
        .unwrap();

    match result {
        Value::Function { params, body, .. } => {
            assert_eq!(params, vec!["x".to_string(), "y".to_string()]);
            assert_eq!(body, "x+y");
        }
        _ => panic!("Expected function value"),
    }
}

#[test]
fn test_identifier_lookup() {
    let mut env = Environment::new();
    let evaluator = Evaluator::new();

    // Set up a variable in environment
    env.define("x".to_string(), Value::Integer(42));

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
    let evaluator = Evaluator::new();

    // Set up variable x in environment first
    env.define("x".to_string(), Value::Integer(5));

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
    let evaluator = Evaluator::new();

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
