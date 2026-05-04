//! Diagnostics, structured logging, and source-span rendering for know-now.
//!
//! Responsibility is defined in PRD section 8.2 (workspace layout).

pub mod diagnostic;
pub mod logging;
pub mod render_json;
pub mod render_text;
pub mod stage;

#[cfg(any(test, feature = "test-support"))]
pub mod test_support;

pub const LOG_SCHEMA_VERSION: &str = "0.1.0";
pub const JSON_SCHEMA_VERSION: &str = "0.1.0";
