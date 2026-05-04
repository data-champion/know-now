use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::context::CommandContext;
use crate::output::{JsonEnvelope, OutputFormat};

#[derive(Debug, clap::Subcommand)]
pub enum IssuesCommand {
    /// List issues (default: unresolved)
    List(IssuesListArgs),

    /// Mark an issue as resolved
    Resolve(IssuesResolveArgs),

    /// Snooze an issue with a reason
    Snooze(IssuesSnoozeArgs),
}

#[derive(Debug, clap::Args)]
pub struct IssuesListArgs {
    /// Filter by status
    #[arg(long, value_enum)]
    pub status: Option<IssueStatusFilter>,
}

#[derive(Debug, clap::Args)]
pub struct IssuesResolveArgs {
    /// Issue ID to resolve
    pub id: String,
}

#[derive(Debug, clap::Args)]
pub struct IssuesSnoozeArgs {
    /// Issue ID to snooze
    pub id: String,

    /// Reason for snoozing
    #[arg(long)]
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum IssueStatusFilter {
    Open,
    InProgress,
    Resolved,
    Snoozed,
    All,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Issue {
    pub id: String,
    pub affected_object: String,
    pub change_type: String,
    pub description: String,
    pub suggested_fix: String,
    pub status: String,
    pub snooze_reason: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

pub fn run(ctx: &CommandContext, cmd: &IssuesCommand) -> anyhow::Result<()> {
    match cmd {
        IssuesCommand::List(args) => run_list(ctx, args),
        IssuesCommand::Resolve(args) => run_resolve(ctx, args),
        IssuesCommand::Snooze(args) => run_snooze(ctx, args),
    }
}

fn issues_path(ctx: &CommandContext) -> PathBuf {
    ctx.project_root.join(".knownow").join("issues.json")
}

fn load_issues(path: &Path) -> Vec<Issue> {
    let Ok(content) = fs::read_to_string(path) else {
        return Vec::new();
    };
    serde_json::from_str(&content).unwrap_or_default()
}

fn save_issues(path: &Path, issues: &[Issue]) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(issues)?;
    fs::write(path, json)?;
    Ok(())
}

fn run_list(ctx: &CommandContext, args: &IssuesListArgs) -> anyhow::Result<()> {
    let path = issues_path(ctx);
    let all_issues = load_issues(&path);

    let filtered: Vec<&Issue> = match args.status {
        Some(IssueStatusFilter::All) => all_issues.iter().collect(),
        Some(IssueStatusFilter::Open) => {
            all_issues.iter().filter(|i| i.status == "open").collect()
        }
        Some(IssueStatusFilter::InProgress) => all_issues
            .iter()
            .filter(|i| i.status == "in_progress")
            .collect(),
        Some(IssueStatusFilter::Resolved) => all_issues
            .iter()
            .filter(|i| i.status == "resolved")
            .collect(),
        Some(IssueStatusFilter::Snoozed) => all_issues
            .iter()
            .filter(|i| i.status == "snoozed")
            .collect(),
        None => all_issues
            .iter()
            .filter(|i| i.status != "resolved")
            .collect(),
    };

    match ctx.format {
        OutputFormat::Json | OutputFormat::Sarif => {
            let envelope = JsonEnvelope::success("issues list", &filtered);
            println!("{}", serde_json::to_string_pretty(&envelope)?);
        }
        OutputFormat::Text | OutputFormat::Quiet => {
            if filtered.is_empty() {
                println!("No issues found.");
            } else {
                println!("{} issue(s):", filtered.len());
                println!();
                for issue in &filtered {
                    print_issue_text(issue);
                }
            }
        }
    }

    Ok(())
}

fn run_resolve(ctx: &CommandContext, args: &IssuesResolveArgs) -> anyhow::Result<()> {
    let path = issues_path(ctx);
    let mut issues = load_issues(&path);

    let idx = issues
        .iter()
        .position(|i| i.id == args.id)
        .ok_or_else(|| anyhow::anyhow!("ISSUE-NOT-FOUND-001: no issue with id '{}'", args.id))?;

    issues[idx].status = "resolved".into();
    issues[idx].updated_at = Some(now_iso8601());

    save_issues(&path, &issues)?;

    match ctx.format {
        OutputFormat::Json | OutputFormat::Sarif => {
            let envelope = JsonEnvelope::success("issues resolve", &issues[idx]);
            println!("{}", serde_json::to_string_pretty(&envelope)?);
        }
        OutputFormat::Text | OutputFormat::Quiet => {
            println!("Resolved issue '{}'.", args.id);
        }
    }

    Ok(())
}

fn run_snooze(ctx: &CommandContext, args: &IssuesSnoozeArgs) -> anyhow::Result<()> {
    let path = issues_path(ctx);
    let mut issues = load_issues(&path);

    let idx = issues
        .iter()
        .position(|i| i.id == args.id)
        .ok_or_else(|| anyhow::anyhow!("ISSUE-NOT-FOUND-001: no issue with id '{}'", args.id))?;

    issues[idx].status = "snoozed".into();
    issues[idx].snooze_reason = Some(args.reason.clone());
    issues[idx].updated_at = Some(now_iso8601());

    save_issues(&path, &issues)?;

    match ctx.format {
        OutputFormat::Json | OutputFormat::Sarif => {
            let envelope = JsonEnvelope::success("issues snooze", &issues[idx]);
            println!("{}", serde_json::to_string_pretty(&envelope)?);
        }
        OutputFormat::Text | OutputFormat::Quiet => {
            println!("Snoozed issue '{}': {}", args.id, args.reason);
        }
    }

    Ok(())
}

fn print_issue_text(issue: &Issue) {
    let status_marker = match issue.status.as_str() {
        "open" => "[OPEN]",
        "in_progress" => "[IN PROGRESS]",
        "resolved" => "[RESOLVED]",
        "snoozed" => "[SNOOZED]",
        _ => "[?]",
    };
    println!("  {} {} — {}", status_marker, issue.id, issue.description);
    println!(
        "    object: {}, change: {}",
        issue.affected_object, issue.change_type
    );
    if !issue.suggested_fix.is_empty() {
        println!("    fix: {}", issue.suggested_fix);
    }
    if let Some(ref reason) = issue.snooze_reason {
        println!("    snooze reason: {reason}");
    }
    println!();
}

fn now_iso8601() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
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
