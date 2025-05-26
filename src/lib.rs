//! Library crate exposing the core calculator functionality and REPL.
pub mod environment;
pub mod errors;
pub mod evaluator;
pub mod interning;
pub mod jupyter;
pub mod parser;
pub mod repl;
#[cfg(test)]
mod tests {
    use super::evaluator::evaluate_expression;
    // Note: Codegen-specific tests that were here (implicitly from codegen_legacy.rs) are removed.

    #[test]
    fn test_basic_arithmetic() {
        assert_eq!(evaluate_expression("1+2").unwrap(), 3);
    }

    #[test]
    fn test_additive_associativity() {
        assert_eq!(evaluate_expression("1+2-3").unwrap(), 0);
    }

    #[test]
    fn test_complex_expressions() {
        assert_eq!(evaluate_expression("2 * (3 + 4)  + 5").unwrap(), 19);
    }

    #[test]
    fn test_chained_factorial() {
        assert_eq!(evaluate_expression("3!!").unwrap(), 720);
    }

    #[test]
    fn test_division_by_zero() {
        let err = evaluate_expression("1/0").unwrap_err();
        assert_eq!(err.code().unwrap().to_string(), "DIVISION_BY_ZERO");
    }

    #[test]
    fn test_empty_input() {
        let err = evaluate_expression("").unwrap_err();
        // Empty input is treated as a syntax error by the parser
        assert_eq!(err.to_string(), "Syntax error in expression");
        assert_eq!(err.code().unwrap().to_string(), "SYNTAX_ERROR");
    }

    #[test]
    fn test_exponent_too_large_error() {
        let err = evaluate_expression("2^64").unwrap_err();
        assert_eq!(err.code().unwrap().to_string(), "EXPONENT_TOO_LARGE");
    }

    #[test]
    fn test_factorial() {
        assert_eq!(evaluate_expression("5!").unwrap(), 120);
    }

    #[test]
    fn test_integer_overflow() {
        let err = evaluate_expression("9999999999999999*999999").unwrap_err();
        assert_eq!(err.code().unwrap().to_string(), "INTEGER_OVERFLOW");
    }

    #[test]
    fn test_double_unary() {
        assert_eq!(evaluate_expression("--1").unwrap(), 1);
    }

    #[test]
    fn test_missing_operand_error() {
        let err = evaluate_expression("1+").unwrap_err();
        // Trailing operator triggers a syntax error
        assert_eq!(err.code().unwrap().to_string(), "SYNTAX_ERROR");
    }

    #[test]
    fn test_invalid_number() {
        let err = evaluate_expression("abc").unwrap_err();
        // Identifiers now produce an "undefined variable" error instead of syntax error
        assert_eq!(err.code().unwrap().to_string(), "OTHER_ERROR");
    }

    #[test]
    fn test_factorial_limits() {
        let err = evaluate_expression("21!").unwrap_err();
        assert_eq!(err.code().unwrap().to_string(), "FACTORIAL_TOO_LARGE");
    }

    #[test]
    fn test_negative_exponent() {
        let err = evaluate_expression("2^-1").unwrap_err();
        assert_eq!(err.code().unwrap().to_string(), "NEGATIVE_EXPONENT");
    }

    #[test]
    fn test_parentheses_mismatch() {
        let err = evaluate_expression("(1+2").unwrap_err();
        assert_eq!(err.code().unwrap().to_string(), "SYNTAX_ERROR");
    }

    #[test]
    fn test_power_mul_precedence() {
        assert_eq!(evaluate_expression("2^3*4").unwrap(), 32);
    }

    #[test]
    fn test_power_associativity() {
        assert_eq!(evaluate_expression("2^3^2").unwrap(), 512);
    }

    #[test]
    fn test_syntax_error() {
        let err = evaluate_expression("1+*2").unwrap_err();
        assert_eq!(err.code().unwrap().to_string(), "SYNTAX_ERROR");
    }

    #[test]
    fn test_nested_group_factorial() {
        assert_eq!(evaluate_expression("(3!)!").unwrap(), 720);
    }

    #[test]
    fn test_power_vs_unary_precedence() {
        assert_eq!(evaluate_expression("-2^2").unwrap(), -4);
    }

    #[test]
    fn test_operator_precedence() {
        assert_eq!(evaluate_expression("1+2*3").unwrap(), 7);
    }

    #[test]
    fn test_power_operator() {
        assert_eq!(evaluate_expression("2^3").unwrap(), 8);
    }

    #[test]
    fn test_unary_minus() {
        assert_eq!(evaluate_expression("-5").unwrap(), -5);
    }

    #[test]
    fn test_whitespace_handling() {
        assert_eq!(evaluate_expression("   1    +     2   ").unwrap(), 3);
    }

    #[test]
    fn test_modulo_operator() {
        assert_eq!(evaluate_expression("5%2").unwrap(), 1);
        assert_eq!(evaluate_expression("2+5%3").unwrap(), 2 + 5 % 3);
        assert_eq!(evaluate_expression("10%3*2").unwrap(), 2);
    }

    #[test]
    fn test_modulo_by_zero_error() {
        let err = evaluate_expression("5%0").unwrap_err();
        assert_eq!(err.code().unwrap().to_string(), "DIVISION_BY_ZERO");
    }
}
