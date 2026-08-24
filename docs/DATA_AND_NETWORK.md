# Data transfer and network policy

## Backups

LinguaRay exports one versioned ZIP archive containing the current
`settings.json`, translation history and favourites, vocabulary, and glossary
books. Restore validates archive paths and size, stages every supported file,
and keeps a rollback copy until the new data has been installed.

Provider keys are never embedded in an archive. Settings contain only
`linguaray-secret://` references; the corresponding values remain in macOS
Keychain or Windows Credential Manager. A backup restored on another computer
therefore requires provider credentials to be entered again.

The format manifest is `manifest.json`:

```json
{
  "format": "linguaray-backup",
  "version": 1,
  "createdAt": 0,
  "includesSecrets": false
}
```

This is a LinguaRay-to-LinguaRay backup. It does not import data from the
retired Tauri prototype.

## Network modes

All Rust translation, OCR, dictionary, and model-discovery adapters are built
through one proxy-aware HTTP client factory. Changing the setting rebuilds the
provider engine; update checks use the same policy on the Dart side.

- **System** reads the active macOS System Configuration or Windows WinINet
  HTTP/HTTPS proxy and bypass list. Environment proxy variables are the safe
  fallback when the native setting cannot be read.
- **Direct** explicitly disables proxy discovery.
- **Custom** accepts an `http://` or `https://` proxy URL and a comma-separated
  bypass list. Proxy credentials are rejected because ordinary settings may
  not contain secrets.

The loopback action API always binds to `127.0.0.1` and is never sent through
a proxy.

## Updates

The updater accepts only the canonical `LinguaRay-macos.dmg` or
`LinguaRay-windows-x64.exe` release asset. It requires an exact matching entry
in `SHA256SUMS.txt`, verifies SHA-256 after download, and then verifies the
Developer ID or Authenticode signature before enabling installation. The
installer signer must match the running LinguaRay release (Apple Team ID on
macOS, certificate subject on Windows), so an unrelated valid certificate is
not accepted.
