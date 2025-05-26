use crate::environment::Value;
use serde_json::{Value as JsonValue, json};
use std::collections::HashMap;

/// Formats wabznasm values for display in Jupyter
pub struct DisplayFormatter;

impl DisplayFormatter {
    /// Convert a wabznasm Value to Jupyter display data
    pub fn format_value(value: &Value) -> HashMap<String, JsonValue> {
        let mut display_data = HashMap::new();

        match value {
            Value::Integer(n) => {
                // Display integers as plain text and HTML
                display_data.insert("text/plain".to_string(), json!(n.to_string()));
                display_data.insert(
                    "text/html".to_string(),
                    json!(format!("<span class=\"nb-integer\">{}</span>", n)),
                );
            }
            Value::Function { params, body, .. } => {
                // Display functions with their signature
                let signature = if params.is_empty() {
                    format!("{{{}}}", body)
                } else {
                    format!("{{[{}] {}}}", params.join(";"), body)
                };

                display_data.insert("text/plain".to_string(), json!(signature));
                display_data.insert(
                    "text/html".to_string(),
                    json!(format!(
                        "<div class=\"nb-function\">\
                         <span class=\"nb-function-keyword\">function</span> \
                         <code>{}</code>\
                         </div>",
                        signature
                    )),
                );
            }
        }

        display_data
    }

    /// Format an execution result (which may be None for assignments)
    pub fn format_result(result: &Option<Value>) -> HashMap<String, JsonValue> {
        match result {
            Some(value) => Self::format_value(value),
            None => {
                // For assignments or statements with no return value
                HashMap::new()
            }
        }
    }

    /// Create CSS styles for wabznasm output
    pub fn get_css_styles() -> &'static str {
        r#"
        <style>
        .nb-integer {
            color: #0066cc;
            font-weight: bold;
        }
        .nb-function {
            background-color: #f8f9fa;
            border: 1px solid #e9ecef;
            border-radius: 4px;
            padding: 8px;
            margin: 4px 0;
        }
        .nb-function-keyword {
            color: #d73a49;
            font-weight: bold;
        }
        .nb-function code {
            background-color: transparent;
            color: #24292e;
            font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace;
        }
        </style>
        "#
    }
}

/// Trait for converting wabznasm values to display representations
pub trait JupyterDisplay {
    fn to_display_data(&self) -> HashMap<String, JsonValue>;
}

impl JupyterDisplay for Value {
    fn to_display_data(&self) -> HashMap<String, JsonValue> {
        DisplayFormatter::format_value(self)
    }
}

impl JupyterDisplay for Option<Value> {
    fn to_display_data(&self) -> HashMap<String, JsonValue> {
        DisplayFormatter::format_result(self)
    }
}
