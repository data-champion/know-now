use std::sync::atomic::{AtomicBool, Ordering};

use uuid::Uuid;

#[derive(Debug)]
pub struct LaunchToken {
    value: String,
    used: AtomicBool,
}

impl Default for LaunchToken {
    fn default() -> Self {
        Self::new()
    }
}

impl LaunchToken {
    pub fn new() -> Self {
        Self {
            value: Uuid::new_v4().to_string(),
            used: AtomicBool::new(false),
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn try_consume(&self, candidate: &str) -> bool {
        if candidate != self.value {
            return false;
        }
        self.used
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    pub fn is_used(&self) -> bool {
        self.used.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_use_token() {
        let token = LaunchToken::new();
        let val = token.value().to_owned();

        assert!(!token.is_used());
        assert!(token.try_consume(&val));
        assert!(token.is_used());
        assert!(!token.try_consume(&val));
    }

    #[test]
    fn wrong_value_rejected() {
        let token = LaunchToken::new();
        assert!(!token.try_consume("wrong"));
        assert!(!token.is_used());
    }
}
