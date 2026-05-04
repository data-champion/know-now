use crate::context::CommandContext;

#[derive(Debug, clap::Subcommand)]
pub enum ConfigCommand {
    /// Show effective configuration
    Inspect(ConfigInspectArgs),
}

#[derive(Debug, clap::Args)]
pub struct ConfigInspectArgs;

pub fn run(_ctx: &CommandContext, cmd: &ConfigCommand) -> anyhow::Result<()> {
    match cmd {
        ConfigCommand::Inspect(_) => {
            anyhow::bail!("config inspect not yet implemented (Phase 2A: know-now-63g.16)")
        }
    }
}
