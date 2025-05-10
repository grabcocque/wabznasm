//! Memory-mapped columnar storage for wabznasm
//!
//! This crate provides a high-performance storage layer for wabznasm using:
//! - Arrow2 for columnar data layouts and type system
//! - MessagePack for fast binary serialization
//! - Memory-mapped files for zero-copy data access
//! - Splayed table format (one file per column)

pub mod config;
pub mod error;
pub mod schema;
pub mod storage;
pub mod table;
pub mod value;

pub use config::QStoreConfig;
pub use error::{StorageError, StorageResult};
pub use schema::{ColumnSchema, TableSchema};
pub use storage::SplayedTable;
pub use table::Table;
pub use value::ScalarValue;
