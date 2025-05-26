use crate::errors::{EvalError, EvalErrorKind};
use serde_json::{Value as JsonValue, json};
use std::collections::HashMap;

// General error type for Jupyter operations that might cross threads
pub type BoxedError = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type JupyterResult<T> = Result<T, BoxedError>;

// Error type for operations confined to a single thread (e.g., local file access)
pub type BoxedLocalError = Box<dyn std::error::Error + 'static>;
pub type JupyterLocalResult<T> = Result<T, BoxedLocalError>;

/// Converts wabznasm errors to Jupyter error format
pub struct JupyterErrorFormatter;

impl JupyterErrorFormatter {
    /// Convert an EvalError to Jupyter error format
    pub fn format_error(error: &EvalError, source_code: &str) -> HashMap<String, JsonValue> {
        let mut error_data = HashMap::new();
        error_data.insert("ename".to_string(), json!("WabznasmError"));
        error_data.insert("evalue".to_string(), json!(error.to_string()));
        let traceback = Self::create_traceback(error, source_code);
        error_data.insert("traceback".to_string(), json!(traceback));
        error_data
    }

    /// Create a traceback with syntax highlighting and location info
    pub fn create_traceback(error: &EvalError, _source_code: &str) -> Vec<String> {
        let mut traceback = Vec::new();
        traceback.push(format!("WabznasmError: {}", error));
        // TODO: Add source location from error.span when implemented
        match &error.kind {
            EvalErrorKind::Other(msg) => traceback.push(format!("Error: {}", msg)),
            EvalErrorKind::DivisionByZero => traceback.push("Error: Division by zero".to_string()),
            EvalErrorKind::IntegerOverflow(msg) => {
                traceback.push(format!("Error: Integer overflow: {}", msg))
            }
            _ => traceback.push(format!("Error: {}", error.kind)),
        }
        traceback
    }

    /// Format error for HTML display with syntax highlighting
    pub fn format_error_html(error: &EvalError, source_code: &str) -> String {
        let traceback = Self::create_traceback(error, source_code);
        let mut html = String::from(r#"<div class="nb-error">"#);
        html.push_str(r#"<pre class="nb-traceback">"#);
        for line in traceback {
            // Preserve original logic for potential different stylings
            if line.starts_with(">>> ") {
                // Assuming this was for input lines if they were part of traceback
                html.push_str(&format!(
                    r#"<span class="nb-error-line">{}</span>"#,
                    html_escape::encode_text(&line)
                ));
            } else if line.trim().starts_with("^") {
                // Assuming this was for error pointers
                html.push_str(&format!(
                    r#"<span class="nb-error-pointer">{}</span>"#,
                    html_escape::encode_text(&line)
                ));
            } else {
                html.push_str(&html_escape::encode_text(&line));
            }
            html.push('\n');
        }
        html.push_str("</pre></div>");
        html
    }

    /// Get CSS styles for error display
    pub fn get_error_css() -> &'static str {
        r#"
        <style>
        .nb-error {
            background-color: #fff5f5;
            border: 1px solid #feb2b2;
            border-radius: 4px;
            padding: 12px;
            margin: 8px 0;
        }
        .nb-traceback {
            font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace;
            font-size: 13px;
            line-height: 1.4;
            color: #2d3748;
            background-color: transparent;
            margin: 0;
            white-space: pre-wrap;
        }
        .nb-error-line {
            background-color: #fed7d7;
            color: #c53030;
        }
        .nb-error-pointer {
            color: #e53e3e;
            font-weight: bold;
        }
        </style>
        "#
    }
}

/// Trait for converting errors to Jupyter format
pub trait JupyterError {
    fn to_jupyter_error(&self, source_code: &str) -> HashMap<String, JsonValue>;
    fn to_jupyter_error_html(&self, source_code: &str) -> String;
}

impl JupyterError for EvalError {
    fn to_jupyter_error(&self, source_code: &str) -> HashMap<String, JsonValue> {
        JupyterErrorFormatter::format_error(self, source_code)
    }

    fn to_jupyter_error_html(&self, source_code: &str) -> String {
        JupyterErrorFormatter::format_error_html(self, source_code)
    }
}
