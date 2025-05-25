use jupyter_protocol::ConnectionInfo;
use std::fs;
use std::path::Path;

/// Represents the Jupyter connection configuration, loaded from a file.
/// This is an alias for `jupyter_protocol::ConnectionInfo`.
pub type ConnectionConfig = ConnectionInfo;

/// Extension trait to add convenience methods for loading ConnectionConfig from files.
pub trait ConnectionConfigExt {
    /// Load connection config from a JSON file.
    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized;

    /// Get shell socket URL
    fn shell_url(&self) -> String;

    /// Get IOPub socket URL
    fn iopub_url(&self) -> String;

    /// Get heartbeat socket URL
    fn hb_url(&self) -> String;
}

impl ConnectionConfigExt for ConnectionConfig {
    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: ConnectionConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    fn shell_url(&self) -> String {
        format!("{}://{}:{}", self.transport, self.ip, self.shell_port)
    }

    fn iopub_url(&self) -> String {
        format!("{}://{}:{}", self.transport, self.ip, self.iopub_port)
    }

    fn hb_url(&self) -> String {
        format!("{}://{}:{}", self.transport, self.ip, self.hb_port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_from_file() {
        // A minimal valid connection file JSON
        let config_json = r#"{
            "shell_port": 54321,
            "iopub_port": 54322,
            "stdin_port": 54323,
            "control_port": 54324,
            "hb_port": 54325,
            "ip": "127.0.0.1",
            "key": "test-key",
            "transport": "tcp",
            "signature_scheme": "hmac-sha256",
            "kernel_name": "wabznasm_test_kernel"
        }"#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), config_json).unwrap();

        let config = ConnectionConfig::from_file(temp_file.path()).unwrap();
        assert_eq!(config.shell_port, 54321);
        assert_eq!(config.key, "test-key");
        assert_eq!(config.kernel_name, Some("wabznasm_test_kernel".to_string()));
    }
}
