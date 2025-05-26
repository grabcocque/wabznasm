use crate::jupyter::errors::JupyterResult;
use crate::jupyter::{
    display::JupyterDisplay, errors::JupyterErrorFormatter, session::JupyterSession,
    signature::SignatureSigner as JP_SignatureSigner,
};
use chrono::Utc;
use jupyter_protocol::{
    ExecuteReply, ExecuteRequest, Header, KernelInfoReply, LanguageInfo, ReplyStatus,
    ShutdownRequest, messaging::CodeMirrorMode, messaging::ExecutionCount,
};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use zeromq::{PubSocket, SocketSend, ZmqMessage};

/// Custom shutdown reply struct used by kernel.rs
pub struct CustomShutdownReply {
    pub restart: bool,
}

/// Simplified message structure for IOPub, defined locally
#[derive(serde::Serialize)]
struct SimplifiedMessage {
    header: Header,
    parent_header: Option<Header>,
    metadata: HashMap<String, JsonValue>,
    content: JsonValue,
}

/// The main Jupyter kernel implementation for wabznasm
pub struct WabznasmJupyterKernel {
    /// Persistent session that maintains environment across cells
    session: JupyterSession,
    /// IOPub socket for broadcasting output
    iopub_socket: Arc<Mutex<PubSocket>>,
    /// Signature signer for IOPub messages
    signer: Arc<JP_SignatureSigner>,
}

impl WabznasmJupyterKernel {
    /// Create a new kernel instance
    pub fn new(iopub_socket: Arc<Mutex<PubSocket>>, signer: Arc<JP_SignatureSigner>) -> Self {
        Self {
            session: JupyterSession::new(),
            iopub_socket,
            signer,
        }
    }

    /// Handle kernel_info_request
    pub fn kernel_info(&self, _parent_header: &Header) -> KernelInfoReply {
        KernelInfoReply {
            status: ReplyStatus::Ok,
            protocol_version: "5.3".to_string(),
            implementation: "wabznasm".to_string(),
            implementation_version: env!("CARGO_PKG_VERSION").to_string(),
            language_info: LanguageInfo {
                name: "wabznasm".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                mimetype: "text/plain".to_string(),
                file_extension: ".wz".to_string(),
                pygments_lexer: "text".to_string(),
                codemirror_mode: CodeMirrorMode::Simple("text".to_string()),
                nbconvert_exporter: "script".to_string(),
            },
            banner: "Wabznasm Kernel".to_string(),
            help_links: vec![],
            debugger: false,
            error: None,
        }
    }

    /// Handle execute_request
    pub async fn execute_request(
        &mut self,
        request: ExecuteRequest,
        parent_header: &Header,
    ) -> ExecuteReply {
        let code = &request.code;

        // Send busy status
        {
            let iopub_header = self.create_iopub_header(parent_header, "status".to_string());
            let status_content = serde_json::json!({
                "execution_state": "busy"
            });
            let msg = SimplifiedMessage {
                header: iopub_header,
                parent_header: Some(parent_header.clone()),
                metadata: HashMap::new(),
                content: status_content,
            };
            let mut iopub_guard = self.iopub_socket.lock().await;
            if let Ok(zmq_msg) = construct_zmq_message_for_iopub(&msg, &self.signer) {
                if let Err(e) = iopub_guard.send(zmq_msg).await {
                    eprintln!("Failed to send busy status: {}", e);
                }
            }
        }

        let exec_reply_content = match self.session.execute(code) {
            Ok(result) => {
                let display_data_map = result.to_display_data();
                if !display_data_map.is_empty() {
                    let iopub_header =
                        self.create_iopub_header(parent_header, "execute_result".to_string());
                    let exec_result_content = serde_json::json!({
                        "execution_count": self.session.execution_count(),
                        "data": display_data_map,
                        "metadata": {}
                    });
                    let msg = SimplifiedMessage {
                        header: iopub_header,
                        parent_header: Some(parent_header.clone()),
                        metadata: HashMap::new(),
                        content: exec_result_content,
                    };
                    let mut iopub_guard = self.iopub_socket.lock().await;
                    if let Ok(zmq_msg) = construct_zmq_message_for_iopub(&msg, &self.signer) {
                        if let Err(e) = iopub_guard.send(zmq_msg).await {
                            eprintln!("Failed to send execute_result: {}", e);
                        }
                    }
                }
                ExecuteReply {
                    status: ReplyStatus::Ok,
                    execution_count: ExecutionCount::new(self.session.execution_count() as usize),
                    payload: vec![],
                    user_expressions: None,
                    error: None,
                }
            }
            Err(eval_error) => {
                let ename = "WabznasmError".to_string();
                let evalue = eval_error.to_string();
                let traceback = JupyterErrorFormatter::create_traceback(&eval_error, code);

                let iopub_header = self.create_iopub_header(parent_header, "error".to_string());
                let error_content = serde_json::json!({
                    "ename": ename.clone(),
                    "evalue": evalue.clone(),
                    "traceback": traceback.clone()
                });
                let msg = SimplifiedMessage {
                    header: iopub_header,
                    parent_header: Some(parent_header.clone()),
                    metadata: HashMap::new(),
                    content: error_content,
                };
                let mut iopub_guard = self.iopub_socket.lock().await;
                if let Ok(zmq_msg) = construct_zmq_message_for_iopub(&msg, &self.signer) {
                    if let Err(e) = iopub_guard.send(zmq_msg).await {
                        eprintln!("Failed to send error IOPub: {}", e);
                    }
                }

                ExecuteReply {
                    status: ReplyStatus::Error,
                    execution_count: ExecutionCount::new(self.session.execution_count() as usize),
                    payload: vec![],
                    user_expressions: None,
                    error: None, // Using JSON in IOPub instead
                }
            }
        };

        // Send idle status
        {
            let iopub_header = self.create_iopub_header(parent_header, "status".to_string());
            let status_content = serde_json::json!({
                "execution_state": "idle"
            });
            let msg = SimplifiedMessage {
                header: iopub_header,
                parent_header: Some(parent_header.clone()),
                metadata: HashMap::new(),
                content: status_content,
            };
            let mut iopub_guard = self.iopub_socket.lock().await;
            if let Ok(zmq_msg) = construct_zmq_message_for_iopub(&msg, &self.signer) {
                if let Err(e) = iopub_guard.send(zmq_msg).await {
                    eprintln!("Failed to send idle status: {}", e);
                }
            }
        }
        exec_reply_content
    }

    /// Handle shutdown_request
    pub fn shutdown_request(
        &mut self,
        request: ShutdownRequest,
        _parent_header: &Header,
    ) -> CustomShutdownReply {
        self.session.reset();
        println!("WabznasmJupyterKernel: Shutdown requested, session reset.");
        CustomShutdownReply {
            restart: request.restart,
        }
    }

    /// Create a header for IOPub messages
    fn create_iopub_header(&self, parent_header: &Header, msg_type: String) -> Header {
        Header {
            msg_id: Uuid::new_v4().to_string(),
            session: parent_header.session.clone(),
            username: parent_header.username.clone(),
            date: Utc::now(),
            msg_type,
            version: parent_header.version.clone(),
        }
    }
}

/// Construct a ZMQ message from a SimplifiedMessage for IOPub publishing
fn construct_zmq_message_for_iopub(
    message: &SimplifiedMessage,
    signer: &JP_SignatureSigner,
) -> JupyterResult<ZmqMessage> {
    let header_bytes = serde_json::to_vec(&message.header)?;
    let parent_header_bytes = match &message.parent_header {
        Some(header) => serde_json::to_vec(header)?,
        None => serde_json::to_vec(&serde_json::json!({}))?,
    };
    let metadata_bytes = serde_json::to_vec(&message.metadata)?;
    let content_bytes = serde_json::to_vec(&message.content)?;

    let signature = signer.sign(&[
        &header_bytes,
        &parent_header_bytes,
        &metadata_bytes,
        &content_bytes,
    ])?;

    let frames_data: Vec<Vec<u8>> = vec![
        message.header.msg_type.as_bytes().to_vec(),
        b"<IDS|MSG>".to_vec(),
        signature.into_bytes(),
        header_bytes,
        parent_header_bytes,
        metadata_bytes,
        content_bytes,
    ];

    if frames_data.is_empty() {
        return Err(Box::from(
            "Cannot create ZMQ message from empty frames_data",
        ));
    }
    let mut zmq_msg = ZmqMessage::from(frames_data[0].clone());
    for frame in frames_data.iter().skip(1) {
        zmq_msg.push_back(frame.clone().into());
    }
    Ok(zmq_msg)
}
