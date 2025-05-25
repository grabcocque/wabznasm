pub mod connection;
pub mod display;
pub mod errors;
pub mod handler; // This will contain the JupyterKernelProtocol implementation
pub mod kernel; // Restored for low-level jupyter-protocol approach
pub mod message_parser;
pub mod session;
pub mod signature;
