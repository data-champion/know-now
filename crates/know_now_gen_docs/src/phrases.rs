/// Stakeholder-friendly translations of technical constraints (PRD §11.5).
pub fn friendly_constraint(raw: &str) -> String {
    let lower = raw.to_lowercase();

    if let Some(len) = parse_varchar_length(&lower) {
        return format!("Up to {len} characters");
    }
    if lower.contains("not null") {
        return "Required value".to_owned();
    }
    if lower.contains("unique") {
        return "Must be unique".to_owned();
    }
    if lower.contains("check") && lower.contains("> 0") {
        return "Must be positive".to_owned();
    }
    if lower.contains("check") && lower.contains(">= 0") {
        return "Must not be negative".to_owned();
    }
    if lower.contains("default") {
        return format!("Defaults to: {raw}");
    }

    raw.to_owned()
}

pub fn friendly_classification(raw: &str) -> String {
    match raw.to_lowercase().as_str() {
        "public" => "Public".to_owned(),
        "internal" => "Internal use only".to_owned(),
        "confidential" => "Confidential".to_owned(),
        "restricted" => "Restricted access".to_owned(),
        _ => raw.to_owned(),
    }
}

fn parse_varchar_length(lower: &str) -> Option<u32> {
    let s = lower.strip_prefix("varchar(")?;
    let s = s.strip_suffix(')')?;
    s.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn varchar_translated() {
        assert_eq!(friendly_constraint("VARCHAR(320)"), "Up to 320 characters");
        assert_eq!(friendly_constraint("varchar(255)"), "Up to 255 characters");
    }

    #[test]
    fn not_null_translated() {
        assert_eq!(friendly_constraint("NOT NULL"), "Required value");
    }

    #[test]
    fn unique_translated() {
        assert_eq!(friendly_constraint("UNIQUE"), "Must be unique");
    }

    #[test]
    fn positive_check_translated() {
        assert_eq!(friendly_constraint("CHECK(amount > 0)"), "Must be positive");
    }

    #[test]
    fn non_negative_check_translated() {
        assert_eq!(
            friendly_constraint("CHECK(quantity >= 0)"),
            "Must not be negative"
        );
    }

    #[test]
    fn default_translated() {
        assert_eq!(friendly_constraint("DEFAULT 0"), "Defaults to: DEFAULT 0");
    }

    #[test]
    fn unknown_constraint_passes_through() {
        assert_eq!(friendly_constraint("FOREIGN KEY (x)"), "FOREIGN KEY (x)");
    }

    #[test]
    fn classification_internal() {
        assert_eq!(friendly_classification("internal"), "Internal use only");
    }

    #[test]
    fn classification_confidential() {
        assert_eq!(friendly_classification("confidential"), "Confidential");
    }

    #[test]
    fn classification_unknown_passes_through() {
        assert_eq!(friendly_classification("custom_level"), "custom_level");
    }
}
