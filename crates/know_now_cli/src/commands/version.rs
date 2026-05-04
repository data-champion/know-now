use know_now_codegen::artifact::ArtifactKind;
use know_now_codegen::registry::{CapabilityRegistry, DialectSupport, GeneratorCapability};

use crate::context::CommandContext;
use crate::output::{JsonEnvelope, OutputFormat};

#[derive(Debug, clap::Args)]
pub struct VersionArgs {
    /// Show detailed capability information
    #[arg(long)]
    pub capabilities: bool,
}

pub fn run(ctx: &CommandContext, args: &VersionArgs) -> anyhow::Result<()> {
    if args.capabilities {
        emit_capabilities(ctx)
    } else {
        emit_version(ctx)
    }
}

fn emit_version(ctx: &CommandContext) -> anyhow::Result<()> {
    match ctx.format {
        OutputFormat::Json => {
            let payload = VersionPayload {
                engine_version: env!("CARGO_PKG_VERSION"),
                metadata_schema_version: "1.0",
                generator_contract_version: know_now_contract::CONTRACT_SCHEMA_VERSION,
                lockfile_schema_version: know_now_lock::CURRENT_SCHEMA_VERSION,
            };
            let envelope = JsonEnvelope::success("version", &payload);
            println!("{}", serde_json::to_string_pretty(&envelope)?);
        }
        OutputFormat::Quiet => {}
        _ => println!("know-now {}", env!("CARGO_PKG_VERSION")),
    }
    Ok(())
}

fn emit_capabilities(ctx: &CommandContext) -> anyhow::Result<()> {
    let registry = build_registry();
    match ctx.format {
        OutputFormat::Json => {
            let envelope = JsonEnvelope::success("version", registry.generators());
            println!("{}", serde_json::to_string_pretty(&envelope)?);
        }
        OutputFormat::Quiet => {}
        _ => {
            println!("know-now {}", env!("CARGO_PKG_VERSION"));
            println!();
            for gen in registry.generators() {
                println!("Generator: {}", gen.name);
                println!("  version: {}", gen.version);
                println!("  contract versions: {}", gen.contract_versions.join(", "));
                println!(
                    "  artifact kinds: {}",
                    gen.artifact_kinds
                        .iter()
                        .map(|k| serde_json::to_string(k).unwrap_or_default())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                if !gen.supported_dialects.is_empty() {
                    for d in &gen.supported_dialects {
                        println!("  dialect: {} ({})", d.dialect, d.versions.join(", "));
                    }
                }
                if !gen.validation_gates.is_empty() {
                    println!("  validation gates: {}", gen.validation_gates.join(", "));
                }
                if !gen.unsupported_constructs.is_empty() {
                    println!("  unsupported: {}", gen.unsupported_constructs.join(", "));
                }
                if !gen.experimental_features.is_empty() {
                    println!("  experimental: {}", gen.experimental_features.join(", "));
                }
                println!();
            }
            println!("Output formats: text, json, sarif, quiet");
        }
    }
    Ok(())
}

#[derive(serde::Serialize)]
#[allow(clippy::struct_field_names)]
struct VersionPayload {
    engine_version: &'static str,
    metadata_schema_version: &'static str,
    generator_contract_version: &'static str,
    lockfile_schema_version: &'static str,
}

#[allow(clippy::too_many_lines)]
pub(crate) fn build_registry() -> CapabilityRegistry {
    let mut reg = CapabilityRegistry::new();

    reg.register(GeneratorCapability {
        name: "know_now_gen_postgres".into(),
        version: "0.1.0".into(),
        contract_versions: vec!["1.0".into()],
        artifact_kinds: vec![ArtifactKind::PostgresDdl],
        supported_dialects: vec![DialectSupport {
            dialect: "postgres".into(),
            versions: vec!["14".into(), "15".into(), "16".into(), "17".into()],
        }],
        supported_logical_types: vec![
            "integer",
            "bigint",
            "smallint",
            "decimal",
            "float",
            "double",
            "boolean",
            "string",
            "text",
            "date",
            "time",
            "timestamp",
            "timestamp_tz",
            "uuid",
            "json",
            "jsonb",
            "binary",
            "interval",
            "array",
        ]
        .into_iter()
        .map(Into::into)
        .collect(),
        supported_semantic_types: vec![
            "email",
            "phone",
            "url",
            "currency",
            "percentage",
            "ip_address",
            "postal_code",
            "country",
        ]
        .into_iter()
        .map(Into::into)
        .collect(),
        validation_gates: vec!["parse_validation".into(), "schema_validation".into()],
        unsupported_constructs: vec![],
        experimental_features: vec![],
    });

    reg.register(GeneratorCapability {
        name: "know_now_gen_dbt".into(),
        version: "0.1.0".into(),
        contract_versions: vec!["1.0".into()],
        artifact_kinds: vec![ArtifactKind::DbtModel, ArtifactKind::DbtSchema],
        supported_dialects: vec![],
        supported_logical_types: vec![],
        supported_semantic_types: vec![],
        validation_gates: vec!["parse_validation".into()],
        unsupported_constructs: vec![],
        experimental_features: vec![],
    });

    reg.register(GeneratorCapability {
        name: "know_now_gen_docs".into(),
        version: "0.1.0".into(),
        contract_versions: vec!["1.0".into()],
        artifact_kinds: vec![ArtifactKind::MarkdownDoc],
        supported_dialects: vec![],
        supported_logical_types: vec![],
        supported_semantic_types: vec![],
        validation_gates: vec!["parse_validation".into()],
        unsupported_constructs: vec![],
        experimental_features: vec![],
    });

    reg.register(GeneratorCapability {
        name: "know_now_gen_er".into(),
        version: "0.1.0".into(),
        contract_versions: vec!["1.0".into()],
        artifact_kinds: vec![ArtifactKind::MermaidDiagram, ArtifactKind::MarkdownDoc],
        supported_dialects: vec![],
        supported_logical_types: vec![],
        supported_semantic_types: vec![],
        validation_gates: vec!["parse_validation".into()],
        unsupported_constructs: vec![],
        experimental_features: vec![],
    });

    reg.register(GeneratorCapability {
        name: "know_now_gen_quality".into(),
        version: "0.1.0".into(),
        contract_versions: vec!["1.0".into()],
        artifact_kinds: vec![ArtifactKind::QualityContract],
        supported_dialects: vec![],
        supported_logical_types: vec![],
        supported_semantic_types: vec![],
        validation_gates: vec!["parse_validation".into()],
        unsupported_constructs: vec![],
        experimental_features: vec![],
    });

    reg.register(GeneratorCapability {
        name: "know_now_gen_fixtures".into(),
        version: "0.1.0".into(),
        contract_versions: vec!["1.0".into()],
        artifact_kinds: vec![ArtifactKind::MarkdownDoc],
        supported_dialects: vec![],
        supported_logical_types: vec![],
        supported_semantic_types: vec![],
        validation_gates: vec!["parse_validation".into()],
        unsupported_constructs: vec![],
        experimental_features: vec![],
    });

    reg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_all_generators() {
        let reg = build_registry();
        assert_eq!(reg.generators().len(), 6);
        assert!(reg.find_by_name("know_now_gen_postgres").is_some());
        assert!(reg.find_by_name("know_now_gen_dbt").is_some());
        assert!(reg.find_by_name("know_now_gen_docs").is_some());
        assert!(reg.find_by_name("know_now_gen_er").is_some());
        assert!(reg.find_by_name("know_now_gen_fixtures").is_some());
        assert!(reg.find_by_name("know_now_gen_quality").is_some());
    }

    #[test]
    fn postgres_supports_contract_v1() {
        let reg = build_registry();
        assert!(reg.supports_contract_version("know_now_gen_postgres", "1.0"));
    }

    #[test]
    fn version_payload_serializes() {
        let payload = VersionPayload {
            engine_version: "0.1.0",
            metadata_schema_version: "1.0",
            generator_contract_version: "1.0",
            lockfile_schema_version: "1.0",
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("engine_version"));
        assert!(json.contains("lockfile_schema_version"));
    }
}
