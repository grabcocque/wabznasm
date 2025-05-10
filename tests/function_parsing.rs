use wabznasm::parser::parse_expression;

#[test]
fn test_function_definition_parsing() {
    // Test function definition: f: {x+1}
    let tree = parse_expression("f: {x+1}").unwrap();
    assert_eq!(tree.root_node().kind(), "source_file");
    println!("Function definition AST: {}", tree.root_node().to_sexp());
}

#[test]
fn test_function_call_parsing() {
    // Test function call: f[5]
    let tree = parse_expression("f[5]").unwrap();
    assert_eq!(tree.root_node().kind(), "source_file");
    println!("Function call AST: {}", tree.root_node().to_sexp());
}

#[test]
fn test_identifier_parsing() {
    // Test identifier: abc
    let tree = parse_expression("abc").unwrap();
    assert_eq!(tree.root_node().kind(), "source_file");
    println!("Identifier AST: {}", tree.root_node().to_sexp());
}

#[test]
fn test_basic_expression_still_works() {
    // Ensure basic expressions still parse correctly
    let tree = parse_expression("2+3").unwrap();
    assert_eq!(tree.root_node().kind(), "source_file");
    println!("Basic expression AST: {}", tree.root_node().to_sexp());
}
