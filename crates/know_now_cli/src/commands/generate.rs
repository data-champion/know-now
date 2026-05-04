use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use know_now_codegen::artifact::{ArtifactDescriptor as CodegenArtifact, ArtifactKind};
use know_now_codegen::generator::Generator;
use know_now_contract::contract::GeneratorContract;
use know_now_diagnostics::diagnostic::{Diagnostic, Severity};
use know_now_gen_dbt::DbtGenerator;
use know_now_gen_docs::DocsGenerator;
use know_now_gen_er::ErDiagramGenerator;
use know_now_gen_fixtures::FixtureGenerator;
use know_now_gen_postgres::PostgresGenerator;
use know_now_gen_quality::QualityContractGenerator;
use know_now_lock::check::{self, LOCK_CORRUPT_005, LOCK_MISSING_004, LOCK_STALE_003};
use know_now_lock::lockfile::Lockfile;
use know_now_lock::LOCKFILE_NAME;
use know_now_policy::engine::PolicyPack;
use know_now_toolchain::{RunRecord, RunResult, VolatileStateStore};
use know_now_writer::edit_detection::check_for_manual_edits;
use know_now_writer::generation::{ArtifactDescriptor as WriterArtifact, GenerationSession};
use know_now_writer::manifest::{ManifestV1, PolicyRef, TargetDatabase};
use know_now_writer::manifest_builder::{sha256_hex, ArtifactInput, ManifestBuilder};
use know_now_writer::path_safety::validate_artifact_path;
use know_now_writer::stale_detection::{
    detect_stale, detect_untracked, stale_warnings, untracked_warnings,
};

use crate::commands::lock::resolve_current_versions;
use crate::context::CommandContext;
use crate::output::{JsonEnvelope, OutputFormat};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum GenerateTarget {
    Ddl,
    Dbt,
    Quality,
    Docs,
    Diagrams,
    Review,
    Fixtures,
    All,
    #[value(hide = true)]
    Changed,
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
    #[arg(long, value_enum, value_delimiter = ',')]
    pub target: Option<Vec<GenerateTarget>>,

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

    /// Require migration-safe generation semantics (Phase 3)
    #[arg(long)]
    pub migration_safe: bool,
}

#[allow(clippy::too_many_lines)]
pub fn run(ctx: &CommandContext, args: &GenerateArgs) -> anyhow::Result<()> {
    if args.migration_safe {
        usage_error(
            "Phase 3 feature: --migration-safe is not available in Phase 2A (track via S_P3_DIFF / S_P3_MIGRATIONS)",
        );
    }
    if args.changed {
        usage_error(
            "Phase 3 feature: --changed is not available in Phase 2A (track via S_P3_DIFF / S_P3_MIGRATIONS)",
        );
    }

    let targets = resolve_targets(args);

    if args.locked {
        ensure_locked_matches(ctx)?;
    }

    let metadata = crate::commands::load_project_metadata(ctx)?;
    let mut diagnostics = Vec::new();

    let (_id_result, id_diags) = know_now_identity::check_ids(&metadata);
    diagnostics.extend(id_diags);

    let build_result = know_now_validate::builder::build_project_graph(&metadata);
    diagnostics.extend(build_result.diagnostics);

    let policy = know_now_policy::dc_standard::DcStandard;
    diagnostics.extend(know_now_policy::engine::evaluate_policy(&policy, &metadata));

    diagnostics.sort_by(|a, b| b.severity.cmp(&a.severity));
    let has_errors = diagnostics.iter().any(Diagnostic::is_error);
    let warning_count = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .count();

    let strict_mode = args.strict || args.fail_on_warnings;
    if has_errors || (strict_mode && warning_count > 0) {
        emit_validation_failures(ctx, &diagnostics, strict_mode)?;
        std::process::exit(crate::exit_code::VALIDATION_ERROR);
    }

    let graph = build_result
        .graph
        .ok_or_else(|| anyhow::anyhow!("project graph unavailable after validation"))?;
    let contract = know_now_core::projection::project_graph_to_contract(&graph);

    let generated_artifacts = run_generators(&contract, &targets)?;
    let planned_artifacts = build_plan(&generated_artifacts);

    if args.dry_run {
        emit_success(
            ctx,
            &GenerateOutcome {
                dry_run: true,
                targets: targets.iter().copied().map(target_name).collect(),
                planned_artifacts,
                warnings: diagnostics_to_warnings(&diagnostics),
                generated_count: generated_artifacts.len(),
                manifest_written: false,
                stale_artifacts: vec![],
                untracked_files: vec![],
            },
        )?;
        return Ok(());
    }

    let target_dir = ctx.project_root.join("generated");
    let knownow_dir = ctx.project_root.join(".knownow");

    let mut warnings = diagnostics_to_warnings(&diagnostics);
    match check_for_manual_edits(&target_dir, args.accept_generated_overwrite) {
        Ok(w) => warnings.extend(w),
        Err(edited) => {
            let edited_list = edited
                .iter()
                .map(|e| {
                    format!(
                        "{} (expected {}, found {})",
                        e.path, e.expected_hash, e.actual_hash
                    )
                })
                .collect::<Vec<_>>()
                .join("; ");
            anyhow::bail!(
                "manual edits detected in generated artifacts; rerun with --accept-generated-overwrite to proceed: {edited_list}"
            );
        }
    }

    let lockfile_hash = lockfile_hash(ctx)?;
    let policy_info = policy.info();

    let mut builder = ManifestBuilder::new(env!("CARGO_PKG_VERSION"), &contract.contract_version)
        .project_id(contract.project.as_ref().map_or("", |p| p.name.as_str()))
        .input_hash_from_bytes(&serde_json::to_vec(&metadata)?)
        .lockfile_hash(&lockfile_hash)
        .policy(PolicyRef {
            pack: policy_info.pack,
            version: policy_info.version,
            hash: policy_info.hash,
        })
        .target_database(target_database_from_metadata(&metadata));

    for artifact in &generated_artifacts {
        builder.add_artifact(ArtifactInput {
            path: artifact.path.clone(),
            kind: artifact_kind_name(&artifact.kind).to_owned(),
            artifact_id: artifact.artifact_id.clone(),
            generator: artifact.generator.clone(),
            generator_version: artifact.generator_version.clone(),
            content: artifact.content.as_bytes().to_vec(),
            metadata_object_ids: artifact.metadata_object_ids.clone(),
            trace: vec![],
        });
    }

    let previous_manifest = load_previous_manifest(&target_dir);
    let mut new_manifest = builder.build();

    let mut stale_artifacts = Vec::new();
    let mut untracked_files = Vec::new();

    if let Some(ref previous) = previous_manifest {
        let stale = detect_stale(previous, &new_manifest);
        stale_artifacts = stale
            .iter()
            .map(|s| s.path.display().to_string())
            .collect::<Vec<_>>();
        warnings.extend(stale_warnings(&stale));

        let untracked = detect_untracked(previous, &target_dir);
        untracked_files = untracked
            .iter()
            .map(|u| u.path.display().to_string())
            .collect::<Vec<_>>();
        warnings.extend(untracked_warnings(&untracked));
    }

    for warning in &warnings {
        new_manifest.warnings.push(warning.clone());
    }

    let run_id = new_run_id();
    let session = GenerationSession::begin(&knownow_dir, &run_id, target_dir.clone())?;

    for artifact in &generated_artifacts {
        let safe = validate_artifact_path(&artifact.path, &target_dir)
            .map_err(|e| anyhow::anyhow!("{}: {}", e.code(), e))?;
        session.write_artifact(&WriterArtifact {
            relative_path: safe.as_path().to_path_buf(),
            content: artifact.content.as_bytes().to_vec(),
        })?;
    }

    if let Some(ref previous) = previous_manifest {
        let staging_root = knownow_dir.join("staging").join(&run_id);
        preserve_existing_files(
            previous,
            &new_manifest,
            &target_dir,
            &staging_root,
            args.prune,
            &mut warnings,
        )?;
    }

    session.write_artifact(&WriterArtifact {
        relative_path: PathBuf::from("manifest.json"),
        content: new_manifest.to_json_pretty().into_bytes(),
    })?;

    session.validate(validate_artifact)?;
    session.promote()?;

    persist_run_record(ctx, &run_id, &new_manifest)?;

    emit_success(
        ctx,
        &GenerateOutcome {
            dry_run: false,
            targets: targets.iter().copied().map(target_name).collect(),
            planned_artifacts,
            warnings,
            generated_count: generated_artifacts.len(),
            manifest_written: true,
            stale_artifacts,
            untracked_files,
        },
    )?;

    Ok(())
}

fn usage_error(message: &str) -> ! {
    eprintln!("error: {message}");
    std::process::exit(crate::exit_code::USAGE_ERROR);
}

fn resolve_targets(args: &GenerateArgs) -> Vec<GenerateTarget> {
    let raw_targets = args.target.clone().unwrap_or_else(supported_targets);
    let mut selected = Vec::new();
    let mut seen = BTreeSet::new();

    for target in raw_targets {
        match target {
            GenerateTarget::All => {
                for supported in supported_targets() {
                    if seen.insert(supported) {
                        selected.push(supported);
                    }
                }
            }
            GenerateTarget::Ddl
            | GenerateTarget::Dbt
            | GenerateTarget::Quality
            | GenerateTarget::Docs
            | GenerateTarget::Diagrams
            | GenerateTarget::Fixtures => {
                if seen.insert(target) {
                    selected.push(target);
                }
            }
            GenerateTarget::Changed => {
                usage_error(
                    "Phase 3 feature: --target changed is not available in Phase 2A (track via S_P3_DIFF / S_P3_MIGRATIONS)",
                );
            }
            GenerateTarget::Review => {
                usage_error(
                    "Phase 2B feature: requested --target value is not available in this Phase 2A build",
                );
            }
        }
    }

    selected
}

fn supported_targets() -> Vec<GenerateTarget> {
    vec![
        GenerateTarget::Ddl,
        GenerateTarget::Dbt,
        GenerateTarget::Quality,
        GenerateTarget::Docs,
        GenerateTarget::Diagrams,
        GenerateTarget::Fixtures,
    ]
}

fn ensure_locked_matches(ctx: &CommandContext) -> anyhow::Result<()> {
    let lock_path = ctx.project_root.join(LOCKFILE_NAME);
    if !lock_path.exists() {
        anyhow::bail!("{LOCK_MISSING_004}: lockfile not found; run 'know-now lock update'");
    }

    let lockfile =
        Lockfile::read_from(&lock_path).map_err(|e| anyhow::anyhow!("{LOCK_CORRUPT_005}: {e}"))?;
    let resolved = resolve_current_versions();
    let result = check::check_lockfile(&lockfile, &resolved);

    if result.is_ok() {
        return Ok(());
    }

    let drift_summary = result
        .drifted_fields
        .iter()
        .map(|d| {
            format!(
                "{}: locked={} resolved={}",
                d.field, d.locked_value, d.resolved_value
            )
        })
        .collect::<Vec<_>>()
        .join("; ");

    anyhow::bail!("{LOCK_STALE_003}: lockfile drift detected: {drift_summary}");
}

fn run_generators(
    contract: &GeneratorContract,
    targets: &[GenerateTarget],
) -> anyhow::Result<Vec<CodegenArtifact>> {
    let mut artifacts = Vec::new();

    for target in targets {
        match target {
            GenerateTarget::Ddl => {
                let generator = PostgresGenerator::new();
                let generated = generator
                    .generate(contract)
                    .map_err(|e| join_generation_errors(&e))?;
                artifacts.extend(generated);
            }
            GenerateTarget::Docs => {
                let generator = DocsGenerator::new();
                let generated = generator
                    .generate(contract)
                    .map_err(|e| join_generation_errors(&e))?;
                artifacts.extend(generated);
            }
            GenerateTarget::Dbt => {
                let generator = DbtGenerator::new();
                let generated = generator
                    .generate(contract)
                    .map_err(|e| join_generation_errors(&e))?;
                artifacts.extend(generated);
            }
            GenerateTarget::Quality => {
                let generator = QualityContractGenerator::new();
                let generated = generator
                    .generate(contract)
                    .map_err(|e| join_generation_errors(&e))?;
                artifacts.extend(generated);
            }
            GenerateTarget::Diagrams => {
                let generator = ErDiagramGenerator::new();
                let generated = generator
                    .generate(contract)
                    .map_err(|e| join_generation_errors(&e))?;
                artifacts.extend(generated);
            }
            GenerateTarget::Fixtures => {
                let generator = FixtureGenerator::new();
                let generated = generator
                    .generate(contract)
                    .map_err(|e| join_generation_errors(&e))?;
                artifacts.extend(generated);
            }
            _ => unreachable!("targets were gated before generation"),
        }
    }

    artifacts.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(artifacts)
}

fn join_generation_errors(
    errors: &[know_now_codegen::generator::GenerationError],
) -> anyhow::Error {
    let details = errors
        .iter()
        .map(|e| format!("{}: {}", e.code, e.message))
        .collect::<Vec<_>>()
        .join("; ");
    anyhow::anyhow!("generation failed: {details}")
}

fn build_plan(artifacts: &[CodegenArtifact]) -> Vec<PlannedArtifact> {
    artifacts
        .iter()
        .map(|a| PlannedArtifact {
            path: a.path.clone(),
            kind: artifact_kind_name(&a.kind).to_owned(),
            generator: a.generator.clone(),
        })
        .collect()
}

fn diagnostics_to_warnings(diagnostics: &[Diagnostic]) -> Vec<String> {
    diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .map(|d| format!("{}: {}", d.code, d.message))
        .collect()
}

fn artifact_kind_name(kind: &ArtifactKind) -> &'static str {
    match kind {
        ArtifactKind::PostgresDdl => "postgres_ddl",
        ArtifactKind::DbtModel => "dbt_model",
        ArtifactKind::DbtSchema => "dbt_schema",
        ArtifactKind::DbtTest => "dbt_test",
        ArtifactKind::QualityContract => "quality_contract",
        ArtifactKind::MarkdownDoc => "markdown_doc",
        ArtifactKind::MermaidDiagram => "mermaid_diagram",
    }
}

fn target_name(target: GenerateTarget) -> String {
    match target {
        GenerateTarget::Ddl => "ddl",
        GenerateTarget::Dbt => "dbt",
        GenerateTarget::Quality => "quality",
        GenerateTarget::Docs => "docs",
        GenerateTarget::Diagrams => "diagrams",
        GenerateTarget::Review => "review",
        GenerateTarget::Fixtures => "fixtures",
        GenerateTarget::All => "all",
        GenerateTarget::Changed => "changed",
    }
    .to_owned()
}

fn target_database_from_metadata(
    metadata: &know_now_metadata::authoring::AuthoringMetadata,
) -> TargetDatabase {
    metadata.target_database.as_ref().map_or(
        TargetDatabase {
            kind: String::new(),
            version: String::new(),
            compatibility_floor: String::new(),
        },
        |db| TargetDatabase {
            kind: db.kind.clone(),
            version: db.version.clone().unwrap_or_default(),
            compatibility_floor: db.compatibility_floor.clone().unwrap_or_default(),
        },
    )
}

fn lockfile_hash(ctx: &CommandContext) -> anyhow::Result<String> {
    let lock_path = ctx.project_root.join(LOCKFILE_NAME);
    if lock_path.exists() {
        Ok(sha256_hex(&fs::read(lock_path)?))
    } else {
        Ok(sha256_hex(b""))
    }
}

fn load_previous_manifest(target_dir: &Path) -> Option<ManifestV1> {
    let manifest_path = target_dir.join("manifest.json");
    let content = fs::read_to_string(manifest_path).ok()?;
    ManifestV1::from_json(&content).ok()
}

fn preserve_existing_files(
    previous_manifest: &ManifestV1,
    new_manifest: &ManifestV1,
    target_dir: &Path,
    staging_root: &Path,
    prune_mode: PruneMode,
    warnings: &mut Vec<String>,
) -> anyhow::Result<()> {
    let stale = detect_stale(previous_manifest, new_manifest);
    let untracked = detect_untracked(previous_manifest, target_dir);

    if prune_mode == PruneMode::None {
        for artifact in stale {
            let src = target_dir.join(&artifact.path);
            let dst = staging_root.join(&artifact.path);
            if src.exists() {
                copy_file(&src, &dst)?;
            } else {
                warnings.push(format!(
                    "WRITER-STALE-MISSING: stale artifact '{}' already absent from disk",
                    artifact.path.display()
                ));
            }
        }
    }

    for file in untracked {
        let src = target_dir.join(&file.path);
        let dst = staging_root.join(&file.path);
        if src.exists() && !dst.exists() {
            copy_file(&src, &dst)?;
        }
    }

    Ok(())
}

fn copy_file(src: &Path, dst: &Path) -> anyhow::Result<()> {
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(src, dst)?;
    Ok(())
}

fn new_run_id() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("run_{}_{}", secs, std::process::id())
}

fn persist_run_record(
    ctx: &CommandContext,
    run_id: &str,
    manifest: &ManifestV1,
) -> anyhow::Result<()> {
    let store = VolatileStateStore::new(&ctx.project_root, 50)?;
    let now = now_iso8601();

    store.persist_run(&RunRecord {
        run_id: run_id.to_owned(),
        started_at: now.clone(),
        finished_at: now,
        command: "generate".to_owned(),
        result: RunResult::Success,
        manifest_hash: sha256_hex(manifest.to_json_pretty().as_bytes()),
        duration_ms: 0,
    })?;

    Ok(())
}

fn now_iso8601() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let hours = (secs / 3600) % 24;
    let minutes = (secs / 60) % 60;
    let seconds = secs % 60;
    let days = secs / 86_400;
    let (year, month, day) = days_to_date(days);
    format!("{year:04}-{month:02}-{day:02}T{hours:02}:{minutes:02}:{seconds:02}Z")
}

fn days_to_date(days_since_epoch: u64) -> (u64, u64, u64) {
    let z = days_since_epoch + 719_468;
    let era = z / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

fn emit_validation_failures(
    ctx: &CommandContext,
    diagnostics: &[Diagnostic],
    strict_mode: bool,
) -> anyhow::Result<()> {
    match ctx.format {
        OutputFormat::Json => {
            let payload = ValidationFailurePayload {
                strict_mode,
                diagnostics,
            };
            let envelope = JsonEnvelope::error("generate", &payload);
            println!("{}", serde_json::to_string_pretty(&envelope)?);
        }
        OutputFormat::Quiet => {}
        OutputFormat::Text | OutputFormat::Sarif => {
            for d in diagnostics {
                let path_suffix = d
                    .yaml_path
                    .as_ref()
                    .map_or(String::new(), |p| format!(" at {p}"));
                println!("  {}: [{}] {}{path_suffix}", d.severity, d.code, d.message);
                if let Some(ref help) = d.help {
                    println!("    help: {help}");
                }
            }
            if strict_mode {
                println!();
                println!("Generate failed due to --strict/--fail-on-warnings.");
            }
        }
    }
    Ok(())
}

fn emit_success(ctx: &CommandContext, outcome: &GenerateOutcome) -> anyhow::Result<()> {
    match ctx.format {
        OutputFormat::Json => {
            let envelope = JsonEnvelope::success("generate", outcome);
            println!("{}", serde_json::to_string_pretty(&envelope)?);
        }
        OutputFormat::Quiet => {}
        OutputFormat::Text | OutputFormat::Sarif => {
            println!("Generation plan:");
            for artifact in &outcome.planned_artifacts {
                println!(
                    "  - {} [{} via {}]",
                    artifact.path, artifact.kind, artifact.generator
                );
            }
            println!();

            if outcome.dry_run {
                println!(
                    "Dry run complete. Planned {} artifact(s). No files were written.",
                    outcome.generated_count
                );
            } else {
                println!(
                    "Generation complete. Wrote {} artifact(s) and manifest.",
                    outcome.generated_count
                );
            }

            for warning in &outcome.warnings {
                println!("  warning: {warning}");
            }
        }
    }

    Ok(())
}

fn validate_artifact(path: &Path, content: &[u8]) -> Result<(), String> {
    let path_str = path.to_string_lossy();

    if path_str == "manifest.json" {
        return Ok(());
    }

    let text = std::str::from_utf8(content).map_err(|e| format!("invalid UTF-8: {e}"))?;

    if path_str.ends_with(".sql") {
        validate_sql(text, &path_str)?;
    } else if path_str.ends_with(".yml") || path_str.ends_with(".yaml") {
        validate_yaml(text, &path_str)?;
    } else if path_str.ends_with(".json") {
        validate_json(text, &path_str)?;
    } else if path_str.ends_with(".mmd") {
        validate_mermaid(text, &path_str)?;
    } else if path_str.ends_with(".md") {
        validate_markdown_links(text, &path_str)?;
    }

    Ok(())
}

fn validate_sql(text: &str, path: &str) -> Result<(), String> {
    let cleaned = strip_ownership_comment(text);

    if cleaned.trim().is_empty() {
        return Ok(());
    }

    if cleaned.contains("{%") || cleaned.contains("{{") {
        return validate_dbt_sql(cleaned, path);
    }

    let dialect = sqlparser::dialect::PostgreSqlDialect {};
    sqlparser::parser::Parser::parse_sql(&dialect, cleaned)
        .map_err(|e| format!("{path}: SQL parse error: {e}"))?;
    Ok(())
}

fn validate_dbt_sql(text: &str, path: &str) -> Result<(), String> {
    let stripped = strip_jinja(text);

    if stripped.trim().is_empty() || stripped.trim() == "select" || stripped.trim() == "select *" {
        return Ok(());
    }

    let dialect = sqlparser::dialect::PostgreSqlDialect {};
    match sqlparser::parser::Parser::parse_sql(&dialect, &stripped) {
        Ok(_) => Ok(()),
        Err(_) => {
            if has_balanced_jinja(text) {
                Ok(())
            } else {
                Err(format!("{path}: unbalanced Jinja blocks"))
            }
        }
    }
}

fn strip_jinja(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' && matches!(chars.peek(), Some('%' | '{')) {
            let close = if chars.peek() == Some(&'%') {
                "%}"
            } else {
                "}}"
            };
            chars.next();
            let mut block = String::new();
            let mut found_close = false;
            for ch in chars.by_ref() {
                block.push(ch);
                if block.ends_with(close) {
                    found_close = true;
                    break;
                }
            }
            if !found_close {
                result.push_str("__jinja__");
                continue;
            }
            if close == "}}" {
                result.push_str("__placeholder__");
            }
        } else {
            result.push(c);
        }
    }
    result
}

fn has_balanced_jinja(text: &str) -> bool {
    let open_expr = text.matches("{{").count();
    let close_expr = text.matches("}}").count();
    let open_block = text.matches("{%").count();
    let close_block = text.matches("%}").count();
    open_expr == close_expr && open_block == close_block
}

fn strip_ownership_comment(text: &str) -> &str {
    let trimmed = text.trim_start();
    if trimmed.starts_with("--") {
        trimmed
            .find('\n')
            .map_or("", |pos| &trimmed[pos + 1..])
    } else {
        text
    }
}

fn validate_yaml(text: &str, path: &str) -> Result<(), String> {
    let cleaned = strip_yaml_ownership_comment(text);
    if cleaned.trim().is_empty() {
        return Ok(());
    }

    let mut valid_lines = 0;
    let mut in_multiline = false;
    for line in cleaned.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if in_multiline {
            if !line.starts_with(' ') && !line.starts_with('\t') {
                in_multiline = false;
            } else {
                valid_lines += 1;
                continue;
            }
        }
        if trimmed.ends_with(": |") || trimmed.ends_with(": >") {
            in_multiline = true;
        }
        if trimmed.starts_with("- ") || trimmed.contains(": ") || trimmed.ends_with(':') {
            valid_lines += 1;
        }
    }

    if valid_lines == 0 && !cleaned.trim().is_empty() {
        return Err(format!("{path}: YAML appears to have no valid entries"));
    }

    Ok(())
}

fn strip_yaml_ownership_comment(text: &str) -> &str {
    let trimmed = text.trim_start();
    if trimmed.starts_with('#') {
        trimmed
            .find('\n')
            .map_or("", |pos| &trimmed[pos + 1..])
    } else {
        text
    }
}

fn validate_json(text: &str, path: &str) -> Result<(), String> {
    serde_json::from_str::<serde_json::Value>(text)
        .map_err(|e| format!("{path}: JSON parse error: {e}"))?;
    Ok(())
}

fn validate_mermaid(text: &str, path: &str) -> Result<(), String> {
    let cleaned = text.trim();
    if cleaned.is_empty() {
        return Ok(());
    }

    let first_line = cleaned.lines().next().unwrap_or("").trim();

    let valid_starts = [
        "erDiagram",
        "flowchart",
        "graph",
        "sequenceDiagram",
        "classDiagram",
        "stateDiagram",
        "gantt",
        "pie",
        "gitgraph",
    ];

    if !valid_starts.iter().any(|s| first_line.starts_with(s)) {
        return Err(format!(
            "{path}: Mermaid diagram must start with a valid diagram type, found: '{first_line}'"
        ));
    }

    let mut brace_depth: i32 = 0;
    for line in cleaned.lines() {
        let trimmed_line = line.trim();
        let is_relationship = trimmed_line.contains("--")
            && (trimmed_line.starts_with('}') || trimmed_line.contains("||") || trimmed_line.contains("o{"));
        if is_relationship {
            continue;
        }
        for c in trimmed_line.chars() {
            match c {
                '{' => brace_depth += 1,
                '}' => brace_depth -= 1,
                _ => {}
            }
        }
        if brace_depth < 0 {
            return Err(format!("{path}: unbalanced braces in Mermaid diagram"));
        }
    }
    if brace_depth != 0 {
        return Err(format!(
            "{path}: unbalanced braces in Mermaid diagram (depth: {brace_depth})"
        ));
    }

    Ok(())
}

fn validate_markdown_links(text: &str, path: &str) -> Result<(), String> {
    let mut warnings = Vec::new();
    for (line_no, line) in text.lines().enumerate() {
        let mut chars = line.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '[' {
                let mut link_text = String::new();
                let mut found_close = false;
                for ch in chars.by_ref() {
                    if ch == ']' {
                        found_close = true;
                        break;
                    }
                    link_text.push(ch);
                }
                if found_close && chars.peek() == Some(&'(') {
                    chars.next();
                    let mut url = String::new();
                    for ch in chars.by_ref() {
                        if ch == ')' {
                            break;
                        }
                        url.push(ch);
                    }
                    if url.is_empty() {
                        warnings.push(format!(
                            "{path}:{}: empty link target for [{link_text}]",
                            line_no + 1
                        ));
                    }
                }
            }
        }
    }

    if !warnings.is_empty() {
        return Err(warnings.join("; "));
    }

    Ok(())
}

#[derive(serde::Serialize)]
struct GenerateOutcome {
    dry_run: bool,
    targets: Vec<String>,
    planned_artifacts: Vec<PlannedArtifact>,
    warnings: Vec<String>,
    generated_count: usize,
    manifest_written: bool,
    stale_artifacts: Vec<String>,
    untracked_files: Vec<String>,
}

#[derive(serde::Serialize)]
struct PlannedArtifact {
    path: String,
    kind: String,
    generator: String,
}

#[derive(serde::Serialize)]
struct ValidationFailurePayload<'a> {
    strict_mode: bool,
    diagnostics: &'a [Diagnostic],
}
