//! Pure request policy: Bearer compare, Origin reject, rate limit.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

pub const DEFAULT_RATE: usize = 60;
pub const RATE_WINDOW: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthError {
    Missing,
    Mismatch,
}

/// Constant-time Bearer compare. `token` is the expected raw token (no "Bearer ").
pub fn authorize(authorization: Option<&str>, expected: &str) -> Result<(), AuthError> {
    let raw = authorization.ok_or(AuthError::Missing)?;
    let got = raw
        .strip_prefix("Bearer ")
        .or_else(|| raw.strip_prefix("bearer "))
        .ok_or(AuthError::Missing)?;
    if !constant_eq(got.as_bytes(), expected.as_bytes()) {
        return Err(AuthError::Mismatch);
    }
    Ok(())
}

/// Any Origin header is rejected (no CORS).
pub fn reject_origin(origin: Option<&str>) -> bool {
    origin.is_some()
}

fn constant_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut acc = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        acc |= x ^ y;
    }
    acc == 0
}

/// Sliding-window limiter (max `limit` events per `RATE_WINDOW`).
pub struct RateLimiter {
    hits: VecDeque<Instant>,
    limit: usize,
}

impl RateLimiter {
    pub fn new(limit: usize) -> Self {
        Self {
            hits: VecDeque::new(),
            limit,
        }
    }

    pub fn allow(&mut self, now: Instant) -> bool {
        let cutoff = now.checked_sub(RATE_WINDOW).unwrap_or(now);
        while self.hits.front().is_some_and(|t| *t < cutoff) {
            self.hits.pop_front();
        }
        if self.hits.len() >= self.limit {
            return false;
        }
        self.hits.push_back(now);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bearer_accepts_exact_token() {
        assert!(authorize(Some("Bearer secret"), "secret").is_ok());
    }

    #[test]
    fn bearer_rejects_missing_and_mismatch() {
        assert_eq!(authorize(None, "secret"), Err(AuthError::Missing));
        assert_eq!(
            authorize(Some("secret"), "secret"),
            Err(AuthError::Missing)
        );
        assert_eq!(
            authorize(Some("Bearer nope"), "secret"),
            Err(AuthError::Mismatch)
        );
    }

    #[test]
    fn origin_any_value_is_rejected() {
        assert!(reject_origin(Some("http://evil.test")));
        assert!(!reject_origin(None));
    }

    #[test]
    fn rate_limiter_trips_after_limit() {
        let mut lim = RateLimiter::new(2);
        let t0 = Instant::now();
        assert!(lim.allow(t0));
        assert!(lim.allow(t0));
        assert!(!lim.allow(t0));
    }
}
