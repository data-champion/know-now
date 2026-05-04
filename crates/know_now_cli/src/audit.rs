use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use serde::Serialize;

const AUDIT_LOG_FILE: &str = "audit.log";
const MAX_AUDIT_SIZE: u64 = 10 * 1024 * 1024; // 10 MiB

#[derive(Debug, Serialize)]
pub struct AuditEntry {
    pub timestamp: String,
    pub command: String,
    pub engine_version: String,
    pub project_root: String,
    pub result: String,
    pub error_code: Option<String>,
}

pub fn append_audit_entry(knownow_dir: &Path, entry: &AuditEntry) {
    if let Err(e) = try_append(knownow_dir, entry) {
        eprintln!("audit: failed to write audit log: {e}");
    }
}

fn try_append(knownow_dir: &Path, entry: &AuditEntry) -> std::io::Result<()> {
    fs::create_dir_all(knownow_dir)?;
    let log_path = knownow_dir.join(AUDIT_LOG_FILE);

    rotate_if_needed(&log_path, knownow_dir)?;

    let mut line = serde_json::to_string(entry).unwrap_or_else(|_| "{}".into());
    line.push('\n');

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;
    file.write_all(line.as_bytes())?;
    Ok(())
}

fn rotate_if_needed(log_path: &Path, knownow_dir: &Path) -> std::io::Result<()> {
    let Ok(metadata) = fs::metadata(log_path) else {
        return Ok(());
    };

    if metadata.len() < MAX_AUDIT_SIZE {
        return Ok(());
    }

    let archive_dir = knownow_dir.join("audit");
    fs::create_dir_all(&archive_dir)?;

    let timestamp = now_compact();
    let archive_path = archive_dir.join(format!("{timestamp}.log"));
    fs::rename(log_path, archive_path)?;

    Ok(())
}

pub fn now_iso8601() -> String {
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

fn now_compact() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = secs / 86_400;
    let (year, month, day) = days_to_date(days);
    format!("{year:04}{month:02}{day:02}_{secs}")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_entry_serializes_to_json() {
        let entry = AuditEntry {
            timestamp: "2026-05-04T12:00:00Z".into(),
            command: "generate".into(),
            engine_version: "0.1.0".into(),
            project_root: "/tmp/test".into(),
            result: "success".into(),
            error_code: None,
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"command\":\"generate\""));
        assert!(json.contains("\"result\":\"success\""));
    }

    #[test]
    fn audit_entry_with_error_code() {
        let entry = AuditEntry {
            timestamp: "2026-05-04T12:00:00Z".into(),
            command: "validate".into(),
            engine_version: "0.1.0".into(),
            project_root: "/tmp/test".into(),
            result: "failure".into(),
            error_code: Some("VAL-001".into()),
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"error_code\":\"VAL-001\""));
    }

    #[test]
    fn append_creates_audit_log() {
        let tmp = tempfile::tempdir().unwrap();
        let knownow = tmp.path().join(".knownow");
        let entry = AuditEntry {
            timestamp: "2026-05-04T12:00:00Z".into(),
            command: "test".into(),
            engine_version: "0.1.0".into(),
            project_root: tmp.path().display().to_string(),
            result: "success".into(),
            error_code: None,
        };
        append_audit_entry(&knownow, &entry);
        let log = fs::read_to_string(knownow.join("audit.log")).unwrap();
        assert!(log.contains("\"command\":\"test\""));
        assert!(log.ends_with('\n'));
    }

    #[test]
    fn multiple_appends_create_multiple_lines() {
        let tmp = tempfile::tempdir().unwrap();
        let knownow = tmp.path().join(".knownow");
        for i in 0..3 {
            let entry = AuditEntry {
                timestamp: format!("2026-05-04T12:00:0{i}Z"),
                command: format!("cmd{i}"),
                engine_version: "0.1.0".into(),
                project_root: tmp.path().display().to_string(),
                result: "success".into(),
                error_code: None,
            };
            append_audit_entry(&knownow, &entry);
        }
        let log = fs::read_to_string(knownow.join("audit.log")).unwrap();
        let lines: Vec<_> = log.lines().collect();
        assert_eq!(lines.len(), 3);
        for line in &lines {
            let _: serde_json::Value = serde_json::from_str(line).expect("each line is valid JSON");
        }
    }

    #[test]
    fn no_secrets_in_audit_entry() {
        let entry = AuditEntry {
            timestamp: "2026-05-04T12:00:00Z".into(),
            command: "generate --locked".into(),
            engine_version: "0.1.0".into(),
            project_root: "$HOME/project".into(),
            result: "success".into(),
            error_code: None,
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(!json.contains("password"), "audit must not contain passwords");
        assert!(!json.contains("token"), "audit must not contain tokens");
        assert!(!json.contains("secret"), "audit must not contain secrets");
    }
}
