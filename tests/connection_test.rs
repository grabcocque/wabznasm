// Test for connection config parsing and kernel construction
use tempfile::NamedTempFile;
use wabznasm::jupyter::connection::{ConnectionConfig, ConnectionConfigExt};
use wabznasm::jupyter::kernel::JupyterKernelRunner;

#[tokio::test]
async fn test_connection_config_parsing() {
    // Create a test connection file
    let config_json = serde_json::json!({
        "transport": "tcp",
        "ip": "127.0.0.1",
        "control_port": 50000,
        "hb_port": 50001,
        "iopub_port": 50002,
        "stdin_port": 50003,
        "shell_port": 50004,
        "signature_scheme": "hmac-sha256",
        "key": "test-key-12345"
    });

    let temp_file = NamedTempFile::new().unwrap();
    std::fs::write(
        temp_file.path(),
        serde_json::to_string_pretty(&config_json).unwrap(),
    )
    .unwrap();

    // Test config parsing
    let config =
        ConnectionConfig::from_file(temp_file.path()).expect("Failed to parse connection config");

    // Verify config was parsed correctly by checking some fields
    assert_eq!(config.shell_port, 50004);
    assert_eq!(config.iopub_port, 50002);
    assert_eq!(config.hb_port, 50001);
    assert!(config.key.contains("test-key"));

    // Test kernel runner construction (this verifies signature schemes work)
    let _kernel_runner = JupyterKernelRunner::new(config).expect("Failed to create kernel runner");

    println!("✅ Connection config parsing and kernel construction test passed");
}
