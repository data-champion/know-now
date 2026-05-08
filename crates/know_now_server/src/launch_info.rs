//! Launch info file persistence — `<project_root>/.knownow/launch.json`.
//!
//! Written by `know-now serve` at startup so `know-now session-url` and dev
//! tooling can discover the launch URL without scraping stdout. Removed on
//! graceful shutdown. The token in this file is the same single-use launch
//! token printed to stdout — file permissions (0600 on Unix) are the
//! authorization boundary.

use std::net::IpAddr;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const LAUNCH_INFO_SCHEMA_VERSION: u32 = 1;
pub const LAUNCH_INFO_RELATIVE_PATH: &str = ".knownow/launch.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchInfo {
    pub version: u32,
    pub scheme: String,
    pub host: String,
    pub port: u16,
    pub token: String,
    pub url: String,
    pub pid: u32,
}

impl LaunchInfo {
    #[must_use]
    pub fn new(host: IpAddr, port: u16, token: String, url: String) -> Self {
        Self {
            version: LAUNCH_INFO_SCHEMA_VERSION,
            scheme: "http".to_owned(),
            host: host.to_string(),
            port,
            token,
            url,
            pid: std::process::id(),
        }
    }
}

#[must_use]
pub fn launch_info_path(project_root: &Path) -> PathBuf {
    project_root.join(LAUNCH_INFO_RELATIVE_PATH)
}

/// Write launch info to `<project_root>/.knownow/launch.json` with restrictive
/// permissions on Unix (0600). Creates `.knownow/` if missing.
///
/// # Errors
/// Returns an error if `.knownow/` cannot be created, the file cannot be
/// written, or serialization fails.
pub fn write_launch_info(project_root: &Path, info: &LaunchInfo) -> std::io::Result<PathBuf> {
    let dir = project_root.join(".knownow");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("launch.json");
    let contents = serde_json::to_string_pretty(info)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    write_file_restricted(&path, contents.as_bytes())?;
    Ok(path)
}

/// Best-effort removal. Ignores errors — caller is in shutdown path.
pub fn remove_launch_info(path: &Path) {
    let _ = std::fs::remove_file(path);
}

/// Read launch info from `<project_root>/.knownow/launch.json`.
///
/// # Errors
/// Returns an error if the file is missing, unreadable, or cannot be parsed
/// as a current-schema [`LaunchInfo`].
pub fn read_launch_info(project_root: &Path) -> std::io::Result<LaunchInfo> {
    let path = launch_info_path(project_root);
    let bytes = std::fs::read(&path)?;
    serde_json::from_slice(&bytes)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

#[cfg(unix)]
fn write_file_restricted(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;

    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?;
    f.write_all(contents)?;
    Ok(())
}

#[cfg(not(unix))]
fn write_file_restricted(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    std::fs::write(path, contents)
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use super::*;

    #[test]
    fn round_trip_serde() {
        let info = LaunchInfo::new(
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            3827,
            "abc-123".to_owned(),
            "http://127.0.0.1:3827/__open?launch_token=abc-123".to_owned(),
        );
        let json = serde_json::to_string(&info).unwrap();
        let parsed: LaunchInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.token, "abc-123");
        assert_eq!(parsed.port, 3827);
        assert_eq!(parsed.version, LAUNCH_INFO_SCHEMA_VERSION);
    }

    #[test]
    fn write_and_read_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let info = LaunchInfo::new(
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            3827,
            "tok".to_owned(),
            "http://127.0.0.1:3827/__open?launch_token=tok".to_owned(),
        );
        let path = write_launch_info(dir.path(), &info).unwrap();
        assert!(path.ends_with("launch.json"));
        let read = read_launch_info(dir.path()).unwrap();
        assert_eq!(read.token, "tok");
    }

    #[test]
    fn remove_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let info = LaunchInfo::new(
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            3827,
            "tok".to_owned(),
            "http://127.0.0.1:3827/__open?launch_token=tok".to_owned(),
        );
        let path = write_launch_info(dir.path(), &info).unwrap();
        remove_launch_info(&path);
        remove_launch_info(&path);
        assert!(!path.exists());
    }

    #[cfg(unix)]
    #[test]
    fn unix_permissions_are_0600() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let info = LaunchInfo::new(
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            3827,
            "tok".to_owned(),
            "http://127.0.0.1:3827/__open?launch_token=tok".to_owned(),
        );
        let path = write_launch_info(dir.path(), &info).unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }
}
