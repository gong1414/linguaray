#!/usr/bin/env bash
set -euo pipefail

bundle_kind="${1:-app}"
if [[ "$bundle_kind" != "app" && "$bundle_kind" != "dmg" ]]; then
  echo "usage: $0 [app|dmg]" >&2
  exit 2
fi

# macOS TCC grants are bound to a stable code-signing identity. A linker-only
# ad-hoc identity changes after every rebuild, so System Settings can show an
# old grant while the current process is correctly reported as untrusted.
if [[ "$(uname -s)" == "Darwin" && -z "${APPLE_SIGNING_IDENTITY:-}" ]]; then
  local_identity="$({ security find-identity -v -p codesigning 2>/dev/null || true; } \
    | sed -nE 's/^[[:space:]]*[0-9]+\) [0-9A-F]+ "(Apple Development:[^"]+)"$/\1/p' \
    | head -n 1)"
  if [[ -z "$local_identity" ]]; then
    echo "error: no Apple Development signing identity found." >&2
    echo "Install one or set APPLE_SIGNING_IDENTITY explicitly." >&2
    exit 1
  fi
  export APPLE_SIGNING_IDENTITY="$local_identity"
fi

exec pnpm tauri build \
  --config src-tauri/tauri.noupdater.conf.json \
  --bundles "$bundle_kind"
