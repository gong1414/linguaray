#!/bin/bash
set -euo pipefail
APP="apps/desktop/flutter/build/macos/Build/Products/Release/LinguaRay.app"
DIST="apps/desktop/flutter/dist"
DMG="$DIST/LinguaRay-macos.dmg"
TMP_DIR="${RUNNER_TEMP:?}/linguaray-package-macos"
mkdir -p "$DIST" "$TMP_DIR"
# The single macOS download must actually run on both supported architectures.
lipo -verify_arch arm64 x86_64 "$APP/Contents/MacOS/LinguaRay"
SIGNED=false
CERT_PATH="$TMP_DIR/signing.p12"
KEYCHAIN="$TMP_DIR/signing.keychain-db"
cleanup() {
  security delete-keychain "$KEYCHAIN" >/dev/null 2>&1 || true
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

if [[ -n "${MACOS_SIGNING_CERTIFICATE_BASE64:-}${MACOS_SIGNING_CERTIFICATE_PASSWORD:-}${APPLE_TEAM_ID:-}${APPLE_ID:-}${APPLE_APP_SPECIFIC_PASSWORD:-}" ]]; then
  : "${MACOS_SIGNING_CERTIFICATE_BASE64:?incomplete macOS signing configuration}"
  : "${MACOS_SIGNING_CERTIFICATE_PASSWORD:?incomplete macOS signing configuration}"
  : "${APPLE_TEAM_ID:?incomplete macOS signing configuration}"
  : "${APPLE_ID:?incomplete macOS signing configuration}"
  : "${APPLE_APP_SPECIFIC_PASSWORD:?incomplete macOS signing configuration}"
  KEYCHAIN_PASSWORD="$(openssl rand -hex 24)"
  printf '%s' "$MACOS_SIGNING_CERTIFICATE_BASE64" | base64 --decode > "$CERT_PATH"
  security create-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN"
  security set-keychain-settings -lut 21600 "$KEYCHAIN"
  security unlock-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN"
  security import "$CERT_PATH" -k "$KEYCHAIN" -P "$MACOS_SIGNING_CERTIFICATE_PASSWORD" -T /usr/bin/codesign
  security set-key-partition-list -S apple-tool:,apple: -s -k "$KEYCHAIN_PASSWORD" "$KEYCHAIN" >/dev/null
  IDENTITY="$(security find-identity -v -p codesigning "$KEYCHAIN" | awk -F'"' '/Developer ID Application/{print $2; exit}')"
  test -n "$IDENTITY"
  codesign --force --deep --options runtime --timestamp \
    --entitlements apps/desktop/flutter/macos/Runner/Release.entitlements \
    --keychain "$KEYCHAIN" --sign "$IDENTITY" "$APP"
  SIGNED=true
fi
codesign --verify --deep --strict "$APP"
mkdir -p "$TMP_DIR/root"
ditto "$APP" "$TMP_DIR/root/LinguaRay.app"
ln -s /Applications "$TMP_DIR/root/Applications"
hdiutil create -volname LinguaRay -srcfolder "$TMP_DIR/root" -ov -format UDZO "$DMG"
if [[ "$SIGNED" == true ]]; then
  codesign --force --timestamp --keychain "$KEYCHAIN" --sign "$IDENTITY" "$DMG"
  codesign --verify --strict "$DMG"
  xcrun notarytool submit "$DMG" --apple-id "$APPLE_ID" \
    --password "$APPLE_APP_SPECIFIC_PASSWORD" --team-id "$APPLE_TEAM_ID" --wait
  xcrun stapler staple "$DMG"
  xcrun stapler validate "$DMG"
fi
printf '{"platformSigned":%s}\n' "$SIGNED" > "$DIST/macos-signing.json"
