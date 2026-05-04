use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParserBudgets {
    pub max_file_bytes: u64,
    pub max_nesting_depth: u32,
    pub max_total_nodes: u64,
    pub max_anchor_count: u32,
    pub max_alias_count: u32,
    pub max_keys_per_mapping: u32,
}

impl Default for ParserBudgets {
    fn default() -> Self {
        Self {
            max_file_bytes: 4 * 1024 * 1024,
            max_nesting_depth: 32,
            max_total_nodes: 1_000_000,
            max_anchor_count: 0,
            max_alias_count: 0,
            max_keys_per_mapping: 10_000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BudgetViolation {
    FileSize { actual: u64, limit: u64 },
    NestingDepth { actual: u32, limit: u32 },
    TotalNodes { actual: u64, limit: u64 },
    AnchorCount { actual: u32, limit: u32 },
    AliasCount { actual: u32, limit: u32 },
    KeysPerMapping { actual: u32, limit: u32 },
}

impl BudgetViolation {
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::FileSize { .. } => "META-PAR-SIZE",
            Self::NestingDepth { .. } => "META-PAR-NESTING",
            Self::TotalNodes { .. } => "META-PAR-NODES",
            Self::AnchorCount { .. } => "META-PAR-ANCHOR",
            Self::AliasCount { .. } => "META-PAR-ALIAS",
            Self::KeysPerMapping { .. } => "META-PAR-KEYS",
        }
    }
}

impl std::fmt::Display for BudgetViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileSize { actual, limit } => {
                write!(
                    f,
                    "{}: file is {actual} bytes, limit is {limit}",
                    self.code()
                )
            }
            Self::NestingDepth { actual, limit } => {
                write!(
                    f,
                    "{}: nesting depth {actual} exceeds limit {limit}",
                    self.code()
                )
            }
            Self::TotalNodes { actual, limit } => {
                write!(f, "{}: {actual} nodes exceeds limit {limit}", self.code())
            }
            Self::AnchorCount { actual, limit } => {
                write!(f, "{}: {actual} anchors exceeds limit {limit}", self.code())
            }
            Self::AliasCount { actual, limit } => {
                write!(f, "{}: {actual} aliases exceeds limit {limit}", self.code())
            }
            Self::KeysPerMapping { actual, limit } => {
                write!(
                    f,
                    "{}: {actual} keys per mapping exceeds limit {limit}",
                    self.code()
                )
            }
        }
    }
}

impl ParserBudgets {
    /// # Errors
    /// Returns `BudgetViolation::FileSize` if `size` exceeds `max_file_bytes`.
    pub fn check_file_size(&self, size: u64) -> Result<(), BudgetViolation> {
        if size > self.max_file_bytes {
            return Err(BudgetViolation::FileSize {
                actual: size,
                limit: self.max_file_bytes,
            });
        }
        Ok(())
    }

    /// # Errors
    /// Returns `BudgetViolation::NestingDepth` if `depth` exceeds `max_nesting_depth`.
    pub fn check_nesting_depth(&self, depth: u32) -> Result<(), BudgetViolation> {
        if depth > self.max_nesting_depth {
            return Err(BudgetViolation::NestingDepth {
                actual: depth,
                limit: self.max_nesting_depth,
            });
        }
        Ok(())
    }

    /// # Errors
    /// Returns `BudgetViolation::TotalNodes` if `count` exceeds `max_total_nodes`.
    pub fn check_total_nodes(&self, count: u64) -> Result<(), BudgetViolation> {
        if count > self.max_total_nodes {
            return Err(BudgetViolation::TotalNodes {
                actual: count,
                limit: self.max_total_nodes,
            });
        }
        Ok(())
    }

    /// # Errors
    /// Returns `BudgetViolation::AnchorCount` if `count` exceeds `max_anchor_count`.
    pub fn check_anchor_count(&self, count: u32) -> Result<(), BudgetViolation> {
        if count > self.max_anchor_count {
            return Err(BudgetViolation::AnchorCount {
                actual: count,
                limit: self.max_anchor_count,
            });
        }
        Ok(())
    }

    /// # Errors
    /// Returns `BudgetViolation::AliasCount` if `count` exceeds `max_alias_count`.
    pub fn check_alias_count(&self, count: u32) -> Result<(), BudgetViolation> {
        if count > self.max_alias_count {
            return Err(BudgetViolation::AliasCount {
                actual: count,
                limit: self.max_alias_count,
            });
        }
        Ok(())
    }

    /// # Errors
    /// Returns `BudgetViolation::KeysPerMapping` if `count` exceeds `max_keys_per_mapping`.
    pub fn check_keys_per_mapping(&self, count: u32) -> Result<(), BudgetViolation> {
        if count > self.max_keys_per_mapping {
            return Err(BudgetViolation::KeysPerMapping {
                actual: count,
                limit: self.max_keys_per_mapping,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_finite() {
        let b = ParserBudgets::default();
        assert_eq!(b.max_file_bytes, 4 * 1024 * 1024);
        assert_eq!(b.max_nesting_depth, 32);
        assert_eq!(b.max_total_nodes, 1_000_000);
        assert_eq!(b.max_anchor_count, 0);
        assert_eq!(b.max_alias_count, 0);
        assert_eq!(b.max_keys_per_mapping, 10_000);
    }

    #[test]
    fn file_size_within_budget() {
        let b = ParserBudgets::default();
        assert!(b.check_file_size(1024).is_ok());
    }

    #[test]
    fn file_size_exceeded() {
        let b = ParserBudgets {
            max_file_bytes: 100,
            ..Default::default()
        };
        let err = b.check_file_size(200).unwrap_err();
        assert_eq!(err.code(), "META-PAR-SIZE");
        assert!(err.to_string().contains("200"));
        assert!(err.to_string().contains("100"));
    }

    #[test]
    fn nesting_depth_within_budget() {
        let b = ParserBudgets::default();
        assert!(b.check_nesting_depth(10).is_ok());
    }

    #[test]
    fn nesting_depth_exceeded() {
        let b = ParserBudgets {
            max_nesting_depth: 5,
            ..Default::default()
        };
        let err = b.check_nesting_depth(6).unwrap_err();
        assert_eq!(err.code(), "META-PAR-NESTING");
    }

    #[test]
    fn total_nodes_exceeded() {
        let b = ParserBudgets {
            max_total_nodes: 100,
            ..Default::default()
        };
        let err = b.check_total_nodes(101).unwrap_err();
        assert_eq!(err.code(), "META-PAR-NODES");
    }

    #[test]
    fn anchors_banned_by_default() {
        let b = ParserBudgets::default();
        let err = b.check_anchor_count(1).unwrap_err();
        assert_eq!(err.code(), "META-PAR-ANCHOR");
    }

    #[test]
    fn aliases_banned_by_default() {
        let b = ParserBudgets::default();
        let err = b.check_alias_count(1).unwrap_err();
        assert_eq!(err.code(), "META-PAR-ALIAS");
    }

    #[test]
    fn keys_per_mapping_exceeded() {
        let b = ParserBudgets {
            max_keys_per_mapping: 5,
            ..Default::default()
        };
        let err = b.check_keys_per_mapping(6).unwrap_err();
        assert_eq!(err.code(), "META-PAR-KEYS");
    }

    #[test]
    fn zero_anchor_budget_rejects_one() {
        let b = ParserBudgets::default();
        assert_eq!(b.max_anchor_count, 0);
        assert!(b.check_anchor_count(0).is_ok());
        assert!(b.check_anchor_count(1).is_err());
    }

    #[test]
    fn budgets_json_roundtrip() {
        let b = ParserBudgets::default();
        let json = serde_json::to_string(&b).unwrap();
        let parsed: ParserBudgets = serde_json::from_str(&json).unwrap();
        assert_eq!(b, parsed);
    }

    #[test]
    fn custom_budgets_roundtrip() {
        let b = ParserBudgets {
            max_file_bytes: 10 * 1024 * 1024,
            max_nesting_depth: 64,
            max_total_nodes: 5_000_000,
            max_anchor_count: 10,
            max_alias_count: 10,
            max_keys_per_mapping: 50_000,
        };
        let json = serde_json::to_string(&b).unwrap();
        let parsed: ParserBudgets = serde_json::from_str(&json).unwrap();
        assert_eq!(b, parsed);
    }

    #[test]
    fn error_codes_are_stable() {
        let violations = [
            BudgetViolation::FileSize {
                actual: 0,
                limit: 0,
            },
            BudgetViolation::NestingDepth {
                actual: 0,
                limit: 0,
            },
            BudgetViolation::TotalNodes {
                actual: 0,
                limit: 0,
            },
            BudgetViolation::AnchorCount {
                actual: 0,
                limit: 0,
            },
            BudgetViolation::AliasCount {
                actual: 0,
                limit: 0,
            },
            BudgetViolation::KeysPerMapping {
                actual: 0,
                limit: 0,
            },
        ];

        let codes = [
            "META-PAR-SIZE",
            "META-PAR-NESTING",
            "META-PAR-NODES",
            "META-PAR-ANCHOR",
            "META-PAR-ALIAS",
            "META-PAR-KEYS",
        ];

        for (v, expected) in violations.iter().zip(codes.iter()) {
            assert_eq!(v.code(), *expected);
        }
    }

    #[test]
    fn boundary_values() {
        let b = ParserBudgets {
            max_file_bytes: 100,
            max_nesting_depth: 10,
            ..Default::default()
        };
        assert!(b.check_file_size(100).is_ok());
        assert!(b.check_file_size(101).is_err());
        assert!(b.check_nesting_depth(10).is_ok());
        assert!(b.check_nesting_depth(11).is_err());
    }
}
