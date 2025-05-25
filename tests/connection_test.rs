// Simple test to validate kernel startup without Jupyter
use std::process::Command;
use tempfile::NamedTempFile;

#[test]
fn test_kernel_starts_and_binds_correctly() {
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

    println!(
        "🧪 Testing kernel startup with config: {}",
        temp_file.path().display()
    );

    // Try to start the kernel for 2 seconds
    let output = Command::new("timeout")
        .arg("2s")
        .arg("cargo")
        .arg("run")
        .arg("--")
        .arg("jupyter")
        .arg("start")
        .arg(temp_file.path())
        .output()
        .expect("Failed to start kernel");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("📤 Stdout:\n{}", stdout);
    println!("📤 Stderr:\n{}", stderr);

    // Check if it started successfully
    assert!(stdout.contains("🚀 Starting Wabznasm Jupyter kernel"));
    assert!(stdout.contains("Shell socket bound to tcp://127.0.0.1:50004"));
    assert!(stdout.contains("IOPub socket bound to tcp://127.0.0.1:50002"));
    assert!(stdout.contains("Heartbeat socket bound to tcp://127.0.0.1:50001"));
    assert!(stdout.contains("✅ Kernel is ready for connections"));

    println!("✅ Kernel startup test passed");
}
