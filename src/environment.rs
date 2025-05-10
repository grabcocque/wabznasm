//! Lexical environment for variable and function bindings
//!
//! This module implements lexical scoping with nested environments, allowing for:
//! - Variable and function bindings
//! - Lexical scoping with parent environments
//! - Efficient lookup with scope chain traversal
//! - Support for closures and nested function definitions

use std::collections::HashMap;
use std::rc::Rc;
use crate::errors::{EvalError, EvalErrorKind};
use tree_sitter::Node;

/// A value that can be stored in the environment
#[derive(Debug, Clone)]
pub enum Value {
    /// Integer value
    Integer(i64),
    /// Function value with parameters, body, and captured environment
    Function {
        /// Function parameter names (empty for no params, single element for one param)
        params: Vec<String>,
        /// Function body as source code (we'll store the AST node later)
        body: String,
        /// Captured lexical environment (closure)
        closure: Option<Rc<Environment>>,
    },
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Function { params: p1, body: b1, .. }, Value::Function { params: p2, body: b2, .. }) => {
                // Compare functions by structure, not closure (since Rc<Environment> is hard to compare)
                p1 == p2 && b1 == b2
            }
            _ => false,
        }
    }
}

impl Value {
    /// Convert value to integer if possible
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Value::Integer(n) => Some(*n),
            _ => None,
        }
    }

    /// Check if value is a function
    pub fn is_function(&self) -> bool {
        matches!(self, Value::Function { .. })
    }

    /// Get function arity (parameter count)
    pub fn arity(&self) -> Option<usize> {
        match self {
            Value::Function { params, .. } => Some(params.len()),
            _ => None,
        }
    }
}

/// Lexical environment for variable and function bindings
#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    /// Variable bindings in this scope
    bindings: HashMap<String, Value>,
    /// Parent environment for lexical scoping
    parent: Option<Rc<Environment>>,
}

impl Environment {
    /// Create a new empty environment with no parent (global scope)
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            parent: None,
        }
    }

    /// Create a new environment with a parent (nested scope)
    pub fn with_parent(parent: Rc<Environment>) -> Self {
        Self {
            bindings: HashMap::new(),
            parent: Some(parent),
        }
    }

    /// Bind a value to a name in the current environment
    pub fn define(&mut self, name: String, value: Value) {
        self.bindings.insert(name, value);
    }

    /// Look up a value by name, searching up the scope chain
    pub fn lookup(&self, name: &str) -> Option<&Value> {
        // First check current environment
        if let Some(value) = self.bindings.get(name) {
            return Some(value);
        }

        // Then check parent environments
        if let Some(parent) = &self.parent {
            return parent.lookup(name);
        }

        None
    }

    /// Look up a value by name, returning an error if not found
    pub fn get(&self, name: &str, node: Node) -> Result<&Value, EvalError> {
        self.lookup(name).ok_or_else(|| {
            EvalError::new(
                EvalErrorKind::Other(format!("Undefined variable: {}", name)),
                node,
            )
        })
    }

    /// Check if a name is defined in this environment (not searching parents)
    pub fn has_local(&self, name: &str) -> bool {
        self.bindings.contains_key(name)
    }

    /// Check if a name is defined anywhere in the scope chain
    pub fn has(&self, name: &str) -> bool {
        self.lookup(name).is_some()
    }

    /// Get all variable names in this environment
    pub fn local_names(&self) -> Vec<&String> {
        self.bindings.keys().collect()
    }

    /// Create a new child environment for function calls
    pub fn extend(&self) -> Environment {
        Environment::with_parent(Rc::new(self.clone()))
    }

    /// Create a new child environment and bind parameters to arguments
    pub fn bind_parameters(
        &self,
        params: &[String],
        args: &[Value],
        node: Node,
    ) -> Result<Environment, EvalError> {
        if params.len() != args.len() {
            return Err(EvalError::new(
                EvalErrorKind::Other(format!(
                    "Arity mismatch: expected {} arguments, got {}",
                    params.len(),
                    args.len()
                )),
                node,
            ));
        }

        let mut child_env = self.extend();
        for (param, arg) in params.iter().zip(args.iter()) {
            child_env.define(param.clone(), arg.clone());
        }

        Ok(child_env)
    }

    /// Get the size of this environment (number of bindings)
    pub fn size(&self) -> usize {
        self.bindings.len()
    }

    /// Check if this environment is empty
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_basic_operations() {
        let mut env = Environment::new();

        // Test binding and lookup
        env.define("x".to_string(), Value::Integer(42));
        assert_eq!(env.lookup("x"), Some(&Value::Integer(42)));
        assert_eq!(env.lookup("y"), None);

        // Test has operations
        assert!(env.has("x"));
        assert!(!env.has("y"));
        assert!(env.has_local("x"));
        assert!(!env.has_local("y"));
    }

    #[test]
    fn test_environment_lexical_scoping() {
        let mut global = Environment::new();
        global.define("global_var".to_string(), Value::Integer(1));

        let mut local = Environment::with_parent(Rc::new(global));
        local.define("local_var".to_string(), Value::Integer(2));

        // Local environment can see both local and global variables
        assert_eq!(local.lookup("local_var"), Some(&Value::Integer(2)));
        assert_eq!(local.lookup("global_var"), Some(&Value::Integer(1)));

        // Local variables shadow global ones
        local.define("global_var".to_string(), Value::Integer(10));
        assert_eq!(local.lookup("global_var"), Some(&Value::Integer(10)));
    }

    #[test]
    fn test_environment_extension() {
        let mut env = Environment::new();
        env.define("x".to_string(), Value::Integer(42));

        let child = env.extend();
        assert_eq!(child.lookup("x"), Some(&Value::Integer(42)));
        assert!(child.has("x"));
        assert!(!child.has_local("x")); // Not in local scope
    }

    #[test]
    fn test_parameter_binding() {
        let env = Environment::new();
        let params = vec!["x".to_string(), "y".to_string()];
        let args = vec![Value::Integer(1), Value::Integer(2)];

        // Create a dummy node for testing (this would normally come from the parser)
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&crate::parser::language()).unwrap();
        let tree = parser.parse("2+3", None).unwrap();
        let node = tree.root_node();

        let child_env = env.bind_parameters(&params, &args, node).unwrap();

        assert_eq!(child_env.lookup("x"), Some(&Value::Integer(1)));
        assert_eq!(child_env.lookup("y"), Some(&Value::Integer(2)));
    }

    #[test]
    fn test_function_value() {
        let func = Value::Function {
            params: vec!["x".to_string()],
            body: "x+1".to_string(),
            closure: None,
        };

        assert!(func.is_function());
        assert_eq!(func.arity(), Some(1));
        assert_eq!(func.as_integer(), None);
    }

    #[test]
    fn test_environment_properties() {
        let mut env = Environment::new();
        assert!(env.is_empty());
        assert_eq!(env.size(), 0);

        env.define("x".to_string(), Value::Integer(42));
        assert!(!env.is_empty());
        assert_eq!(env.size(), 1);

        let names = env.local_names();
        assert_eq!(names.len(), 1);
        assert!(names.contains(&&"x".to_string()));
    }
}
