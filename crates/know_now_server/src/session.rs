use std::collections::HashSet;
use std::sync::Mutex;

use uuid::Uuid;

#[derive(Debug)]
pub struct SessionStore {
    sessions: Mutex<HashSet<String>>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashSet::new()),
        }
    }

    pub fn create_session(&self) -> String {
        let id = Uuid::new_v4().to_string();
        self.sessions
            .lock()
            .expect("session lock poisoned")
            .insert(id.clone());
        id
    }

    pub fn validate(&self, session_id: &str) -> bool {
        self.sessions
            .lock()
            .expect("session lock poisoned")
            .contains(session_id)
    }

    pub fn invalidate(&self, session_id: &str) {
        self.sessions
            .lock()
            .expect("session lock poisoned")
            .remove(session_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_validate() {
        let store = SessionStore::new();
        let id = store.create_session();
        assert!(store.validate(&id));
    }

    #[test]
    fn invalidate_removes_session() {
        let store = SessionStore::new();
        let id = store.create_session();
        store.invalidate(&id);
        assert!(!store.validate(&id));
    }

    #[test]
    fn unknown_session_rejected() {
        let store = SessionStore::new();
        assert!(!store.validate("nonexistent"));
    }
}
