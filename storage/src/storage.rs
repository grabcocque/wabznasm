//! Core storage implementation with memory-mapped splayed tables

use crate::{
    config::QStoreConfig,
    error::{StorageError, StorageResult},
    table::Row,
    value::ScalarValue,
};
use memmap2::{Mmap, MmapOptions};
use std::{
    collections::HashMap,
    fs::{create_dir_all, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
};

/// Column data stored in memory-mapped files
struct ColumnData {
    /// Memory-mapped file for reading
    mmap: Option<Mmap>,
    /// File handle for writing
    file: File,
    /// Path to the column file
    path: PathBuf,
    /// Number of values in this column
    count: usize,
}

/// Splayed table storage - one file per column
pub struct SplayedTable {
    config: QStoreConfig,
    columns: HashMap<String, ColumnData>,
    row_count: usize,
}

impl SplayedTable {
    /// Create a new splayed table
    pub fn new(config: QStoreConfig) -> StorageResult<Self> {
        // Create table directory
        let table_path = config.table_path();
        create_dir_all(&table_path)?;

        Ok(Self {
            config,
            columns: HashMap::new(),
            row_count: 0,
        })
    }

    /// Open an existing splayed table
    pub fn open(config: QStoreConfig) -> StorageResult<Self> {
        let table_path = config.table_path();
        if !table_path.exists() {
            return Err(StorageError::Configuration(
                format!("Table directory does not exist: {:?}", table_path)
            ));
        }

        let mut columns = HashMap::new();
        let mut row_count = 0;

        // Scan directory for column files
        for entry in std::fs::read_dir(&table_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(column_name) = path.file_name().and_then(|n| n.to_str()) {
                    let file = OpenOptions::new()
                        .read(true)
                        .append(true)
                        .open(&path)?;

                    // Count entries in this column file to determine row count
                    let count = Self::count_entries_in_file(&path)?;
                    row_count = row_count.max(count);

                    let mmap = if file.metadata()?.len() > 0 {
                        Some(unsafe { MmapOptions::new().map(&file)? })
                    } else {
                        None
                    };

                    columns.insert(column_name.to_string(), ColumnData {
                        mmap,
                        file,
                        path: path.clone(),
                        count,
                    });
                }
            }
        }

        Ok(Self {
            config,
            columns,
            row_count,
        })
    }

    /// Get the number of rows in the table
    pub fn count(&self) -> StorageResult<usize> {
        Ok(self.row_count)
    }

    /// Insert a row into the table
    pub fn put(&mut self, row: Row) -> StorageResult<()> {
        // Ensure all columns exist
        for column_name in row.keys() {
            if !self.columns.contains_key(column_name) {
                self.ensure_column_exists(column_name)?;
            }
        }

        // Collect column names to avoid borrow checker issues
        let column_names: Vec<String> = self.columns.keys().cloned().collect();

        // Write values to each column file
        for column_name in column_names {
            let value = row.get(&column_name).cloned().unwrap_or(ScalarValue::Null);
            if let Some(column_data) = self.columns.get_mut(&column_name) {
                Self::write_value_to_column_static(column_data, &value)?;
            }
        }

        self.row_count += 1;
        Ok(())
    }

    /// Get a row by index
    pub fn get(&self, index: usize) -> StorageResult<Row> {
        if index >= self.row_count {
            return Err(StorageError::InvalidRowIndex {
                index,
                max: self.row_count
            });
        }

        let mut row = Row::new();

        for (column_name, column_data) in &self.columns {
            let value = self.read_value_from_column(column_data, index)?;
            row.insert(column_name.clone(), value);
        }

        Ok(row)
    }

    /// Ensure a column file exists
    fn ensure_column_exists(&mut self, column_name: &str) -> StorageResult<()> {
        if self.columns.contains_key(column_name) {
            return Ok(());
        }

        let column_path = self.config.column_path(column_name);
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(&column_path)?;

        self.columns.insert(column_name.to_string(), ColumnData {
            mmap: None,
            file,
            path: column_path,
            count: 0,
        });

        Ok(())
    }

    /// Write a value to a column file using MessagePack
    fn write_value_to_column_static(column_data: &mut ColumnData, value: &ScalarValue) -> StorageResult<()> {
        let encoded = rmp_serde::to_vec(value)?;

        // Write length prefix (4 bytes) followed by data
        let len_bytes = (encoded.len() as u32).to_le_bytes();
        column_data.file.write_all(&len_bytes)?;
        column_data.file.write_all(&encoded)?;
        column_data.file.flush()?;

        column_data.count += 1;

        // Invalidate mmap since file has been modified
        column_data.mmap = None;

        Ok(())
    }

    /// Read a value from a column file
    fn read_value_from_column(&self, column_data: &ColumnData, index: usize) -> StorageResult<ScalarValue> {
        if index >= column_data.count {
            return Ok(ScalarValue::Null);
        }

        // For now, read from file directly (not optimized)
        // In a full implementation, we'd use the mmap for reading
        let mut file = File::open(&column_data.path)?;

        // Skip to the target entry
        for _ in 0..index {
            // Read length prefix
            let mut len_bytes = [0u8; 4];
            file.read_exact(&mut len_bytes)?;
            let len = u32::from_le_bytes(len_bytes) as usize;

            // Skip the data
            file.seek(SeekFrom::Current(len as i64))?;
        }

        // Read the target entry
        let mut len_bytes = [0u8; 4];
        file.read_exact(&mut len_bytes)?;
        let len = u32::from_le_bytes(len_bytes) as usize;

        let mut data = vec![0u8; len];
        file.read_exact(&mut data)?;

        let value: ScalarValue = rmp_serde::from_slice(&data)?;
        Ok(value)
    }

    /// Count entries in a column file
    fn count_entries_in_file(path: &PathBuf) -> StorageResult<usize> {
        let mut file = File::open(path)?;
        let mut count = 0;

        loop {
            // Try to read length prefix
            let mut len_bytes = [0u8; 4];
            match file.read_exact(&mut len_bytes) {
                Ok(()) => {
                    let len = u32::from_le_bytes(len_bytes) as usize;
                    // Skip the data
                    file.seek(SeekFrom::Current(len as i64))?;
                    count += 1;
                }
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(StorageError::Io(e)),
            }
        }

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_config() -> (QStoreConfig, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = QStoreConfig::new(temp_dir.path(), "test_table".to_string());
        (config, temp_dir)
    }

    #[test]
    fn test_splayed_table_creation() {
        let (config, _temp_dir) = create_test_config();
        let table = SplayedTable::new(config).unwrap();
        assert_eq!(table.count().unwrap(), 0);
    }

    #[test]
    fn test_splayed_table_insert_and_get() {
        let (config, _temp_dir) = create_test_config();
        let mut table = SplayedTable::new(config).unwrap();

        let mut row = Row::new();
        row.insert("id".to_string(), ScalarValue::Int64(1));
        row.insert("name".to_string(), ScalarValue::Utf8("test".to_string()));

        table.put(row.clone()).unwrap();
        assert_eq!(table.count().unwrap(), 1);

        let retrieved_row = table.get(0).unwrap();
        assert_eq!(retrieved_row.get("id"), row.get("id"));
        assert_eq!(retrieved_row.get("name"), row.get("name"));
    }

    #[test]
    fn test_splayed_table_multiple_rows() {
        let (config, _temp_dir) = create_test_config();
        let mut table = SplayedTable::new(config).unwrap();

        // Insert multiple rows
        for i in 0..10 {
            let mut row = Row::new();
            row.insert("id".to_string(), ScalarValue::Int64(i));
            row.insert("value".to_string(), ScalarValue::Float64(i as f64 * 3.14));
            table.put(row).unwrap();
        }

        assert_eq!(table.count().unwrap(), 10);

        // Check specific rows
        let row_5 = table.get(5).unwrap();
        assert_eq!(row_5.get("id"), Some(&ScalarValue::Int64(5)));
        assert_eq!(row_5.get("value"), Some(&ScalarValue::Float64(5.0 * 3.14)));
    }

    #[test]
    fn test_splayed_table_open_existing() {
        let (config, _temp_dir) = create_test_config();

        // Create and populate table
        {
            let mut table = SplayedTable::new(config.clone()).unwrap();
            let mut row = Row::new();
            row.insert("test".to_string(), ScalarValue::Utf8("hello".to_string()));
            table.put(row).unwrap();
        }

        // Open existing table
        let table = SplayedTable::open(config).unwrap();
        assert_eq!(table.count().unwrap(), 1);

        let row = table.get(0).unwrap();
        assert_eq!(row.get("test"), Some(&ScalarValue::Utf8("hello".to_string())));
    }
}
