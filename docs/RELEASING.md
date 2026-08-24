# Releasing LinguaRay

LinguaRay publishes signed desktop installers for macOS and Windows only. A
tagged release never falls back to an unsigned archive.

## Required repository secrets

| Secret | Purpose |
| --- | --- |
| `MACOS_SIGNING_CERTIFICATE_BASE64` | Developer ID Application certificate in PKCS#12 form |
| `MACOS_SIGNING_CERTIFICATE_PASSWORD` | Password for that PKCS#12 file |
| `APPLE_TEAM_ID` | Stable Apple signing team used by update identity checks |
| `APPLE_ID` | Notary service account |
| `APPLE_APP_SPECIFIC_PASSWORD` | Notary service credential |
| `WINDOWS_CERTIFICATE_BASE64` | Authenticode certificate in PFX form |
| `WINDOWS_CERTIFICATE_PASSWORD` | Password for that PFX file |

The workflow imports certificates into temporary runner storage and removes
them after packaging. Secrets must never be committed or printed.

## Release contract

1. Update `apps/desktop/flutter/pubspec.yaml`; the version before `+` must
   match the tag, for example `0.6.0` and `v0.6.0`.
2. Complete CI on macOS and Windows.
3. Push the version tag.
4. The release workflow builds and signs the app payloads, notarizes the macOS
   DMG, builds the per-user Windows Inno Setup installer, signs that installer,
   packages the optional integrations, and creates a draft GitHub Release.
5. Review the draft and publish it manually.

The updater depends on these exact asset names:

- `LinguaRay-macos.dmg`
- `LinguaRay-windows-x64.exe`
- `SHA256SUMS.txt`

`SHA256SUMS.txt` is created only after both platform artifacts and integration
archives have been merged. Update installation requires its exact checksum
entry and a signer identity matching the running LinguaRay release.

## Windows protocol registration

The Inno Setup installer registers `linguaray://` under the current user's
`Software\Classes` registry hive and removes the key during uninstall. Do not
publish the raw Flutter build directory as a supported installation: it has no
installer lifecycle or protocol registration.

## Supported targets

There is no Linux runner, build, installer, release asset, or support promise.
The supported release targets are macOS 13 or newer and Windows 10 or newer.
