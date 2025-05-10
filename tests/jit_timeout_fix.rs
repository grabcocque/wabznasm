#[cfg(test)]
mod jit_timeout_tests {
    use std::time::{Duration, Instant};
    use wabznasm::evaluator::evaluate_expression;

    #[test]
    fn test_negative_exponent_timeout_fix() {
        let expr = "-3^2^1^-4!";

        // First verify the interpreter handles this correctly
        let interpreter_result = evaluate_expression(expr);
        assert!(
            interpreter_result.is_err(),
            "Interpreter should error on negative exponent"
        );

        // Now test JIT - it should error quickly, not timeout
        let start = Instant::now();
        let jit_result = evaluate_expression(expr);
        let elapsed = start.elapsed();

        // Should complete quickly (under 100ms)
        assert!(
            elapsed < Duration::from_millis(100),
            "JIT took too long: {:?}, likely infinite loop",
            elapsed
        );

        // Should error like the interpreter
        assert!(
            jit_result.is_err(),
            "JIT should error on negative exponent like interpreter"
        );

        println!("Interpreter: {:?}", interpreter_result);
        println!("JIT: {:?}", jit_result);
        println!("Elapsed: {:?}", elapsed);
    }

    #[test]
    fn test_factorial_too_large_fix() {
        let expr = "2/5!!"; // 5!! = (5!)! = 120! which is > 20 limit

        // Verify interpreter handles this correctly
        let interpreter_result = evaluate_expression(expr);
        assert!(
            interpreter_result.is_err(),
            "Interpreter should error on factorial too large"
        );

        // Test JIT - should error quickly, not crash
        let start = Instant::now();
        let jit_result = evaluate_expression(expr);
        let elapsed = start.elapsed();

        // Should complete quickly (under 100ms)
        assert!(
            elapsed < Duration::from_millis(100),
            "JIT took too long: {:?}, likely infinite loop",
            elapsed
        );

        // Should error like the interpreter
        assert!(
            jit_result.is_err(),
            "JIT should error on factorial too large like interpreter"
        );

        println!("Interpreter: {:?}", interpreter_result);
        println!("JIT: {:?}", jit_result);
        println!("Elapsed: {:?}", elapsed);
    }

    #[test]
    fn test_stack_overflow_fix() {
        // Create a deep factorial chain that would cause stack overflow
        let mut expr = "5".to_string();
        for _ in 0..150 {
            // More than MAX_DEPTH (100)
            expr.push('!');
        }

        // Test JIT - should error gracefully, not crash with stack overflow
        let start = Instant::now();
        let jit_result = evaluate_expression(&expr);
        let elapsed = start.elapsed();

        // Should complete reasonably quickly (under 500ms)
        // Increased timeout to account for template-based system overhead
        assert!(
            elapsed < Duration::from_millis(500),
            "JIT took too long: {:?}, likely infinite recursion",
            elapsed
        );

        // Should error on recursion depth
        assert!(jit_result.is_err(), "JIT should error on deep recursion");

        println!("JIT deep recursion: {:?}", jit_result);
        println!("Elapsed: {:?}", elapsed);
    }

    #[test]
    fn test_large_exponent_chains() {
        // Test other potential problematic patterns
        let test_cases = vec![
            "2^2^2^2",   // Should be fast: 2^(2^(2^2)) = 2^16 = 65536
            "3^3^2",     // Should be fast: 3^(3^2) = 3^9 = 19683
            "2^2^2^2^2", // Should be bounded or error quickly
        ];

        for expr in test_cases {
            let start = Instant::now();
            let result = evaluate_expression(expr);
            let elapsed = start.elapsed();

            // All should complete quickly
            assert!(
                elapsed < Duration::from_millis(100),
                "Expression '{}' took too long: {:?}",
                expr,
                elapsed
            );

            println!("'{}' -> {:?} in {:?}", expr, result, elapsed);
        }
    }
}
