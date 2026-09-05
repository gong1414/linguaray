# Releasing LinguaRay

LinguaRay builds a universal macOS DMG and a Windows x64 per-user installer.
The release workflow verifies both platforms and signs a single update feed
with Ed25519 before publishing. The app checks this feed at startup, every
six hours while resident, and after resume when a check is due. Users can turn
automatic checks off or check manually in Settings → Updates.

## Repository configuration

The required `LINGUARAY_UPDATE_SIGNING_KEY` Actions secret contains a base64
encoded 32-byte Ed25519 private seed. Its matching public key is committed in
`assets/update/public-key.json` and
`apps/desktop/flutter/lib/src/config/update_signing_key.dart`.

The configured key has a recovery copy in the maintainer's macOS Keychain:
service `io.github.gong1414.linguaray.release-signing`, account `ed25519-v1`.
Never commit or print the private seed. Do not regenerate it during routine
releases: existing installations trust the embedded public key. Key rotation
requires a separately planned migration for those installations.

GitHub Actions must be enabled. The workflow needs `contents: write` only in
its publishing job; the repository can retain read-only default permissions.
No personal access token or GitHub token is embedded in the application.

### Optional platform signing

Ed25519 authenticates release provenance and the exact installer digest. It
does not replace Gatekeeper notarization or Windows Authenticode reputation.
To enable operating-system signing, configure the complete group of secrets:

| Platform | Secrets |
| --- | --- |
| macOS | `MACOS_SIGNING_CERTIFICATE_BASE64` (Developer ID Application PKCS#12), `MACOS_SIGNING_CERTIFICATE_PASSWORD`, `APPLE_TEAM_ID`, `APPLE_ID`, `APPLE_APP_SPECIFIC_PASSWORD` |
| Windows | `WINDOWS_CERTIFICATE_BASE64` (PFX), `WINDOWS_CERTIFICATE_PASSWORD` |

Without a platform's certificate group, its installer remains OS-unsigned and
requires the user's system installation confirmation. Partial configuration,
failed signing or failed notarization stops the release; these failures never
silently fall back to unsigned output. Certificates are removed from runner
storage after packaging. A platform-signed installation refuses an update
that loses platform signing or changes publisher identity.

## Validate and publish

1. Update `apps/desktop/flutter/pubspec.yaml`. The version before `+` must
   equal the release tag, for example `0.6.1+20` and `v0.6.1`.
2. For a rehearsal, run **Actions → Release → Run workflow** on the desired
   branch, supply the tag, and leave **publish** off. This runs the checks,
   builds both installers, signs the feed, and uploads `verified-release`
   as a workflow artifact without creating a public release.
3. Merge the tested source into `main`, then push its stable version tag.
   Alternatively, run Release on `main` with the matching tag and **publish**
   enabled. Publishing requires the checked-out commit to be on `main`.
4. The workflow assembles a draft, uploads all assets, then makes the complete
   release public and marks it latest. An existing public release cannot be
   overwritten; fix problems in a new patch version. A failed draft may be
   retried from the same source commit.

Release artifacts have these fixed names:

- `LinguaRay-macos.dmg` — contains both arm64 and x86_64 application code
- `LinguaRay-windows-x64.exe` — Inno Setup installer, not a raw build directory
- `update.json` — signed payload containing version, names, sizes and SHA256
- `SHA256SUMS.txt` — downloadable checksums for all assets, including the feed
- Optional integration template archives

The public feed is
[the latest release's update.json](https://github.com/gong1414/linguaray/releases/latest/download/update.json).
This direct release URL avoids anonymous GitHub REST API rate limits. Release
notes use GitHub's authenticated API only inside Actions.

## Client verification and installation

The client verifies the signature with its embedded public key before trusting
the version, platform, filename, byte count or digest. Download URLs are derived
from the fixed repository, signed stable tag and expected artifact name. The
client bounds feed/download sizes, rejects truncated files, verifies SHA256,
and checks platform identity when required. It repeats verification just
before handing the installer to the system.

Updates require a click to download and another to open the installer. On
macOS, the DMG opens for the user to quit LinguaRay and replace it in
Applications. On Windows, Inno Setup handles closing and replacing the app,
with a post-install launch option. This release does **not** silently replace
a running app or install at exit.

The old v0.6.0 ZIP builds do not understand this signed feed. Install v0.6.1
once from its release assets to bootstrap automatic checks for subsequent
versions. Before the first new release is published, the feed URL is not yet
available and an update check reports a connection/check failure.

## Local verification

```bash
python3 -m pip install -r scripts/release-requirements.txt
python3 -m unittest discover -s scripts -p 'test_release_manifest.py'
python3 scripts/release_manifest.py --tag v0.6.1
dart run melos run analyze
dart run melos run test
```

The signing tests generate ephemeral keys and synthetic files. Client tests
exercise signature tampering, platform selection, download integrity, installer
handoff and periodic checks without executing an installer. Production private
keys are not required to run the test suite.

### Native desktop golden baselines

Golden screenshots use each operating system's own fonts. Refresh Windows
baselines on Windows, not by overriding the platform theme on macOS. Run
**Actions → CI → Run workflow → refresh_windows_goldens** to generate a
`windows-golden-review` artifact. Review the images and commit the intended
changes; normal CI never regenerates or silently accepts them. All desktop
snapshots live in `apps/desktop/flutter/test/goldens/catalog/`.

## Supported targets and protocol registration

Supported releases target macOS 13 or newer and Windows 10 or newer. There is
no Linux installer or support promise. Inno Setup registers `linguaray://`
under the current user's `Software\Classes` hive and removes it on uninstall.
