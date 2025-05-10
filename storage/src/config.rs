//! Configuration for storage layers

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for a Q-style storage system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QStoreConfig {
    /// Base directory for storing table data
    pub data_dir: PathBuf,
    /// Table name
    pub table_name: String,
    /// Maximum file size before splitting (bytes)
    pub max_file_size: usize,
    /// Enable compression for column files
    pub enable_compression: bool,
    /// Buffer size for memory-mapped files
    pub mmap_buffer_size: usize,
}

impl QStoreConfig {
    /// Create a new configuration with default values
    pub fn new<P: Into<PathBuf>>(data_dir: P, table_name: String) -> Self {
        Self {
            data_dir: data_dir.into(),
            table_name,
            max_file_size: 1024 * 1024 * 1024, // 1GB default
            enable_compression: false,         // Start simple, add compression later
            mmap_buffer_size: 8192,            // 8KB buffer
        }
    }

    /// Get the path for a specific column file
    pub fn column_path(&self, column_name: &str) -> PathBuf {
        self.data_dir.join(&self.table_name).join(column_name)
    }

    /// Get the table directory path
    pub fn table_path(&self) -> PathBuf {
        self.data_dir.join(&self.table_name)
    }

    /// Set compression enabled/disabled
    pub fn with_compression(mut self, enabled: bool) -> Self {
        self.enable_compression = enabled;
        self
    }

    /// Set maximum file size
    pub fn with_max_file_size(mut self, size: usize) -> Self {
        self.max_file_size = size;
        self
    }

    /// Set memory map buffer size
    pub fn with_mmap_buffer_size(mut self, size: usize) -> Self {
        self.mmap_buffer_size = size;
        self
    }
}

impl Default for QStoreConfig {
    fn default() -> Self {
        Self::new("data", "default_table".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_paths() {
        let temp_dir = TempDir::new().unwrap();
        let config = QStoreConfig::new(temp_dir.path(), "test_table".to_string());

        let table_path = config.table_path();
        assert_eq!(table_path, temp_dir.path().join("test_table"));

        let column_path = config.column_path("price");
        assert_eq!(
            column_path,
            temp_dir.path().join("test_table").join("price")
        );
    }

    #[test]
    fn test_config_builder() {
        let config = QStoreConfig::default()
            .with_compression(true)
            .with_max_file_size(2048)
            .with_mmap_buffer_size(4096);

        assert!(config.enable_compression);
        assert_eq!(config.max_file_size, 2048);
        assert_eq!(config.mmap_buffer_size, 4096);
    }
}
