use std::collections::{HashMap, HashSet};

use oauth2::PkceCodeVerifier;
use time::{Duration, OffsetDateTime};

/// The PKCE Verifier store
/// Internally, this is a HashMap of (csrf_token, (verifier, expiry))
pub struct PkceStore {
    pub verifiers: HashMap<String, (PkceCodeVerifier, OffsetDateTime)>,
    /// CSRF tokens that originated from the mobile app — need a deep-link redirect on callback.
    pub mobile_sessions: HashSet<String>,
}

impl PkceStore {
    /// Create a new PkceStore
    pub fn new() -> Self {
        Self {
            verifiers: HashMap::new(),
            mobile_sessions: HashSet::new(),
        }
    }

    /// Mark a CSRF token as originating from the mobile app.
    pub fn mark_mobile(&mut self, csrf_token: &str) {
        self.mobile_sessions.insert(csrf_token.to_owned());
    }

    /// Check (and consume) whether a CSRF token was marked as mobile.
    pub fn take_mobile(&mut self, csrf_token: &str) -> bool {
        self.mobile_sessions.remove(csrf_token)
    }

    /// Insert a new PKCE verifier into the store
    pub fn insert(&mut self, csrf_token: String, verifier: PkceCodeVerifier, expiry: Duration) {
        let expiry_time = OffsetDateTime::now_utc() + expiry;
        self.verifiers.insert(csrf_token, (verifier, expiry_time));
    }

    /// Get a PKCE verifier from the store, given a CSRF token
    /// Will remove the verifier from the store if it has expired.
    pub fn get(&mut self, csrf_token: &str) -> Option<PkceCodeVerifier> {
        // TODO: move this to a background task
        self.cleanup();

        // likely the worst way to do this, but probably necessary
        // as the code is in a hashmap...
        let code = self.verifiers.get(csrf_token);
        let verifier_internal = if let Some((verifier, expiry)) = code {
            if *expiry > OffsetDateTime::now_utc() {
                Some(verifier.secret())
            } else {
                None
            }
        } else {
            None
        };
        verifier_internal.map(|secret| PkceCodeVerifier::new(secret.to_owned()))
    }

    pub fn cleanup(&mut self) {
        let now = OffsetDateTime::now_utc();
        self.verifiers.retain(|_, (_, expiry)| *expiry > now);
    }
}
