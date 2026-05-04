//! Repository task runner crate for know-now maintenance workflows.
//!
//! Responsibility is defined in PRD section 8.2 (workspace layout).

use std::{
    fmt, fs,
    io::{self, Write},
    path::Path,
    process::{Command, ExitCode, Stdio},
};

use anyhow::Context;
use clap::{Parser, Subcommand, ValueEnum};
use know_now_audit::redaction;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Parser)]
#[command(author, version, about = "Repository task runner for know-now")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Run the canonical local quality gate.
    Check,
    /// Fixture-related utilities.
    Fixtures {
        #[command(subcommand)]
        command: FixtureCommands,
    },
    /// Run benchmark suite.
    Bench,
    /// Release preparation utilities.
    Release {
        #[command(subcommand)]
        command: ReleaseCommands,
    },
    /// Documentation utilities.
    Docs {
        #[command(subcommand)]
        command: DocsCommands,
    },
    /// E2E orchestration entrypoint.
    E2e {
        #[arg(value_enum)]
        phase: E2ePhase,
    },
    /// Event stream utilities.
    Logs {
        #[command(subcommand)]
        command: LogCommands,
    },
    /// Redaction fuzz helpers.
    Redaction {
        #[command(subcommand)]
        command: RedactionCommands,
    },
    /// Project graph helpers.
    Graph {
        #[command(subcommand)]
        command: GraphCommands,
    },
    /// Snapshot-review utilities.
    Snapshot {
        #[command(subcommand)]
        command: SnapshotCommands,
    },
}

#[derive(Debug, Subcommand)]
enum FixtureCommands {
    /// Regenerate fixtures (requires --confirm).
    Regen {
        #[arg(long)]
        confirm: bool,
    },
    /// Report fixture drift and return non-zero when drift exists.
    Diff {
        /// Emit machine-readable JSON summary.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
enum ReleaseCommands {
    /// Prepare release metadata for a version.
    Prepare { version: String },
}

#[derive(Debug, Subcommand)]
enum DocsCommands {
    /// Build documentation site.
    Build,
    /// Validate documentation cross-references.
    Check,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum E2ePhase {
    Foundation,
    Phase1,
    Phase2A,
    Phase2B,
    Phase3,
    All,
}

#[derive(Debug, Subcommand)]
enum LogCommands {
    /// Validate captured events against committed schema.
    Validate { events_path: String },
}

#[derive(Debug, Subcommand)]
enum RedactionCommands {
    /// Run redaction fuzz harness.
    Fuzz {
        #[arg(long)]
        seed: Option<String>,
        #[arg(long)]
        iterations: Option<u32>,
    },
}

#[derive(Debug, Subcommand)]
enum GraphCommands {
    /// Emit graph health report.
    Beads,
}

#[derive(Debug, Subcommand)]
enum SnapshotCommands {
    /// Interactive snapshot review wrapper.
    Review,
}

#[derive(Debug, Clone, Copy)]
enum StepStatus {
    Pass,
    Skip,
    Fail,
}

impl fmt::Display for StepStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pass => write!(f, "PASS"),
            Self::Skip => write!(f, "SKIP"),
            Self::Fail => write!(f, "FAIL"),
        }
    }
}

#[derive(Debug)]
struct StepResult {
    name: &'static str,
    status: StepStatus,
    detail: String,
}

#[derive(Debug, Serialize)]
struct FixtureDiffSummary {
    status: &'static str,
    files: Vec<FixtureDiffEntry>,
}

#[derive(Debug, Serialize)]
struct FixtureDiffEntry {
    status: String,
    path: String,
}

pub fn entry() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            let _ = writeln!(io::stderr(), "error: {err}");
            ExitCode::from(1)
        }
    }
}

fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Check => cmd_check(),
        Commands::Fixtures { command } => match command {
            FixtureCommands::Regen { confirm } => cmd_fixtures_regen(confirm),
            FixtureCommands::Diff { json } => cmd_fixtures_diff(json),
        },
        Commands::Bench => cmd_bench(),
        Commands::Release { command } => match command {
            ReleaseCommands::Prepare { version } => cmd_release_prepare(&version),
        },
        Commands::Docs { command } => match command {
            DocsCommands::Build => cmd_docs_build(),
            DocsCommands::Check => cmd_docs_check(),
        },
        Commands::E2e { phase } => cmd_e2e(phase),
        Commands::Logs { command } => match command {
            LogCommands::Validate { events_path } => cmd_logs_validate(&events_path),
        },
        Commands::Redaction { command } => match command {
            RedactionCommands::Fuzz { seed, iterations } => {
                cmd_redaction_fuzz(seed.as_deref(), iterations)
            }
        },
        Commands::Graph { command } => match command {
            GraphCommands::Beads => cmd_graph_beads(),
        },
        Commands::Snapshot { command } => match command {
            SnapshotCommands::Review => cmd_snapshot_review(),
        },
    }
}

fn cmd_check() -> anyhow::Result<()> {
    let mut results = Vec::new();

    results.push(run_cargo_step("fmt", ["fmt", "--all", "--", "--check"]));
    if last_failed(&results) {
        return finish_check(&results);
    }

    results.push(run_cargo_step(
        "clippy",
        [
            "clippy",
            "--all-targets",
            "--all-features",
            "--",
            "-D",
            "warnings",
        ],
    ));
    if last_failed(&results) {
        return finish_check(&results);
    }

    results.push(run_cargo_step("deny", ["deny", "check"]));
    if last_failed(&results) {
        return finish_check(&results);
    }

    results.push(run_cargo_step("tests", ["test", "--workspace"]));
    if last_failed(&results) {
        return finish_check(&results);
    }

    results.push(run_fitness_step());
    if last_failed(&results) {
        return finish_check(&results);
    }

    results.push(run_fixture_diff_step());
    finish_check(&results)
}

fn finish_check(results: &[StepResult]) -> anyhow::Result<()> {
    let mut failed = false;
    let mut stdout = io::stdout();

    writeln!(stdout, "xtask check summary")?;
    for result in results {
        writeln!(
            stdout,
            "  {:<8} {:<10} {}",
            result.status, result.name, result.detail
        )?;
        if matches!(result.status, StepStatus::Fail) {
            failed = true;
        }
    }

    if failed {
        anyhow::bail!("xtask check failed")
    }

    writeln!(stdout, "xtask check: PASS")?;
    Ok(())
}

fn cmd_fixtures_regen(confirm: bool) -> anyhow::Result<()> {
    if !confirm {
        anyhow::bail!("refusing to regenerate fixtures without --confirm")
    }

    let regen_script = Path::new("tests/fixtures/regen.sh");
    if regen_script.exists() {
        run_external_command_dyn("bash", &["tests/fixtures/regen.sh"])?;
        let mut stdout = io::stdout();
        writeln!(
            stdout,
            "Fixture regeneration completed via tests/fixtures/regen.sh."
        )?;
        return Ok(());
    }

    let fixtures_dir = Path::new("fixtures");
    let generated_dir = Path::new("generated");
    if !fixtures_dir.exists() && !generated_dir.exists() {
        let mut stdout = io::stdout();
        writeln!(
            stdout,
            "No fixtures/ or generated/ directories found; nothing to regenerate."
        )?;
        return Ok(());
    }

    anyhow::bail!(
        "fixture directories exist but no regeneration driver was found at tests/fixtures/regen.sh"
    )
}

fn cmd_fixtures_diff(json: bool) -> anyhow::Result<()> {
    let summary = fixture_diff_summary()?;

    if json {
        let mut stdout = io::stdout();
        serde_json::to_writer_pretty(&mut stdout, &summary)?;
        writeln!(stdout)?;
    } else {
        print_fixture_diff_text(&summary)?;
    }

    if summary.files.is_empty() {
        return Ok(());
    }

    anyhow::bail!("fixture drift detected")
}

fn cmd_release_prepare(version: &str) -> anyhow::Result<()> {
    if !is_semver_triplet(version) {
        anyhow::bail!("version must match <major>.<minor>.<patch>, got `{version}`")
    }

    let mut stdout = io::stdout();
    writeln!(stdout, "Release preparation checklist for {version}:")?;
    writeln!(
        stdout,
        "  1. Update crate versions and compatibility metadata."
    )?;
    writeln!(
        stdout,
        "  2. Regenerate fixtures and classify fixture diff."
    )?;
    writeln!(stdout, "  3. Draft changelog entries and release notes.")?;
    writeln!(stdout, "  4. Run `cargo xtask check` before tagging.")?;
    Ok(())
}

fn cmd_bench() -> anyhow::Result<()> {
    run_external_command("cargo", ["bench", "--workspace"])
}

fn cmd_docs_build() -> anyhow::Result<()> {
    if command_available("mdbook") {
        return run_external_command("mdbook", ["build", "docs"]);
    }

    run_external_command("cargo", ["doc", "--workspace", "--no-deps"])
}

fn cmd_docs_check() -> anyhow::Result<()> {
    let adr_readme_path = Path::new("docs/adr/README.md");
    let adr_dir = Path::new("docs/adr");
    if !adr_readme_path.exists() || !adr_dir.exists() {
        anyhow::bail!("expected docs/adr/README.md and docs/adr/ to exist")
    }

    let readme_contents = fs::read_to_string(adr_readme_path)?;
    let mut missing = Vec::new();

    for entry in fs::read_dir(adr_dir)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();

        if !file_name.ends_with(".md")
            || file_name == "README.md"
            || file_name == "0000-template.md"
        {
            continue;
        }

        if !readme_contents.contains(file_name.as_ref()) {
            missing.push(file_name.to_string());
        }
    }

    if missing.is_empty() {
        let mut stdout = io::stdout();
        writeln!(stdout, "ADR index is in sync.")?;
        return Ok(());
    }

    missing.sort_unstable();
    anyhow::bail!("ADR index missing entries for: {}", missing.join(", "))
}

fn cmd_e2e(phase: E2ePhase) -> anyhow::Result<()> {
    let phases = if matches!(phase, E2ePhase::All) {
        vec![
            E2ePhase::Foundation,
            E2ePhase::Phase1,
            E2ePhase::Phase2A,
            E2ePhase::Phase2B,
            E2ePhase::Phase3,
        ]
    } else {
        vec![phase]
    };

    let mut overall_ok = true;
    let mut stdout = io::stdout();
    writeln!(stdout, "xtask e2e summary:")?;

    for selected in phases {
        let result = run_e2e_phase(selected);
        let (status, detail) = match result {
            Ok(detail) => ("PASS", detail),
            Err(err) => {
                overall_ok = false;
                ("FAIL", err.to_string())
            }
        };

        writeln!(
            stdout,
            "  {:<8} {:<10} {}",
            status,
            selected.as_str(),
            detail
        )?;
    }

    if overall_ok {
        return Ok(());
    }

    anyhow::bail!("one or more E2E phases failed")
}

fn run_e2e_phase(phase: E2ePhase) -> anyhow::Result<String> {
    match phase {
        E2ePhase::Foundation => {
            run_external_command("cargo", ["test", "-p", "know_now_fitness"])?;
            Ok("cargo test -p know_now_fitness".to_string())
        }
        E2ePhase::Phase1 => run_e2e_script("tests/e2e/phase-1.sh", phase.as_str()),
        E2ePhase::Phase2A => run_e2e_script("tests/e2e/phase-2a.sh", phase.as_str()),
        E2ePhase::Phase2B => run_e2e_script("tests/e2e/phase-2b.sh", phase.as_str()),
        E2ePhase::Phase3 => run_e2e_script("tests/e2e/phase-3.sh", phase.as_str()),
        E2ePhase::All => unreachable!("expanded by caller"),
    }
}

fn run_e2e_script(script_path: &str, phase_name: &str) -> anyhow::Result<String> {
    if !Path::new(script_path).exists() {
        anyhow::bail!("missing E2E driver for {phase_name}: {script_path}")
    }

    run_external_command_dyn("bash", &[script_path])?;
    Ok(script_path.to_string())
}

fn cmd_logs_validate(events_path: &str) -> anyhow::Result<()> {
    let path = Path::new(events_path);
    if !path.exists() {
        anyhow::bail!("events file does not exist: {events_path}")
    }

    let schema_path = Path::new("tests/logging/expected_event_shape.schema.json");
    let required = required_fields_from_schema(schema_path)?;
    let contents = fs::read_to_string(path)
        .with_context(|| format!("failed to read events file: {events_path}"))?;

    let mut validated = 0usize;
    for (index, line) in contents.lines().enumerate() {
        let line_number = index + 1;
        if line.trim().is_empty() {
            continue;
        }

        let value: Value = serde_json::from_str(line)
            .with_context(|| format!("invalid JSON at /line/{line_number}"))?;
        let object = value
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("expected object at /line/{line_number}"))?;

        for field in &required {
            if !object.contains_key(field) {
                anyhow::bail!(
                    "schema violation at /line/{line_number}/{field}: required field missing"
                );
            }
        }

        validated += 1;
    }

    if validated == 0 {
        anyhow::bail!("no JSON events found in {events_path}")
    }

    let mut stdout = io::stdout();
    writeln!(
        stdout,
        "Validated {validated} events against {} required schema field(s).",
        required.len()
    )?;
    Ok(())
}

fn required_fields_from_schema(schema_path: &Path) -> anyhow::Result<Vec<String>> {
    if !schema_path.exists() {
        anyhow::bail!(
            "missing schema file {}; expected from logging bead",
            schema_path.display()
        )
    }

    let schema_contents = fs::read_to_string(schema_path)
        .with_context(|| format!("failed reading schema file {}", schema_path.display()))?;
    let value: Value = serde_json::from_str(&schema_contents)
        .with_context(|| format!("invalid JSON schema {}", schema_path.display()))?;

    let Some(required) = value.get("required").and_then(Value::as_array) else {
        return Ok(Vec::new());
    };

    required
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::to_owned)
                .ok_or_else(|| anyhow::anyhow!("schema `required` contains non-string entries"))
        })
        .collect()
}

fn cmd_redaction_fuzz(seed: Option<&str>, iterations: Option<u32>) -> anyhow::Result<()> {
    let chosen_seed = seed.unwrap_or("0x5eed");
    let chosen_iterations = iterations.unwrap_or(10_000);
    let iteration_count =
        usize::try_from(chosen_iterations).context("iterations value does not fit into usize")?;
    let mut state = parse_seed(chosen_seed);
    let mut detected = 0usize;
    let mut failed = 0usize;

    let mut stdout = io::stdout();
    writeln!(
        stdout,
        "Running redaction fuzz baseline (seed: {chosen_seed}, iterations: {chosen_iterations})..."
    )?;

    if !Path::new("crates/know_now_audit/src/redaction.rs").exists() {
        anyhow::bail!("redaction module is missing; cannot run redaction fuzz baseline")
    }

    for index in 0..iteration_count {
        let sample = redaction_fuzz_sample(index, &mut state);
        if !redaction::contains_secret(&sample) {
            continue;
        }

        detected += 1;
        let redacted = redaction::redact(&sample);
        if redacted == sample || redaction::contains_secret(&redacted) {
            failed += 1;
            if failed <= 5 {
                writeln!(
                    stdout,
                    "  leak sample {index}: {}",
                    sample.chars().take(80).collect::<String>()
                )?;
            }
        }
    }

    if detected == 0 {
        anyhow::bail!("redaction fuzz generated zero secret-shaped samples")
    }

    if failed > 0 {
        anyhow::bail!("redaction fuzz found {failed} unredacted sample(s)")
    }

    writeln!(
        stdout,
        "Redaction fuzz detected {detected} secret-shaped sample(s) with zero leaks."
    )?;

    run_external_command("cargo", ["test", "-p", "know_now_audit", "redaction"])?;
    Ok(())
}

fn cmd_graph_beads() -> anyhow::Result<()> {
    let stats_raw = run_command_capture("br", ["stats", "--json"])?;
    let ready_raw = run_command_capture("br", ["ready", "--json"])?;
    let cycles_raw = run_command_capture("br", ["dep", "cycles"])?;

    let stats_json: Value =
        serde_json::from_str(&stats_raw).context("failed to parse `br stats --json` output")?;
    let ready_json: Value =
        serde_json::from_str(&ready_raw).context("failed to parse `br ready --json` output")?;

    let open = stats_json
        .get("summary")
        .and_then(|summary| summary.get("open_issues"))
        .and_then(Value::as_i64)
        .unwrap_or_default();
    let blocked = stats_json
        .get("summary")
        .and_then(|summary| summary.get("blocked_issues"))
        .and_then(Value::as_i64)
        .unwrap_or_default();
    let ready_count = ready_json.as_array().map_or(0usize, std::vec::Vec::len);
    let cycles_lower = cycles_raw.to_ascii_lowercase();
    let has_cycles = !(cycles_lower.contains("no dependency cycles")
        || cycles_lower.contains("no cycles detected"));

    let mut stdout = io::stdout();
    writeln!(stdout, "beads graph summary")?;
    writeln!(stdout, "  open: {open}")?;
    writeln!(stdout, "  blocked: {blocked}")?;
    writeln!(stdout, "  ready: {ready_count}")?;
    writeln!(
        stdout,
        "  cycles: {}",
        if has_cycles { "yes" } else { "no" }
    )?;

    if has_cycles {
        anyhow::bail!("dependency cycles detected:\n{cycles_raw}");
    }

    Ok(())
}

fn cmd_snapshot_review() -> anyhow::Result<()> {
    let probe = Command::new("cargo")
        .args(["insta", "--help"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context("failed to probe cargo-insta availability")?;

    if !probe.success() {
        anyhow::bail!("cargo-insta is not installed (`cargo install cargo-insta`)")
    }

    run_external_command("cargo", ["insta", "test"])
}

fn run_cargo_step<const N: usize>(name: &'static str, args: [&'static str; N]) -> StepResult {
    let status = Command::new("cargo")
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    match status {
        Ok(exit) if exit.success() => StepResult {
            name,
            status: StepStatus::Pass,
            detail: "ok".to_string(),
        },
        Ok(exit) => StepResult {
            name,
            status: StepStatus::Fail,
            detail: format!("exit status {exit}"),
        },
        Err(err) => StepResult {
            name,
            status: StepStatus::Fail,
            detail: format!("failed to execute cargo: {err}"),
        },
    }
}

fn run_external_command<const N: usize>(program: &str, args: [&str; N]) -> anyhow::Result<()> {
    run_external_command_dyn(program, &args)
}

fn run_external_command_dyn(program: &str, args: &[&str]) -> anyhow::Result<()> {
    let status = Command::new(program)
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if status.success() {
        return Ok(());
    }

    anyhow::bail!("`{program}` exited with status {status}")
}

fn run_command_capture<const N: usize>(program: &str, args: [&str; N]) -> anyhow::Result<String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .with_context(|| format!("failed to execute `{program}`"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("`{program}` failed with status {}: {stderr}", output.status);
    }

    String::from_utf8(output.stdout).context("command output was not valid UTF-8")
}

fn command_available(name: &str) -> bool {
    Command::new(name)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn run_fitness_step() -> StepResult {
    if std::path::Path::new("crates/know_now_fitness/Cargo.toml").exists() {
        return run_cargo_step("fitness", ["test", "-p", "know_now_fitness"]);
    }

    StepResult {
        name: "fitness",
        status: StepStatus::Skip,
        detail: "fitness harness crate not present yet".to_string(),
    }
}

fn run_fixture_diff_step() -> StepResult {
    match fixture_diff_summary() {
        Ok(summary) if summary.files.is_empty() => StepResult {
            name: "fixtures",
            status: StepStatus::Pass,
            detail: "no fixture drift".to_string(),
        },
        Ok(summary) => StepResult {
            name: "fixtures",
            status: StepStatus::Fail,
            detail: format!("{} changed fixture path(s)", summary.files.len()),
        },
        Err(err) => StepResult {
            name: "fixtures",
            status: StepStatus::Fail,
            detail: format!("failed to inspect fixture drift: {err}"),
        },
    }
}

fn fixture_diff_summary() -> anyhow::Result<FixtureDiffSummary> {
    let output = Command::new("git")
        .args([
            "-c",
            "core.quotepath=false",
            "status",
            "--porcelain=v1",
            "--untracked-files=all",
            "--",
            "fixtures",
            "generated",
        ])
        .output()?;

    if !output.status.success() {
        anyhow::bail!(
            "git status failed while reading fixture drift (exit status {})",
            output.status
        );
    }

    let stdout = String::from_utf8(output.stdout)?;
    let files = stdout
        .lines()
        .filter_map(parse_fixture_line)
        .collect::<Vec<_>>();

    let status = if files.is_empty() { "clean" } else { "drift" };
    Ok(FixtureDiffSummary { status, files })
}

fn parse_fixture_line(line: &str) -> Option<FixtureDiffEntry> {
    if line.len() < 4 {
        return None;
    }

    let (status_chunk, rest) = line.split_at(2);
    let path = rest.trim_start();
    if path.is_empty() {
        return None;
    }

    let status = status_chunk.trim();
    let normalized_status = if status.is_empty() { "??" } else { status };

    Some(FixtureDiffEntry {
        status: normalized_status.to_string(),
        path: path.to_string(),
    })
}

fn print_fixture_diff_text(summary: &FixtureDiffSummary) -> anyhow::Result<()> {
    let mut stdout = io::stdout();

    if summary.files.is_empty() {
        writeln!(
            stdout,
            "No fixture drift detected in fixtures/ or generated/."
        )?;
        return Ok(());
    }

    writeln!(stdout, "Fixture drift detected:")?;
    for entry in &summary.files {
        writeln!(stdout, "  [{}] {}", entry.status, entry.path)?;
    }

    Ok(())
}

fn last_failed(results: &[StepResult]) -> bool {
    results
        .last()
        .is_some_and(|result| matches!(result.status, StepStatus::Fail))
}

fn parse_seed(seed: &str) -> u64 {
    if let Some(hex) = seed.strip_prefix("0x") {
        return u64::from_str_radix(hex, 16).unwrap_or(0x5eed_u64);
    }

    if let Ok(parsed) = seed.parse::<u64>() {
        return parsed;
    }

    seed.as_bytes().iter().fold(0x5eed_u64, |acc, byte| {
        acc.wrapping_mul(1_099_511_628_211)
            .wrapping_add(u64::from(*byte))
    })
}

fn next_u64(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

fn random_charset_token(state: &mut u64, len: usize, charset: &[u8]) -> String {
    let mut token = String::with_capacity(len);
    for _ in 0..len {
        let index = (next_u64(state) as usize) % charset.len();
        token.push(char::from(charset[index]));
    }
    token
}

fn redaction_fuzz_sample(index: usize, state: &mut u64) -> String {
    const UPPER_NUM: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    const BASE64: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    const ALNUM: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

    match index % 5 {
        0 => format!("key=AKIA{}", random_charset_token(state, 16, UPPER_NUM)),
        1 => format!("password={}", random_charset_token(state, 40, ALNUM)),
        2 => format!(
            "-----BEGIN RSA PRIVATE KEY-----\n{}\n-----END RSA PRIVATE KEY-----",
            random_charset_token(state, 48, BASE64)
        ),
        3 => format!(
            "Bearer eyJ{}.eyJ{}.{}",
            random_charset_token(state, 12, ALNUM),
            random_charset_token(state, 14, ALNUM),
            random_charset_token(state, 18, ALNUM)
        ),
        _ => random_charset_token(state, 56, BASE64),
    }
}

fn is_semver_triplet(value: &str) -> bool {
    let mut parts = value.split('.');
    let (Some(major), Some(minor), Some(patch), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return false;
    };

    [major, minor, patch]
        .iter()
        .all(|segment| !segment.is_empty() && segment.chars().all(|ch| ch.is_ascii_digit()))
}

impl E2ePhase {
    fn as_str(self) -> &'static str {
        match self {
            Self::Foundation => "foundation",
            Self::Phase1 => "phase-1",
            Self::Phase2A => "phase-2a",
            Self::Phase2B => "phase-2b",
            Self::Phase3 => "phase-3",
            Self::All => "all",
        }
    }
}
