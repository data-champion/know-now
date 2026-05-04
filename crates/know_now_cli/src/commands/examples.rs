use crate::context::CommandContext;

#[derive(Debug, clap::Subcommand)]
pub enum ExamplesCommand {
    /// List available example projects
    List(ExamplesListArgs),
}

#[derive(Debug, clap::Args)]
pub struct ExamplesListArgs;

pub fn run(_ctx: &CommandContext, cmd: &ExamplesCommand) -> anyhow::Result<()> {
    match cmd {
        ExamplesCommand::List(_) => {
            anyhow::bail!("examples list not yet implemented (Phase 2A: know-now-63g.16)")
        }
    }
}
