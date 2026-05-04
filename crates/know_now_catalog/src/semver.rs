/// Parsed semver triple.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    pub major: u64,
    pub minor: Option<u64>,
    pub patch: Option<u64>,
}

impl Version {
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('.').collect();
        let major = parts.first()?.parse().ok()?;
        let minor = parts.get(1).and_then(|p| p.parse().ok());
        let patch = parts.get(2).and_then(|p| p.parse().ok());
        Some(Self { major, minor, patch })
    }
}

/// A version range like "1.0.x", "1.x", "1.0.3", or "1".
#[derive(Debug, Clone)]
pub struct VersionRange {
    pub major: u64,
    pub minor: RangePart,
    pub patch: RangePart,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RangePart {
    Exact(u64),
    Wildcard,
    Absent,
}

impl VersionRange {
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('.').collect();
        let major = parts.first()?.parse().ok()?;
        let minor = parts.get(1).map_or(RangePart::Absent, |p| {
            if *p == "x" || *p == "*" {
                RangePart::Wildcard
            } else {
                p.parse().map_or(RangePart::Absent, RangePart::Exact)
            }
        });
        let patch = parts.get(2).map_or(RangePart::Absent, |p| {
            if *p == "x" || *p == "*" {
                RangePart::Wildcard
            } else {
                p.parse().map_or(RangePart::Absent, RangePart::Exact)
            }
        });
        Some(Self { major, minor, patch })
    }

    pub fn matches(&self, version: &Version) -> bool {
        if version.major != self.major {
            return false;
        }
        match (&self.minor, version.minor) {
            (RangePart::Wildcard | RangePart::Absent, _) => return true,
            (RangePart::Exact(expected), Some(actual)) if *expected != actual => return false,
            (RangePart::Exact(_), None) => return false,
            _ => {}
        }
        match (&self.patch, version.patch) {
            (RangePart::Wildcard | RangePart::Absent, _) => true,
            (RangePart::Exact(expected), Some(actual)) => *expected == actual,
            (RangePart::Exact(_), None) => false,
        }
    }
}

pub fn matches_any(version_str: &str, ranges: &[String]) -> bool {
    let Some(version) = Version::parse(version_str) else {
        return false;
    };
    ranges
        .iter()
        .filter_map(|r| VersionRange::parse(r))
        .any(|range| range.matches(&version))
}

/// Classify what kind of version difference exists.
pub fn classify_version_diff(installed: &str, ranges: &[String]) -> VersionDiff {
    let Some(version) = Version::parse(installed) else {
        return VersionDiff::Unknown;
    };

    if ranges.iter().filter_map(|r| VersionRange::parse(r)).any(|range| range.matches(&version)) {
        return VersionDiff::None;
    }

    let approved_versions: Vec<Version> = ranges
        .iter()
        .filter_map(|r| Version::parse(r))
        .collect();

    if approved_versions.is_empty() {
        let parsed_ranges: Vec<VersionRange> = ranges.iter().filter_map(|r| VersionRange::parse(r)).collect();
        if parsed_ranges.is_empty() {
            return VersionDiff::Unknown;
        }
        let any_same_major = parsed_ranges.iter().any(|r| r.major == version.major);
        if any_same_major {
            let any_same_minor = parsed_ranges.iter().any(|r| {
                r.major == version.major && match (&r.minor, version.minor) {
                    (RangePart::Exact(e), Some(a)) => *e == a,
                    (RangePart::Wildcard, _) => true,
                    _ => false,
                }
            });
            if any_same_minor {
                return VersionDiff::Patch;
            }
            return VersionDiff::Minor;
        }
        return VersionDiff::Major;
    }

    let same_major = approved_versions.iter().any(|v| v.major == version.major);
    if !same_major {
        return VersionDiff::Major;
    }

    let same_minor = approved_versions.iter().any(|v| {
        v.major == version.major && v.minor == version.minor
    });
    if !same_minor {
        return VersionDiff::Minor;
    }

    VersionDiff::Patch
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionDiff {
    None,
    Patch,
    Minor,
    Major,
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_version() {
        let v = Version::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, Some(2));
        assert_eq!(v.patch, Some(3));
    }

    #[test]
    fn parse_version_major_only() {
        let v = Version::parse("1").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, None);
        assert_eq!(v.patch, None);
    }

    #[test]
    fn range_wildcard_patch() {
        let range = VersionRange::parse("1.0.x").unwrap();
        assert!(range.matches(&Version::parse("1.0.0").unwrap()));
        assert!(range.matches(&Version::parse("1.0.99").unwrap()));
        assert!(!range.matches(&Version::parse("1.1.0").unwrap()));
        assert!(!range.matches(&Version::parse("2.0.0").unwrap()));
    }

    #[test]
    fn range_wildcard_minor() {
        let range = VersionRange::parse("1.x").unwrap();
        assert!(range.matches(&Version::parse("1.0.0").unwrap()));
        assert!(range.matches(&Version::parse("1.99.0").unwrap()));
        assert!(!range.matches(&Version::parse("2.0.0").unwrap()));
    }

    #[test]
    fn range_exact() {
        let range = VersionRange::parse("1.0.3").unwrap();
        assert!(range.matches(&Version::parse("1.0.3").unwrap()));
        assert!(!range.matches(&Version::parse("1.0.4").unwrap()));
    }

    #[test]
    fn range_major_only() {
        let range = VersionRange::parse("1").unwrap();
        assert!(range.matches(&Version::parse("1.0.0").unwrap()));
        assert!(range.matches(&Version::parse("1.5.3").unwrap()));
        assert!(!range.matches(&Version::parse("2.0.0").unwrap()));
    }

    #[test]
    fn matches_any_works() {
        let ranges = vec!["1.0.x".into(), "1.1.x".into()];
        assert!(matches_any("1.0.5", &ranges));
        assert!(matches_any("1.1.0", &ranges));
        assert!(!matches_any("1.2.0", &ranges));
        assert!(!matches_any("2.0.0", &ranges));
    }

    #[test]
    fn classify_none_for_matching() {
        let ranges = vec!["1.0.x".into()];
        assert_eq!(classify_version_diff("1.0.5", &ranges), VersionDiff::None);
    }

    #[test]
    fn classify_patch_diff() {
        let ranges = vec!["1.0.3".into()];
        assert_eq!(classify_version_diff("1.0.5", &ranges), VersionDiff::Patch);
    }

    #[test]
    fn classify_minor_diff() {
        let ranges = vec!["1.0.x".into()];
        assert_eq!(classify_version_diff("1.2.0", &ranges), VersionDiff::Minor);
    }

    #[test]
    fn classify_major_diff() {
        let ranges = vec!["1.0.x".into()];
        assert_eq!(classify_version_diff("2.0.0", &ranges), VersionDiff::Major);
    }

    #[test]
    fn classify_unknown_for_unparseable() {
        let ranges = vec!["1.0.x".into()];
        assert_eq!(classify_version_diff("abc", &ranges), VersionDiff::Unknown);
    }
}
