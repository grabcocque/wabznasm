use crate::jupyter::errors::JupyterResult;
use crate::jupyter::signature::SignatureVerifier;
use crate::jupyter::{ByteSlice, IdentityFrames};
use jupyter_protocol::{Header, JupyterMessageContent};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use zeromq::ZmqMessage;

/// Represents a fully parsed Jupyter message, including routing and deserialized content.
#[derive(Debug, Clone)]
pub struct ParsedMessage {
    /// Message identities (routing info from ZMQ).
    pub identities: IdentityFrames,
    /// HMAC signature of the message parts.
    pub signature: String,
    /// Deserialized message header.
    pub header: Header,
    /// Deserialized parent message header (if present).
    pub parent_header: Option<Header>, // Or JsonValue if keeping it raw from protocol
    /// Deserialized message metadata.
    pub metadata: HashMap<String, JsonValue>, // Or a typed Metadata struct if preferred
    /// Deserialized and typed message content.
    pub content: JupyterMessageContent,
}

impl ParsedMessage {
    /// Parse a raw ZeroMQ message into a structured ParsedMessage.
    /// This involves validating the signature and deserializing message parts.
    pub fn parse(
        zmq_msg: &ZmqMessage,
        verifier: &SignatureVerifier, // For HMAC validation
    ) -> JupyterResult<Self> {
        let frames: Vec<ByteSlice> = zmq_msg.iter().map(|bytes| bytes.as_ref()).collect();

        if frames.len() < 5 {
            // Identities can be empty, then <IDS|MSG>, sig, header, parent, meta, content
            return Err(format!(
                "Invalid message: expected at least 5 frames after identities + delimiter, got {}",
                frames.len()
            )
            .into());
        }

        let delimiter_pos = frames
            .iter()
            .position(|frame| *frame == b"<IDS|MSG>")
            .ok_or("Missing delimiter '<IDS|MSG>'")?;

        let identities: IdentityFrames =
            frames[..delimiter_pos].iter().map(|f| f.to_vec()).collect();

        let signature_bytes = frames
            .get(delimiter_pos + 1)
            .ok_or("Missing signature frame")?;
        let header_bytes = frames
            .get(delimiter_pos + 2)
            .ok_or("Missing header frame")?;
        let parent_header_bytes = frames
            .get(delimiter_pos + 3)
            .ok_or("Missing parent_header frame")?;
        let metadata_bytes = frames
            .get(delimiter_pos + 4)
            .ok_or("Missing metadata frame")?;
        let content_bytes = frames
            .get(delimiter_pos + 5)
            .ok_or("Missing content frame")?;

        // Verify signature - convert signature_bytes to &str for verify method
        let signature_str = std::str::from_utf8(signature_bytes)?;
        let signature_valid = verifier.verify(
            signature_str,
            &[
                header_bytes,
                parent_header_bytes,
                metadata_bytes,
                content_bytes,
            ],
        )?;

        if !signature_valid {
            return Err("Signature verification failed".into());
        }

        let signature = signature_str.to_string(); // Store the verified signature string

        // Deserialize parts
        let header: Header = serde_json::from_slice(header_bytes)?;

        // Parent header can be an empty dict {} if not present
        let parent_header_json: JsonValue = serde_json::from_slice(parent_header_bytes)?;
        let parent_header = if parent_header_json.is_object()
            && parent_header_json.as_object().unwrap().is_empty()
        {
            None
        } else {
            Some(serde_json::from_value(parent_header_json)?) // Attempt to parse as Header
        };

        let metadata: HashMap<String, JsonValue> = serde_json::from_slice(metadata_bytes)?;

        // Deserialize content based on header.msg_type
        // jupyter-protocol 0.6.0 has JupyterMessageContent::from_type_and_content
        let content_json: JsonValue = serde_json::from_slice(content_bytes)?;
        let content = JupyterMessageContent::from_type_and_content(&header.msg_type, content_json)?;

        Ok(ParsedMessage {
            identities,
            signature,
            header,
            parent_header,
            metadata,
            content,
        })
    }

    // Helper to create a ZmqMessage from a ParsedMessage (or rather, from its components)
    // This would be used by the kernel_runner to send replies.
    // It will take Header, parent_header (Option<Header>), metadata, content (JupyterMessageContent)
    // and an hmac_key to sign the message.
    // This is a more complex function involving serialization and signing.
    // For now, we focus on parsing.
}

// It might also be useful to have a function to construct a ZmqMessage for sending.
// pub fn construct_zmq_message(
//     identities: &[Vec<u8>],
//     header: &Header,
//     parent_header: Option<&Header>,
//     metadata: &HashMap<String, JsonValue>,
//     content: &JupyterMessageContent,
//     signer: &jupyter_protocol::SignatureSigner, // For signing
// ) -> Result<ZmqMessage, Box<dyn std::error::Error + Send + Sync>> {
// Serialize header, parent_header (or empty dict), metadata, content
// Sign the parts
// Construct ZmqMessage with identities, <IDS|MSG>, sig, and serialized parts
//    Err("Not yet implemented".into())
// }

#[cfg(test)]
mod tests {
    // Tests would require a mock SignatureVerifier and example ZmqMessages.
    // Also, construction of ZmqMessage for testing would need a SignatureSigner.
}
