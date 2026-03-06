//! Client-side session state tracking for the Realtime API.
//!
//! Mirrors the server's session state by processing `session.created` and
//! `session.updated` events.

use super::types::Session;

/// Tracks the current session state based on server events.
#[derive(Debug, Clone, Default)]
pub struct SessionState {
    session: Option<Session>,
}

impl SessionState {
    /// Create a new, uninitialized session state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Update from a `session.created` or `session.updated` event.
    pub fn update(&mut self, session: Session) {
        self.session = Some(session);
    }

    /// Get the current session, if initialized.
    pub fn get(&self) -> Option<&Session> {
        self.session.as_ref()
    }

    /// Get the session ID.
    pub fn id(&self) -> Option<&str> {
        self.session.as_ref().and_then(|s| s.id.as_deref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_state_is_empty() {
        let state = SessionState::new();
        assert!(state.get().is_none());
        assert!(state.id().is_none());
    }

    #[test]
    fn update_sets_session() {
        let mut state = SessionState::new();
        let session = Session {
            id: Some("sess_123".into()),
            instructions: Some("Be helpful.".into()),
            ..Default::default()
        };
        state.update(session);

        assert!(state.get().is_some());
        assert_eq!(state.id(), Some("sess_123"));
        assert_eq!(
            state.get().unwrap().instructions.as_deref(),
            Some("Be helpful.")
        );
    }

    #[test]
    fn update_replaces_session() {
        let mut state = SessionState::new();
        state.update(Session {
            id: Some("sess_1".into()),
            ..Default::default()
        });
        state.update(Session {
            id: Some("sess_2".into()),
            ..Default::default()
        });

        assert_eq!(state.id(), Some("sess_2"));
    }

    #[test]
    fn id_returns_none_when_session_has_no_id() {
        let mut state = SessionState::new();
        state.update(Session {
            id: None,
            ..Default::default()
        });

        assert!(state.get().is_some());
        assert!(state.id().is_none());
    }

    #[test]
    fn default_is_same_as_new() {
        let state1 = SessionState::new();
        let state2 = SessionState::default();
        assert!(state1.get().is_none());
        assert!(state2.get().is_none());
    }
}
