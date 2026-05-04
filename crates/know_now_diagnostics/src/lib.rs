//! Diagnostics, structured logging, and source-span rendering for know-now.
//!
//! Responsibility is defined in PRD section 8.2 (workspace layout).

pub mod logging;
pub mod stage;

#[cfg(any(test, feature = "test-support"))]
pub mod test_support;

pub const LOG_SCHEMA_VERSION: &str = "0.1.0";
