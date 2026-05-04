//! CLI entrypoint crate for know-now.
//!
//! Responsibility is defined in PRD section 8.2 (workspace layout).
#![allow(clippy::missing_errors_doc)]

pub mod audit;
pub mod commands;
pub mod context;
pub mod output;

pub const JSON_ENVELOPE_VERSION: &str = "1";

pub mod exit_code {
    pub const SUCCESS: i32 = 0;
    pub const VALIDATION_ERROR: i32 = 1;
    pub const USAGE_ERROR: i32 = 2;
}
