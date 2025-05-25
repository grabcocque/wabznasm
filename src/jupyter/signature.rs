use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::fmt;

pub type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone)]
pub struct SignatureVerifier {
    key: Vec<u8>,
    scheme: String,
}

#[derive(Debug, Clone)]
pub struct SignatureSigner {
    key: Vec<u8>,
    scheme: String,
}

#[derive(Debug)]
pub enum SignatureError {
    InvalidScheme(String),
    InvalidSignature,
    MacError(String),
}

impl fmt::Display for SignatureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SignatureError::InvalidScheme(scheme) => {
                write!(f, "Invalid signature scheme: {}", scheme)
            }
            SignatureError::InvalidSignature => write!(f, "Invalid signature"),
            SignatureError::MacError(msg) => write!(f, "MAC error: {}", msg),
        }
    }
}

impl std::error::Error for SignatureError {}

impl SignatureVerifier {
    pub fn new(scheme: String, key: &[u8]) -> Result<Self, SignatureError> {
        match scheme.as_str() {
            "hmac-sha256" => Ok(Self {
                key: key.to_vec(),
                scheme,
            }),
            _ => Err(SignatureError::InvalidScheme(scheme)),
        }
    }

    pub fn verify(&self, signature: &str, message_parts: &[&[u8]]) -> Result<bool, SignatureError> {
        match self.scheme.as_str() {
            "hmac-sha256" => {
                let mut mac = HmacSha256::new_from_slice(&self.key)
                    .map_err(|e| SignatureError::MacError(e.to_string()))?;

                for part in message_parts {
                    mac.update(part);
                }

                let expected_signature = hex::encode(mac.finalize().into_bytes());
                Ok(expected_signature == signature)
            }
            _ => Err(SignatureError::InvalidScheme(self.scheme.clone())),
        }
    }
}

impl SignatureSigner {
    pub fn new(scheme: String, key: &[u8]) -> Result<Self, SignatureError> {
        match scheme.as_str() {
            "hmac-sha256" => Ok(Self {
                key: key.to_vec(),
                scheme,
            }),
            _ => Err(SignatureError::InvalidScheme(scheme)),
        }
    }

    pub fn sign(&self, message_parts: &[&[u8]]) -> Result<String, SignatureError> {
        match self.scheme.as_str() {
            "hmac-sha256" => {
                let mut mac = HmacSha256::new_from_slice(&self.key)
                    .map_err(|e| SignatureError::MacError(e.to_string()))?;

                for part in message_parts {
                    mac.update(part);
                }

                Ok(hex::encode(mac.finalize().into_bytes()))
            }
            _ => Err(SignatureError::InvalidScheme(self.scheme.clone())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_round_trip() {
        let key = b"test-key";
        let scheme = "hmac-sha256".to_string();

        let signer = SignatureSigner::new(scheme.clone(), key).unwrap();
        let verifier = SignatureVerifier::new(scheme, key).unwrap();

        let message_parts = vec![b"hello".as_slice(), b"world".as_slice()];
        let signature = signer.sign(&message_parts).unwrap();

        assert!(verifier.verify(&signature, &message_parts).unwrap());
    }

    #[test]
    fn test_signature_mismatch() {
        let key = b"test-key";
        let scheme = "hmac-sha256".to_string();

        let signer = SignatureSigner::new(scheme.clone(), key).unwrap();
        let verifier = SignatureVerifier::new(scheme, key).unwrap();

        let message_parts = vec![b"hello".as_slice(), b"world".as_slice()];
        let signature = signer.sign(&message_parts).unwrap();

        let wrong_parts = vec![b"hello".as_slice(), b"wrong".as_slice()];
        assert!(!verifier.verify(&signature, &wrong_parts).unwrap());
    }
}
