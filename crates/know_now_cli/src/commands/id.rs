use std::fs;

use crate::context::CommandContext;
use crate::output::OutputFormat;

#[derive(Debug, clap::Subcommand)]
pub enum IdCommand {
    /// List missing or required IDs
    Check(IdCheckArgs),
    /// Propose deterministic IDs to stdout
    Suggest(IdSuggestArgs),
    /// Preview or apply stable ID backfill
    Backfill(IdBackfillArgs),
}

#[derive(Debug, clap::Args)]
pub struct IdCheckArgs;

#[derive(Debug, clap::Args)]
pub struct IdSuggestArgs;

#[derive(Debug, clap::Args)]
pub struct IdBackfillArgs {
    /// Actually write changes (default is dry-run)
    #[arg(long)]
    pub apply: bool,

    /// Allow apply on a dirty working tree
    #[arg(long)]
    pub allow_dirty: bool,
}

pub fn run(ctx: &CommandContext, cmd: &IdCommand) -> anyhow::Result<()> {
    let metadata = crate::commands::load_project_metadata(ctx)?;

    match cmd {
        IdCommand::Check(_) => {
            let (result, _diagnostics) = know_now_identity::check_ids(&metadata);
            match ctx.format {
                OutputFormat::Json => {
                    let envelope = crate::output::JsonEnvelope::success("id check", &result);
                    println!("{}", serde_json::to_string_pretty(&envelope)?);
                }
                OutputFormat::Quiet => {}
                _ => {
                    if result.missing.is_empty()
                        && result.invalid.is_empty()
                        && result.duplicate.is_empty()
                    {
                        println!("All objects have valid stable IDs.");
                    } else {
                        for m in &result.missing {
                            println!(
                                "  missing: {} '{}' at {} (suggested: {})",
                                m.object_type, m.name, m.yaml_path, m.suggested_id
                            );
                        }
                        for i in &result.invalid {
                            println!(
                                "  invalid: {} '{}' at {}: {}",
                                i.object_type, i.id, i.yaml_path, i.reason
                            );
                        }
                        for d in &result.duplicate {
                            println!("  duplicate: '{}' at {:?}", d.id, d.locations);
                        }
                    }
                }
            }
            if !result.duplicate.is_empty() {
                std::process::exit(crate::exit_code::VALIDATION_ERROR);
            }
            Ok(())
        }
        IdCommand::Suggest(_) => {
            let suggestions = know_now_identity::suggest_all_ids(&metadata);
            match ctx.format {
                OutputFormat::Json => {
                    let envelope = crate::output::JsonEnvelope::success("id suggest", &suggestions);
                    println!("{}", serde_json::to_string_pretty(&envelope)?);
                }
                OutputFormat::Quiet => {}
                _ => {
                    if suggestions.is_empty() {
                        println!("All objects already have stable IDs.");
                    } else {
                        for s in &suggestions {
                            println!("  {} '{}': id: {}", s.object_type, s.name, s.suggested_id);
                        }
                    }
                }
            }
            Ok(())
        }
        IdCommand::Backfill(args) => run_backfill(ctx, &metadata, args),
    }
}

fn run_backfill(
    ctx: &CommandContext,
    metadata: &know_now_metadata::authoring::AuthoringMetadata,
    args: &IdBackfillArgs,
) -> anyhow::Result<()> {
    let suggestions = know_now_identity::suggest_all_ids(metadata);

    if suggestions.is_empty() {
        println!("No missing IDs found.");
        return Ok(());
    }

    if !args.apply {
        let preview = know_now_identity::backfill_preview(metadata);
        if ctx.format != OutputFormat::Quiet {
            print!("{preview}");
            println!("\nRun with --apply to write changes.");
        }
        return Ok(());
    }

    let metadata_dir = ctx.project_root.join("metadata");
    let yaml_files = collect_yaml_files(&metadata_dir)?;

    let timestamp = now_compact();
    let backup_dir = ctx
        .project_root
        .join(".knownow")
        .join("backups")
        .join(&timestamp);
    fs::create_dir_all(&backup_dir)?;

    let mut patched_count = 0;
    for file_path in &yaml_files {
        let content = fs::read_to_string(file_path)?;
        let relative = file_path
            .strip_prefix(&ctx.project_root)
            .unwrap_or(file_path);

        let file_patches = find_patches_for_file(&content, &suggestions);
        if file_patches.is_empty() {
            continue;
        }

        let backup_path = backup_dir.join(relative);
        if let Some(parent) = backup_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&backup_path, &content)?;

        let updated_content = apply_patches(&content, &file_patches);
        fs::write(file_path, &updated_content)?;
        patched_count += file_patches.len();

        println!("  patched {} ({} IDs added)", relative.display(), file_patches.len());
    }

    println!();
    println!(
        "Applied {} ID(s) across {} file(s). Backups at: {}",
        patched_count,
        yaml_files.len(),
        backup_dir.display()
    );

    Ok(())
}

struct Patch {
    line_index: usize,
    id_line: String,
}

fn find_patches_for_file(
    content: &str,
    suggestions: &[know_now_identity::MissingId],
) -> Vec<Patch> {
    let lines: Vec<&str> = content.lines().collect();
    let mut patches = Vec::new();

    for suggestion in suggestions {
        if let Some(patch) = find_insertion_point(&lines, suggestion) {
            patches.push(patch);
        }
    }

    patches.sort_by(|a, b| b.line_index.cmp(&a.line_index));
    patches
}

fn find_insertion_point(
    lines: &[&str],
    suggestion: &know_now_identity::MissingId,
) -> Option<Patch> {
    if suggestion.object_type == "relationship" {
        return find_relationship_insertion(lines, suggestion);
    }

    let name_pattern = format!("name: {}", suggestion.name);
    let name_pattern_quoted = format!("name: \"{}\"", suggestion.name);
    let list_pattern = format!("- name: {}", suggestion.name);

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("- name:") && trimmed.contains(&suggestion.name) {
            let indent_len = line.len() - line.trim_start().len();
            let indent = " ".repeat(indent_len + 2);
            return Some(Patch {
                line_index: i + 1,
                id_line: format!("{indent}id: {}", suggestion.suggested_id),
            });
        } else if trimmed == name_pattern
            || trimmed == name_pattern_quoted
            || trimmed == list_pattern
        {
            let indent_len = line.len() - line.trim_start().len();
            let indent = " ".repeat(indent_len);
            return Some(Patch {
                line_index: i + 1,
                id_line: format!("{indent}id: {}", suggestion.suggested_id),
            });
        }
    }
    None
}

fn find_relationship_insertion(
    lines: &[&str],
    suggestion: &know_now_identity::MissingId,
) -> Option<Patch> {
    let parts: Vec<&str> = suggestion.name.splitn(2, '→').collect();
    if parts.len() != 2 {
        return None;
    }
    let (from, to) = (parts[0], parts[1]);

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let is_list_item = trimmed.starts_with("- from_entity:");
        let is_plain = trimmed.starts_with("from_entity:");

        if !is_list_item && !is_plain {
            continue;
        }

        if !trimmed.contains(from) {
            continue;
        }

        let to_line = lines.get(i + 1).map(|l| l.trim());
        if to_line.is_some_and(|t| t.starts_with("to_entity:") && t.contains(to)) {
            let indent_len = line.len() - line.trim_start().len();
            let indent = if is_list_item {
                " ".repeat(indent_len + 2)
            } else {
                " ".repeat(indent_len)
            };
            return Some(Patch {
                line_index: i + 1,
                id_line: format!("{indent}id: {}", suggestion.suggested_id),
            });
        }
    }
    None
}

fn apply_patches(content: &str, patches: &[Patch]) -> String {
    let mut lines: Vec<String> = content.lines().map(String::from).collect();

    for patch in patches {
        if patch.line_index <= lines.len() {
            lines.insert(patch.line_index, patch.id_line.clone());
        }
    }

    let mut result = lines.join("\n");
    if content.ends_with('\n') {
        result.push('\n');
    }
    result
}

fn collect_yaml_files(dir: &std::path::Path) -> anyhow::Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();
    if !dir.is_dir() {
        return Ok(files);
    }
    for entry in fs::read_dir(dir)?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            files.extend(collect_yaml_files(&path)?);
        } else if path
            .extension()
            .is_some_and(|ext| ext == "yml" || ext == "yaml")
        {
            files.push(path);
        }
    }
    Ok(files)
}

fn now_compact() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{secs}")
}
