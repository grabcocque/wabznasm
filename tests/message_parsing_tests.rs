use chrono::Utc;
use jupyter_protocol::{Header, JupyterMessageContent, KernelInfoRequest};
use wabznasm::jupyter::message_parser::ParsedMessage;
use wabznasm::jupyter::signature::{SignatureSigner, SignatureVerifier};

fn create_test_kernel_info_request_message() -> zeromq::ZmqMessage {
    let signer = SignatureSigner::new("hmac-sha256".to_string(), b"test-key").unwrap();

    let header = Header {
        msg_id: "test-msg-id".to_string(),
        session: "test-session".to_string(),
        username: "test-user".to_string(),
        date: Utc::now(),
        msg_type: "kernel_info_request".to_string(),
        version: "5.3".to_string(),
    };

    let kernel_info_request = KernelInfoRequest {};
    let content = JupyterMessageContent::KernelInfoRequest(kernel_info_request);
    let metadata: std::collections::HashMap<String, serde_json::Value> =
        std::collections::HashMap::new();

    // Serialize message parts
    let header_bytes = serde_json::to_vec(&header).unwrap();
    let parent_header_bytes = serde_json::to_vec(&serde_json::json!({})).unwrap();
    let metadata_bytes = serde_json::to_vec(&metadata).unwrap();
    let content_bytes = serde_json::to_vec(&content).unwrap();

    let signature = signer
        .sign(&[
            &header_bytes,
            &parent_header_bytes,
            &metadata_bytes,
            &content_bytes,
        ])
        .unwrap();

    // Build ZMQ message
    let mut frames = Vec::new();
    frames.push(b"router-id".to_vec());
    frames.push(b"<IDS|MSG>".to_vec());
    frames.push(signature.into_bytes());
    frames.push(header_bytes);
    frames.push(parent_header_bytes);
    frames.push(metadata_bytes);
    frames.push(content_bytes);

    let mut zmq_msg = zeromq::ZmqMessage::from(frames[0].clone());
    for frame in frames.into_iter().skip(1) {
        zmq_msg.push_back(frame.into());
    }
    zmq_msg
}

#[test]
fn test_message_parsing_roundtrip() {
    let verifier = SignatureVerifier::new("hmac-sha256".to_string(), b"test-key").unwrap();
    let zmq_msg = create_test_kernel_info_request_message();

    println!("📦 Created test message with {} frames", zmq_msg.len());

    match ParsedMessage::parse(&zmq_msg, &verifier) {
        Ok(parsed) => {
            println!("✅ Message parsed successfully");
            println!("📋 Message type: {}", parsed.header.msg_type);
            println!("📋 Session: {}", parsed.header.session);
            println!("📋 Identities: {} items", parsed.identities.len());

            // Check that it's a kernel_info_request
            match parsed.content {
                JupyterMessageContent::KernelInfoRequest(_) => {
                    println!("✅ Correctly identified as KernelInfoRequest");
                }
                _ => {
                    panic!("❌ Wrong message type parsed");
                }
            }
        }
        Err(e) => {
            panic!("❌ Failed to parse message: {}", e);
        }
    }
}

#[test]
fn test_message_parsing_bad_signature() {
    let verifier = SignatureVerifier::new("hmac-sha256".to_string(), b"wrong-key").unwrap();
    let zmq_msg = create_test_kernel_info_request_message();

    match ParsedMessage::parse(&zmq_msg, &verifier) {
        Ok(_) => {
            panic!("❌ Should have failed with bad signature");
        }
        Err(e) => {
            println!("✅ Correctly rejected bad signature: {}", e);
        }
    }
}

#[test]
fn test_message_parsing_frame_structure() {
    let zmq_msg = create_test_kernel_info_request_message();

    println!("🔍 Analyzing message frame structure:");
    for (i, frame) in zmq_msg.iter().enumerate() {
        if i < 3 {
            match std::str::from_utf8(frame) {
                Ok(s) => println!("Frame {}: {:?}", i, s),
                Err(_) => println!("Frame {}: {} bytes (binary)", i, frame.len()),
            }
        } else {
            println!("Frame {}: {} bytes (JSON/binary)", i, frame.len());
        }
    }

    // Check expected frame count
    let expected_frames = 7; // router-id + delimiter + signature + header + parent_header + metadata + content
    assert_eq!(
        zmq_msg.len(),
        expected_frames,
        "Expected {} frames, got {}",
        expected_frames,
        zmq_msg.len()
    );

    println!("✅ Frame structure is correct");
}
