//! Scalar value types aligned with Arrow2

use crate::schema::SimpleDataType;
use arrow2::datatypes::DataType;
use serde::{Deserialize, Serialize};

/// Scalar values that can be stored in columns
/// Aligned with Arrow2's type system for zero-copy compatibility
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScalarValue {
    /// Null value
    Null,
    /// Boolean value
    Boolean(bool),
    /// 8-bit signed integer
    Int8(i8),
    /// 16-bit signed integer
    Int16(i16),
    /// 32-bit signed integer
    Int32(i32),
    /// 64-bit signed integer
    Int64(i64),
    /// 8-bit unsigned integer
    UInt8(u8),
    /// 16-bit unsigned integer
    UInt16(u16),
    /// 32-bit unsigned integer
    UInt32(u32),
    /// 64-bit unsigned integer
    UInt64(u64),
    /// 32-bit floating point
    Float32(f32),
    /// 64-bit floating point
    Float64(f64),
    /// UTF-8 string
    Utf8(String),
    /// Binary data
    Binary(Vec<u8>),
    /// Timestamp (nanoseconds since Unix epoch)
    Timestamp(i64),
}

impl ScalarValue {
    /// Get the Arrow2 DataType for this value
    pub fn data_type(&self) -> DataType {
        match self {
            ScalarValue::Null => DataType::Null,
            ScalarValue::Boolean(_) => DataType::Boolean,
            ScalarValue::Int8(_) => DataType::Int8,
            ScalarValue::Int16(_) => DataType::Int16,
            ScalarValue::Int32(_) => DataType::Int32,
            ScalarValue::Int64(_) => DataType::Int64,
            ScalarValue::UInt8(_) => DataType::UInt8,
            ScalarValue::UInt16(_) => DataType::UInt16,
            ScalarValue::UInt32(_) => DataType::UInt32,
            ScalarValue::UInt64(_) => DataType::UInt64,
            ScalarValue::Float32(_) => DataType::Float32,
            ScalarValue::Float64(_) => DataType::Float64,
            ScalarValue::Utf8(_) => DataType::Utf8,
            ScalarValue::Binary(_) => DataType::Binary,
            ScalarValue::Timestamp(_) => {
                DataType::Timestamp(arrow2::datatypes::TimeUnit::Nanosecond, None)
            }
        }
    }

    /// Get the SimpleDataType for this value
    pub fn simple_data_type(&self) -> SimpleDataType {
        match self {
            ScalarValue::Null => SimpleDataType::Null,
            ScalarValue::Boolean(_) => SimpleDataType::Boolean,
            ScalarValue::Int8(_) => SimpleDataType::Int8,
            ScalarValue::Int16(_) => SimpleDataType::Int16,
            ScalarValue::Int32(_) => SimpleDataType::Int32,
            ScalarValue::Int64(_) => SimpleDataType::Int64,
            ScalarValue::UInt8(_) => SimpleDataType::UInt8,
            ScalarValue::UInt16(_) => SimpleDataType::UInt16,
            ScalarValue::UInt32(_) => SimpleDataType::UInt32,
            ScalarValue::UInt64(_) => SimpleDataType::UInt64,
            ScalarValue::Float32(_) => SimpleDataType::Float32,
            ScalarValue::Float64(_) => SimpleDataType::Float64,
            ScalarValue::Utf8(_) => SimpleDataType::Utf8,
            ScalarValue::Binary(_) => SimpleDataType::Binary,
            ScalarValue::Timestamp(_) => SimpleDataType::Timestamp,
        }
    }

    /// Check if this value is null
    pub fn is_null(&self) -> bool {
        matches!(self, ScalarValue::Null)
    }

    /// Convert to i64 if possible
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            ScalarValue::Int8(v) => Some(*v as i64),
            ScalarValue::Int16(v) => Some(*v as i64),
            ScalarValue::Int32(v) => Some(*v as i64),
            ScalarValue::Int64(v) => Some(*v),
            ScalarValue::UInt8(v) => Some(*v as i64),
            ScalarValue::UInt16(v) => Some(*v as i64),
            ScalarValue::UInt32(v) => Some(*v as i64),
            ScalarValue::UInt64(v) => Some(*v as i64),
            _ => None,
        }
    }

    /// Convert to f64 if possible
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            ScalarValue::Float32(v) => Some(*v as f64),
            ScalarValue::Float64(v) => Some(*v),
            v => v.as_i64().map(|i| i as f64),
        }
    }

    /// Convert to string if possible
    pub fn as_str(&self) -> Option<&str> {
        match self {
            ScalarValue::Utf8(s) => Some(s),
            _ => None,
        }
    }
}

impl From<bool> for ScalarValue {
    fn from(value: bool) -> Self {
        ScalarValue::Boolean(value)
    }
}

impl From<i8> for ScalarValue {
    fn from(value: i8) -> Self {
        ScalarValue::Int8(value)
    }
}

impl From<i16> for ScalarValue {
    fn from(value: i16) -> Self {
        ScalarValue::Int16(value)
    }
}

impl From<i32> for ScalarValue {
    fn from(value: i32) -> Self {
        ScalarValue::Int32(value)
    }
}

impl From<i64> for ScalarValue {
    fn from(value: i64) -> Self {
        ScalarValue::Int64(value)
    }
}

impl From<u8> for ScalarValue {
    fn from(value: u8) -> Self {
        ScalarValue::UInt8(value)
    }
}

impl From<u16> for ScalarValue {
    fn from(value: u16) -> Self {
        ScalarValue::UInt16(value)
    }
}

impl From<u32> for ScalarValue {
    fn from(value: u32) -> Self {
        ScalarValue::UInt32(value)
    }
}

impl From<u64> for ScalarValue {
    fn from(value: u64) -> Self {
        ScalarValue::UInt64(value)
    }
}

impl From<f32> for ScalarValue {
    fn from(value: f32) -> Self {
        ScalarValue::Float32(value)
    }
}

impl From<f64> for ScalarValue {
    fn from(value: f64) -> Self {
        ScalarValue::Float64(value)
    }
}

impl From<String> for ScalarValue {
    fn from(value: String) -> Self {
        ScalarValue::Utf8(value)
    }
}

impl From<&str> for ScalarValue {
    fn from(value: &str) -> Self {
        ScalarValue::Utf8(value.to_string())
    }
}

impl From<Vec<u8>> for ScalarValue {
    fn from(value: Vec<u8>) -> Self {
        ScalarValue::Binary(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scalar_value_data_types() {
        assert_eq!(ScalarValue::Int64(42).data_type(), DataType::Int64);
        assert_eq!(ScalarValue::Float64(3.14).data_type(), DataType::Float64);
        assert_eq!(
            ScalarValue::Utf8("hello".to_string()).data_type(),
            DataType::Utf8
        );
        assert_eq!(ScalarValue::Boolean(true).data_type(), DataType::Boolean);
    }

    #[test]
    fn test_scalar_value_conversions() {
        let val = ScalarValue::Int64(42);
        assert_eq!(val.as_i64(), Some(42));
        assert_eq!(val.as_f64(), Some(42.0));
        assert_eq!(val.as_str(), None);

        let val = ScalarValue::Utf8("hello".to_string());
        assert_eq!(val.as_str(), Some("hello"));
        assert_eq!(val.as_i64(), None);
    }

    #[test]
    fn test_scalar_value_from_primitives() {
        assert_eq!(ScalarValue::from(42i64), ScalarValue::Int64(42));
        assert_eq!(ScalarValue::from(3.14f64), ScalarValue::Float64(3.14));
        assert_eq!(
            ScalarValue::from("hello"),
            ScalarValue::Utf8("hello".to_string())
        );
        assert_eq!(ScalarValue::from(true), ScalarValue::Boolean(true));
    }
}
