//! UUID generation utilities (S0 §5.1 + §8.5 migration protocol).
//!
//! - Legacy provider migration: deterministic UUIDv5 so crash-replay is idempotent.
//! - New user-created providers: UUIDv4.
//! - DB-loss recovery ("provider/<uuid>" keys): UUIDv5 from the secret_ref.

use uuid::Uuid;

/// The fixed LinguaRay namespace for UUIDv5 generation.
/// Generated once, hardcoded. Used for deterministic provider UUIDs across
/// crash replays (same input → same UUID).
pub const NAMESPACE_LINGUARAY: Uuid = Uuid::from_bytes([
    0x4c, 0x69, 0x6e, 0x67, // "Ling"
    0x75, 0x61, 0x52, 0x61, // "uaRa"
    0x79, 0x00, 0x00, 0x00, // "y\0\0\0"
    0x00, 0x00, 0x00, 0x00, // padding
]);

/// Deterministic UUIDv5 for a legacy provider migration.
/// `UUIDv5(NAMESPACE_LINGUARAY, "linguaray:legacy-provider:" + legacy_id)`.
/// Re-running after a crash produces the same UUID → idempotent.
pub fn legacy_provider_uuid(legacy_id: &str) -> Uuid {
    let name = format!("linguaray:legacy-provider:{}", legacy_id);
    Uuid::new_v5(&NAMESPACE_LINGUARAY, name.as_bytes())
}

/// Deterministic UUIDv5 for a recovered key (DB-loss recovery).
/// `UUIDv5(NAMESPACE_LINGUARAY, "linguaray:recovered-key:" + secret_ref)`.
pub fn recovered_key_uuid(secret_ref: &str) -> Uuid {
    let name = format!("linguaray:recovered-key:{}", secret_ref);
    Uuid::new_v5(&NAMESPACE_LINGUARAY, name.as_bytes())
}

/// Random UUIDv4 for new user-created providers.
pub fn new_provider_uuid() -> Uuid {
    Uuid::new_v4()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_uuid_is_deterministic() {
        let a = legacy_provider_uuid("openai");
        let b = legacy_provider_uuid("openai");
        assert_eq!(a, b, "same legacy_id must produce same UUID");
    }

    #[test]
    fn legacy_uuid_differs_for_different_ids() {
        let a = legacy_provider_uuid("openai");
        let b = legacy_provider_uuid("anthropic");
        assert_ne!(a, b);
    }

    #[test]
    fn recovered_key_uuid_is_deterministic() {
        let a = recovered_key_uuid("provider/abc-123");
        let b = recovered_key_uuid("provider/abc-123");
        assert_eq!(a, b);
    }

    #[test]
    fn new_provider_uuid_is_unique() {
        let a = new_provider_uuid();
        let b = new_provider_uuid();
        assert_ne!(a, b);
    }
}
