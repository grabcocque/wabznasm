use crate::environment::Environment;

/// Type alias for cleaner code
type ExecuteResult = Result<Option<crate::environment::Value>, crate::errors::EvalError>;

/// Manages the persistent environment state across Jupyter cells
pub struct JupyterSession {
    /// The persistent environment that maintains state across cell executions
    environment: Environment,
    /// Execution counter for cells
    execution_count: u32,
}

impl JupyterSession {
    /// Create a new Jupyter session with an empty environment
    pub fn new() -> Self {
        Self {
            environment: Environment::new(),
            execution_count: 0,
        }
    }

    /// Get a clone of the current environment for read operations
    pub fn get_environment(&self) -> Environment {
        self.environment.clone()
    }

    /// Execute code in the session environment and return the result
    pub fn execute(&mut self, code: &str) -> ExecuteResult {
        self.execution_count += 1;

        // Parse the code
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&crate::parser::language())
            .map_err(|e| {
                // Create a dummy tree to get a valid node
                let dummy_tree = tree_sitter::Parser::new().parse("", None).unwrap();
                crate::errors::EvalError::new(
                    crate::errors::EvalErrorKind::Other(format!("Language error: {}", e)),
                    dummy_tree.root_node(),
                )
            })?;

        let tree = parser.parse(code, None).ok_or_else(|| {
            // Create a dummy tree for error reporting
            let dummy_tree = parser.parse("", None).unwrap();
            crate::errors::EvalError::new(
                crate::errors::EvalErrorKind::Other("Failed to parse input".to_string()),
                dummy_tree.root_node(),
            )
        })?;

        let root = tree.root_node();

        // If there's a syntax error, return it
        if root.has_error() {
            return Err(crate::errors::EvalError::new(
                crate::errors::EvalErrorKind::Other("Syntax error in input".to_string()),
                root,
            ));
        }

        // Execute in the persistent environment
        let evaluator = crate::evaluator::Evaluator::new();

        // Check if this is an expression or assignment
        let child_count = root.child_count();
        if child_count == 0 {
            return Ok(None);
        }

        let mut last_result = None;

        // Process each top-level statement
        for i in 0..child_count {
            let child = root.child(i).unwrap();

            // Skip whitespace and comments
            if child.kind() == "comment" || child.is_extra() {
                continue;
            }

            let result = evaluator.eval_with_env(child, code, &mut self.environment)?;
            last_result = Some(result);
        }

        Ok(last_result)
    }

    /// Get the current execution count
    pub fn execution_count(&self) -> u32 {
        self.execution_count
    }

    /// Reset the session environment
    pub fn reset(&mut self) {
        self.environment = Environment::new();
        self.execution_count = 0;
    }
}

impl Default for JupyterSession {
    fn default() -> Self {
        Self::new()
    }
}
