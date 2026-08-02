# S0 Erratum — Phase 5 Migration Verification Direction (revised)

**Status:** Proposed erratum to frozen S0 spec (`2026-08-01-linguaray-product-baseline.md` §8.5).
**Date:** 2026-08-02. *(rev-1 incorrectly dated 2026-07-31, which predates the frozen S0.)*
**Reason:** The frozen §8.5 Phase 5 says "Assert every DB profile's `secret_ref` exists in `provider_keys`." This is directionally wrong for keyless providers and blocks migration. This erratum clarifies verification scope, `needs_key` storage, and orphan-key detection before S2a references it.

---

## 1. Phase 5 verification direction: keystore → DB, key-bearing only

1. **Enumerate keys** from the v2 keystore's `provider_keys`.
2. For each key, assert a DB profile exists with `secret_ref` matching that key AND `status != 'deleted'`.
3. **Do NOT** assert the reverse (every DB profile has a key). Keyless providers are valid.

### Orphan-key rule (deleted tombstones)

A key in the keystore whose only matching DB profile is `status = 'deleted'` (tombstone) is an **orphan** — verification fails. A deleted provider must have had its key removed in delete step 2; if the key persists, the migration is incomplete.

- `status = 'deleting'`: NOT an orphan. The key is expected to still exist (delete step 2 hasn't run yet). `resume_deletions` will remove it.
- `status = 'deleted'`: the key should already be gone. If it persists → orphan → verification fails → `MigrationIncomplete`.

---

## 2. `needs_key` — stored DB column with CHECK

`needs_key` is a **stored column** on `providers` (NOT a derived runtime property), constrained by CHECK:

```sql
needs_key INTEGER NOT NULL CHECK (needs_key IN (0, 1))
```

This eliminates ambiguity. The value is set at profile creation and never changes:

| Provider type | `needs_key` |
|---------------|-------------|
| Local (Ollama) | 0 |
| Traditional free engine (Google) | 0 |
| AI preset (OpenAI/Anthropic/Gemini) | 1 |
| Unknown/custom repair profile | 1 (fail-closed: requires a key before callable) |

### Phase 5 does NOT check keyless providers

Profiles with `needs_key = 0` are never required to have a key. Phase 5 only verifies keys that actually exist in the keystore have a matching non-deleted profile.

### Post-archive recovery (keystore emptied)

After `archive_keystore` / `reset_keystore` (keystore becomes empty/missing):
1. Phase 5 re-runs: zero keys in keystore → verification passes trivially.
2. DB transaction: `UPDATE providers SET enabled = 0 WHERE needs_key = 1` (AI providers lost their keys → disabled). Keyless providers (`needs_key = 0`) stay enabled.
3. Clear active selection + consent (slots may reference now-disabled providers).
4. `migration_complete = 1`, `DataReadiness = Ready`.
5. User re-enters keys via `provider_set_key`, which re-enables the profile.

The recovery SQL is now executable directly (`WHERE needs_key = 1`), not a pseudo-derivation.

---

## 3. DB-loss recovery (keystore v2, DB rebuilt)

When the DB is lost/rebuilt but the keystore is v2:

1. Enumerate `provider_keys` from the v2 keystore.
2. For `"provider/<uuid>"` keys (new-style):
   - **Parseable UUID:** create repair profile with that UUID, `secret_ref = "provider/<uuid>"`, `template_id = "unknown"`, `protocol = "custom_http"`, `endpoint = ""`, `needs_key = 1`, `enabled = 0`.
   - **Unparseable:** generate a **deterministic UUIDv5** from the full `secret_ref` string (`UUIDv5(NAMESPACE, "linguaray:recovered-key:" + secret_ref)`). NOT UUIDv4 — crash replay must be idempotent.
3. For legacy keys (no `"provider/"` prefix): same preset/catalog lookup as normal migration; unknown → repair profile (`needs_key = 1`, `enabled = 0`).
4. Phase 5 verifies: each enumerated key has a matching non-deleted profile → passes.

---

## 4. Traditional engine provider catalog

The frozen spec maps `fallback_engine` to a provider UUID, but `TraditionalEngine` has no `endpoint()` method and `base_url` is private. S2a introduces a **static catalog**:

```rust
pub struct TraditionalProviderCatalog {
    pub template_id: &'static str,
    pub label: &'static str,
    pub endpoint: &'static str,
    pub needs_key: bool,  // always false for traditional free engines
}

pub fn traditional_catalog() -> &'static [TraditionalProviderCatalog] {
    &[
        TraditionalProviderCatalog {
            template_id: "google", label: "Google Translate",
            endpoint: "https://translate.google.com", needs_key: false,
        },
        // DeepL, Baidu, etc. added here in future slices
    ]
}
```

This is static data, separate from the runtime `TraditionalEngine` trait. It provides `endpoint` + `template_id` for creating a `ProviderProfile` row. The trait is not modified.
