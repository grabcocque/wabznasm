use crate::environment::{Environment, Value};
use crate::errors::{EvalError, EvalErrorKind};
use crate::interning::InternedString;
use crate::parser::{parse_expression, query_expression};
use bumpalo::Bump;
use lasso::Rodeo;
use miette::Report;
use std::sync::Arc;
use tree_sitter::Node;

// Type aliases for cleaner code
type EvalInternedStringListResult = Result<Vec<InternedString>, EvalError>;
type EvalValueListResult = Result<Vec<Value>, EvalError>;

fn get_node_text<'a>(node: Node<'a>, source: &'a str) -> Result<&'a str, String> {
    node.utf8_text(source.as_bytes()).map_err(|e| e.to_string())
}

fn calculate_factorial(n: i64, node: Node) -> Result<i64, EvalError> {
    if n < 0 {
        return Err(EvalError::new(EvalErrorKind::FactorialOfNegative, node));
    }
    if n > 20 {
        return Err(EvalError::new(EvalErrorKind::FactorialTooLarge, node));
    }
    let mut result = 1i64;
    for i in 1..=n {
        result = result.checked_mul(i).ok_or_else(|| {
            EvalError::new(
                EvalErrorKind::IntegerOverflow("factorial computation".into()),
                node,
            )
        })?;
    }
    Ok(result)
}

fn evaluate_binary_operation(
    left: i64,
    right: i64,
    op: &str,
    node: Node,
    op_node: Node,
) -> Result<i64, EvalError> {
    match op {
        "+" => left
            .checked_add(right)
            .ok_or_else(|| EvalError::new(EvalErrorKind::IntegerOverflow("addition".into()), node)),
        "-" => left.checked_sub(right).ok_or_else(|| {
            EvalError::new(EvalErrorKind::IntegerOverflow("subtraction".into()), node)
        }),
        "*" => left.checked_mul(right).ok_or_else(|| {
            EvalError::new(
                EvalErrorKind::IntegerOverflow("multiplication".into()),
                node,
            )
        }),
        "/" => {
            if right == 0 {
                Err(EvalError::new(EvalErrorKind::DivisionByZero, node))
            } else {
                Ok(left / right)
            }
        }
        "%" => {
            if right == 0 {
                Err(EvalError::new(EvalErrorKind::DivisionByZero, node))
            } else {
                Ok(left % right)
            }
        }
        _ => Err(EvalError::new(
            EvalErrorKind::UnknownOperator(op.into()),
            op_node,
        )),
    }
}

/// Visitor struct that encapsulates evaluation logic with environment support.
/// Each evaluator instance maintains its own session-scoped string interner.
pub struct Evaluator {
    /// Session-scoped string interner for this evaluator instance
    string_interner: Rodeo,
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl Evaluator {
    pub fn new() -> Self {
        Evaluator {
            string_interner: Rodeo::default(),
        }
    }

    /// Get a reference to the session-scoped string interner
    pub fn interner(&self) -> &Rodeo {
        &self.string_interner
    }

    /// Intern a string using this evaluator's session-scoped interner
    pub fn intern(&mut self, s: &str) -> InternedString {
        self.string_interner.get_or_intern(s)
    }

    /// Resolve an interned string back to its value
    pub fn resolve(&self, key: InternedString) -> &str {
        self.string_interner.resolve(&key)
    }

    /// Evaluate a node with an environment, returning a Value
    /// This is the main public API that automatically uses bumpalo for temporaries
    pub fn eval_with_env(
        &mut self,
        node: Node<'_>,
        src: &str,
        env: &mut Environment,
    ) -> Result<Value, EvalError> {
        // Create arena for this evaluation call - scoped to this invocation
        let arena = Bump::new();
        self.eval_with_env_and_arena(node, src, env, &arena)
    }

    /// Internal evaluation method that uses provided bumpalo arena for temporaries
    pub fn eval_with_env_and_arena(
        &mut self,
        node: Node<'_>,
        src: &str,
        env: &mut Environment,
        arena: &Bump,
    ) -> Result<Value, EvalError> {
        match node.kind() {
            // Handle source_file with multiple children
            "source_file" => {
                let mut last_result = None;
                let mut cursor = node.walk();

                for child in node.named_children(&mut cursor) {
                    match child.kind() {
                        "comment" => continue, // Skip comments
                        "statement" => {
                            last_result =
                                Some(self.eval_with_env_and_arena(child, src, env, arena)?);
                        }
                        _ => {
                            // Handle other non-comment nodes as statements
                            last_result =
                                Some(self.eval_with_env_and_arena(child, src, env, arena)?);
                        }
                    }
                }

                last_result.ok_or_else(|| {
                    EvalError::new(
                        EvalErrorKind::Other("No statements found in source file".into()),
                        node,
                    )
                })
            }

            // Delegate to child for wrapper nodes
            "expression" | "statement" => {
                let child = self.named_child(node)?;
                self.eval_with_env_and_arena(child, src, env, arena)
            }

            // Arithmetic expressions (return integer values)
            "number" => Ok(Value::Integer(self.visit_number_raw(node, src)?)),
            "additive" | "multiplicative" => {
                Ok(Value::Integer(self.visit_binary_raw(node, src, env)?))
            }
            "primary" => self.visit_primary_with_env(node, src, env),
            "unary" => Ok(Value::Integer(self.visit_unary_raw(node, src, env)?)),
            "power" => Ok(Value::Integer(self.visit_power_raw(node, src, env)?)),
            "postfix" => Ok(Value::Integer(self.visit_postfix_raw(node, src, env)?)),

            // Variable and function operations - use interned optimized versions
            "identifier" => self.visit_identifier_interned(node, src, env),
            "assignment" => self.visit_assignment_with_arena(node, src, env, arena),
            "function_call" => self.visit_function_call_with_arena(node, src, env, arena),

            other => Err(EvalError::new(
                EvalErrorKind::Other(format!("Unexpected node type: {}", other)),
                node,
            )),
        }
    }

    /// Legacy method for backward compatibility (integers only)
    pub fn eval(&mut self, node: Node<'_>, src: &str) -> Result<i64, EvalError> {
        let mut env = Environment::new();
        match self.eval_with_env(node, src, &mut env)? {
            Value::Integer(n) => Ok(n),
            _ => Err(EvalError::new(
                EvalErrorKind::Other("Expected integer value".into()),
                node,
            )),
        }
    }

    fn named_child<'a>(&self, node: Node<'a>) -> Result<Node<'a>, EvalError> {
        node.named_child(0)
            .ok_or_else(|| EvalError::new(EvalErrorKind::MissingOperand, node))
    }

    fn child<'a>(&self, node: Node<'a>, field: &str) -> Result<Node<'a>, EvalError> {
        node.child_by_field_name(field)
            .ok_or_else(|| EvalError::new(EvalErrorKind::MissingOperand, node))
    }

    fn op_text<'a>(&self, node: Node<'a>, src: &'a str) -> Result<&'a str, EvalError> {
        node.utf8_text(src.as_bytes()).map_err(|e| {
            EvalError::new(
                EvalErrorKind::Other(format!("Failed to read operator: {}", e)),
                node,
            )
        })
    }

    fn visit_number(&self, node: Node<'_>, src: &str) -> Result<i64, EvalError> {
        let txt = get_node_text(node, src).map_err(|e| {
            EvalError::new(
                EvalErrorKind::Other(format!("Failed to get number text: {}", e)),
                node,
            )
        })?;
        txt.parse::<i64>()
            .map_err(|e| EvalError::new(EvalErrorKind::InvalidNumber(e.to_string()), node))
    }

    // New environment-aware visitor methods

    /// Visit identifier with interned string optimization
    fn visit_identifier_interned(
        &mut self,
        node: Node,
        src: &str,
        env: &Environment,
    ) -> Result<Value, EvalError> {
        let name =
            get_node_text(node, src).map_err(|e| EvalError::new(EvalErrorKind::Other(e), node))?;

        // Intern the identifier name for efficient lookup using session-scoped interner
        let interned_name = self.intern(name);

        // Try interned lookup first, fall back to string lookup for compatibility
        if let Some(value) = env.lookup_interned(interned_name) {
            Ok(value.clone())
        } else {
            env.get(name, node, &mut self.string_interner).cloned()
        }
    }

    /// Visit assignment with arena support: name: value or name: {body}
    fn visit_assignment_with_arena(
        &mut self,
        node: Node,
        src: &str,
        env: &mut Environment,
        arena: &Bump,
    ) -> Result<Value, EvalError> {
        let name_node = node
            .child_by_field_name("name")
            .ok_or_else(|| EvalError::new(EvalErrorKind::MissingOperand, node))?;
        let value_node = node
            .child_by_field_name("value")
            .ok_or_else(|| EvalError::new(EvalErrorKind::MissingOperand, node))?;

        let name = get_node_text(name_node, src)
            .map_err(|e| EvalError::new(EvalErrorKind::Other(e), name_node))?;

        let value = match value_node.kind() {
            "function_body" => self.visit_function_body_with_arena(value_node, src, env, arena)?,
            _ => self.eval_with_env_and_arena(value_node, src, env, arena)?,
        };

        // Intern the variable name for efficient storage and lookup
        let interned_name = self.intern(name);
        env.define_interned(interned_name, value.clone());
        Ok(value)
    }

    /// Visit function body with arena support: {expr} or {[params] expr}
    fn visit_function_body_with_arena(
        &mut self,
        node: Node,
        src: &str,
        env: &Environment,
        arena: &Bump,
    ) -> Result<Value, EvalError> {
        let body_node = node
            .child_by_field_name("body")
            .ok_or_else(|| EvalError::new(EvalErrorKind::MissingOperand, node))?;
        let params_node = node.child_by_field_name("params");

        let body_text = get_node_text(body_node, src)
            .map_err(|e| EvalError::new(EvalErrorKind::Other(e), body_node))?;

        let params = if let Some(params_node) = params_node {
            self.extract_parameter_list_with_arena(params_node, src, arena)?
        } else {
            vec![]
        };

        let param_names: Vec<String> = params
            .iter()
            .map(|&p| self.resolve(p).to_string())
            .collect();
        Ok(Value::new_function(
            &param_names,
            body_text,
            Some(Arc::new(env.clone())),
            &mut self.string_interner,
        ))
    }

    /// Visit function call with arena support: func[args]
    fn visit_function_call_with_arena(
        &mut self,
        node: Node,
        src: &str,
        env: &mut Environment,
        arena: &Bump,
    ) -> Result<Value, EvalError> {
        let func_node = node
            .child_by_field_name("function")
            .ok_or_else(|| EvalError::new(EvalErrorKind::MissingOperand, node))?;
        let args_node = node.child_by_field_name("args");

        // Get function value using interned lookup
        let func_value = self.visit_identifier_interned(func_node, src, env)?;
        let (params, body, closure) = match func_value {
            Value::Function {
                params,
                body,
                closure,
            } => (params, body, closure),
            _ => {
                return Err(EvalError::new(
                    EvalErrorKind::Other("Cannot call non-function value".into()),
                    func_node,
                ));
            }
        };

        // Evaluate arguments using arena
        let args = if let Some(args_node) = args_node {
            self.extract_argument_list_with_arena(args_node, src, env, arena)?
        } else {
            vec![]
        };

        // Create function execution environment using arena
        let base_env = closure.as_ref().map(|c| c.as_ref()).unwrap_or(env);
        let mut call_env = base_env.bind_parameters_with_arena(
            &params,
            &args,
            node,
            arena,
            &mut self.string_interner,
        )?;

        // Parse and evaluate function body
        let body_str = self.resolve(body).to_string();
        let tree = parse_expression(&body_str).map_err(|e| {
            EvalError::new(
                EvalErrorKind::Other(format!("Function body parse error: {}", e)),
                node,
            )
        })?;

        self.eval_with_env_and_arena(tree.root_node(), &body_str, &mut call_env, arena)
    }

    // Bumpalo-enhanced parameter/argument extraction methods

    /// Extract parameter names using bumpalo for temporary allocations
    /// Returns Vec<InternedString> for memory efficiency
    fn extract_parameter_list_with_arena(
        &mut self,
        node: Node,
        src: &str,
        arena: &Bump,
    ) -> EvalInternedStringListResult {
        // Use bumpalo Vec for temporary collection
        let mut temp_params = bumpalo::collections::Vec::new_in(arena);

        // Walk through all named children that are identifiers
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            if child.kind() == "identifier" {
                let param_name = get_node_text(child, src)
                    .map_err(|e| EvalError::new(EvalErrorKind::Other(e), child))?;
                // Store as arena-allocated string slice initially
                let arena_str = arena.alloc_str(param_name);
                temp_params.push(arena_str);
            }
        }

        // Convert to Vec<InternedString> for return (intern the strings)
        let result = temp_params.iter().map(|s| self.intern(s)).collect();
        Ok(result)
    }

    /// Extract argument values using bumpalo for temporary allocations
    /// Returns Vec<Value> for compatibility, but uses arena internally
    fn extract_argument_list_with_arena(
        &mut self,
        node: Node,
        src: &str,
        env: &mut Environment,
        arena: &Bump,
    ) -> EvalValueListResult {
        // Use bumpalo Vec for temporary collection
        let mut temp_args = bumpalo::collections::Vec::new_in(arena);

        // Walk through all named children looking for argument expressions
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            if child.kind() == "expression" {
                let arg_value = self.eval_with_env_and_arena(child, src, env, arena)?;
                temp_args.push(arg_value);
            }
        }

        // Convert to owned Vec<Value> for return (long-lived data)
        Ok(temp_args.into_iter().collect())
    }

    /// Visit primary with environment support
    fn visit_primary_with_env(
        &mut self,
        node: Node,
        src: &str,
        env: &mut Environment,
    ) -> Result<Value, EvalError> {
        let child = self.named_child(node)?;
        self.eval_with_env(child, src, env)
    }

    // Raw arithmetic methods (return i64) - updated to work with environments

    fn visit_number_raw(&self, node: Node, src: &str) -> Result<i64, EvalError> {
        self.visit_number(node, src)
    }

    fn visit_binary_raw(
        &mut self,
        node: Node,
        src: &str,
        env: &mut Environment,
    ) -> Result<i64, EvalError> {
        // Binary operation if operator field exists, otherwise fallback to single child
        if let Some(opn) = node.child_by_field_name("operator") {
            let lhs = self.child(node, "left")?;
            let rhs = self.child(node, "right")?;

            let left_val = match self.eval_with_env(lhs, src, env)? {
                Value::Integer(n) => n,
                _ => {
                    return Err(EvalError::new(
                        EvalErrorKind::Other("Expected integer in arithmetic".into()),
                        lhs,
                    ));
                }
            };

            let right_val = match self.eval_with_env(rhs, src, env)? {
                Value::Integer(n) => n,
                _ => {
                    return Err(EvalError::new(
                        EvalErrorKind::Other("Expected integer in arithmetic".into()),
                        rhs,
                    ));
                }
            };

            let op = self.op_text(opn, src)?;
            evaluate_binary_operation(left_val, right_val, op, node, opn)
        } else {
            // Single child fallback
            let child = self.named_child(node)?;
            match self.eval_with_env(child, src, env)? {
                Value::Integer(n) => Ok(n),
                _ => Err(EvalError::new(
                    EvalErrorKind::Other("Expected integer".into()),
                    child,
                )),
            }
        }
    }

    fn visit_unary_raw(
        &mut self,
        node: Node,
        src: &str,
        env: &mut Environment,
    ) -> Result<i64, EvalError> {
        if let Some(_op) = node.child_by_field_name("operator") {
            let operand_node = self.child(node, "operand")?;
            let operand = match self.eval_with_env(operand_node, src, env)? {
                Value::Integer(n) => n,
                _ => {
                    return Err(EvalError::new(
                        EvalErrorKind::Other("Expected integer in unary operation".into()),
                        operand_node,
                    ));
                }
            };
            Ok(-operand)
        } else {
            let child = self.named_child(node)?;
            match self.eval_with_env(child, src, env)? {
                Value::Integer(n) => Ok(n),
                _ => Err(EvalError::new(
                    EvalErrorKind::Other("Expected integer".into()),
                    child,
                )),
            }
        }
    }

    fn visit_power_raw(
        &mut self,
        node: Node,
        src: &str,
        env: &mut Environment,
    ) -> Result<i64, EvalError> {
        if let Some(_op) = node.child_by_field_name("operator") {
            let base_node = self.child(node, "base")?;
            let exp_node = self.child(node, "exponent")?;

            let base = match self.eval_with_env(base_node, src, env)? {
                Value::Integer(n) => n,
                _ => {
                    return Err(EvalError::new(
                        EvalErrorKind::Other("Expected integer in power operation".into()),
                        base_node,
                    ));
                }
            };

            let exponent = match self.eval_with_env(exp_node, src, env)? {
                Value::Integer(n) => n,
                _ => {
                    return Err(EvalError::new(
                        EvalErrorKind::Other("Expected integer in power operation".into()),
                        exp_node,
                    ));
                }
            };

            if exponent < 0 {
                return Err(EvalError::new(EvalErrorKind::NegativeExponent, node));
            }
            if exponent > 63 {
                return Err(EvalError::new(EvalErrorKind::ExponentTooLarge, node));
            }

            base.checked_pow(exponent as u32).ok_or_else(|| {
                EvalError::new(
                    EvalErrorKind::IntegerOverflow("exponentiation".into()),
                    node,
                )
            })
        } else {
            let child = self.named_child(node)?;
            match self.eval_with_env(child, src, env)? {
                Value::Integer(n) => Ok(n),
                _ => Err(EvalError::new(
                    EvalErrorKind::Other("Expected integer".into()),
                    child,
                )),
            }
        }
    }

    fn visit_postfix_raw(
        &mut self,
        node: Node,
        src: &str,
        env: &mut Environment,
    ) -> Result<i64, EvalError> {
        if let Some(_op) = node.child_by_field_name("operator") {
            let operand_node = self.child(node, "operand")?;
            let operand = match self.eval_with_env(operand_node, src, env)? {
                Value::Integer(n) => n,
                _ => {
                    return Err(EvalError::new(
                        EvalErrorKind::Other("Expected integer in factorial".into()),
                        operand_node,
                    ));
                }
            };
            calculate_factorial(operand, node)
        } else {
            let child = self.named_child(node)?;
            match self.eval_with_env(child, src, env)? {
                Value::Integer(n) => Ok(n),
                _ => Err(EvalError::new(
                    EvalErrorKind::Other("Expected integer".into()),
                    child,
                )),
            }
        }
    }
}

/// Entry point invoked by query_expression
/// Adapter for visitor-based evaluation
pub fn eval_node(node: Node<'_>, src: &str) -> Result<i64, EvalError> {
    let mut evaluator = Evaluator::new();
    evaluator.eval(node, src)
}

/// Parses the input string into a TreeSitter parse tree.
/// Evaluates mathematical expressions using tree-walking interpretation.
///
/// This is the original expression evaluator that walks the AST directly without
/// generating LLVM IR. It's useful for comparison, debugging, and scenarios where
/// JIT compilation overhead isn't warranted.
///
/// # Examples
///
/// Basic arithmetic:
/// ```
/// # use wabznasm::evaluator::evaluate_expression;
/// assert_eq!(evaluate_expression("1+2").unwrap(), 3);
/// assert_eq!(evaluate_expression("10-3").unwrap(), 7);
/// assert_eq!(evaluate_expression("4*5").unwrap(), 20);
/// assert_eq!(evaluate_expression("15/3").unwrap(), 5);
/// assert_eq!(evaluate_expression("17%5").unwrap(), 2);
/// ```
///
/// Operator precedence:
/// ```
/// # use wabznasm::evaluator::evaluate_expression;
/// assert_eq!(evaluate_expression("2+3*4").unwrap(), 14);
/// assert_eq!(evaluate_expression("(2+3)*4").unwrap(), 20);
/// ```
///
/// Power and factorial operations:
/// ```
/// # use wabznasm::evaluator::evaluate_expression;
/// assert_eq!(evaluate_expression("2^3").unwrap(), 8);
/// assert_eq!(evaluate_expression("5!").unwrap(), 120);
/// assert_eq!(evaluate_expression("3!!").unwrap(), 720);
/// ```
///
/// Error handling:
/// ```
/// # use wabznasm::evaluator::evaluate_expression;
/// let result = evaluate_expression("1/0");
/// assert!(result.is_err());
///
/// let result = evaluate_expression("(-3)!");
/// assert!(result.is_err());
/// ```
///
/// # Returns
///
/// - `Ok(i64)` - The computed result
/// - `Err(Report)` - Error information for syntax or runtime errors
pub fn evaluate_expression(input: &str) -> Result<i64, Report> {
    let tree = parse_expression(input)?;
    query_expression(&tree, input, eval_node)
}
