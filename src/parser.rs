use crate::errors::Span;
use miette::{LabeledSpan, NamedSource, Report, miette};
use tree_sitter::{Node, Parser as Tsparser, Query, QueryCursor, StreamingIterator, Tree};

include!(concat!(env!("OUT_DIR"), "/calc.rs"));

/// Parses the input string into a TreeSitter parse tree.
///
/// This function converts mathematical expressions into an Abstract Syntax Tree (AST)
/// that can be used for evaluation, code generation, or analysis.
///
/// # Examples
///
/// Parse a simple expression:
/// ```
/// # use wabznasm::parser::parse_expression;
/// let tree = parse_expression("1+2").unwrap();
/// assert_eq!(tree.root_node().kind(), "source_file");
/// ```
///
/// Parse complex expressions:
/// ```
/// # use wabznasm::parser::parse_expression;
/// let tree = parse_expression("2^3 + 4!").unwrap();
/// assert!(!tree.root_node().has_error());
/// ```
///
/// Parse errors are detected via error nodes in the tree:
/// ```
/// # use wabznasm::parser::parse_expression;
/// let tree = parse_expression("1+").unwrap();
/// assert!(tree.root_node().has_error()); // Has error nodes
/// ```
///
/// # Returns
///
/// - `Ok(Tree)` - A parsed syntax tree ready for evaluation
/// - `Err(Report)` - Detailed error information for invalid syntax
pub fn parse_expression(input: &str) -> Result<Tree, Report> {
    let mut parser = Tsparser::new();
    parser
        .set_language(&language())
        .map_err(|e| miette!("parser error: {}", e))?;
    parser
        .parse(input, None)
        .ok_or_else(|| miette!("Failed to parse expression"))
}

/// Checks for syntax errors using a TreeSitter query against the parse tree,
/// and if none are found, invokes the provided callback on the root node.
pub fn query_expression<T, F>(tree: &Tree, input: &str, eval: F) -> Result<T, Report>
where
    F: Fn(Node, &str) -> Result<T, crate::errors::EvalError>,
{
    let root = tree.root_node();
    if root.has_error() {
        let mut cursor = QueryCursor::new();
        let query =
            // Query::new takes Language by value
            Query::new(&language(), "(ERROR) @error").map_err(|e| miette!("Query error: {}", e))?;

        let bytes = input.as_bytes();
        let mut matches = cursor.matches(&query, root, bytes);
        let mut error_nodes = Vec::new();
        while let Some(m) = matches.next() {
            for capture in m.captures {
                error_nodes.push(capture.node);
            }
        }

        if let Some(error_node) = error_nodes.first() {
            let span = Span::from(*error_node);
            return Err(miette!(
                labels = vec![LabeledSpan::at(span.start..span.end, "Syntax error here")],
                code = "SYNTAX_ERROR",
                "Syntax error in expression"
            )
            .with_source_code(NamedSource::new("calc", input.to_string())));
        }
        return Err(miette!(code = "SYNTAX_ERROR", "Syntax error in expression"));
    }
    // Delegate AST evaluation to the provided callback
    eval(root, input).map_err(|e| e.with_source(input).into())
}
