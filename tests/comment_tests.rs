use lasso::Rodeo;
use wabznasm::environment::{Environment, Value};
use wabznasm::evaluator::Evaluator;
use wabznasm::parser::parse_expression;

#[test]
fn test_backslash_comment_only() {
    let mut env = Environment::new();
    let mut evaluator = Evaluator::new();
    let mut interner = Rodeo::new();

    // Test that only backslash comments work (to avoid division ambiguity)
    let tree = parse_expression("x: 42 \\ This is a backslash comment").unwrap();
    let result = evaluator
        .eval_with_env(
            tree.root_node(),
            "x: 42 \\ This is a backslash comment",
            &mut env,
        )
        .unwrap();

    // Should return the assigned value
    assert_eq!(result, Value::Integer(42));

    // Variable should be set despite comment
    assert_eq!(env.lookup("x", &mut interner), Some(&Value::Integer(42)));
}

#[test]
fn test_end_of_line_comment() {
    let mut env = Environment::new();
    let mut evaluator = Evaluator::new();
    let mut interner = Rodeo::new();

    // Test end-of-line comment with \
    let tree = parse_expression("x: 42 \\ This is an end-of-line comment").unwrap();
    let result = evaluator
        .eval_with_env(
            tree.root_node(),
            "x: 42 \\ This is an end-of-line comment",
            &mut env,
        )
        .unwrap();

    // Should return the assigned value
    assert_eq!(result, Value::Integer(42));

    // Variable should be set despite comment
    assert_eq!(env.lookup("x", &mut interner), Some(&Value::Integer(42)));
}

#[test]
fn test_mixed_comments() {
    let mut env = Environment::new();
    let mut evaluator = Evaluator::new();

    // First define the function
    let tree =
        parse_expression("add: {[x;y] x + y}  \\ inline comment about the function").unwrap();
    evaluator
        .eval_with_env(
            tree.root_node(),
            "add: {[x;y] x + y}  \\ inline comment about the function",
            &mut env,
        )
        .unwrap();

    // Then call the function
    let tree = parse_expression("add[2; 3]   \\ should return 5").unwrap();
    let result = evaluator
        .eval_with_env(tree.root_node(), "add[2; 3]   \\ should return 5", &mut env)
        .unwrap();

    // Check that the result is correct
    assert_eq!(result, Value::Integer(5));
}
