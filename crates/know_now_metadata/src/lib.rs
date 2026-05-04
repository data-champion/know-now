//! Metadata types and parsing crate for know-now.
//!
//! Responsibility is defined in PRD section 8.2 (workspace layout).

pub mod authoring;
pub mod budgets;
pub mod fixtures;
pub mod parser;
pub mod span;

#[cfg(any(test, feature = "test-support"))]
pub mod test_support;
