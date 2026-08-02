# S0 Erratum — Phase 5 Migration Verification Direction

**Status:** Proposed erratum to frozen S0 spec (`2026-08-01-linguaray-product-baseline.md` §8.5).
**Date:** 2026-07-31.
**Reason:** The frozen §8.5 Phase 5 says "Read the v2 keystore back. Assert every DB profile's `secret_ref` exists in `provider_keys`." This is directionally wrong for keyless providers and blocks migration in legitimate scenarios. This erratum clarifies the verification scope before S2a implementation references it.

---

## Problem

The frozen text asserts **every** DB profile has a key in `provider_keys`. But several provider types legitimately have no key:

- **Local providers** (Ollama): `is_local = true`, `needs_key = false`. No key in the keystore.
- **Traditional engines configured as fallback** (Google): free, keyless. No key.
- **Key-missing profiles**: a user may create a provider and not yet enter a key. The profile exists in the DB; the key is absent.

Under the frozen text, Phase 5 would fail for all of these, blocking migration permanently.

## Erratum

Phase 5 verification is **keystore → DB direction, scoped to key-bearing profiles only**:

1. **Enumerate keys** from the v2 keystore's `provider_keys`.
2. For each key found, assert a DB profile exists with `secret_ref` matching that key.
3. **Do NOT** assert the reverse (that every DB profile has a key). Keyless providers are valid.

### `needs_key` classification

A provider `needs_key` if and only if:
- `is_local = false` (not a loopback/local provider), AND
- `protocol ≠ google_translate` (traditional free engines are keyless), AND
- The provider's template/preset declares `needs_key = true`.

`needs_key` is a **derived property** from `is_local` + `protocol` + preset metadata, not a stored column. It is computed at read time by joining the profile's `template_id` against the preset catalog.

### Keyless vs profile-missing-key

| State | `needs_key` | key in keystore | Valid? |
|-------|-------------|-----------------|--------|
| Local provider (Ollama) | false | absent | ✅ Valid — no key needed |
| Traditional engine (Google free) | false | absent | ✅ Valid — no key needed |
| AI provider, key entered | true | present | ✅ Valid |
| AI provider, key not yet entered | true | absent | ✅ Valid (provisional) — profile exists, user can enter key later. UI shows "key missing". Not callable until key set. |

All four states pass Phase 5. The only state that fails is: **a key exists in the keystore but no DB profile has a matching `secret_ref`** (an orphaned key — the profile was deleted but the key wasn't removed, or the DB was lost).

### Archive keystore recovery

After the user archives a corrupt keystore (`archive_keystore` → `.broken-<ts>`):
1. The keystore is empty (`Missing`).
2. Phase 5 re-runs: enumerates zero keys from the empty keystore → verification passes trivially (no keys to check).
3. Existing DB profiles lose their keys. AI providers with `needs_key = true` are set to `enabled = false` in a DB transaction (key-missing state). The user re-enters keys via `provider_set_key`.
4. `migration_complete = 1`. `DataReadiness = Ready`.
5. **Phase 5 does not permanently fail.**

### DB-loss recovery (keystore v2, DB rebuilt)

When the DB is lost but the keystore is v2:
1. Enumerate `provider_keys` from the v2 keystore.
2. For `"provider/<uuid>"` keys: create repair profiles (`template_id = "unknown"`, `protocol = "custom_http"`, `endpoint = ""`, `enabled = false`). The uuid is extracted from the key if parseable.
3. For legacy keys (no `"provider/"` prefix): same preset-lookup logic as normal migration.
4. Phase 5 verifies these newly-created profiles against the keys → passes.

---

## Traditional engine provider catalog (new)

The frozen spec maps `fallback_engine` to a provider UUID, but the current `TraditionalEngine` trait has no `endpoint()` method and `base_url` is private. S2a introduces a **traditional provider catalog**:

```rust
pub struct TraditionalProviderCatalog {
    pub template_id: &'static str,   // "google", "deepl", "baidu", ...
    pub label: &'static str,
    pub endpoint: &'static str,       // the base URL for this engine
    pub needs_key: bool,
}
```

This catalog is static data (like `providers::presets()`), separate from the runtime `TraditionalEngine` trait. It provides the `endpoint` + `template_id` needed to create a `ProviderProfile` row for a traditional engine. Google's entry:

```rust
TraditionalProviderCatalog {
    template_id: "google",
    label: "Google Translate",
    endpoint: "https://translate.google.com",
    needs_key: false,
}
```

The `TraditionalEngine` trait may later gain an `endpoint()` method that returns this, but S2a does not require modifying the trait — the static catalog suffices for migration.
