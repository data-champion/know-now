//! Repository task runner crate for know-now maintenance workflows.
//!
//! Responsibility is defined in PRD section 8.2 (workspace layout).

use std::{
    collections::BTreeMap,
    fmt::{self, Write as _},
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use anyhow::Context;
use clap::{Args, Parser, Subcommand, ValueEnum};
use know_now_audit::redaction;
use serde::{Deserialize, Serialize};
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
    Bench(BenchArgs),
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

#[derive(Debug, Clone, Args)]
struct BenchArgs {
    /// Number of timing samples per benchmark case.
    #[arg(long, default_value_t = 3, value_parser = clap::value_parser!(u32).range(1..=20))]
    runs: u32,

    /// Allowed regression percentage versus baseline before failing.
    #[arg(long, default_value_t = 20.0)]
    max_regression_pct: f64,

    /// Path to the benchmark baseline JSON.
    #[arg(long, default_value = "benchmarks/baseline.json")]
    baseline: String,

    /// Update baseline file from current measurements.
    #[arg(long, default_value_t = false)]
    update_baseline: bool,

    /// Allow baseline regressions above threshold.
    #[arg(long, default_value_t = false)]
    allow_regression: bool,

    /// Run optional peak-memory check for 100-entity generation (NFR-P11).
    #[arg(long, default_value_t = false)]
    memory_check: bool,

    /// Peak-memory budget in MiB for NFR-P11.
    #[arg(long, default_value_t = 512)]
    memory_budget_mib: u64,
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
    /// Generate release notes from classified commits.
    Notes {
        /// Git ref range (e.g., v0.1.0..HEAD).
        #[arg(long, default_value = "HEAD~20..HEAD")]
        range: String,
    },
    /// Check commit messages for convention compliance.
    CheckCommits {
        /// Git ref range to check.
        #[arg(long, default_value = "HEAD~10..HEAD")]
        range: String,
    },
}

#[derive(Debug, Subcommand)]
enum DocsCommands {
    /// Build documentation site.
    Build,
    /// Validate documentation cross-references.
    Check,
    /// Regenerate docs/user/cli-reference.md from clap definitions.
    CliRef,
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

#[derive(Debug, Serialize)]
struct BenchReport {
    version: u32,
    runs: u32,
    max_regression_pct: f64,
    baseline_path: String,
    allow_regression: bool,
    memory_check: bool,
    memory_budget_mib: u64,
    cases: Vec<BenchCaseResult>,
    deferred_cases: Vec<BenchDeferredCase>,
    memory_case: Option<BenchMemoryCaseResult>,
    passed: bool,
}

#[derive(Debug, Serialize)]
struct BenchCaseResult {
    id: &'static str,
    requirement: &'static str,
    budget_ms: f64,
    samples_ms: Vec<f64>,
    mean_ms: f64,
    min_ms: f64,
    max_ms: f64,
    baseline_ms: Option<f64>,
    regression_pct: Option<f64>,
    within_budget: bool,
    regression_within_limit: Option<bool>,
    status: &'static str,
    detail: String,
}

#[derive(Debug, Serialize)]
struct BenchDeferredCase {
    id: &'static str,
    requirement: &'static str,
    reason: &'static str,
}

#[derive(Debug, Serialize)]
struct BenchMemoryCaseResult {
    id: &'static str,
    requirement: &'static str,
    budget_mib: u64,
    observed_mib: Option<f64>,
    within_budget: Option<bool>,
    status: &'static str,
    detail: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct BenchBaseline {
    version: u32,
    cases: BTreeMap<String, f64>,
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
        Commands::Bench(args) => cmd_bench(&args),
        Commands::Release { command } => match command {
            ReleaseCommands::Prepare { version } => cmd_release_prepare(&version),
            ReleaseCommands::Notes { range } => cmd_release_notes(&range),
            ReleaseCommands::CheckCommits { range } => cmd_check_commits(&range),
        },
        Commands::Docs { command } => match command {
            DocsCommands::Build => cmd_docs_build(),
            DocsCommands::Check => cmd_docs_check(),
            DocsCommands::CliRef => cmd_docs_cli_ref(),
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

    results.push(run_rustdoc_step());
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

fn cmd_release_notes(range: &str) -> anyhow::Result<()> {
    let commits = git_log_oneline(range)?;
    if commits.is_empty() {
        anyhow::bail!("no commits found in range {range}")
    }

    let classified = classify_commits(&commits);
    let mut stdout = io::stdout();

    writeln!(stdout, "# Release Notes\n")?;

    let sections: &[(&str, &str)] = &[
        ("breaking", "Breaking Changes"),
        ("feat", "Features"),
        ("fix", "Bug Fixes"),
        ("refactor", "Refactoring"),
        ("perf", "Performance"),
        ("docs", "Documentation"),
        ("test", "Tests"),
        ("build", "Build"),
        ("chore", "Chores"),
        ("other", "Other"),
    ];

    for (key, heading) in sections {
        let items: Vec<_> = classified.iter().filter(|c| c.category == *key).collect();
        if items.is_empty() {
            continue;
        }
        writeln!(stdout, "## {heading}\n")?;
        for item in &items {
            writeln!(stdout, "- {}", item.summary)?;
            if let Some(classification) = &item.output_classification {
                writeln!(stdout, "  Output classification: {classification}")?;
            }
        }
        writeln!(stdout)?;
    }

    Ok(())
}

fn cmd_check_commits(range: &str) -> anyhow::Result<()> {
    let raw = run_command_capture("git", ["log", "--format=%H %s%n%b%n---END---", range])?;
    let commit_blocks: Vec<&str> = raw.split("---END---").collect();
    let mut violations = Vec::new();

    for block in &commit_blocks {
        let block = block.trim();
        if block.is_empty() {
            continue;
        }

        let lines: Vec<&str> = block.lines().collect();
        let Some(first_line) = lines.first() else {
            continue;
        };

        let subject = first_line.split_once(' ').map_or("", |x| x.1);
        let body = lines[1..].join("\n");

        if subject.contains('!') && subject.contains(':') && !body.contains("BREAKING CHANGE:") {
            violations.push(format!(
                "breaking commit missing BREAKING CHANGE footer: {subject}"
            ));
        }
    }

    let mut stdout = io::stdout();
    if violations.is_empty() {
        writeln!(stdout, "All commits pass convention checks.")?;
        return Ok(());
    }

    for v in &violations {
        writeln!(stdout, "  VIOLATION: {v}")?;
    }
    anyhow::bail!("{} commit convention violation(s)", violations.len())
}

struct ClassifiedCommit {
    category: String,
    summary: String,
    output_classification: Option<String>,
}

fn classify_commits(oneline_commits: &[String]) -> Vec<ClassifiedCommit> {
    oneline_commits
        .iter()
        .map(|line| {
            let subject = line.split_once(' ').map_or(line.as_str(), |x| x.1);
            let (category, summary) = parse_conventional_subject(subject);
            ClassifiedCommit {
                category,
                summary: summary.to_owned(),
                output_classification: None,
            }
        })
        .collect()
}

fn parse_conventional_subject(subject: &str) -> (String, &str) {
    let Some(colon_pos) = subject.find(':') else {
        return ("other".into(), subject);
    };

    let prefix = &subject[..colon_pos];
    let message = subject[colon_pos + 1..].trim_start();

    let type_part = prefix
        .split('(')
        .next()
        .unwrap_or(prefix)
        .trim_end_matches('!');

    let category = match type_part {
        "feat" => "feat",
        "fix" => "fix",
        "refactor" => "refactor",
        "perf" => "perf",
        "docs" | "doc" => "docs",
        "test" | "tests" => "test",
        "build" | "ci" => "build",
        "chore" => "chore",
        _ => "other",
    };

    if prefix.contains('!') {
        return ("breaking".into(), message);
    }

    (category.into(), message)
}

fn git_log_oneline(range: &str) -> anyhow::Result<Vec<String>> {
    let raw = run_command_capture("git", ["log", "--oneline", range])?;
    Ok(raw.lines().map(String::from).collect())
}

fn cmd_bench(args: &BenchArgs) -> anyhow::Result<()> {
    let know_now = ensure_know_now_binary()?;
    let baseline_path = Path::new(&args.baseline);
    let baseline = load_benchmark_baseline(baseline_path)?;

    let run_root = benchmark_run_root()?;
    let cases = run_primary_bench_cases(args, &know_now, baseline.as_ref(), &run_root)?;
    let deferred_cases = deferred_bench_cases();
    let memory_case = maybe_run_memory_benchmark(args, &know_now, &run_root)?;

    let all_case_passed = cases.iter().all(|case| case.status == "pass");
    let memory_passed = memory_case
        .as_ref()
        .is_none_or(|case| case.status == "pass" || case.status == "skip");

    if args.update_baseline {
        let baseline_payload = baseline_from_cases(&cases);
        write_benchmark_baseline(baseline_path, &baseline_payload)?;
    }

    let report = BenchReport {
        version: 1,
        runs: args.runs,
        max_regression_pct: args.max_regression_pct,
        baseline_path: args.baseline.clone(),
        allow_regression: args.allow_regression,
        memory_check: args.memory_check,
        memory_budget_mib: args.memory_budget_mib,
        cases,
        deferred_cases,
        memory_case,
        passed: all_case_passed && memory_passed,
    };

    write_benchmark_report(&report)?;
    print_benchmark_summary(&report)?;

    if report.passed {
        return Ok(());
    }

    anyhow::bail!("benchmark gate failed")
}

fn run_primary_bench_cases(
    args: &BenchArgs,
    know_now: &Path,
    baseline: Option<&BenchBaseline>,
    run_root: &Path,
) -> anyhow::Result<Vec<BenchCaseResult>> {
    let p3_projects = prepare_synthetic_projects(run_root, "p3_validate_100", 100, args.runs)?;
    let p4_projects = prepare_synthetic_projects(run_root, "p4_generate_10", 10, args.runs)?;
    let p5_projects = prepare_synthetic_projects(run_root, "p5_generate_100", 100, args.runs)?;

    Ok(vec![
        run_benchmark_case(
            "p1_cli_startup",
            "NFR-P1",
            500.0,
            args,
            baseline.and_then(|b| b.cases.get("p1_cli_startup").copied()),
            |_run_index| run_know_now_command(know_now, None, &["version"]),
        ),
        run_benchmark_case(
            "p2_help_output",
            "NFR-P2",
            200.0,
            args,
            baseline.and_then(|b| b.cases.get("p2_help_output").copied()),
            |_run_index| run_know_now_command(know_now, None, &["--help"]),
        ),
        run_benchmark_case(
            "p3_validate_100_entities",
            "NFR-P3",
            2_000.0,
            args,
            baseline.and_then(|b| b.cases.get("p3_validate_100_entities").copied()),
            |run_index| {
                run_know_now_command(know_now, Some(&p3_projects[run_index]), &["validate"])
            },
        ),
        run_benchmark_case(
            "p4_generate_10_entities",
            "NFR-P4",
            5_000.0,
            args,
            baseline.and_then(|b| b.cases.get("p4_generate_10_entities").copied()),
            |run_index| {
                run_know_now_command(know_now, Some(&p4_projects[run_index]), &["generate"])
            },
        ),
        run_benchmark_case(
            "p5_generate_100_entities",
            "NFR-P5",
            60_000.0,
            args,
            baseline.and_then(|b| b.cases.get("p5_generate_100_entities").copied()),
            |run_index| {
                run_know_now_command(know_now, Some(&p5_projects[run_index]), &["generate"])
            },
        ),
    ])
}

fn deferred_bench_cases() -> Vec<BenchDeferredCase> {
    vec![
        BenchDeferredCase {
            id: "p6_incremental_regeneration",
            requirement: "NFR-P6",
            reason: "Phase-3 incremental target (`--changed`) is not implemented yet.",
        },
        BenchDeferredCase {
            id: "p7_custom_reference_scan",
            requirement: "NFR-P7",
            reason: "Custom-reference scan harness is pending extension-model implementation.",
        },
        BenchDeferredCase {
            id: "p8_dashboard_fcp",
            requirement: "NFR-P8",
            reason: "Dashboard web-vitals perf harness is tracked under Playwright/web profiling work.",
        },
        BenchDeferredCase {
            id: "p9_entity_list_api",
            requirement: "NFR-P9",
            reason: "API p95 latency gate needs a load-harness in phase-3 server testing.",
        },
        BenchDeferredCase {
            id: "p12_timing_breakdown",
            requirement: "NFR-P12",
            reason: "Per-stage timing JSON is not emitted by the pipeline yet.",
        },
        BenchDeferredCase {
            id: "p13_parallel_generation",
            requirement: "NFR-P13",
            reason: "Parallel generation safety is covered by determinism/fitness tests, not microbench timings.",
        },
    ]
}

fn maybe_run_memory_benchmark(
    args: &BenchArgs,
    know_now: &Path,
    run_root: &Path,
) -> anyhow::Result<Option<BenchMemoryCaseResult>> {
    if !args.memory_check {
        return Ok(None);
    }
    let memory_project = prepare_synthetic_project(&run_root.join("p11_memory"), 100)?;
    Ok(Some(run_memory_benchmark(
        know_now,
        &memory_project,
        args.memory_budget_mib,
    )))
}

fn baseline_from_cases(cases: &[BenchCaseResult]) -> BenchBaseline {
    BenchBaseline {
        version: 1,
        cases: cases
            .iter()
            .map(|case| (case.id.to_string(), case.mean_ms))
            .collect(),
    }
}

fn ensure_know_now_binary() -> anyhow::Result<PathBuf> {
    let binary_name = if cfg!(windows) {
        "know-now.exe"
    } else {
        "know-now"
    };
    let path = Path::new("target").join("debug").join(binary_name);
    if path.exists() {
        return Ok(path);
    }

    run_external_command("cargo", ["build", "-p", "know_now_cli"])?;
    if path.exists() {
        return Ok(path);
    }

    anyhow::bail!("expected know-now binary at {} after build", path.display())
}

fn load_benchmark_baseline(path: &Path) -> anyhow::Result<Option<BenchBaseline>> {
    if !path.exists() {
        return Ok(None);
    }
    let payload = fs::read_to_string(path)
        .with_context(|| format!("failed to read baseline file {}", path.display()))?;
    let baseline: BenchBaseline = serde_json::from_str(&payload)
        .with_context(|| format!("failed to parse benchmark baseline {}", path.display()))?;
    Ok(Some(baseline))
}

fn write_benchmark_baseline(path: &Path, baseline: &BenchBaseline) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create baseline parent directory {}",
                parent.display()
            )
        })?;
    }
    let encoded = serde_json::to_vec_pretty(baseline)?;
    fs::write(path, encoded)
        .with_context(|| format!("failed to write benchmark baseline {}", path.display()))
}

fn benchmark_run_root() -> anyhow::Result<PathBuf> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock appears to be before UNIX_EPOCH")?;
    let root = std::env::temp_dir().join(format!(
        "know-now-bench-{}-{}",
        std::process::id(),
        now.as_nanos()
    ));
    fs::create_dir_all(&root)
        .with_context(|| format!("failed to create benchmark run root {}", root.display()))?;
    Ok(root)
}

fn prepare_synthetic_projects(
    root: &Path,
    case_key: &str,
    entity_count: usize,
    runs: u32,
) -> anyhow::Result<Vec<PathBuf>> {
    let mut projects = Vec::new();
    for run_index in 0..runs {
        let project_path = root.join(case_key).join(format!("run-{run_index}"));
        let project = prepare_synthetic_project(&project_path, entity_count)?;
        projects.push(project);
    }
    Ok(projects)
}

fn prepare_synthetic_project(project_root: &Path, entity_count: usize) -> anyhow::Result<PathBuf> {
    let metadata_dir = project_root.join("metadata");
    fs::create_dir_all(&metadata_dir).with_context(|| {
        format!(
            "failed to create synthetic metadata directory {}",
            metadata_dir.display()
        )
    })?;
    let project_yml = metadata_dir.join("project.yml");
    let contents = synthetic_project_yaml(entity_count);
    fs::write(&project_yml, contents).with_context(|| {
        format!(
            "failed to write synthetic metadata {}",
            project_yml.display()
        )
    })?;
    Ok(project_root.to_path_buf())
}

fn synthetic_project_yaml(entity_count: usize) -> String {
    let mut out = String::from("version: \"1.0\"\nentities:\n");
    for idx in 0..entity_count {
        let ordinal = idx + 1;
        let entity_slug = format!("entity_{ordinal:03}");
        let entity_id = format!("ent_{ordinal:03}");
        let attr_id = format!("attr_{ordinal:03}_id");
        let attr_name = format!("attr_{ordinal:03}_name");
        let attr_status = format!("attr_{ordinal:03}_status");
        let attr_created_at = format!("attr_{ordinal:03}_created_at");

        let _ = writeln!(out, "  - id: {entity_id}");
        let _ = writeln!(out, "    name: {entity_slug}");
        let _ = writeln!(out, "    attributes:");
        let _ = writeln!(out, "      - id: {attr_id}");
        let _ = writeln!(out, "        name: id");
        let _ = writeln!(out, "        logical_type: integer");
        let _ = writeln!(out, "        required: true");
        let _ = writeln!(out, "      - id: {attr_name}");
        let _ = writeln!(out, "        name: {entity_slug}_name");
        let _ = writeln!(out, "        logical_type: string");
        let _ = writeln!(out, "        required: true");
        let _ = writeln!(out, "      - id: {attr_status}");
        let _ = writeln!(out, "        name: {entity_slug}_status");
        let _ = writeln!(out, "        logical_type: string");
        let _ = writeln!(out, "      - id: {attr_created_at}");
        let _ = writeln!(out, "        name: {entity_slug}_created_at");
        let _ = writeln!(out, "        logical_type: timestamp");
    }
    out
}

fn run_know_now_command(
    know_now: &Path,
    project: Option<&Path>,
    args: &[&str],
) -> anyhow::Result<()> {
    let mut command = Command::new(know_now);
    if let Some(project_root) = project {
        command.arg("--project").arg(project_root);
    }
    command
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let status = command.status().with_context(|| {
        format!(
            "failed to execute {} {}",
            know_now.display(),
            args.join(" ")
        )
    })?;
    if status.success() {
        return Ok(());
    }
    anyhow::bail!(
        "`{}` {} exited with status {}",
        know_now.display(),
        args.join(" "),
        status
    )
}

fn run_benchmark_case<F>(
    case_id: &'static str,
    requirement: &'static str,
    budget_ms: f64,
    args: &BenchArgs,
    baseline_ms: Option<f64>,
    mut run_once: F,
) -> BenchCaseResult
where
    F: FnMut(usize) -> anyhow::Result<()>,
{
    let mut samples_ms = Vec::new();
    let mut command_failure = None;

    for run_index in 0..args.runs {
        let run_index = usize::try_from(run_index).unwrap_or(usize::MAX);
        let started = Instant::now();
        match run_once(run_index) {
            Ok(()) => {
                let elapsed_ms = started.elapsed().as_secs_f64() * 1_000.0;
                samples_ms.push(elapsed_ms);
            }
            Err(err) => {
                command_failure = Some(err.to_string());
                break;
            }
        }
    }

    if let Some(err) = command_failure {
        return BenchCaseResult {
            id: case_id,
            requirement,
            budget_ms,
            samples_ms,
            mean_ms: 0.0,
            min_ms: 0.0,
            max_ms: 0.0,
            baseline_ms,
            regression_pct: None,
            within_budget: false,
            regression_within_limit: None,
            status: "fail",
            detail: format!("command failed: {err}"),
        };
    }

    if samples_ms.is_empty() {
        return BenchCaseResult {
            id: case_id,
            requirement,
            budget_ms,
            samples_ms,
            mean_ms: 0.0,
            min_ms: 0.0,
            max_ms: 0.0,
            baseline_ms,
            regression_pct: None,
            within_budget: false,
            regression_within_limit: None,
            status: "fail",
            detail: "no successful samples captured".to_string(),
        };
    }

    let (mean_ms, min_ms, max_ms) = benchmark_sample_stats(&samples_ms);
    let within_budget = mean_ms <= budget_ms;
    let regression_pct = baseline_ms.and_then(|baseline| {
        if baseline <= f64::EPSILON {
            None
        } else {
            Some(((mean_ms - baseline) / baseline) * 100.0)
        }
    });

    let regression_within_limit = regression_pct.map(|pct| {
        if pct <= args.max_regression_pct {
            true
        } else {
            args.allow_regression
        }
    });

    let regression_text = match (regression_pct, regression_within_limit) {
        (Some(pct), Some(true)) if pct > args.max_regression_pct => {
            format!("regression {pct:.2}% (allowed)")
        }
        (Some(pct), Some(true)) => format!("regression {pct:.2}%"),
        (Some(pct), Some(false)) => format!(
            "regression {pct:.2}% exceeds {:.2}%",
            args.max_regression_pct
        ),
        _ => "no baseline".to_string(),
    };

    let status = if within_budget && regression_within_limit.unwrap_or(true) {
        "pass"
    } else {
        "fail"
    };

    BenchCaseResult {
        id: case_id,
        requirement,
        budget_ms,
        samples_ms,
        mean_ms,
        min_ms,
        max_ms,
        baseline_ms,
        regression_pct,
        within_budget,
        regression_within_limit,
        status,
        detail: format!("mean {mean_ms:.2}ms vs budget {budget_ms:.2}ms; {regression_text}"),
    }
}

fn benchmark_sample_stats(samples_ms: &[f64]) -> (f64, f64, f64) {
    let sum: f64 = samples_ms.iter().sum();
    let count_u32 = u32::try_from(samples_ms.len()).unwrap_or(u32::MAX);
    let mean = sum / f64::from(count_u32);
    let min = samples_ms.iter().copied().fold(f64::INFINITY, f64::min);
    let max = samples_ms.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    (mean, min, max)
}

fn run_memory_benchmark(know_now: &Path, project: &Path, budget_mib: u64) -> BenchMemoryCaseResult {
    if !Path::new("/usr/bin/time").exists() {
        return bench_memory_skip(budget_mib, "/usr/bin/time not available on this host");
    }

    let temp_file = std::env::temp_dir().join(format!(
        "know-now-memory-{}-{}.txt",
        std::process::id(),
        rand_suffix()
    ));

    let status = Command::new("/usr/bin/time")
        .arg("-f")
        .arg("%M")
        .arg("-o")
        .arg(&temp_file)
        .arg(know_now)
        .arg("--project")
        .arg(project)
        .arg("generate")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    let status = match status {
        Ok(status) => status,
        Err(err) => {
            return bench_memory_fail(
                budget_mib,
                None,
                format!("failed to run memory check: {err}"),
            )
        }
    };

    if !status.success() {
        return bench_memory_fail(
            budget_mib,
            None,
            format!("memory-check command exited with status {status}"),
        );
    }

    let raw = match fs::read_to_string(&temp_file) {
        Ok(raw) => raw,
        Err(err) => {
            return bench_memory_fail(
                budget_mib,
                None,
                format!("failed to read memory sample: {err}"),
            )
        }
    };

    let kib = match raw.trim().parse::<f64>() {
        Ok(kib) => kib,
        Err(err) => {
            return bench_memory_fail(
                budget_mib,
                None,
                format!("failed to parse memory sample `{}`: {err}", raw.trim()),
            );
        }
    };

    let observed_mib = kib / 1024.0;
    let Ok(budget_mib_u32) = u32::try_from(budget_mib) else {
        return bench_memory_fail(
            budget_mib,
            Some(observed_mib),
            format!("memory budget exceeds supported range: {budget_mib} MiB"),
        );
    };
    let within_budget = observed_mib <= f64::from(budget_mib_u32);
    if within_budget {
        return BenchMemoryCaseResult {
            id: "p11_peak_memory_100_entities",
            requirement: "NFR-P11",
            budget_mib,
            observed_mib: Some(observed_mib),
            within_budget: Some(true),
            status: "pass",
            detail: format!(
                "observed peak memory {observed_mib:.2} MiB vs budget {budget_mib} MiB"
            ),
        };
    }
    bench_memory_fail(
        budget_mib,
        Some(observed_mib),
        format!("observed peak memory {observed_mib:.2} MiB vs budget {budget_mib} MiB"),
    )
}

fn bench_memory_skip(budget_mib: u64, detail: &str) -> BenchMemoryCaseResult {
    BenchMemoryCaseResult {
        id: "p11_peak_memory_100_entities",
        requirement: "NFR-P11",
        budget_mib,
        observed_mib: None,
        within_budget: None,
        status: "skip",
        detail: detail.to_string(),
    }
}

fn bench_memory_fail(
    budget_mib: u64,
    observed_mib: Option<f64>,
    detail: String,
) -> BenchMemoryCaseResult {
    BenchMemoryCaseResult {
        id: "p11_peak_memory_100_entities",
        requirement: "NFR-P11",
        budget_mib,
        observed_mib,
        within_budget: Some(false),
        status: "fail",
        detail,
    }
}

fn rand_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

fn write_benchmark_report(report: &BenchReport) -> anyhow::Result<()> {
    let dir = Path::new("target").join("benchmarks");
    fs::create_dir_all(&dir).with_context(|| {
        format!(
            "failed to create benchmark report directory {}",
            dir.display()
        )
    })?;
    let report_path = dir.join("latest.json");
    let payload = serde_json::to_vec_pretty(report)?;
    fs::write(&report_path, payload)
        .with_context(|| format!("failed to write benchmark report {}", report_path.display()))
}

fn print_benchmark_summary(report: &BenchReport) -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    writeln!(stdout, "Benchmark gate summary:")?;
    writeln!(stdout, "  runs per case: {}", report.runs)?;
    writeln!(
        stdout,
        "  max regression threshold: {:.2}%",
        report.max_regression_pct
    )?;
    writeln!(stdout, "  baseline path: {}", report.baseline_path)?;

    for case in &report.cases {
        writeln!(
            stdout,
            "  [{:4}] {:<28} mean={:>9.2}ms budget={:>8.2}ms ({})",
            case.status.to_ascii_uppercase(),
            case.id,
            case.mean_ms,
            case.budget_ms,
            case.detail
        )?;
    }

    if let Some(memory_case) = &report.memory_case {
        writeln!(
            stdout,
            "  [{:4}] {:<28} {}",
            memory_case.status.to_ascii_uppercase(),
            memory_case.id,
            memory_case.detail
        )?;
    }

    if !report.deferred_cases.is_empty() {
        writeln!(stdout, "  deferred cases:")?;
        for deferred in &report.deferred_cases {
            writeln!(
                stdout,
                "    - {} ({}) {}",
                deferred.id, deferred.requirement, deferred.reason
            )?;
        }
    }

    let report_path = Path::new("target/benchmarks/latest.json");
    writeln!(stdout, "  report: {}", report_path.display())?;
    Ok(())
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

fn cmd_docs_cli_ref() -> anyhow::Result<()> {
    let know_now = Path::new("target/debug/know-now");
    if !know_now.exists() {
        run_external_command("cargo", ["build", "-p", "know_now_cli"])?;
    }

    let commands: &[&[&str]] = &[
        &[],
        &["init"],
        &["validate"],
        &["check"],
        &["schema"],
        &["generate"],
        &["lock", "update"],
        &["lock", "check"],
        &["id", "check"],
        &["id", "suggest"],
        &["examples", "list"],
        &["config", "inspect"],
        &["version"],
    ];

    let mut sections = Vec::new();

    sections.push(
        "# CLI reference\n\n\
         > Auto-generated by `cargo xtask docs cli-ref`. Do not edit by hand.\n"
            .to_string(),
    );

    for args in commands {
        let mut cmd_args: Vec<&str> = args.to_vec();
        cmd_args.push("--help");

        let output = Command::new(know_now)
            .args(&cmd_args)
            .output()
            .with_context(|| format!("failed to run know-now {}", args.join(" ")))?;

        let help_text = String::from_utf8(output.stdout)
            .context("know-now --help output was not valid UTF-8")?;

        let heading = if args.is_empty() {
            "know-now".to_string()
        } else {
            format!("know-now {}", args.join(" "))
        };

        sections.push(format!("## `{heading}`\n\n```\n{help_text}```\n"));
    }

    let content = sections.join("\n");
    let out_path = Path::new("docs/user/cli-reference.md");
    fs::write(out_path, &content).context("failed to write docs/user/cli-reference.md")?;

    let mut stdout = io::stdout();
    writeln!(
        stdout,
        "Wrote {} bytes to {}",
        content.len(),
        out_path.display()
    )?;
    Ok(())
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
        E2ePhase::Foundation => run_e2e_foundation(),
        E2ePhase::Phase1 => run_e2e_script("tests/e2e/phase-1.sh", phase.as_str()),
        E2ePhase::Phase2A => run_e2e_script("tests/e2e/phase-2a.sh", phase.as_str()),
        E2ePhase::Phase2B => run_e2e_script("tests/e2e/phase-2b.sh", phase.as_str()),
        E2ePhase::Phase3 => run_e2e_script("tests/e2e/phase-3.sh", phase.as_str()),
        E2ePhase::All => unreachable!("expanded by caller"),
    }
}

fn run_e2e_foundation() -> anyhow::Result<String> {
    let mut stdout = io::stdout();
    let mut steps_passed = 0u32;

    writeln!(stdout, "  [foundation] step 1: cargo build --workspace")?;
    run_external_command("cargo", ["build", "--workspace"])?;
    steps_passed += 1;

    writeln!(stdout, "  [foundation] step 2: cargo test --workspace")?;
    run_external_command("cargo", ["test", "--workspace"])?;
    steps_passed += 1;

    writeln!(stdout, "  [foundation] step 3: cargo fmt --all -- --check")?;
    run_external_command("cargo", ["fmt", "--all", "--", "--check"])?;
    steps_passed += 1;

    writeln!(
        stdout,
        "  [foundation] step 4: cargo clippy --all-targets -- -D warnings"
    )?;
    run_external_command(
        "cargo",
        [
            "clippy",
            "--all-targets",
            "--all-features",
            "--",
            "-D",
            "warnings",
        ],
    )?;
    steps_passed += 1;

    writeln!(stdout, "  [foundation] step 5: cargo deny check")?;
    run_external_command("cargo", ["deny", "check"])?;
    steps_passed += 1;

    writeln!(stdout, "  [foundation] step 6: architecture fitness tests")?;
    run_external_command("cargo", ["test", "-p", "know_now_fitness"])?;
    steps_passed += 1;

    writeln!(stdout, "  [foundation] step 7: frontend boot")?;
    if Path::new("web/package.json").exists() {
        let web = Path::new("web");
        run_external_in_dir("pnpm", &["install", "--frozen-lockfile"], web)?;
        run_external_in_dir("pnpm", &["typecheck"], web)?;
        run_external_in_dir("pnpm", &["build"], web)?;
        steps_passed += 1;
    } else {
        writeln!(stdout, "    skipped (web/package.json not found)")?;
    }

    writeln!(stdout, "  [foundation] step 8: beads graph hygiene")?;
    if command_available("br") {
        let cycles_raw = run_command_capture("br", ["dep", "cycles"])?;
        let cycles_lower = cycles_raw.to_ascii_lowercase();
        if !(cycles_lower.contains("no dependency cycles")
            || cycles_lower.contains("no cycles detected"))
        {
            anyhow::bail!("dependency cycles detected in beads graph:\n{cycles_raw}");
        }
        steps_passed += 1;
    } else {
        writeln!(stdout, "    skipped (br not available)")?;
    }

    writeln!(stdout, "  [foundation] step 9: docs check")?;
    cmd_docs_check()?;
    steps_passed += 1;

    Ok(format!("{steps_passed} steps passed"))
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

fn run_external_in_dir(program: &str, args: &[&str], dir: &Path) -> anyhow::Result<()> {
    let status = Command::new(program)
        .args(args)
        .current_dir(dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("failed to execute `{program}` in {}", dir.display()))?;

    if status.success() {
        return Ok(());
    }

    anyhow::bail!(
        "`{program}` in {} exited with status {status}",
        dir.display()
    )
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

fn run_rustdoc_step() -> StepResult {
    let status = Command::new("cargo")
        .args(["doc", "--workspace", "--no-deps"])
        .env("RUSTDOCFLAGS", "-D warnings")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    match status {
        Ok(exit) if exit.success() => StepResult {
            name: "rustdoc",
            status: StepStatus::Pass,
            detail: "ok".to_string(),
        },
        Ok(exit) => StepResult {
            name: "rustdoc",
            status: StepStatus::Fail,
            detail: format!("exit status {exit}"),
        },
        Err(err) => StepResult {
            name: "rustdoc",
            status: StepStatus::Fail,
            detail: format!("failed to execute cargo doc: {err}"),
        },
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
