pub mod connection;
pub mod display;
pub mod errors;
pub mod handler; // This will contain the JupyterKernelProtocol implementation
pub mod kernel; // Restored for low-level jupyter-protocol approach
pub mod message_parser;
pub mod session;
pub mod signature;

// Type aliases to reduce complexity warnings
pub type DynError = Box<dyn std::error::Error>;
pub type DynSendSyncError = Box<dyn std::error::Error + Send + Sync>;
pub type ByteSlice<'a> = &'a [u8];
pub type ByteSlices<'a> = &'a [&'a [u8]];
pub type IdentityFrames = Vec<Vec<u8>>;
