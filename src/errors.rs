use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;
use tree_sitter::Node;

/// Span in the source code
#[derive(Debug, Clone, Copy)]
pub struct Span {
    /// Byte offset of the start position
    pub start: usize,
    /// Byte offset of the end position
    pub end: usize,
}

impl From<Node<'_>> for Span {
    fn from(node: Node) -> Self {
        Self {
            start: node.start_byte(),
            end: node.end_byte(),
        }
    }
}

impl From<Span> for SourceSpan {
    fn from(span: Span) -> Self {
        (span.start, span.end - span.start).into()
    }
}

/// Error that occurs during parse tree evaluation
#[derive(Error, Debug)]
#[error("{kind}")]
pub struct EvalError {
    #[source]
    pub kind: EvalErrorKind,

    pub span: SourceSpan,

    pub src: Option<NamedSource<String>>,
}

/// Specific kinds of errors that can occur during evaluation
#[derive(Error, Debug)]
pub enum EvalErrorKind {
    #[error("Division by zero")]
    DivisionByZero,

    #[error("Integer overflow: {0}")]
    IntegerOverflow(String),

    #[error("Negative exponent")]
    NegativeExponent,

    #[error("Exponent too large")]
    ExponentTooLarge,

    #[error("Factorial of negative number")]
    FactorialOfNegative,

    #[error("Factorial too large")]
    FactorialTooLarge,

    #[error("Invalid number: {0}")]
    InvalidNumber(String),

    #[error("Unknown operator: {0}")]
    UnknownOperator(String),

    #[error("Missing operand")]
    MissingOperand,

    #[error("{0}")]
    Other(String),
}

impl EvalErrorKind {
    /// Returns a machine-readable error code for this error kind.
    pub fn code(&self) -> &'static str {
        match self {
            EvalErrorKind::DivisionByZero => "DIVISION_BY_ZERO",
            EvalErrorKind::IntegerOverflow(_) => "INTEGER_OVERFLOW",
            EvalErrorKind::NegativeExponent => "NEGATIVE_EXPONENT",
            EvalErrorKind::ExponentTooLarge => "EXPONENT_TOO_LARGE",
            EvalErrorKind::FactorialOfNegative => "FACTORIAL_OF_NEGATIVE",
            EvalErrorKind::FactorialTooLarge => "FACTORIAL_TOO_LARGE",
            EvalErrorKind::InvalidNumber(_) => "INVALID_NUMBER",
            EvalErrorKind::UnknownOperator(_) => "UNKNOWN_OPERATOR",
            EvalErrorKind::MissingOperand => "MISSING_OPERAND",
            EvalErrorKind::Other(_) => "OTHER_ERROR",
        }
    }
}

// Implement Diagnostic manually to include machine-readable error codes.
// Provide diagnostic error codes for better programmatic matching
impl Diagnostic for EvalError {
    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        Some(Box::new(self.kind.code().to_string()))
    }
}

impl EvalError {
    /// Creates a new EvalError without source context.
    pub fn new(kind: EvalErrorKind, node: Node) -> Self {
        Self {
            kind,
            span: Span::from(node).into(),
            src: None,
        }
    }

    /// Attaches the source code to the error for reporting.
    pub fn with_source(mut self, source: &str) -> Self {
        self.src = Some(NamedSource::new("calc", source.to_string()));
        self
    }
}
