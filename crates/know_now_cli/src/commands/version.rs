use crate::context::CommandContext;
use crate::output::{JsonEnvelope, OutputFormat};

#[derive(Debug, clap::Args)]
pub struct VersionArgs {
    /// Show detailed capability information
    #[arg(long)]
    pub capabilities: bool,
}

pub fn run(ctx: &CommandContext, args: &VersionArgs) -> anyhow::Result<()> {
    let version = env!("CARGO_PKG_VERSION");

    if args.capabilities {
        let caps = capabilities();
        match ctx.format {
            OutputFormat::Json => {
                let envelope = JsonEnvelope::success("version", &caps);
                println!("{}", serde_json::to_string_pretty(&envelope)?);
            }
            OutputFormat::Quiet => {}
            _ => {
                println!("know-now {version}");
                println!();
                println!("Generators:");
                for gen in &caps.generators {
                    println!("  {:<30} {}", gen.name, gen.version);
                }
                println!();
                println!("Output formats: text, json, sarif, quiet");
            }
        }
    } else {
        match ctx.format {
            OutputFormat::Json => {
                let envelope = JsonEnvelope::success("version", version);
                println!("{}", serde_json::to_string_pretty(&envelope)?);
            }
            OutputFormat::Quiet => {}
            _ => println!("know-now {version}"),
        }
    }

    Ok(())
}

#[derive(Debug, serde::Serialize)]
struct Capabilities {
    version: String,
    generators: Vec<GeneratorInfo>,
    output_formats: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
struct GeneratorInfo {
    name: String,
    version: String,
}

fn capabilities() -> Capabilities {
    Capabilities {
        version: env!("CARGO_PKG_VERSION").to_owned(),
        generators: vec![
            GeneratorInfo {
                name: "know_now_gen_postgres".to_owned(),
                version: "0.1.0".to_owned(),
            },
            GeneratorInfo {
                name: "know_now_gen_docs".to_owned(),
                version: "0.1.0".to_owned(),
            },
        ],
        output_formats: vec![
            "text".to_owned(),
            "json".to_owned(),
            "sarif".to_owned(),
            "quiet".to_owned(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capabilities_includes_both_generators() {
        let caps = capabilities();
        assert_eq!(caps.generators.len(), 2);
        assert_eq!(caps.generators[0].name, "know_now_gen_postgres");
        assert_eq!(caps.generators[1].name, "know_now_gen_docs");
    }
}
