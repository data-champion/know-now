use std::path::PathBuf;
use std::process;
use std::time::Duration;

use clap::{Parser, Subcommand};

use know_now_cli::audit::{self, AuditEntry};
use know_now_cli::commands::{
    admin, check, config, diff, doctor, examples, explain, generate, id, init, issues, lock,
    policy, review, schema, serve, session, support, validate, version,
};
use know_now_cli::context::CommandContext;
use know_now_cli::exit_code;
use know_now_cli::output::OutputFormat;
use know_now_toolchain::project_lock;

/// Local-first metadata-driven data platform generation engine
#[derive(Debug, Parser)]
#[command(name = "know-now", version, about, long_about = None)]
struct Cli {
    /// Output format
    #[arg(long, value_enum, global = true, default_value = "text")]
    format: OutputFormat,

    /// Enable verbose output with pipeline steps and timings
    #[arg(long, global = true)]
    verbose: bool,

    /// Enable debug output with diagnostic details
    #[arg(long, global = true)]
    debug: bool,

    /// Path to configuration file
    #[arg(long, global = true)]
    config: Option<PathBuf>,

    /// Path to project root
    #[arg(long, global = true)]
    project: Option<PathBuf>,

    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Initialize a new know-now project
    Init(init::InitArgs),

    /// Parse and validate metadata files
    Validate(validate::ValidateArgs),

    /// Run recommended local and CI checks
    Check(check::CheckArgs),

    /// Export JSON Schema for metadata files
    Schema(schema::SchemaArgs),

    /// Generate artifacts from validated metadata
    Generate(generate::GenerateArgs),

    /// Compare current metadata against a baseline
    Diff(diff::DiffArgs),

    /// Check project, toolchain, and configuration health
    Doctor(doctor::DoctorArgs),

    /// Explain generated artifacts, trace metadata origins
    Explain(explain::ExplainArgs),

    /// Track and manage deprecation issues
    #[command(subcommand)]
    Issues(issues::IssuesCommand),

    /// Lockfile operations
    #[command(subcommand)]
    Lock(lock::LockCommand),

    /// Stable object ID operations
    #[command(subcommand)]
    Id(id::IdCommand),

    /// Example project operations
    #[command(subcommand)]
    Examples(examples::ExamplesCommand),

    /// Policy pack status and rule documentation
    #[command(subcommand)]
    Policy(policy::PolicyCommand),

    /// Export review packs for stakeholder review
    #[command(subcommand)]
    Review(review::ReviewCommand),

    /// Create a sanitized support bundle for diagnostics
    Support(support::SupportBundleArgs),

    /// Start the local development server
    Serve(serve::ServeArgs),

    /// Print the launch URL for a running `know-now serve` (reads .knownow/launch.json)
    SessionUrl(session::SessionUrlArgs),

    /// Administrative fleet-wide operations
    #[command(subcommand)]
    Admin(admin::AdminCommand),

    /// Configuration operations
    #[command(subcommand)]
    Config(config::ConfigCommand),

    /// Show version information
    Version(version::VersionArgs),
}

fn main() {
    let cli = Cli::parse();

    let project_root = cli
        .project
        .unwrap_or_else(|| std::env::current_dir().expect("failed to determine current directory"));

    let ctx = CommandContext {
        format: cli.format,
        verbose: cli.verbose,
        debug: cli.debug,
        no_color: cli.no_color,
        project_root,
        config_path: cli.config,
    };

    let lock_guard = if requires_write_lock(&cli.command) {
        let locks_dir = ctx.project_root.join(".knownow").join("locks");
        let cmd_name = write_lock_command_name(&cli.command);
        match project_lock::acquire(
            &locks_dir,
            cmd_name,
            Duration::from_secs(project_lock::DEFAULT_LOCK_TIMEOUT_SECS),
        ) {
            Ok(guard) => Some(guard),
            Err(e) => {
                eprintln!("error: {e}");
                process::exit(exit_code::VALIDATION_ERROR);
            }
        }
    } else {
        None
    };

    let result = match &cli.command {
        Command::Init(args) => init::run(&ctx, args),
        Command::Validate(args) => validate::run(&ctx, args),
        Command::Check(args) => check::run(&ctx, args),
        Command::Schema(args) => schema::run(&ctx, args),
        Command::Generate(args) => generate::run(&ctx, args),
        Command::Diff(args) => diff::run(&ctx, args),
        Command::Doctor(args) => doctor::run(&ctx, args),
        Command::Explain(args) => explain::run(&ctx, args),
        Command::Issues(cmd) => issues::run(&ctx, cmd),
        Command::Lock(cmd) => lock::run(&ctx, cmd),
        Command::Id(cmd) => id::run(&ctx, cmd),
        Command::Examples(cmd) => examples::run(&ctx, cmd),
        Command::Policy(cmd) => policy::run(&ctx, cmd),
        Command::Review(cmd) => review::run(&ctx, cmd),
        Command::Serve(args) => serve::run(&ctx, args),
        Command::SessionUrl(args) => session::run(&ctx, args),
        Command::Admin(cmd) => admin::run(&ctx, cmd),
        Command::Support(args) => support::run(&ctx, args),
        Command::Config(cmd) => config::run(&ctx, cmd),
        Command::Version(args) => version::run(&ctx, args),
    };

    drop(lock_guard);

    let knownow_dir = ctx.project_root.join(".knownow");
    let (audit_result, audit_error_code) = match &result {
        Ok(()) => ("success", None),
        Err(e) => ("failure", Some(e.to_string())),
    };
    audit::append_audit_entry(
        &knownow_dir,
        &AuditEntry {
            timestamp: audit::now_iso8601(),
            command: command_name(&cli.command).to_owned(),
            engine_version: env!("CARGO_PKG_VERSION").to_owned(),
            project_root: ctx.project_root.display().to_string(),
            result: audit_result.to_owned(),
            error_code: audit_error_code,
        },
    );

    if let Err(err) = result {
        eprintln!("error: {err}");
        process::exit(exit_code::VALIDATION_ERROR);
    }
}

fn requires_write_lock(command: &Command) -> bool {
    matches!(
        command,
        Command::Init(_) | Command::Generate(_) | Command::Lock(lock::LockCommand::Update(_))
    )
}

fn command_name(command: &Command) -> &'static str {
    match command {
        Command::Init(_) => "init",
        Command::Validate(_) => "validate",
        Command::Check(_) => "check",
        Command::Schema(_) => "schema",
        Command::Generate(_) => "generate",
        Command::Diff(_) => "diff",
        Command::Doctor(_) => "doctor",
        Command::Explain(_) => "explain",
        Command::Issues(_) => "issues",
        Command::Lock(_) => "lock",
        Command::Id(_) => "id",
        Command::Examples(_) => "examples",
        Command::Policy(_) => "policy",
        Command::Review(_) => "review",
        Command::Serve(_) => "serve",
        Command::SessionUrl(_) => "session-url",
        Command::Admin(_) => "admin",
        Command::Support(_) => "support",
        Command::Config(_) => "config",
        Command::Version(_) => "version",
    }
}

fn write_lock_command_name(command: &Command) -> &'static str {
    match command {
        Command::Init(_) => "init",
        Command::Generate(_) => "generate",
        Command::Lock(lock::LockCommand::Update(_)) => "lock update",
        _ => "unknown",
    }
}
