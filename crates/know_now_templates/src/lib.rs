//! Restricted template pack rendering crate for know-now.
//!
//! Produces artifact descriptors only — never writes files directly.
//! Custom packs cannot register native functions, filters, tests, or loaders.

mod filters;
mod manifest;
mod render;

pub use manifest::{
    Licensing, Limits, PackManifest, Permissions, RendererRef, TrustLevel, ValidationError,
    validate_manifest,
};
pub use render::{ArtifactDescriptor, RenderError, RenderReport, render_pack};
