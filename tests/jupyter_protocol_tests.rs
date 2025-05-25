use chrono::Utc;
use jupyter_protocol::{ExecuteRequest, Header, JupyterMessageContent};
use std::sync::Arc;
use tokio::sync::Mutex;
use wabznasm::jupyter::handler::WabznasmJupyterKernel;
use wabznasm::jupyter::signature::SignatureSigner;
use zeromq::{PubSocket, Socket};

fn create_test_header() -> Header {
    Header {
        msg_id: "test-msg-id".to_string(),
        session: "test-session".to_string(),
        username: "test-user".to_string(),
        date: Utc::now(),
        msg_type: "test".to_string(),
        version: "5.3".to_string(),
    }
}

async fn create_test_kernel() -> WabznasmJupyterKernel {
    let iopub_socket = Arc::new(Mutex::new(PubSocket::new()));
    let signer = Arc::new(SignatureSigner::new("hmac-sha256".to_string(), b"test-key").unwrap());
    WabznasmJupyterKernel::new(iopub_socket, signer)
}

#[tokio::test]
async fn test_kernel_info_reply_structure() {
    let kernel = create_test_kernel().await;
    let header = create_test_header();

    let reply = kernel.kernel_info(&header);

    // Test that all required fields are present
    assert_eq!(reply.status, jupyter_protocol::ReplyStatus::Ok);
    assert_eq!(reply.protocol_version, "5.3");
    assert_eq!(reply.implementation, "wabznasm");
    assert!(!reply.implementation_version.is_empty());

    // Test language info
    assert_eq!(reply.language_info.name, "wabznasm");
    assert_eq!(reply.language_info.file_extension, ".wz");
    assert_eq!(reply.language_info.mimetype, "text/plain");

    println!("✅ KernelInfoReply structure is valid");
}

#[tokio::test]
async fn test_execute_request_basic() {
    let mut kernel = create_test_kernel().await;
    let header = create_test_header();

    let execute_request = ExecuteRequest {
        code: "1 + 2".to_string(),
        silent: false,
        store_history: true,
        user_expressions: None,
        allow_stdin: false,
        stop_on_error: true,
    };

    let reply = kernel.execute_request(execute_request, &header).await;

    // Should succeed with basic arithmetic
    assert_eq!(reply.status, jupyter_protocol::ReplyStatus::Ok);
    // ExecutionCount doesn't have a get() method, just check it exists
    println!("Execution count: {:?}", reply.execution_count);

    println!("✅ Execute request works for basic arithmetic");
}

#[tokio::test]
async fn test_execute_request_error() {
    let mut kernel = create_test_kernel().await;
    let header = create_test_header();

    let execute_request = ExecuteRequest {
        code: "invalid syntax here !@#".to_string(),
        silent: false,
        store_history: true,
        user_expressions: None,
        allow_stdin: false,
        stop_on_error: true,
    };

    let reply = kernel.execute_request(execute_request, &header).await;

    // Should fail with syntax error
    assert_eq!(reply.status, jupyter_protocol::ReplyStatus::Error);

    println!("✅ Execute request properly handles errors");
}

#[test]
fn test_kernel_info_json_serialization() {
    use serde_json;

    // Create a test KernelInfoReply and serialize it to see the exact format
    let reply = jupyter_protocol::KernelInfoReply {
        status: jupyter_protocol::ReplyStatus::Ok,
        protocol_version: "5.3".to_string(),
        implementation: "wabznasm".to_string(),
        implementation_version: "0.1.0".to_string(),
        language_info: jupyter_protocol::LanguageInfo {
            name: "wabznasm".to_string(),
            version: "0.1.0".to_string(),
            mimetype: "text/plain".to_string(),
            file_extension: ".wz".to_string(),
            pygments_lexer: "text".to_string(),
            codemirror_mode: jupyter_protocol::messaging::CodeMirrorMode::Simple(
                "text".to_string(),
            ),
            nbconvert_exporter: "script".to_string(),
        },
        banner: "Wabznasm Kernel".to_string(),
        help_links: vec![],
        debugger: false,
        error: None,
    };

    let json = serde_json::to_string_pretty(&reply).unwrap();
    println!("📄 KernelInfoReply JSON structure:");
    println!("{}", json);

    // Test that it can be deserialized back
    let _deserialized: jupyter_protocol::KernelInfoReply = serde_json::from_str(&json).unwrap();

    println!("✅ KernelInfoReply serialization works correctly");
}

#[test]
fn test_jupyter_message_content_serialization() {
    use serde_json;

    // Test that JupyterMessageContent can wrap KernelInfoReply correctly
    let reply = jupyter_protocol::KernelInfoReply {
        status: jupyter_protocol::ReplyStatus::Ok,
        protocol_version: "5.3".to_string(),
        implementation: "wabznasm".to_string(),
        implementation_version: "0.1.0".to_string(),
        language_info: jupyter_protocol::LanguageInfo {
            name: "wabznasm".to_string(),
            version: "0.1.0".to_string(),
            mimetype: "text/plain".to_string(),
            file_extension: ".wz".to_string(),
            pygments_lexer: "text".to_string(),
            codemirror_mode: jupyter_protocol::messaging::CodeMirrorMode::Simple(
                "text".to_string(),
            ),
            nbconvert_exporter: "script".to_string(),
        },
        banner: "Wabznasm Kernel".to_string(),
        help_links: vec![],
        debugger: false,
        error: None,
    };

    let message_content = JupyterMessageContent::KernelInfoReply(Box::new(reply));
    let json = serde_json::to_string_pretty(&message_content).unwrap();

    println!("📄 JupyterMessageContent::KernelInfoReply JSON:");
    println!("{}", json);

    println!("✅ JupyterMessageContent serialization works correctly");
}

#[test]
fn test_signature_creation() {
    use wabznasm::jupyter::signature::SignatureSigner;

    let signer = SignatureSigner::new("hmac-sha256".to_string(), b"test-key").unwrap();

    let test_data = [
        b"header".to_vec(),
        b"parent".to_vec(),
        b"metadata".to_vec(),
        b"content".to_vec(),
    ];
    let signature = signer
        .sign(&test_data.iter().map(|v| v.as_slice()).collect::<Vec<_>>())
        .unwrap();

    println!("🔐 Test signature: {}", signature);
    println!("✅ Signature creation works");
}
