# wabznasm: Implementation Plan & Architecture

## Project Vision

wabznasm is a high-performance array processing language and columnar database system inspired by Q/KDB+ but modernized with concepts from APL, J, Clojure, and Joy. It unites:

- **wabznasm Language**: An expressive, Q-inspired query language with powerful functional composition and tacit programming
- **storage**: A column-oriented storage engine using memory-mapped splayed tables for zero-copy data access

The goal is to create a system that combines the terseness and interactive nature of Q with modern language features and Rust's safety and performance characteristics.

## Architecture

### Component Structure

The project is organized as a Cargo workspace with the following crates:

1. **parser**: The language parser, AST, and evaluation engine
2. **storage**: The memory-mapped columnar storage system
3. **repl**: The interactive read-eval-print-loop for user interaction

### Data Model

#### Columnar Storage Foundation

wabznasm uses a columnar data model with memory-mapped files, optimizing for:

1. **Zero-copy access**: Data is accessed directly from memory-mapped files
2. **Cache efficiency**: Column-oriented layout improves cache locality
3. **SIMD compatibility**: Contiguous data enables vectorized processing
4. **Memory efficiency**: Column-level compression reduces footprint

#### Time Series Storage

Time series data is stored as a single splayed table: one column file per field.

- **Column Files**: Each column is stored in a separate file under `data_dir/<column_name>`.
- **Columns**:
  - `time`: Int64 UNIX nanoseconds since epoch
  - Measure columns (e.g., `price`: Float64, `size`: Int64)
  - Tag columns (e.g., `symbol`: Utf8 string), later dictionary-encoded

#### Graph Data

Graphs are represented by two logical splayed tables: **nodes** and **edges**.

- **Nodes Table**:

  - `id` (Int64): unique node identifier
  - Additional property columns per node (e.g., `name`: Utf8, `type`: Utf8)

- **Edges Table**:
  - `src`, `dst` (Int64): source and destination node IDs
  - Additional edge property columns (e.g., `weight`: Float64, `label`: Utf8)

### Optimization Techniques

- **Block-Encoding Layers**:

  - **Delta Encoding** for sorted time stamps with varint packing
  - **Run-Length Encoding (RLE)** for repeated values or low-cardinality columns
  - **Dictionary Encoding** for string/tag columns
  - **Bit-Packing** and compression (e.g., LZ4, Zstd) for binary column files

- **Attributes**:
  - `` `s# ``: Sorted - enables binary search on column
  - `` `u# ``: Unique - enables hash-based lookup
  - `` `g# ``: Grouped - pre-computes group by for column
  - `` `p# ``: Parted - optimizes range queries

## Implementation Plan

The implementation follows a phased approach, focusing on building the system incrementally with performance in mind from the beginning.

### Phase 0: Core Runtime & Atoms

**Focus:** Minimal viable interpreter, basic types.

**Tasks:**

1. Set up Cargo workspace: crates `parser`, `storage`, `repl`
2. Design shared `ScalarValue` enum for atoms (int, float, symbol, etc.)
3. Implement reference-counted or arena-based memory management for values
4. Build a Chumsky-based parser for basic expressions (`1+2`)
5. Implement AST with evaluation for basic arithmetic (`+ - * %`)
6. Create simple REPL using `rustyline` for command input

**Implementation Details:**

- `ScalarValue` should use Arrow2's `DataType` for schema alignment
- parser should handle Q-style right-to-left evaluation order
- Testing framework with unit tests for each atom type

### Phase 1: The Vector - Lists & Basic Functions

**Focus:** Homogeneous lists (vectors), core list operations, simple functions.

**Tasks:**

1. Implement generic list/vector structures with efficient memory layouts
2. Implement vectorized operations that work on entire arrays
3. Add parser support for function calls and definitions (`f: {x+y}`)
4. Implement basic control flow (`if`, `do`, `while`)
5. Set up symbol interning for performance
6. Add APL-inspired rank polymorphism and array primitives

**Implementation Details:**

- Lists should be stored as contiguous arrays of elements where possible
- Use SIMD-friendly layouts for numeric vectors
- Symbol tables need lock-free access for thread safety

### Phase 2: The Dictionary - Associative Data & Tables

**Focus:** Dictionaries, tables (columnar format), basic queries.

**Tasks:**

1. Implement dictionary type (`!` operator) as key-value mapping
2. Build table type as columnar data structure (using flip of dict-of-vectors)
3. Create keyed tables (tables with primary key columns)
4. Implement SQL-like query verbs: `select`, `update`, `delete`, `insert`
5. Add aggregation functions: `sum`, `avg`, `count i by col`
6. Create lexical scoping with `local` keyword
7. Implement extended locale system for namespaces

**Implementation Details:**

- Dictionaries should use high-performance hash tables
- Tables need optimized columnar layout for fast access by column
- Query execution should use columnar evaluation for efficiency

### Phase 3: Splayed-Table Storage

**Focus:** Persistent columnar storage with memory-mapping.

**Tasks:**

1. Design `QStoreConfig` for schema and path configuration
2. Implement column-file format and create one file per column
3. Set up memory-mapped I/O for zero-copy access to column data
4. Create APIs for:
   - `init()`: Load or create column mmaps
   - `put(row)`: Append to column files
   - `count()`: Count rows from file lengths
   - `get(index)`: Read ith entry from each column slice
5. Integrate storage API with query system

**Implementation Details:**

- Use Rust's `memmap2` for safe memory mapping
- Ensure thread-safe access to memory-mapped files
- Design for lock-free readers and append-only writers

### Phase 4: I/O, Persistence & Namespaces

**Focus:** Saving/loading data, system interaction, namespace organization.

**Tasks:**

1. Design serialization format for data structures
2. Implement `save`/`load` for workspaces/variables
3. Create file I/O primitives (`read0`, `:`)
4. Handle system commands (`\t`, `\v`, etc.)
5. Build namespace support (`.ns.var`)
6. Implement workspace persistence model
7. Add system interface functions

**Implementation Details:**

- Serialization format should prioritize speed and space efficiency
- Workspaces need efficient partial loading support
- File I/O should use memory mapping where appropriate

### Phase 5: IPC & Advanced Functions

**Focus:** Inter-process communication, temporal types, advanced operations.

**Tasks:**

1. Implement date/time types and their operations
2. Add complex joins: `aj` (as-of join), `lj` (left join), `wj` (window join)
3. Create iterator adverbs: `/` (over), `\` (scan)
4. Build IPC server via TCP for remote queries
5. Implement error trapping with `'`
6. Add function references with gerund notation

**Implementation Details:**

- Date/time types should use efficient binary representations
- Joins need to be optimized for columnar data
- IPC should use a binary protocol for efficiency
- Design for low-latency network operations

### Phase 6: Advanced Composition - Tacit Programming & Combinators

**Focus:** Enhanced function composition and point-free programming.

**Tasks:**

1. Implement right-to-left pipeline operator `<|`
2. Add stack effect documentation syntax (`/ fn: (input -- output)`)
3. Create function documentation & metadata system (`.doc`, `.meta`)
4. Implement J-inspired function composition:
   - Hooks (dyad from monad and dyad): `f@:g`
   - Forks (triad from three functions): `(f g h)`
   - Caps (dyad from two monads): `f&:g`
5. Add Joy/Forth-inspired combinators (S', K, I, C, W)
6. Implement pattern matching system

**Implementation Details:**

- parser needs to handle complex tacit expressions
- Optimization pass for composition to reduce intermediate allocations
- Documentation should be accessible at runtime for help system

### Phase 7: Advanced Data Structures & Type System

**Focus:** High-performance data structures and enhanced type system.

**Tasks:**

1. Implement first-class data-oriented design:
   - Create `.struct` namespace for memory layout control
   - Add SoA/AoS options for data organization
   - Support alignment and packing hints for SIMD
2. Enhance type system:
   - Add first-class nil handling with propagation
   - Implement optional type annotations
   - Create nil-aware operators (`??`, `?=`, `!?`)
3. Add multiple dispatch:
   - Type-based specialization for functions
   - Runtime optimization based on argument types
4. Implement advanced slicing for multi-dimensional arrays
5. Create partial application syntax and optimization
6. Build context system for implicit parameters:
   - `@.` operator for accessing context values
   - `@@` operator for context modification
   - Thread-local context storage mechanism

**Implementation Details:**

- Memory layouts should align with CPU cache lines
- Nil handling needs to be zero-cost when not used
- Context system should optimize away at compile time when possible

### Phase 8: Optimization - Attributes, Performance

**Focus:** Performance features and explicit optimization controls.

**Tasks:**

1. Implement attributes for lists/tables:
   - `s#` (sorted) for binary search
   - `u#` (unique) for hash lookup
   - `g#` (grouped) for pre-computed grouping
   - `p#` (parted) for range optimization
2. Optimize vectorized operations with SIMD
3. Add high-performance window functions for time series
4. Implement specialized temporal aggregations
5. Create efficient compression for column data
6. Add explicit control namespaces:
   - `.mem` for memory management
   - `.opt` for optimization hints
7. Implement advanced sequence abstractions:
   - Seq protocol for unified sequence operations
   - Transducers for composable transformations
   - Transients for controlled mutability
   - Reactive streams for event processing

**Implementation Details:**

- Attributes should modify in-memory representations for optimal access
- SIMD operations need to use explicit vectorization
- Compression codecs should be pluggable per column
- Sequence abstraction needs lazy evaluation support

## Cross-Cutting Concerns

### Performance & Profiling

- Set up a `benches/` directory with Criterion-based microbenchmarks
- Instrument hot paths for inlining
- Use lock-free, thread-safe data structures
- Design data layouts for cache locality
- Support SIMD/vectorizable code
- Integrate benchmark runs into CI to catch performance regressions

### Testing Strategy

- Unit tests for each component function
- Integration tests for end-to-end flows
- Property-based testing for parser and evaluation
- Benchmark tests to ensure performance targets are met
- Fuzz testing for parser robustness

### Documentation

- API documentation for all public interfaces
- Example-driven documentation for language features
- Reference card for quick syntax lookup
- Developer guide for contributing to the project

### Memory Management

- Use Rust's ownership model for safety
- Implement reference counting or arenas for shared values
- Design for minimal allocations in hot paths
- Create explicit memory control primitives

### Error Handling

- Typed error system categorizing error sources
- Detailed error messages with context
- Recovery mechanisms for non-fatal errors
- Runtime diagnostics for performance issues

## Next Steps

1. Begin Phase 0 implementation in `parser` and `repl` crates
2. Create scaffolding for `storage` with memory-map support
3. Implement basic value types and operations
4. Build simple command parser and evaluator
5. Develop initial REPL functionality
6. Create test suite for core components
