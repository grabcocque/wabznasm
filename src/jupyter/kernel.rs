use crate::jupyter::IdentityFrames;
use crate::jupyter::connection::{ConnectionConfig, ConnectionConfigExt};
use crate::jupyter::errors::JupyterResult;
use crate::jupyter::handler::WabznasmJupyterKernel;
use crate::jupyter::message_parser::ParsedMessage;
use crate::jupyter::signature::{
    SignatureSigner as JP_SignatureSigner, SignatureVerifier as JP_SignatureVerifier,
};
use jupyter_protocol::{
    Header, JupyterMessageContent, ReplyStatus, ShutdownReply as ProtocolShutdownReply,
    ShutdownRequest, messaging::ExecutionState, messaging::Status as ProtocolStatus,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use zeromq::{PubSocket, RepSocket, RouterSocket, Socket, SocketRecv, SocketSend, ZmqMessage};

// Type aliases to reduce complexity
type ZmqSender = Sender<ZmqMessage>;
type ZmqReceiver = Receiver<ZmqMessage>;

pub struct JupyterKernelRunner {
    config: ConnectionConfig,
    kernel_handler: WabznasmJupyterKernel,
    verifier: Arc<JP_SignatureVerifier>,
    signer: Arc<JP_SignatureSigner>,
    /// Channel sender to the IOPub socket actor
    iopub_sender: ZmqSender,
}

impl JupyterKernelRunner {
    pub fn from_file(connection_file_path: &std::path::Path) -> JupyterResult<Self> {
        let config_obj = ConnectionConfig::from_file(connection_file_path)
            .map_err(|e| format!("Failed to load config: {}", e))?;
        Self::new(config_obj)
    }

    pub fn new(config: ConnectionConfig) -> JupyterResult<Self> {
        let key = config.key.as_bytes();
        let verifier = Arc::new(
            JP_SignatureVerifier::new(config.signature_scheme.clone(), key)
                .map_err(|e| e.to_string())?,
        );
        let signer = Arc::new(
            JP_SignatureSigner::new(config.signature_scheme.clone(), key)
                .map_err(|e| e.to_string())?,
        );
        // Spawn an IOPub actor task that binds and sends messages lock-free via channel
        let iopub_url = config.iopub_url();
        let (iopub_sender, mut iopub_receiver): (ZmqSender, ZmqReceiver) =
            tokio::sync::mpsc::channel(1024);
        // IOPub socket actor
        tokio::spawn(async move {
            let mut socket = PubSocket::new();
            if let Err(e) = socket.bind(&iopub_url).await {
                eprintln!("Failed to bind IOPub socket to {}: {}", iopub_url, e);
                return;
            }
            println!("📢 IOPub socket bound to {}", iopub_url);
            while let Some(msg) = iopub_receiver.recv().await {
                if let Err(e) = socket.send(msg).await {
                    eprintln!("IOPub send error: {}", e);
                }
            }
        });
        // Kernel handler uses the same sender for IOPub messages
        let kernel_handler = WabznasmJupyterKernel::new(iopub_sender.clone(), Arc::clone(&signer));
        Ok(Self {
            config,
            kernel_handler,
            verifier,
            signer,
            iopub_sender,
        })
    }

    async fn send_iopub_status(
        &self,
        parent_header: &Header,
        execution_state: &str,
    ) -> JupyterResult<()> {
        let iopub_header = Header {
            msg_id: uuid::Uuid::new_v4().to_string(),
            session: parent_header.session.clone(),
            username: parent_header.username.clone(),
            date: chrono::Utc::now(),
            msg_type: "status".to_string(),
            version: parent_header.version.clone(),
        };

        let status_content = ProtocolStatus {
            execution_state: match execution_state {
                "busy" => ExecutionState::Busy,
                "idle" => ExecutionState::Idle,
                _ => panic!(
                    "Invalid execution state for IOPub status: {}",
                    execution_state
                ),
            },
        };

        let header_bytes = serde_json::to_vec(&iopub_header)?;
        let parent_header_bytes = serde_json::to_vec(&serde_json::json!({}))?;
        let metadata_bytes = serde_json::to_vec(&serde_json::json!({}))?;
        let content_bytes = serde_json::to_vec(&status_content)?;

        let signature = self.signer.sign(&[
            &header_bytes,
            &parent_header_bytes,
            &metadata_bytes,
            &content_bytes,
        ])?;

        let frames: Vec<Vec<u8>> = vec![
            iopub_header.msg_type.as_bytes().to_vec(),
            b"<IDS|MSG>".to_vec(),
            signature.into_bytes(),
            header_bytes,
            parent_header_bytes,
            metadata_bytes,
            content_bytes,
        ];

        let mut zmq_msg = ZmqMessage::from(frames[0].clone());
        for frame_content in frames.iter().skip(1) {
            zmq_msg.push_back(frame_content.clone().into());
        }

        // Debug: Log the status message content
        if let Ok(status_json) = serde_json::to_string_pretty(&status_content) {
            println!("🔍 Status message content: {}", status_json);
        }
        if let Ok(header_json) = serde_json::to_string_pretty(&iopub_header) {
            println!("🔍 Status header: {}", header_json);
        }

        // Send via the IOPub actor channel
        self.iopub_sender.send(zmq_msg).await?;
        println!("📢 Sent IOPub status: {}", execution_state);
        Ok(())
    }

    pub async fn run(&mut self) -> JupyterResult<()> {
        println!("🚀 Starting Wabznasm Jupyter kernel (custom runner)...");
        let mut shell_socket = RouterSocket::new();
        shell_socket.bind(&self.config.shell_url()).await?;
        println!("🐚 Shell socket bound to {}", self.config.shell_url());
        // IOPub socket is bound and managed by the background actor spawned in new()
        let mut hb_socket = RepSocket::new();
        hb_socket.bind(&self.config.hb_url()).await?;
        println!("💓 Heartbeat socket bound to {}", self.config.hb_url());

        let initial_dummy_header_for_status = Header {
            msg_id: uuid::Uuid::new_v4().to_string(),
            session: uuid::Uuid::new_v4().to_string(),
            username: "kernel".to_string(),
            date: chrono::Utc::now(),
            msg_type: "status".to_string(),
            version: "5.3".to_string(),
        };

        if let Err(e) = self
            .send_iopub_status(&initial_dummy_header_for_status, "busy")
            .await
        {
            eprintln!("❌ Failed to send initial IOPub status (busy): {}", e);
        }

        tokio::spawn(async move {
            loop {
                match hb_socket.recv().await {
                    Ok(msg) => {
                        if let Err(e) = hb_socket.send(msg).await {
                            eprintln!("Heartbeat send error: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Heartbeat recv error: {}", e);
                        break;
                    }
                }
            }
        });
        println!("✅ Kernel is ready for connections.");

        if let Err(e) = self
            .send_iopub_status(&initial_dummy_header_for_status, "idle")
            .await
        {
            eprintln!("❌ Failed to send initial IOPub status (idle): {}", e);
        }

        loop {
            let zmq_msg = shell_socket.recv().await?;
            let parsed_msg = match ParsedMessage::parse(&zmq_msg, &self.verifier) {
                Ok(msg) => msg,
                Err(e) => {
                    eprintln!("Error parsing message: {}", e);
                    continue;
                }
            };
            let parent_header_for_reply = parsed_msg.header.clone();
            let reply_metadata = HashMap::new();
            match parsed_msg.content {
                JupyterMessageContent::KernelInfoRequest(_req_content) => {
                    let reply_header = Header {
                        msg_id: uuid::Uuid::new_v4().to_string(),
                        session: parent_header_for_reply.session.clone(),
                        username: parent_header_for_reply.username.clone(),
                        date: chrono::Utc::now(),
                        msg_type: "kernel_info_reply".to_string(),
                        version: parent_header_for_reply.version.clone(),
                    };

                    let kernel_info_reply_content =
                        self.kernel_handler.kernel_info(&parent_header_for_reply);

                    // Debug: Log the kernel_info_reply content
                    if let Ok(info_json) = serde_json::to_string_pretty(&kernel_info_reply_content)
                    {
                        println!("🔍 Kernel info reply content: {}", info_json);
                    }
                    if let Ok(header_json) = serde_json::to_string_pretty(&reply_header) {
                        println!("🔍 Kernel info reply header: {}", header_json);
                    }

                    let reply_msg = construct_zmq_message(
                        &parsed_msg.identities,
                        &reply_header,
                        Some(&parent_header_for_reply),
                        &reply_metadata,
                        &JupyterMessageContent::KernelInfoReply(Box::new(
                            kernel_info_reply_content,
                        )),
                        &self.signer,
                    )
                    .map_err(|e| {
                        eprintln!("❌ Failed to construct kernel_info_reply: {}", e);
                        e
                    })?;
                    match shell_socket.send(reply_msg).await {
                        Ok(_) => println!("📤 Sent kernel_info_reply successfully"),
                        Err(e) => eprintln!("❌ Failed to send kernel_info_reply: {}", e),
                    }
                }
                JupyterMessageContent::ExecuteRequest(req_content) => {
                    let execute_reply_content = self
                        .kernel_handler
                        .execute_request(req_content, &parent_header_for_reply)
                        .await;
                    let reply_header = Header {
                        msg_id: uuid::Uuid::new_v4().to_string(),
                        session: parent_header_for_reply.session.clone(),
                        username: parent_header_for_reply.username.clone(),
                        date: chrono::Utc::now(),
                        msg_type: "execute_reply".to_string(),
                        version: parent_header_for_reply.version.clone(),
                    };
                    let reply_msg = construct_zmq_message(
                        &parsed_msg.identities,
                        &reply_header,
                        Some(&parent_header_for_reply),
                        &reply_metadata,
                        &JupyterMessageContent::ExecuteReply(execute_reply_content),
                        &self.signer,
                    )?;
                    shell_socket.send(reply_msg).await?;
                }
                JupyterMessageContent::ShutdownRequest(req_content) => {
                    let shutdown_request_struct = ShutdownRequest {
                        restart: req_content.restart,
                    };
                    let handler_shutdown_reply = self
                        .kernel_handler
                        .shutdown_request(shutdown_request_struct, &parent_header_for_reply);
                    let protocol_shutdown_reply = ProtocolShutdownReply {
                        restart: handler_shutdown_reply.restart,
                        status: ReplyStatus::Ok,
                        error: None,
                    };
                    let reply_header = Header {
                        msg_id: uuid::Uuid::new_v4().to_string(),
                        session: parent_header_for_reply.session.clone(),
                        username: parent_header_for_reply.username.clone(),
                        date: chrono::Utc::now(),
                        msg_type: "shutdown_reply".to_string(),
                        version: parent_header_for_reply.version.clone(),
                    };
                    let reply_msg = construct_zmq_message(
                        &parsed_msg.identities,
                        &reply_header,
                        Some(&parent_header_for_reply),
                        &reply_metadata,
                        &JupyterMessageContent::ShutdownReply(protocol_shutdown_reply),
                        &self.signer,
                    )?;
                    shell_socket.send(reply_msg).await?;
                    println!("Kernel shutdown requested.");
                    break;
                }
                JupyterMessageContent::InterruptRequest(_) => {
                    let reply_header = Header {
                        msg_id: uuid::Uuid::new_v4().to_string(),
                        session: parent_header_for_reply.session.clone(),
                        username: parent_header_for_reply.username.clone(),
                        date: chrono::Utc::now(),
                        msg_type: "interrupt_reply".to_string(),
                        version: parent_header_for_reply.version.clone(),
                    };
                    let interrupt_reply = ProtocolShutdownReply {
                        restart: false,
                        status: ReplyStatus::Ok,
                        error: None,
                    };
                    let reply_msg = construct_zmq_message(
                        &parsed_msg.identities,
                        &reply_header,
                        Some(&parent_header_for_reply),
                        &reply_metadata,
                        &JupyterMessageContent::ShutdownReply(interrupt_reply),
                        &self.signer,
                    )?;
                    shell_socket.send(reply_msg).await?;
                }
                _ => {
                    println!("⚠️  Unhandled message type: {}", parsed_msg.header.msg_type);
                }
            }
        }
        Ok(())
    }
}

fn construct_zmq_message(
    identities: &IdentityFrames,
    header: &Header,
    parent_header: Option<&Header>,
    metadata: &HashMap<String, serde_json::Value>,
    content: &JupyterMessageContent,
    signer: &JP_SignatureSigner,
) -> JupyterResult<ZmqMessage> {
    let header_bytes = serde_json::to_vec(header)?;
    let parent_header_bytes = match parent_header {
        Some(ph) => serde_json::to_vec(ph)?,
        None => serde_json::to_vec(&serde_json::json!({}))?,
    };
    let metadata_bytes = serde_json::to_vec(metadata)?;
    let content_bytes = serde_json::to_vec(content)?;
    let signature = signer.sign(&[
        &header_bytes,
        &parent_header_bytes,
        &metadata_bytes,
        &content_bytes,
    ])?;
    let mut frames: Vec<Vec<u8>> = identities.to_vec();
    frames.push(b"<IDS|MSG>".to_vec());
    frames.push(signature.into_bytes());
    frames.push(header_bytes);
    frames.push(parent_header_bytes);
    frames.push(metadata_bytes);
    frames.push(content_bytes);
    if frames.is_empty() {
        return Err("No frames to send".into());
    }

    let mut zmq_msg = ZmqMessage::from(frames[0].clone());
    for frame_content in frames.iter().skip(1) {
        zmq_msg.push_back(frame_content.clone().into());
    }
    Ok(zmq_msg)
}
