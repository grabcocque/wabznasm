//! String interning infrastructure for wabznasm
//!
//! This module provides string interning capabilities for function parameters,
//! function bodies, variable names, and other repeated strings throughout the
//! evaluation process. Using string interning reduces memory usage and enables
//! faster string comparisons via pointer equality.

use lasso::Spur;
// ThreadedRodeo and LazyLock are no longer needed as GLOBAL_INTERNER is removed.
// Rodeo might be needed if other parts of this file still construct Rodeo instances directly,
// but for now, assuming it's not if all global functionality is gone.

/// Type alias for interned string keys
///
/// This represents a unique identifier for an interned string.
/// The actual string interner is managed by the `Evaluator` instances.
pub type InternedString = Spur;

// StringInterner type alias removed as it was for the GLOBAL_INTERNER's Rodeo type.
// The GLOBAL_INTERNER itself and its associated functions (intern, resolve, etc.) are removed.
// The InternerStats struct is removed.
// The initialize_interner and preintern_common_strings functions are removed.

// The #[cfg(test)] mod tests { ... } block is removed as it tested the global interner.

// If there are any other functions in this file that were private helpers
// for the global interner, they should also be removed.
// For now, this leaves the file very minimal, only defining InternedString.
