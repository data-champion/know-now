use std::path::PathBuf;

use crate::context::CommandContext;
use crate::output::OutputFormat;

#[derive(Debug, clap::Args)]
pub struct SchemaArgs {
    /// Write schema to a file instead of stdout
    #[arg(long)]
    pub output: Option<PathBuf>,

    /// Write VS Code settings.json fragment for YAML association
    #[arg(long)]
    pub vscode: Option<PathBuf>,
}

pub fn run(ctx: &CommandContext, args: &SchemaArgs) -> anyhow::Result<()> {
    let schema = schemars::schema_for!(know_now_metadata::authoring::AuthoringMetadata);
    let json = serde_json::to_string_pretty(&schema)?;

    if let Some(ref path) = args.output {
        std::fs::write(path, &json)?;
        if ctx.format != OutputFormat::Quiet {
            eprintln!("Schema written to {}", path.display());
        }
    } else if ctx.format == OutputFormat::Quiet {
        // no output
    } else {
        println!("{json}");
    }

    if let Some(ref vscode_path) = args.vscode {
        let schema_path = args.output.as_deref().map_or_else(
            || ".knownow/metadata-schema.json".to_owned(),
            |p| p.to_string_lossy().to_string(),
        );

        let fragment = serde_json::json!({
            "yaml.schemas": {
                schema_path: ["metadata/**/*.yml", "metadata/**/*.yaml"]
            }
        });
        let fragment_json = serde_json::to_string_pretty(&fragment)?;
        std::fs::write(vscode_path, &fragment_json)?;
        if ctx.format != OutputFormat::Quiet {
            eprintln!(
                "VS Code settings fragment written to {}",
                vscode_path.display()
            );
        }
    }

    Ok(())
}
