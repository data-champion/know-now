pub mod check;
pub mod config;
pub mod diff;
pub mod doctor;
pub mod examples;
pub mod explain;
pub mod generate;
pub mod id;
pub mod init;
pub mod issues;
pub mod lock;
pub mod schema;
pub mod validate;
pub mod version;

use crate::context::CommandContext;
use know_now_metadata::authoring::AuthoringMetadata;
use know_now_metadata::budgets::ParserBudgets;

pub fn load_project_metadata(ctx: &CommandContext) -> anyhow::Result<AuthoringMetadata> {
    let metadata_dir = ctx.project_root.join("metadata");
    if !metadata_dir.is_dir() {
        anyhow::bail!(
            "no metadata/ directory found at {}",
            ctx.project_root.display()
        );
    }
    let project =
        know_now_core::project_loader::load_project(&metadata_dir, &ParserBudgets::default())
            .map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(project.metadata)
}
