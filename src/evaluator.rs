use crate::environment::{Environment, Value};
use crate::errors::{EvalError, EvalErrorKind};
use crate::parser::{parse_expression, query_expression};
use miette::Report;
use std::rc::Rc;
use tree_sitter::Node;

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
pub struct Evaluator;

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl Evaluator {
    pub fn new() -> Self {
        Evaluator
    }

    /// Evaluate a node with an environment, returning a Value
    pub fn eval_with_env<'a>(&self, node: Node<'a>, src: &str, env: &mut Environment) -> Result<Value, EvalError> {
        match node.kind() {
            // Delegate to child for wrapper nodes
            "source_file" | "expression" | "statement" => {
                let child = self.named_child(node)?;
                self.eval_with_env(child, src, env)
            }

            // Arithmetic expressions (return integer values)
            "number" => Ok(Value::Integer(self.visit_number_raw(node, src)?)),
            "additive" | "multiplicative" => Ok(Value::Integer(self.visit_binary_raw(node, src, env)?)),
            "primary" => self.visit_primary_with_env(node, src, env),
            "unary" => Ok(Value::Integer(self.visit_unary_raw(node, src, env)?)),
            "power" => Ok(Value::Integer(self.visit_power_raw(node, src, env)?)),
            "postfix" => Ok(Value::Integer(self.visit_postfix_raw(node, src, env)?)),

            // Variable and function operations
            "identifier" => self.visit_identifier(node, src, env),
            "assignment" => self.visit_assignment(node, src, env),
            "function_call" => self.visit_function_call(node, src, env),

            other => Err(EvalError::new(
                EvalErrorKind::Other(format!("Unexpected node type: {}", other)),
                node,
            )),
        }
    }

    /// Legacy method for backward compatibility (integers only)
    pub fn eval<'a>(&self, node: Node<'a>, src: &str) -> Result<i64, EvalError> {
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

    #[allow(dead_code)]
    fn visit_binary<'a>(&self, node: Node<'a>, src: &str) -> Result<i64, EvalError> {
        // Binary operation if operator field exists, otherwise fallback to single child
        if let Some(opn) = node.child_by_field_name("operator") {
            // Get all required fields at once to avoid redundant lookups
            let lhs = node
                .child_by_field_name("left")
                .ok_or_else(|| EvalError::new(EvalErrorKind::MissingOperand, node))?;
            let rhs = node
                .child_by_field_name("right")
                .ok_or_else(|| EvalError::new(EvalErrorKind::MissingOperand, node))?;
            let lv = self.eval(lhs, src)?;
            let rv = self.eval(rhs, src)?;
            let op = self.op_text(opn, src)?;
            evaluate_binary_operation(lv, rv, op, node, opn)
        } else {
            let child = self.named_child(node)?;
            self.eval(child, src)
        }
    }

    #[allow(dead_code)]
    fn visit_unary<'a>(&self, node: Node<'a>, src: &str) -> Result<i64, EvalError> {
        if node.child_by_field_name("operator").is_some() {
            let operand = self.child(node, "operand")?;
            let v = self.eval(operand, src)?;
            v.checked_neg().ok_or_else(|| {
                EvalError::new(EvalErrorKind::IntegerOverflow("negation".into()), node)
            })
        } else {
            let c = self.named_child(node)?;
            self.eval(c, src)
        }
    }

    #[allow(dead_code)]
    fn visit_power<'a>(&self, node: Node<'a>, src: &str) -> Result<i64, EvalError> {
        if node.child_by_field_name("operator").is_some() {
            let base = self.child(node, "base")?;
            let exp = self.child(node, "exponent")?;
            let b = self.eval(base, src)?;
            let e = self.eval(exp, src)?;
            if e < 0 {
                Err(EvalError::new(EvalErrorKind::NegativeExponent, exp))
            } else if e > 63 {
                Err(EvalError::new(EvalErrorKind::ExponentTooLarge, exp))
            } else {
                b.checked_pow(e as u32).ok_or_else(|| {
                    EvalError::new(
                        EvalErrorKind::IntegerOverflow("power operation".into()),
                        node,
                    )
                })
            }
        } else {
            let c = self.named_child(node)?;
            self.eval(c, src)
        }
    }

    #[allow(dead_code)]
    fn visit_postfix<'a>(&self, node: Node<'a>, src: &str) -> Result<i64, EvalError> {
        if node.child_by_field_name("operator").is_some() {
            let operand = self.child(node, "operand")?;
            let v = self.eval(operand, src)?;
            calculate_factorial(v, node)
        } else {
            let c = self.named_child(node)?;
            self.eval(c, src)
        }
    }

    #[allow(dead_code)]
    fn visit_primary<'a>(&self, node: Node<'a>, src: &str) -> Result<i64, EvalError> {
        if let Some(expr) = node.child_by_field_name("expression") {
            self.eval(expr, src)
        } else if let Some(num) = node.named_child(0) {
            self.eval(num, src)
        } else {
            Err(EvalError::new(
                EvalErrorKind::Other("Invalid primary expression".into()),
                node,
            ))
        }
    }

    fn visit_number<'a>(&self, node: Node<'a>, src: &str) -> Result<i64, EvalError> {
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

    /// Visit identifier and lookup in environment
    fn visit_identifier(&self, node: Node, src: &str, env: &Environment) -> Result<Value, EvalError> {
        let name = get_node_text(node, src).map_err(|e| {
            EvalError::new(EvalErrorKind::Other(e), node)
        })?;

        env.get(name, node).cloned()
    }

    /// Visit assignment: name: value or name: {body}
    fn visit_assignment(&self, node: Node, src: &str, env: &mut Environment) -> Result<Value, EvalError> {
        let name_node = node.child_by_field_name("name")
            .ok_or_else(|| EvalError::new(EvalErrorKind::MissingOperand, node))?;
        let value_node = node.child_by_field_name("value")
            .ok_or_else(|| EvalError::new(EvalErrorKind::MissingOperand, node))?;

        let name = get_node_text(name_node, src).map_err(|e| {
            EvalError::new(EvalErrorKind::Other(e), name_node)
        })?;

        let value = match value_node.kind() {
            "function_body" => self.visit_function_body(value_node, src, env)?,
            _ => self.eval_with_env(value_node, src, env)?,
        };

        env.define(name.to_string(), value.clone());
        Ok(value)
    }

    /// Visit function body: {expr} or {[params] expr}
    fn visit_function_body(&self, node: Node, src: &str, env: &Environment) -> Result<Value, EvalError> {
        let body_node = node.child_by_field_name("body")
            .ok_or_else(|| EvalError::new(EvalErrorKind::MissingOperand, node))?;
        let params_node = node.child_by_field_name("params");

        let body_text = get_node_text(body_node, src).map_err(|e| {
            EvalError::new(EvalErrorKind::Other(e), body_node)
        })?;

        let params = if let Some(params_node) = params_node {
            self.extract_parameter_list(params_node, src)?
        } else {
            vec![]
        };

        Ok(Value::Function {
            params,
            body: body_text.to_string(),
            closure: Some(Rc::new(env.clone())),
        })
    }

    /// Extract parameter names from parameter list: [x;y;z]
    #[allow(clippy::type_complexity)]
    fn extract_parameter_list(&self, node: Node, src: &str) -> Result<Vec<String>, EvalError> {
        let mut params = Vec::new();

        // Walk through all named children that are identifiers
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            if child.kind() == "identifier" {
                let param_name = get_node_text(child, src).map_err(|e| {
                    EvalError::new(EvalErrorKind::Other(e), child)
                })?;
                params.push(param_name.to_string());
            }
        }

        Ok(params)
    }

    /// Visit function call: func[args]
    fn visit_function_call(&self, node: Node, src: &str, env: &mut Environment) -> Result<Value, EvalError> {
        let func_node = node.child_by_field_name("function")
            .ok_or_else(|| EvalError::new(EvalErrorKind::MissingOperand, node))?;
        let args_node = node.child_by_field_name("args");

        // Get function value
        let func_value = self.visit_identifier(func_node, src, env)?;
        let (params, body, closure) = match func_value {
            Value::Function { params, body, closure } => (params, body, closure),
            _ => return Err(EvalError::new(
                EvalErrorKind::Other("Cannot call non-function value".into()),
                func_node,
            )),
        };

        // Evaluate arguments
        let args = if let Some(args_node) = args_node {
            self.extract_argument_list(args_node, src, env)?
        } else {
            vec![]
        };

        // Create function execution environment
        let base_env = closure.as_ref().map(|c| c.as_ref()).unwrap_or(env);
        let mut call_env = base_env.bind_parameters(&params, &args, node)?;

        // Parse and evaluate function body
        let tree = parse_expression(&body).map_err(|e| {
            EvalError::new(EvalErrorKind::Other(format!("Function body parse error: {}", e)), node)
        })?;

        self.eval_with_env(tree.root_node(), &body, &mut call_env)
    }

    /// Extract argument values from argument list: expr;expr;expr
    #[allow(clippy::type_complexity)]
    fn extract_argument_list(&self, node: Node, src: &str, env: &mut Environment) -> Result<Vec<Value>, EvalError> {
        let mut args = Vec::new();

        // Walk through all named children looking for argument expressions
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            if child.kind() == "expression" {
                let arg_value = self.eval_with_env(child, src, env)?;
                args.push(arg_value);
            }
        }

        Ok(args)
    }

    /// Visit primary with environment support
    fn visit_primary_with_env(&self, node: Node, src: &str, env: &mut Environment) -> Result<Value, EvalError> {
        let child = self.named_child(node)?;
        self.eval_with_env(child, src, env)
    }

    // Raw arithmetic methods (return i64) - updated to work with environments

    fn visit_number_raw(&self, node: Node, src: &str) -> Result<i64, EvalError> {
        self.visit_number(node, src)
    }

    fn visit_binary_raw(&self, node: Node, src: &str, env: &mut Environment) -> Result<i64, EvalError> {
        // Binary operation if operator field exists, otherwise fallback to single child
        if let Some(opn) = node.child_by_field_name("operator") {
            let lhs = self.child(node, "left")?;
            let rhs = self.child(node, "right")?;

            let left_val = match self.eval_with_env(lhs, src, env)? {
                Value::Integer(n) => n,
                _ => return Err(EvalError::new(
                    EvalErrorKind::Other("Expected integer in arithmetic".into()),
                    lhs,
                )),
            };

            let right_val = match self.eval_with_env(rhs, src, env)? {
                Value::Integer(n) => n,
                _ => return Err(EvalError::new(
                    EvalErrorKind::Other("Expected integer in arithmetic".into()),
                    rhs,
                )),
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

    fn visit_unary_raw(&self, node: Node, src: &str, env: &mut Environment) -> Result<i64, EvalError> {
        if let Some(_op) = node.child_by_field_name("operator") {
            let operand_node = self.child(node, "operand")?;
            let operand = match self.eval_with_env(operand_node, src, env)? {
                Value::Integer(n) => n,
                _ => return Err(EvalError::new(
                    EvalErrorKind::Other("Expected integer in unary operation".into()),
                    operand_node,
                )),
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

    fn visit_power_raw(&self, node: Node, src: &str, env: &mut Environment) -> Result<i64, EvalError> {
        if let Some(_op) = node.child_by_field_name("operator") {
            let base_node = self.child(node, "base")?;
            let exp_node = self.child(node, "exponent")?;

            let base = match self.eval_with_env(base_node, src, env)? {
                Value::Integer(n) => n,
                _ => return Err(EvalError::new(
                    EvalErrorKind::Other("Expected integer in power operation".into()),
                    base_node,
                )),
            };

            let exponent = match self.eval_with_env(exp_node, src, env)? {
                Value::Integer(n) => n,
                _ => return Err(EvalError::new(
                    EvalErrorKind::Other("Expected integer in power operation".into()),
                    exp_node,
                )),
            };

            if exponent < 0 {
                return Err(EvalError::new(EvalErrorKind::NegativeExponent, node));
            }
            if exponent > 63 {
                return Err(EvalError::new(EvalErrorKind::ExponentTooLarge, node));
            }

            base.checked_pow(exponent as u32).ok_or_else(|| {
                EvalError::new(EvalErrorKind::IntegerOverflow("exponentiation".into()), node)
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

    fn visit_postfix_raw(&self, node: Node, src: &str, env: &mut Environment) -> Result<i64, EvalError> {
        if let Some(_op) = node.child_by_field_name("operator") {
            let operand_node = self.child(node, "operand")?;
            let operand = match self.eval_with_env(operand_node, src, env)? {
                Value::Integer(n) => n,
                _ => return Err(EvalError::new(
                    EvalErrorKind::Other("Expected integer in factorial".into()),
                    operand_node,
                )),
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
pub fn eval_node<'a>(node: Node<'a>, src: &str) -> Result<i64, EvalError> {
    Evaluator::new().eval(node, src)
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
