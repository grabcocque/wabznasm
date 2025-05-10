//! High-level table interface

use crate::{
    config::QStoreConfig,
    error::{StorageError, StorageResult},
    schema::TableSchema,
    storage::SplayedTable,
    value::ScalarValue,
};
use std::collections::HashMap;

/// A row of data (column name -> value mapping)
pub type Row = HashMap<String, ScalarValue>;

/// High-level table interface that wraps SplayedTable
pub struct Table {
    schema: TableSchema,
    storage: SplayedTable,
}

impl Table {
    /// Create a new table with the given schema and configuration
    pub fn new(schema: TableSchema, config: QStoreConfig) -> StorageResult<Self> {
        let storage = SplayedTable::new(config)?;
        Ok(Self { schema, storage })
    }

    /// Open an existing table
    pub fn open(schema: TableSchema, config: QStoreConfig) -> StorageResult<Self> {
        let storage = SplayedTable::open(config)?;
        Ok(Self { schema, storage })
    }

    /// Get the table schema
    pub fn schema(&self) -> &TableSchema {
        &self.schema
    }

    /// Get the number of rows in the table
    pub fn row_count(&self) -> StorageResult<usize> {
        self.storage.count()
    }

    /// Insert a row into the table
    pub fn insert(&mut self, row: Row) -> StorageResult<()> {
        // Validate that the row matches the schema
        for column in &self.schema.columns {
            if let Some(value) = row.get(&column.name) {
                if value.simple_data_type() != column.data_type && !value.is_null() {
                    return Err(StorageError::SchemaMismatch {
                        expected: format!("{:?}", column.data_type),
                        actual: format!("{:?}", value.simple_data_type()),
                    });
                }
            } else if !column.nullable {
                return Err(StorageError::SchemaMismatch {
                    expected: format!("non-null value for column {}", column.name),
                    actual: "missing value".to_string(),
                });
            }
        }

        self.storage.put(row)
    }

    /// Get a row by index
    pub fn get(&self, index: usize) -> StorageResult<Row> {
        let row_count = self.row_count()?;
        if index >= row_count {
            return Err(StorageError::InvalidRowIndex {
                index,
                max: row_count,
            });
        }

        self.storage.get(index)
    }

    /// Get a specific column value by row index and column name
    pub fn get_value(&self, row_index: usize, column_name: &str) -> StorageResult<ScalarValue> {
        // Check if column exists in schema
        if self.schema.get_column(column_name).is_none() {
            return Err(StorageError::ColumnNotFound(column_name.to_string()));
        }

        let row = self.get(row_index)?;
        Ok(row.get(column_name).cloned().unwrap_or(ScalarValue::Null))
    }

    /// Get all values for a specific column
    pub fn get_column(&self, column_name: &str) -> StorageResult<Vec<ScalarValue>> {
        // Check if column exists in schema
        if self.schema.get_column(column_name).is_none() {
            return Err(StorageError::ColumnNotFound(column_name.to_string()));
        }

        let row_count = self.row_count()?;
        let mut values = Vec::with_capacity(row_count);

        for i in 0..row_count {
            let value = self.get_value(i, column_name)?;
            values.push(value);
        }

        Ok(values)
    }

    /// Iterate over all rows
    pub fn iter(&self) -> StorageResult<TableIterator> {
        let row_count = self.row_count()?;
        Ok(TableIterator {
            table: self,
            current_index: 0,
            max_index: row_count,
        })
    }

    /// Filter rows based on a predicate
    pub fn filter<F>(&self, predicate: F) -> StorageResult<Vec<Row>>
    where
        F: Fn(&Row) -> bool,
    {
        let mut results = Vec::new();
        let row_count = self.row_count()?;

        for i in 0..row_count {
            let row = self.get(i)?;
            if predicate(&row) {
                results.push(row);
            }
        }

        Ok(results)
    }

    /// Get basic statistics about the table
    pub fn stats(&self) -> StorageResult<TableStats> {
        let row_count = self.row_count()?;
        let column_count = self.schema.column_count();

        Ok(TableStats {
            row_count,
            column_count,
            column_names: self
                .schema
                .column_names()
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
        })
    }
}

/// Iterator over table rows
pub struct TableIterator<'a> {
    table: &'a Table,
    current_index: usize,
    max_index: usize,
}

impl<'a> Iterator for TableIterator<'a> {
    type Item = StorageResult<Row>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index >= self.max_index {
            None
        } else {
            let result = self.table.get(self.current_index);
            self.current_index += 1;
            Some(result)
        }
    }
}

/// Basic statistics about a table
#[derive(Debug, Clone)]
pub struct TableStats {
    pub row_count: usize,
    pub column_count: usize,
    pub column_names: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::SchemaBuilder;
    use tempfile::TempDir;

    fn create_test_table() -> (Table, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = QStoreConfig::new(temp_dir.path(), "test_table".to_string());
        let schema = SchemaBuilder::time_series();
        let table = Table::new(schema, config).unwrap();
        (table, temp_dir)
    }

    #[test]
    fn test_table_creation() {
        let (table, _temp_dir) = create_test_table();
        assert_eq!(table.schema().column_count(), 2);
        assert_eq!(table.row_count().unwrap(), 0);
    }

    #[test]
    fn test_table_insert_and_get() {
        let (mut table, _temp_dir) = create_test_table();

        let mut row = Row::new();
        row.insert("time".to_string(), ScalarValue::Timestamp(1000000000));
        row.insert("value".to_string(), ScalarValue::Float64(3.14));

        table.insert(row.clone()).unwrap();
        assert_eq!(table.row_count().unwrap(), 1);

        let retrieved_row = table.get(0).unwrap();
        assert_eq!(retrieved_row.get("time"), row.get("time"));
        assert_eq!(retrieved_row.get("value"), row.get("value"));
    }

    #[test]
    fn test_table_column_access() {
        let (mut table, _temp_dir) = create_test_table();

        let mut row = Row::new();
        row.insert("time".to_string(), ScalarValue::Timestamp(1000000000));
        row.insert("value".to_string(), ScalarValue::Float64(3.14));
        table.insert(row).unwrap();

        let value = table.get_value(0, "value").unwrap();
        assert_eq!(value, ScalarValue::Float64(3.14));

        let column_values = table.get_column("value").unwrap();
        assert_eq!(column_values.len(), 1);
        assert_eq!(column_values[0], ScalarValue::Float64(3.14));
    }

    #[test]
    fn test_table_schema_validation() {
        let (mut table, _temp_dir) = create_test_table();

        // Try to insert a row with wrong type
        let mut row = Row::new();
        row.insert(
            "time".to_string(),
            ScalarValue::Utf8("not a timestamp".to_string()),
        );
        row.insert("value".to_string(), ScalarValue::Float64(3.14));

        let result = table.insert(row);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            StorageError::SchemaMismatch { .. }
        ));
    }

    #[test]
    fn test_table_stats() {
        let (mut table, _temp_dir) = create_test_table();

        let mut row = Row::new();
        row.insert("time".to_string(), ScalarValue::Timestamp(1000000000));
        row.insert("value".to_string(), ScalarValue::Float64(3.14));
        table.insert(row).unwrap();

        let stats = table.stats().unwrap();
        assert_eq!(stats.row_count, 1);
        assert_eq!(stats.column_count, 2);
        assert_eq!(stats.column_names, vec!["time", "value"]);
    }
}
