#!/usr/bin/env bash
# 真实应用截图基线（hygiene-6；docs/UI-RULES.md 规则 10）。
#
# 用法:
#   bash scripts/real-app-screenshots.sh            # 复用已构建的 .app
#   bash scripts/real-app-screenshots.sh --build    # 先 pnpm build:local
#
# 产出: docs/baselines/real-app/main-settings.png（提交入库作为对照基线）。
# 说明: 通过 LINGUARAY_AUTOSHOW_MAIN 环境变量直启打包后的二进制（绕过托盘
# 隐藏），用 CGWindowID 定位主窗口后 screencapture 抓取。修改 UI 的提交必须
# 更新基线并在 PR 中附新旧对照。终端需要屏幕录制权限，否则截图只有壁纸。
set -euo pipefail
cd "$(dirname "$0")/.."

APP_BUNDLE="src-tauri/target/release/bundle/macos/LinguaRay.app"
OUT_DIR="docs/baselines/real-app"

if [[ "${1:-}" == "--build" || ! -d "$APP_BUNDLE" ]]; then
  echo "[1/4] 本地 release 构建（无 updater 产物）…"
  pnpm build:local
fi

if [[ ! -d "$APP_BUNDLE" ]]; then
  echo "ERROR: 未找到 $APP_BUNDLE（先运行 --build）" >&2
  exit 1
fi

echo "[2/4] 退出已运行实例…"
osascript -e 'tell application "LinguaRay" to quit' >/dev/null 2>&1 || true
sleep 1

echo "[3/4] 以 LINGUARAY_AUTOSHOW_MAIN 直启打包二进制…"
mkdir -p "$OUT_DIR"
LINGUARAY_AUTOSHOW_MAIN=1 "$APP_BUNDLE/Contents/MacOS/LinguaRay" >/dev/null 2>&1 &
APP_PID=$!
# 等主窗口显示 + WebView 渲染完成（首帧加载较慢，宁长勿缺）。
sleep 6

cleanup() {
  kill "$APP_PID" >/dev/null 2>&1 || true
  wait "$APP_PID" 2>/dev/null || true
}
trap cleanup EXIT

WIN_ID="$(swift scripts/windowid.swift || true)"
if [[ -z "$WIN_ID" ]]; then
  echo "ERROR: 未找到 LinguaRay 可见窗口（主窗口未显示？）" >&2
  exit 1
fi

echo "[4/4] 抓取窗口 $WIN_ID → $OUT_DIR/main-settings.png"
screencapture -o -l "$WIN_ID" "$OUT_DIR/main-settings.png"
echo "完成: $OUT_DIR/main-settings.png"
