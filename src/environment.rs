//! Lexical environment for variable and function bindings
//!
//! This module implements lexical scoping with nested environments, allowing for:
//! - Variable and function bindings
//! - Lexical scoping with parent environments
//! - Efficient lookup with scope chain traversal
//! - Support for closures and nested function definitions

use crate::errors::{EvalError, EvalErrorKind};
use crate::interning::InternedString;
use bumpalo::Bump;
use lasso::Rodeo;
use std::collections::HashMap;
use std::sync::Arc;
use tree_sitter::Node;

/// A value that can be stored in the environment
#[derive(Debug, Clone)]
pub enum Value {
    /// Integer value
    Integer(i64),
    /// Function value with parameters, body, and captured environment
    Function {
        /// Function parameter names (empty for no params, single element for one param)
        /// Uses interned strings for memory efficiency and fast comparison
        params: Vec<InternedString>,
        /// Function body as source code (we'll store the AST node later)
        /// Uses interned string for memory efficiency
        body: InternedString,
        /// Captured lexical environment (closure)
        closure: Option<Arc<Environment>>,
    },
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (
                Value::Function {
                    params: p1,
                    body: b1,
                    ..
                },
                Value::Function {
                    params: p2,
                    body: b2,
                    ..
                },
            ) => {
                // Compare functions by structure, not closure (since Arc<Environment> is hard to compare)
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

    /// Get function parameter names as resolved strings
    pub fn param_names(&self, interner: &Rodeo) -> Option<Vec<String>> {
        match self {
            Value::Function { params, .. } => Some(
                params
                    .iter()
                    .map(|&param| interner.resolve(&param).to_string())
                    .collect(),
            ),
            _ => None,
        }
    }

    /// Get function body as resolved string
    pub fn body_source(&self, interner: &Rodeo) -> Option<String> {
        match self {
            Value::Function { body, .. } => Some(interner.resolve(body).to_string()),
            _ => None,
        }
    }

    /// Create a new function value with interned strings
    pub fn new_function(
        param_names: &[String],
        body_source: &str,
        closure: Option<Arc<Environment>>,
        interner: &mut Rodeo,
    ) -> Self {
        let params = param_names
            .iter()
            .map(|name| interner.get_or_intern(name))
            .collect();
        let body = interner.get_or_intern(body_source);
        Value::Function {
            params,
            body,
            closure,
        }
    }
}

/// Lexical environment for variable and function bindings
#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    /// Variable bindings in this scope - using interned strings for variable names
    bindings: HashMap<InternedString, Value>,
    /// Parent environment for lexical scoping
    parent: Option<Arc<Environment>>,
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
    pub fn with_parent(parent: Arc<Environment>) -> Self {
        Self {
            bindings: HashMap::new(),
            parent: Some(parent),
        }
    }

    /// Bind a value to a name in the current environment
    pub fn define(&mut self, name: String, value: Value, interner: &mut Rodeo) {
        let interned_name = interner.get_or_intern(&name);
        self.bindings.insert(interned_name, value);
    }

    /// Bind a value to an interned name in the current environment
    pub fn define_interned(&mut self, name: InternedString, value: Value) {
        self.bindings.insert(name, value);
    }

    /// Look up a value by name, searching up the scope chain
    pub fn lookup(&self, name: &str, interner: &mut Rodeo) -> Option<&Value> {
        // Intern the name for lookup
        let interned_name = interner.get_or_intern(name);

        // First check current environment
        if let Some(value) = self.bindings.get(&interned_name) {
            return Some(value);
        }

        // Then check parent environments
        if let Some(parent) = &self.parent {
            return parent.lookup(name, interner);
        }

        None
    }

    /// Look up a value by interned name, searching up the scope chain
    pub fn lookup_interned(&self, name: InternedString) -> Option<&Value> {
        // First check current environment
        if let Some(value) = self.bindings.get(&name) {
            return Some(value);
        }

        // Then check parent environments
        if let Some(parent) = &self.parent {
            return parent.lookup_interned(name);
        }

        None
    }

    /// Look up a value by name, returning an error if not found
    pub fn get(&self, name: &str, node: Node, interner: &mut Rodeo) -> Result<&Value, EvalError> {
        self.lookup(name, interner).ok_or_else(|| {
            EvalError::new(
                EvalErrorKind::Other(format!("Undefined variable: {}", name)),
                node,
            )
        })
    }

    /// Check if a name is defined in this environment (not searching parents)
    pub fn has_local(&self, name: &str, interner: &mut Rodeo) -> bool {
        let interned_name = interner.get_or_intern(name);
        self.bindings.contains_key(&interned_name)
    }

    /// Check if an interned name is defined in this environment (not searching parents)
    pub fn has_local_interned(&self, name: InternedString) -> bool {
        self.bindings.contains_key(&name)
    }

    /// Check if a name is defined anywhere in the scope chain
    pub fn has(&self, name: &str, interner: &mut Rodeo) -> bool {
        self.lookup(name, interner).is_some()
    }

    /// Get all variable names in this environment as resolved strings
    pub fn local_names(&self, interner: &Rodeo) -> Vec<String> {
        self.bindings
            .keys()
            .map(|&key| interner.resolve(&key).to_string())
            .collect()
    }

    /// Get all variable names in this environment as interned strings
    pub fn local_names_interned(&self) -> Vec<InternedString> {
        self.bindings.keys().copied().collect()
    }

    /// Create a new child environment for function calls
    pub fn extend(&self) -> Environment {
        Environment::with_parent(Arc::new(self.clone()))
    }

    /// Create a new child environment and bind parameters to arguments
    pub fn bind_parameters(
        &self,
        params: &[InternedString],
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
        for (&param, arg) in params.iter().zip(args.iter()) {
            child_env.define_interned(param, arg.clone());
        }

        Ok(child_env)
    }

    /// Create a new child environment and bind parameters to arguments using arena allocation
    pub fn bind_parameters_with_arena(
        &self,
        params: &[InternedString],
        args: &[Value],
        node: Node,
        arena: &Bump,
        interner: &mut Rodeo,
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

        // Use arena for temporary parameter binding operations
        let mut temp_bindings = bumpalo::collections::Vec::new_in(arena);

        // Collect parameter-argument pairs in arena
        for (&param, arg) in params.iter().zip(args.iter()) {
            let param_str = arena.alloc_str(interner.resolve(&param));
            temp_bindings.push((param_str, arg));
        }

        // Create child environment (still uses regular HashMap for permanent storage)
        let mut child_env = self.extend();

        // Bind parameters from arena-allocated temporaries
        for (param_str, arg) in temp_bindings {
            // Convert the arena string back to an interned string for permanent storage
            let param_interned = interner.get_or_intern(param_str);
            child_env.define_interned(param_interned, arg.clone());
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
        let mut interner = Rodeo::default();

        // Test binding and lookup
        env.define("x".to_string(), Value::Integer(42), &mut interner);
        assert_eq!(env.lookup("x", &mut interner), Some(&Value::Integer(42)));
        assert_eq!(env.lookup("y", &mut interner), None);

        // Test has operations
        assert!(env.has("x", &mut interner));
        assert!(!env.has("y", &mut interner));
        assert!(env.has_local("x", &mut interner));
        assert!(!env.has_local("y", &mut interner));
    }

    #[test]
    fn test_environment_lexical_scoping() {
        let mut interner = Rodeo::default();
        let mut global = Environment::new();
        global.define("global_var".to_string(), Value::Integer(1), &mut interner);

        let mut local = Environment::with_parent(Arc::new(global));
        local.define("local_var".to_string(), Value::Integer(2), &mut interner);

        // Local environment can see both local and global variables
        assert_eq!(
            local.lookup("local_var", &mut interner),
            Some(&Value::Integer(2))
        );
        assert_eq!(
            local.lookup("global_var", &mut interner),
            Some(&Value::Integer(1))
        );

        // Local variables shadow global ones
        local.define("global_var".to_string(), Value::Integer(10), &mut interner);
        assert_eq!(
            local.lookup("global_var", &mut interner),
            Some(&Value::Integer(10))
        );
    }

    #[test]
    fn test_environment_extension() {
        let mut interner = Rodeo::default();
        let mut env = Environment::new();
        env.define("x".to_string(), Value::Integer(42), &mut interner);

        let child = env.extend();
        assert_eq!(child.lookup("x", &mut interner), Some(&Value::Integer(42)));
        assert!(child.has("x", &mut interner));
        assert!(!child.has_local("x", &mut interner)); // Not in local scope
    }

    #[test]
    fn test_parameter_binding() {
        let mut interner = Rodeo::default();
        let env = Environment::new();
        let param_names = ["x".to_string(), "y".to_string()];
        let params: Vec<InternedString> = param_names
            .iter()
            .map(|name| interner.get_or_intern(name))
            .collect();
        let args = vec![Value::Integer(1), Value::Integer(2)];

        // Create a dummy node for testing (this would normally come from the parser)
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&crate::parser::language()).unwrap();
        let tree = parser.parse("2+3", None).unwrap();
        let node = tree.root_node();

        let child_env = env.bind_parameters(&params, &args, node).unwrap();

        assert_eq!(
            child_env.lookup("x", &mut interner),
            Some(&Value::Integer(1))
        );
        assert_eq!(
            child_env.lookup("y", &mut interner),
            Some(&Value::Integer(2))
        );
    }

    #[test]
    fn test_function_value() {
        let mut interner = Rodeo::default();
        let func = Value::new_function(&["x".to_string()], "x+1", None, &mut interner);

        assert!(func.is_function());
        assert_eq!(func.arity(), Some(1));
        assert_eq!(func.as_integer(), None);
        assert_eq!(func.param_names(&interner), Some(vec!["x".to_string()]));
        assert_eq!(func.body_source(&interner), Some("x+1".to_string()));
    }

    #[test]
    fn test_environment_properties() {
        let mut interner = Rodeo::default();
        let mut env = Environment::new();
        assert!(env.is_empty());
        assert_eq!(env.size(), 0);

        env.define("x".to_string(), Value::Integer(42), &mut interner);
        assert!(!env.is_empty());
        assert_eq!(env.size(), 1);

        let names = env.local_names(&interner);
        assert_eq!(names.len(), 1);
        assert!(names.contains(&"x".to_string()));
    }
}
