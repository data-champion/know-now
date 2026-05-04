use crate::context::CommandContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum GenerateTarget {
    Ddl,
    Dbt,
    Quality,
    Docs,
    Diagrams,
    Review,
    Fixtures,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum PruneMode {
    Stale,
    None,
}

#[derive(Debug, clap::Args)]
#[allow(clippy::struct_excessive_bools)]
pub struct GenerateArgs {
    /// Preview changes without writing
    #[arg(long)]
    pub dry_run: bool,

    /// Generation targets
    #[arg(long, value_enum, default_value = "all")]
    pub target: GenerateTarget,

    /// Fail on any warning
    #[arg(long)]
    pub strict: bool,

    /// Alias for --strict
    #[arg(long)]
    pub fail_on_warnings: bool,

    /// Require lockfile match
    #[arg(long)]
    pub locked: bool,

    /// Disable caching
    #[arg(long)]
    pub no_cache: bool,

    /// Only generate changed entities
    #[arg(long)]
    pub changed: bool,

    /// Stale artifact handling
    #[arg(long, value_enum, default_value = "none")]
    pub prune: PruneMode,

    /// Overwrite manually edited generated files
    #[arg(long)]
    pub accept_generated_overwrite: bool,
}

pub fn run(_ctx: &CommandContext, _args: &GenerateArgs) -> anyhow::Result<()> {
    anyhow::bail!("generate command not yet implemented (Phase 2A: know-now-63g.15)")
}
