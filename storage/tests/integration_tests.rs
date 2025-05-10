//! Integration tests for the storage layer
//!
//! These tests demonstrate end-to-end table operations including:
//! - Market data time series
//! - Graph node/edge storage
//! - Mixed data type handling
//! - Persistence across restarts
//! - Performance with larger datasets

use storage::{
    QStoreConfig, ScalarValue, Table,
    schema::{ColumnSchema, SchemaBuilder, SimpleDataType},
    table::Row,
};
use tempfile::TempDir;

/// Test time series data operations (market data scenario)
#[test]
fn test_market_data_time_series() {
    let temp_dir = TempDir::new().unwrap();
    let config = QStoreConfig::new(temp_dir.path(), "market_data".to_string());
    let schema = SchemaBuilder::market_data();
    let mut table = Table::new(schema, config).unwrap();

    // Insert market data over time
    let symbols = vec!["AAPL", "GOOGL", "MSFT", "TSLA"];
    let sides = vec!["BUY", "SELL"];
    let base_time = 1640995200000000000i64; // 2022-01-01 00:00:00 UTC in nanoseconds

    for i in 0..100 {
        let mut row = Row::new();
        row.insert(
            "time".to_string(),
            ScalarValue::Timestamp(base_time + (i as i64) * 1000000000),
        ); // 1 second intervals
        row.insert(
            "symbol".to_string(),
            ScalarValue::Utf8(symbols[i % symbols.len()].to_string()),
        );
        row.insert(
            "price".to_string(),
            ScalarValue::Float64(100.0 + (i as f64) * 0.1),
        );
        row.insert(
            "size".to_string(),
            ScalarValue::Int64(100 + (i % 10) as i64),
        );
        row.insert(
            "side".to_string(),
            ScalarValue::Utf8(sides[i % sides.len()].to_string()),
        );

        table.insert(row).unwrap();
    }

    // Verify data
    assert_eq!(table.row_count().unwrap(), 100);

    // Check specific rows
    let first_row = table.get(0).unwrap();
    assert_eq!(
        first_row.get("symbol"),
        Some(&ScalarValue::Utf8("AAPL".to_string()))
    );
    assert_eq!(first_row.get("price"), Some(&ScalarValue::Float64(100.0)));

    let last_row = table.get(99).unwrap();
    assert_eq!(
        last_row.get("time"),
        Some(&ScalarValue::Timestamp(base_time + 99_i64 * 1000000000))
    );

    // Test column access
    let prices = table.get_column("price").unwrap();
    assert_eq!(prices.len(), 100);
    assert_eq!(prices[0], ScalarValue::Float64(100.0));
    assert_eq!(prices[99], ScalarValue::Float64(109.9));

    // Test filtering
    let aapl_trades = table
        .filter(
            |row| matches!(row.get("symbol"), Some(ScalarValue::Utf8(symbol)) if symbol == "AAPL"),
        )
        .unwrap();
    assert_eq!(aapl_trades.len(), 25); // Every 4th trade is AAPL
}

/// Test graph data storage (nodes and edges)
#[test]
fn test_graph_data_storage() {
    let temp_dir = TempDir::new().unwrap();

    // Test nodes table
    let nodes_config = QStoreConfig::new(temp_dir.path(), "nodes".to_string());
    let nodes_schema = SchemaBuilder::graph_nodes();
    let mut nodes_table = Table::new(nodes_schema, nodes_config).unwrap();

    // Insert some nodes
    let node_types = vec!["person", "company", "location"];
    for i in 0..50 {
        let mut row = Row::new();
        row.insert("id".to_string(), ScalarValue::Int64(i as i64));
        row.insert("name".to_string(), ScalarValue::Utf8(format!("Node_{}", i)));
        row.insert(
            "type".to_string(),
            ScalarValue::Utf8(node_types[i % node_types.len()].to_string()),
        );
        nodes_table.insert(row).unwrap();
    }

    // Test edges table
    let edges_config = QStoreConfig::new(temp_dir.path(), "edges".to_string());
    let edges_schema = SchemaBuilder::graph_edges();
    let mut edges_table = Table::new(edges_schema, edges_config).unwrap();

    // Insert some edges
    let edge_labels = vec!["knows", "works_for", "located_in"];
    for i in 0..100 {
        let mut row = Row::new();
        row.insert("src".to_string(), ScalarValue::Int64((i % 50) as i64));
        row.insert("dst".to_string(), ScalarValue::Int64(((i + 1) % 50) as i64));
        row.insert("weight".to_string(), ScalarValue::Float64((i as f64) * 0.1));
        row.insert(
            "label".to_string(),
            ScalarValue::Utf8(edge_labels[i % edge_labels.len()].to_string()),
        );
        edges_table.insert(row).unwrap();
    }

    // Verify data
    assert_eq!(nodes_table.row_count().unwrap(), 50);
    assert_eq!(edges_table.row_count().unwrap(), 100);

    // Test graph queries
    let node_10 = nodes_table.get(10).unwrap();
    assert_eq!(node_10.get("id"), Some(&ScalarValue::Int64(10)));
    assert_eq!(
        node_10.get("name"),
        Some(&ScalarValue::Utf8("Node_10".to_string()))
    );

    // Find edges from node 10
    let edges_from_10 = edges_table
        .filter(|row| matches!(row.get("src"), Some(ScalarValue::Int64(src)) if *src == 10))
        .unwrap();
    assert_eq!(edges_from_10.len(), 2); // node 10 appears twice as source (i=10, i=60)
}

/// Test persistence across table restarts
#[test]
fn test_persistence_across_restarts() {
    let temp_dir = TempDir::new().unwrap();
    let config = QStoreConfig::new(temp_dir.path(), "persistent_table".to_string());
    let schema = SchemaBuilder::time_series();

    // Create and populate table
    {
        let mut table = Table::new(schema.clone(), config.clone()).unwrap();

        for i in 0..20 {
            let mut row = Row::new();
            row.insert(
                "time".to_string(),
                ScalarValue::Timestamp(1000000000 + i as i64),
            );
            row.insert("value".to_string(), ScalarValue::Float64(i as f64 * 3.14));
            table.insert(row).unwrap();
        }

        assert_eq!(table.row_count().unwrap(), 20);
    } // Table goes out of scope, files should be flushed

    // Reopen the table
    {
        let table = Table::open(schema, config).unwrap();
        assert_eq!(table.row_count().unwrap(), 20);

        // Verify data is still there
        let row_10 = table.get(10).unwrap();
        assert_eq!(
            row_10.get("time"),
            Some(&ScalarValue::Timestamp(1000000010))
        );
        assert_eq!(
            row_10.get("value"),
            Some(&ScalarValue::Float64(10.0 * 3.14))
        );

        // Verify all data
        for i in 0..20 {
            let row = table.get(i).unwrap();
            let expected_time = ScalarValue::Timestamp(1000000000 + i as i64);
            assert_eq!(row.get("time"), Some(&expected_time));
            // Use approximate comparison for floating point values
            if let Some(ScalarValue::Float64(actual)) = row.get("value") {
                let expected = i as f64 * 3.14;
                assert!(
                    (actual - expected).abs() < 1e-10,
                    "Expected {}, got {}",
                    expected,
                    actual
                );
            } else {
                panic!("Expected Float64 value");
            }
        }
    }
}

/// Test mixed data types and null handling
#[test]
fn test_mixed_data_types_and_nulls() {
    let temp_dir = TempDir::new().unwrap();
    let config = QStoreConfig::new(temp_dir.path(), "mixed_types".to_string());

    // Create a schema with all supported types
    let schema = storage::schema::TableSchema::new("mixed_types".to_string())
        .add_column(
            ColumnSchema::new_simple("id".to_string(), SimpleDataType::Int64).with_nullable(false),
        )
        .add_column(ColumnSchema::new_simple(
            "name".to_string(),
            SimpleDataType::Utf8,
        ))
        .add_column(ColumnSchema::new_simple(
            "age".to_string(),
            SimpleDataType::Int32,
        ))
        .add_column(ColumnSchema::new_simple(
            "salary".to_string(),
            SimpleDataType::Float64,
        ))
        .add_column(ColumnSchema::new_simple(
            "active".to_string(),
            SimpleDataType::Boolean,
        ))
        .add_column(ColumnSchema::new_simple(
            "data".to_string(),
            SimpleDataType::Binary,
        ))
        .add_column(ColumnSchema::new_simple(
            "created_at".to_string(),
            SimpleDataType::Timestamp,
        ));

    let mut table = Table::new(schema, config).unwrap();

    // Insert rows with various data types and some nulls
    for i in 0..10 {
        let mut row = Row::new();
        row.insert("id".to_string(), ScalarValue::Int64(i as i64));

        if i % 3 == 0 {
            // Some rows have nulls
            row.insert("name".to_string(), ScalarValue::Null);
            row.insert("age".to_string(), ScalarValue::Null);
        } else {
            row.insert(
                "name".to_string(),
                ScalarValue::Utf8(format!("Person_{}", i)),
            );
            row.insert("age".to_string(), ScalarValue::Int32(20 + i as i32));
        }

        row.insert(
            "salary".to_string(),
            ScalarValue::Float64(50000.0 + i as f64 * 1000.0),
        );
        row.insert("active".to_string(), ScalarValue::Boolean(i % 2 == 0));
        row.insert("data".to_string(), ScalarValue::Binary(vec![i as u8; 10]));
        row.insert(
            "created_at".to_string(),
            ScalarValue::Timestamp(1640995200000000000 + i as i64),
        );

        table.insert(row).unwrap();
    }

    assert_eq!(table.row_count().unwrap(), 10);

    // Verify null handling
    let row_0 = table.get(0).unwrap();
    assert_eq!(row_0.get("name"), Some(&ScalarValue::Null));
    assert_eq!(row_0.get("age"), Some(&ScalarValue::Null));
    assert_eq!(row_0.get("salary"), Some(&ScalarValue::Float64(50000.0)));

    let row_1 = table.get(1).unwrap();
    assert_eq!(
        row_1.get("name"),
        Some(&ScalarValue::Utf8("Person_1".to_string()))
    );
    assert_eq!(row_1.get("age"), Some(&ScalarValue::Int32(21)));

    // Test data type conversions
    let ages = table.get_column("age").unwrap();
    assert_eq!(ages.len(), 10);
    assert_eq!(ages[0], ScalarValue::Null);
    assert_eq!(ages[1], ScalarValue::Int32(21));
}

/// Test table statistics and metadata
#[test]
fn test_table_statistics() {
    let temp_dir = TempDir::new().unwrap();
    let config = QStoreConfig::new(temp_dir.path(), "stats_table".to_string());
    let schema = SchemaBuilder::market_data();
    let mut table = Table::new(schema, config).unwrap();

    // Initially empty
    let stats = table.stats().unwrap();
    assert_eq!(stats.row_count, 0);
    assert_eq!(stats.column_count, 5);
    assert_eq!(
        stats.column_names,
        vec!["time", "symbol", "price", "size", "side"]
    );

    // Add some data
    for i in 0..50 {
        let mut row = Row::new();
        row.insert(
            "time".to_string(),
            ScalarValue::Timestamp(1000000000 + i as i64),
        );
        row.insert("symbol".to_string(), ScalarValue::Utf8("TEST".to_string()));
        row.insert("price".to_string(), ScalarValue::Float64(100.0));
        row.insert("size".to_string(), ScalarValue::Int64(100));
        row.insert("side".to_string(), ScalarValue::Utf8("BUY".to_string()));
        table.insert(row).unwrap();
    }

    let stats = table.stats().unwrap();
    assert_eq!(stats.row_count, 50);
    assert_eq!(stats.column_count, 5);
}

/// Test error handling and edge cases
#[test]
fn test_error_handling() {
    let temp_dir = TempDir::new().unwrap();
    let config = QStoreConfig::new(temp_dir.path(), "error_test".to_string());
    let schema = SchemaBuilder::time_series();
    let mut table = Table::new(schema, config.clone()).unwrap();

    // Test invalid row index
    let result = table.get(0);
    assert!(result.is_err());

    // Test schema validation - wrong type
    let mut row = Row::new();
    row.insert(
        "time".to_string(),
        ScalarValue::Utf8("not_a_timestamp".to_string()),
    );
    row.insert("value".to_string(), ScalarValue::Float64(3.14));

    let result = table.insert(row);
    assert!(result.is_err());

    // Test missing non-nullable column
    let mut row = Row::new();
    row.insert("value".to_string(), ScalarValue::Float64(3.14));
    // Missing required "time" field

    let result = table.insert(row);
    assert!(result.is_err());

    // Test opening non-existent table
    let bad_config = QStoreConfig::new(temp_dir.path(), "non_existent".to_string());
    let bad_schema = SchemaBuilder::time_series();
    let result = Table::open(bad_schema, bad_config);
    assert!(result.is_err());
}

/// Performance test with larger dataset
#[test]
fn test_performance_with_larger_dataset() {
    let temp_dir = TempDir::new().unwrap();
    let config = QStoreConfig::new(temp_dir.path(), "perf_test".to_string());
    let schema = SchemaBuilder::market_data();
    let mut table = Table::new(schema, config).unwrap();

    let start_time = std::time::Instant::now();

    // Insert 1000 rows
    for i in 0..1000 {
        let mut row = Row::new();
        row.insert(
            "time".to_string(),
            ScalarValue::Timestamp(1000000000 + i as i64),
        );
        row.insert(
            "symbol".to_string(),
            ScalarValue::Utf8(format!("SYM_{}", i % 10)),
        );
        row.insert(
            "price".to_string(),
            ScalarValue::Float64(100.0 + (i as f64) * 0.01),
        );
        row.insert(
            "size".to_string(),
            ScalarValue::Int64(100 + (i % 100) as i64),
        );
        row.insert(
            "side".to_string(),
            ScalarValue::Utf8(if i % 2 == 0 { "BUY" } else { "SELL" }.to_string()),
        );
        table.insert(row).unwrap();
    }

    let insert_duration = start_time.elapsed();
    println!("Inserted 1000 rows in {:?}", insert_duration);

    assert_eq!(table.row_count().unwrap(), 1000);

    // Test random access performance
    let start_time = std::time::Instant::now();
    for i in (0..1000).step_by(10) {
        let _row = table.get(i).unwrap();
    }
    let read_duration = start_time.elapsed();
    println!("Read 100 random rows in {:?}", read_duration);

    // Test column access performance
    let start_time = std::time::Instant::now();
    let _prices = table.get_column("price").unwrap();
    let column_duration = start_time.elapsed();
    println!(
        "Read entire price column (1000 values) in {:?}",
        column_duration
    );

    // Test filtering performance
    let start_time = std::time::Instant::now();
    let buy_orders = table
        .filter(|row| matches!(row.get("side"), Some(ScalarValue::Utf8(side)) if side == "BUY"))
        .unwrap();
    let filter_duration = start_time.elapsed();
    println!(
        "Filtered 1000 rows (found {} BUY orders) in {:?}",
        buy_orders.len(),
        filter_duration
    );

    assert_eq!(buy_orders.len(), 500); // Half should be BUY orders
}
