//! Schema definitions for tables and columns

use arrow2::datatypes::{DataType, Field};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Simplified data type enum for serialization
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SimpleDataType {
    Null,
    Boolean,
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float32,
    Float64,
    Utf8,
    Binary,
    Timestamp,
}

impl From<SimpleDataType> for DataType {
    fn from(simple: SimpleDataType) -> Self {
        match simple {
            SimpleDataType::Null => DataType::Null,
            SimpleDataType::Boolean => DataType::Boolean,
            SimpleDataType::Int8 => DataType::Int8,
            SimpleDataType::Int16 => DataType::Int16,
            SimpleDataType::Int32 => DataType::Int32,
            SimpleDataType::Int64 => DataType::Int64,
            SimpleDataType::UInt8 => DataType::UInt8,
            SimpleDataType::UInt16 => DataType::UInt16,
            SimpleDataType::UInt32 => DataType::UInt32,
            SimpleDataType::UInt64 => DataType::UInt64,
            SimpleDataType::Float32 => DataType::Float32,
            SimpleDataType::Float64 => DataType::Float64,
            SimpleDataType::Utf8 => DataType::Utf8,
            SimpleDataType::Binary => DataType::Binary,
            SimpleDataType::Timestamp => {
                DataType::Timestamp(arrow2::datatypes::TimeUnit::Nanosecond, None)
            }
        }
    }
}

impl From<&DataType> for SimpleDataType {
    fn from(dt: &DataType) -> Self {
        match dt {
            DataType::Null => SimpleDataType::Null,
            DataType::Boolean => SimpleDataType::Boolean,
            DataType::Int8 => SimpleDataType::Int8,
            DataType::Int16 => SimpleDataType::Int16,
            DataType::Int32 => SimpleDataType::Int32,
            DataType::Int64 => SimpleDataType::Int64,
            DataType::UInt8 => SimpleDataType::UInt8,
            DataType::UInt16 => SimpleDataType::UInt16,
            DataType::UInt32 => SimpleDataType::UInt32,
            DataType::UInt64 => SimpleDataType::UInt64,
            DataType::Float32 => SimpleDataType::Float32,
            DataType::Float64 => SimpleDataType::Float64,
            DataType::Utf8 => SimpleDataType::Utf8,
            DataType::Binary => SimpleDataType::Binary,
            DataType::Timestamp(_, _) => SimpleDataType::Timestamp,
            _ => SimpleDataType::Binary, // Default fallback
        }
    }
}

/// Schema for a single column
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColumnSchema {
    /// Column name
    pub name: String,
    /// Simplified data type for serialization
    pub data_type: SimpleDataType,
    /// Whether the column allows null values
    pub nullable: bool,
    /// Optional metadata
    pub metadata: HashMap<String, String>,
}

impl ColumnSchema {
    /// Create a new column schema
    pub fn new(name: String, data_type: DataType) -> Self {
        Self {
            name,
            data_type: SimpleDataType::from(&data_type),
            nullable: true,
            metadata: HashMap::new(),
        }
    }

    /// Create a new column schema with SimpleDataType
    pub fn new_simple(name: String, data_type: SimpleDataType) -> Self {
        Self {
            name,
            data_type,
            nullable: true,
            metadata: HashMap::new(),
        }
    }

    /// Get the Arrow2 DataType
    pub fn arrow_data_type(&self) -> DataType {
        self.data_type.clone().into()
    }

    /// Set nullable flag
    pub fn with_nullable(mut self, nullable: bool) -> Self {
        self.nullable = nullable;
        self
    }

    /// Add metadata
    pub fn with_metadata<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

/// Schema for a table (collection of columns)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableSchema {
    /// Table name
    pub name: String,
    /// Ordered list of columns
    pub columns: Vec<ColumnSchema>,
    /// Table-level metadata
    pub metadata: HashMap<String, String>,
}

impl TableSchema {
    /// Create a new empty table schema
    pub fn new(name: String) -> Self {
        Self {
            name,
            columns: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a column to the schema
    pub fn add_column(mut self, column: ColumnSchema) -> Self {
        self.columns.push(column);
        self
    }

    /// Add metadata
    pub fn with_metadata<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Get column by name
    pub fn get_column(&self, name: &str) -> Option<&ColumnSchema> {
        self.columns.iter().find(|col| col.name == name)
    }

    /// Get column index by name
    pub fn get_column_index(&self, name: &str) -> Option<usize> {
        self.columns.iter().position(|col| col.name == name)
    }

    /// Get all column names
    pub fn column_names(&self) -> Vec<&str> {
        self.columns.iter().map(|col| col.name.as_str()).collect()
    }

    /// Get the number of columns
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Convert to Arrow2 schema
    pub fn to_arrow_schema(&self) -> arrow2::datatypes::Schema {
        let fields: Vec<Field> = self
            .columns
            .iter()
            .map(|col| Field::new(&col.name, col.arrow_data_type(), col.nullable))
            .collect();

        arrow2::datatypes::Schema::from(fields)
    }
}

/// Builder for creating common table schemas
pub struct SchemaBuilder;

impl SchemaBuilder {
    /// Create a time series schema with timestamp and value columns
    pub fn time_series() -> TableSchema {
        TableSchema::new("timeseries".to_string())
            .add_column(
                ColumnSchema::new_simple("time".to_string(), SimpleDataType::Timestamp)
                    .with_nullable(false),
            )
            .add_column(ColumnSchema::new_simple(
                "value".to_string(),
                SimpleDataType::Float64,
            ))
    }

    /// Create a market data schema
    pub fn market_data() -> TableSchema {
        TableSchema::new("market_data".to_string())
            .add_column(
                ColumnSchema::new_simple("time".to_string(), SimpleDataType::Timestamp)
                    .with_nullable(false),
            )
            .add_column(ColumnSchema::new_simple(
                "symbol".to_string(),
                SimpleDataType::Utf8,
            ))
            .add_column(ColumnSchema::new_simple(
                "price".to_string(),
                SimpleDataType::Float64,
            ))
            .add_column(ColumnSchema::new_simple(
                "size".to_string(),
                SimpleDataType::Int64,
            ))
            .add_column(ColumnSchema::new_simple(
                "side".to_string(),
                SimpleDataType::Utf8,
            ))
    }

    /// Create a graph nodes schema
    pub fn graph_nodes() -> TableSchema {
        TableSchema::new("nodes".to_string())
            .add_column(
                ColumnSchema::new_simple("id".to_string(), SimpleDataType::Int64)
                    .with_nullable(false),
            )
            .add_column(ColumnSchema::new_simple(
                "name".to_string(),
                SimpleDataType::Utf8,
            ))
            .add_column(ColumnSchema::new_simple(
                "type".to_string(),
                SimpleDataType::Utf8,
            ))
    }

    /// Create a graph edges schema
    pub fn graph_edges() -> TableSchema {
        TableSchema::new("edges".to_string())
            .add_column(
                ColumnSchema::new_simple("src".to_string(), SimpleDataType::Int64)
                    .with_nullable(false),
            )
            .add_column(
                ColumnSchema::new_simple("dst".to_string(), SimpleDataType::Int64)
                    .with_nullable(false),
            )
            .add_column(ColumnSchema::new_simple(
                "weight".to_string(),
                SimpleDataType::Float64,
            ))
            .add_column(ColumnSchema::new_simple(
                "label".to_string(),
                SimpleDataType::Utf8,
            ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_schema() {
        let col = ColumnSchema::new_simple("price".to_string(), SimpleDataType::Float64)
            .with_nullable(false)
            .with_metadata("unit", "USD");

        assert_eq!(col.name, "price");
        assert_eq!(col.data_type, SimpleDataType::Float64);
        assert!(!col.nullable);
        assert_eq!(col.get_metadata("unit"), Some(&"USD".to_string()));
    }

    #[test]
    fn test_table_schema() {
        let schema = TableSchema::new("test".to_string())
            .add_column(ColumnSchema::new_simple(
                "id".to_string(),
                SimpleDataType::Int64,
            ))
            .add_column(ColumnSchema::new_simple(
                "name".to_string(),
                SimpleDataType::Utf8,
            ));

        assert_eq!(schema.column_count(), 2);
        assert_eq!(schema.column_names(), vec!["id", "name"]);
        assert_eq!(schema.get_column_index("name"), Some(1));
        assert!(schema.get_column("id").is_some());
    }

    #[test]
    fn test_schema_builders() {
        let ts_schema = SchemaBuilder::time_series();
        assert_eq!(ts_schema.column_count(), 2);
        assert!(ts_schema.get_column("time").is_some());
        assert!(ts_schema.get_column("value").is_some());

        let market_schema = SchemaBuilder::market_data();
        assert_eq!(market_schema.column_count(), 5);
        assert!(market_schema.get_column("symbol").is_some());
        assert!(market_schema.get_column("price").is_some());
    }

    #[test]
    fn test_arrow_schema_conversion() {
        let schema = SchemaBuilder::time_series();
        let arrow_schema = schema.to_arrow_schema();
        assert_eq!(arrow_schema.fields.len(), 2);
    }
}
