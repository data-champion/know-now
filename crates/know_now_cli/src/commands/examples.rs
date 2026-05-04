use crate::context::CommandContext;
use crate::output::{JsonEnvelope, OutputFormat};

#[derive(Debug, clap::Subcommand)]
pub enum ExamplesCommand {
    /// List available example projects
    List(ExamplesListArgs),
}

#[derive(Debug, clap::Args)]
pub struct ExamplesListArgs;

pub fn run(ctx: &CommandContext, cmd: &ExamplesCommand) -> anyhow::Result<()> {
    match cmd {
        ExamplesCommand::List(_) => list(ctx),
    }
}

fn list(ctx: &CommandContext) -> anyhow::Result<()> {
    let examples = built_in_examples();
    match ctx.format {
        OutputFormat::Json => {
            let envelope = JsonEnvelope::success("examples list", &examples);
            println!("{}", serde_json::to_string_pretty(&envelope)?);
        }
        OutputFormat::Quiet => {}
        _ => {
            println!("Available examples:");
            println!();
            for ex in &examples {
                println!("  {:<30} {}", ex.name, ex.description);
                println!(
                    "  {:<30} init: know-now init <name> --profile {}",
                    "", ex.profile
                );
                println!();
            }
        }
    }
    Ok(())
}

#[derive(Debug, serde::Serialize)]
struct ExampleEntry {
    name: &'static str,
    profile: &'static str,
    description: &'static str,
}

fn built_in_examples() -> Vec<ExampleEntry> {
    vec![
        ExampleEntry {
            name: "minimal",
            profile: "minimal",
            description: "Bare-bones project with a single entity. Start here.",
        },
        ExampleEntry {
            name: "consultant-postgres-dbt",
            profile: "consultant-postgres-dbt",
            description: "Postgres + dbt setup for consulting engagements.",
        },
        ExampleEntry {
            name: "dbt-existing-stack",
            profile: "dbt-existing-stack",
            description: "Onboard an existing dbt project into know-now.",
        },
        ExampleEntry {
            name: "governed-team",
            profile: "governed-team",
            description: "Multi-domain project with governance and ownership.",
        },
        ExampleEntry {
            name: "demo-ecommerce",
            profile: "demo",
            description: "Full e-commerce demo (customer, order, product).",
        },
    ]
}
