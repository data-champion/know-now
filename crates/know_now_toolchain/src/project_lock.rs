use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

const LOCK_FILE: &str = "writer.lock";
const LOCK_INFO_FILE: &str = "writer.lock.info";
const POLL_INTERVAL: Duration = Duration::from_millis(200);

pub const DEFAULT_LOCK_TIMEOUT_SECS: u64 = 30;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockInfo {
    pub pid: u32,
    pub command: String,
    pub acquired_at: String,
}

pub struct ProjectLockGuard {
    _file: File,
    info_path: PathBuf,
}

impl Drop for ProjectLockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.info_path);
        // Do NOT delete lock_path here. On POSIX, flock is tied to the inode.
        // Unlinking while the fd is open lets another process create + lock a
        // *new* file at the same path, producing two concurrent lock holders.
        // The OS releases the advisory lock when _file is dropped (fd closed).
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LockError {
    #[error("LOCK-ACQUIRE: could not create lock file at {path}: {reason}")]
    CreateFailed { path: String, reason: String },

    #[error(
        "LOCK-TIMEOUT: could not acquire project lock within {timeout_secs}s. \
         Lock held by PID {holder_pid} ({holder_command}) since {holder_acquired_at}"
    )]
    Timeout {
        timeout_secs: u64,
        holder_pid: u32,
        holder_command: String,
        holder_acquired_at: String,
    },

    #[error(
        "LOCK-TIMEOUT: could not acquire project lock within {timeout_secs}s (no holder info available)"
    )]
    TimeoutNoInfo { timeout_secs: u64 },

    #[error("LOCK-IO: {reason}")]
    Io { reason: String },
}

impl LockError {
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::CreateFailed { .. } => "LOCK-ACQUIRE",
            Self::Timeout { .. } | Self::TimeoutNoInfo { .. } => "LOCK-TIMEOUT",
            Self::Io { .. } => "LOCK-IO",
        }
    }
}

/// Acquire an exclusive advisory lock for write operations.
///
/// The lock file is created at `<locks_dir>/writer.lock` using the OS advisory
/// lock (`flock` on POSIX, `LockFileEx` on Windows). A companion info file
/// records the holder's PID, command, and acquisition time for diagnostics.
///
/// # Errors
///
/// Returns `LockError::Timeout` if the lock cannot be acquired within
/// `timeout`, or `LockError::CreateFailed` on I/O failure.
pub fn acquire(
    locks_dir: &Path,
    command_name: &str,
    timeout: Duration,
) -> Result<ProjectLockGuard, LockError> {
    let lock_path = locks_dir.join(LOCK_FILE);
    let info_path = locks_dir.join(LOCK_INFO_FILE);

    fs::create_dir_all(locks_dir).map_err(|e| LockError::CreateFailed {
        path: locks_dir.display().to_string(),
        reason: e.to_string(),
    })?;

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lock_path)
        .map_err(|e| LockError::CreateFailed {
            path: lock_path.display().to_string(),
            reason: e.to_string(),
        })?;

    let deadline = Instant::now() + timeout;
    loop {
        match file.try_lock() {
            Ok(()) => {
                write_lock_info(&info_path, command_name)?;
                return Ok(ProjectLockGuard {
                    _file: file,
                    info_path,
                });
            }
            Err(std::fs::TryLockError::WouldBlock) => {
                if Instant::now() >= deadline {
                    return Err(timeout_error(&info_path, timeout));
                }
                thread::sleep(POLL_INTERVAL);
            }
            Err(std::fs::TryLockError::Error(e)) => {
                return Err(LockError::Io {
                    reason: e.to_string(),
                });
            }
        }
    }
}

/// Read diagnostic info about the current lock holder, if available.
pub fn read_holder_info(locks_dir: &Path) -> Option<LockInfo> {
    let info_path = locks_dir.join(LOCK_INFO_FILE);
    let content = fs::read_to_string(&info_path).ok()?;
    serde_json::from_str(&content).ok()
}

fn write_lock_info(info_path: &Path, command_name: &str) -> Result<(), LockError> {
    let info = LockInfo {
        pid: std::process::id(),
        command: command_name.to_owned(),
        acquired_at: now_iso8601(),
    };
    let json = serde_json::to_string_pretty(&info).map_err(|e| LockError::Io {
        reason: e.to_string(),
    })?;
    fs::write(info_path, json).map_err(|e| LockError::Io {
        reason: e.to_string(),
    })?;
    Ok(())
}

fn timeout_error(info_path: &Path, timeout: Duration) -> LockError {
    let timeout_secs = timeout.as_secs();
    match fs::read_to_string(info_path)
        .ok()
        .and_then(|s| serde_json::from_str::<LockInfo>(&s).ok())
    {
        Some(info) => LockError::Timeout {
            timeout_secs,
            holder_pid: info.pid,
            holder_command: info.command,
            holder_acquired_at: info.acquired_at,
        },
        None => LockError::TimeoutNoInfo { timeout_secs },
    }
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
    let days = secs / 86400;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("know_now_lock_{name}_{}", std::process::id()))
    }

    #[test]
    fn acquire_and_release() {
        let dir = test_dir("acquire");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let lock_path = dir.join(LOCK_FILE);
        let info_path = dir.join(LOCK_INFO_FILE);

        {
            let guard = acquire(&dir, "generate", Duration::from_secs(5)).unwrap();
            assert!(lock_path.exists());
            assert!(info_path.exists());

            let info: LockInfo =
                serde_json::from_str(&fs::read_to_string(&info_path).unwrap()).unwrap();
            assert_eq!(info.pid, std::process::id());
            assert_eq!(info.command, "generate");
            assert!(!info.acquired_at.is_empty());

            drop(guard);
        }

        assert!(!info_path.exists());
        assert!(
            lock_path.exists(),
            "lock file persists after release (OS releases advisory lock)"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn lock_info_contains_pid_and_command() {
        let dir = test_dir("info");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let guard = acquire(&dir, "lock update", Duration::from_secs(5)).unwrap();
        let info = read_holder_info(&dir).expect("info should be readable");
        assert_eq!(info.pid, std::process::id());
        assert_eq!(info.command, "lock update");
        drop(guard);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn reacquire_after_release() {
        let dir = test_dir("reacquire");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let guard1 = acquire(&dir, "generate", Duration::from_secs(5)).unwrap();
        drop(guard1);

        let guard2 = acquire(&dir, "lock update", Duration::from_secs(5)).unwrap();
        let info = read_holder_info(&dir).unwrap();
        assert_eq!(info.command, "lock update");
        drop(guard2);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn lock_error_codes_are_stable() {
        let errors: Vec<LockError> = vec![
            LockError::CreateFailed {
                path: String::new(),
                reason: String::new(),
            },
            LockError::Timeout {
                timeout_secs: 30,
                holder_pid: 1234,
                holder_command: "generate".into(),
                holder_acquired_at: "2026-01-01T00:00:00Z".into(),
            },
            LockError::TimeoutNoInfo { timeout_secs: 30 },
            LockError::Io {
                reason: String::new(),
            },
        ];

        let codes = ["LOCK-ACQUIRE", "LOCK-TIMEOUT", "LOCK-TIMEOUT", "LOCK-IO"];

        for (error, expected) in errors.iter().zip(codes.iter()) {
            assert_eq!(error.code(), *expected);
        }
    }

    #[test]
    fn timeout_error_includes_holder_info() {
        let dir = test_dir("timeout_info");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let info_path = dir.join(LOCK_INFO_FILE);
        let info = LockInfo {
            pid: 9999,
            command: "generate --locked".into(),
            acquired_at: "2026-05-04T12:00:00Z".into(),
        };
        fs::write(&info_path, serde_json::to_string(&info).unwrap()).unwrap();

        let err = timeout_error(&info_path, Duration::from_secs(30));
        let msg = err.to_string();
        assert!(msg.contains("PID 9999"));
        assert!(msg.contains("generate --locked"));
        assert!(msg.contains("2026-05-04T12:00:00Z"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn timeout_error_without_info_file() {
        let dir = test_dir("timeout_noinfo");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let info_path = dir.join(LOCK_INFO_FILE);
        let err = timeout_error(&info_path, Duration::from_secs(30));
        assert!(err.to_string().contains("no holder info"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn now_iso8601_is_reasonable() {
        let ts = now_iso8601();
        assert!(ts.starts_with("20"));
        assert!(ts.ends_with('Z'));
        assert!(ts.contains('T'));
    }

    #[test]
    fn lock_info_serializes() {
        let info = LockInfo {
            pid: 42,
            command: "generate".into(),
            acquired_at: "2026-05-04T12:00:00Z".into(),
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"pid\":42"));
        assert!(json.contains("\"command\":\"generate\""));
    }
}
