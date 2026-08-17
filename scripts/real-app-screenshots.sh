#!/usr/bin/env bash
# 真实应用截图基线（hygiene-6；docs/UI-RULES.md 规则 10）。
#
# 用法:
#   bash scripts/real-app-screenshots.sh                          # main（设置窗）
#   bash scripts/real-app-screenshots.sh --window input|popup|ocr # 其它三个窗口
#   bash scripts/real-app-screenshots.sh --build                  # 先 pnpm build:local
#
# 产出: docs/baselines/real-app/<窗口名>.png（提交入库作为对照基线）。
# 说明: 通过 LINGUARAY_AUTOSHOW_{MAIN,INPUT,POPUP,OCR} 环境变量直启打包后的
# 二进制（绕过托盘隐藏），再按窗口逻辑尺寸用 scripts/listwin.swift 定位
# CGWindowID 后 screencapture 抓取（尺寸来自 tauri.conf.json：input 720×440、
# popup 460×300；ocr 为全屏覆盖层取最大面积窗口）。修改 UI 的提交必须更新
# 基线并在 PR 中附新旧对照。终端需要屏幕录制权限，否则截图只有壁纸。
set -euo pipefail
cd "$(dirname "$0")/.."

APP_BUNDLE="src-tauri/target/release/bundle/macos/LinguaRay.app"
OUT_DIR="docs/baselines/real-app"

WINDOW="main"
DO_BUILD=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --build) DO_BUILD=1 ;;
    --window) WINDOW="${2:?--window 需要参数: main|input|popup|ocr}"; shift ;;
    -h|--help) grep '^#' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "ERROR: 未知参数 $1（支持 --window main|input|popup|ocr、--build）" >&2; exit 2 ;;
  esac
  shift
done

case "$WINDOW" in
  main)   AUTOSHOW_VAR="LINGUARAY_AUTOSHOW_MAIN";   OUT_FILE="main-settings.png" ;;
  input)  AUTOSHOW_VAR="LINGUARAY_AUTOSHOW_INPUT";  OUT_FILE="input-window.png" ;;
  popup)  AUTOSHOW_VAR="LINGUARAY_AUTOSHOW_POPUP";  OUT_FILE="popup-window.png" ;;
  ocr)    AUTOSHOW_VAR="LINGUARAY_AUTOSHOW_OCR";    OUT_FILE="ocr-window.png" ;;
  *) echo "ERROR: 未知窗口 '$WINDOW'（支持 main|input|popup|ocr）" >&2; exit 2 ;;
esac

if [[ "$DO_BUILD" == 1 || ! -d "$APP_BUNDLE" ]]; then
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

echo "[3/4] 以 $AUTOSHOW_VAR 直启打包二进制（$WINDOW 窗口）…"
mkdir -p "$OUT_DIR"
env "$AUTOSHOW_VAR=1" "$APP_BUNDLE/Contents/MacOS/LinguaRay" >/dev/null 2>&1 &
APP_PID=$!
# 等窗口显示 + WebView 渲染完成（首帧加载较慢，宁长勿缺）。
sleep 6

cleanup() {
  kill "$APP_PID" >/dev/null 2>&1 || true
  wait "$APP_PID" 2>/dev/null || true
}
trap cleanup EXIT

# 按 CGWindowID 选择目标窗口：main 用 windowid.swift（首个 layer-0）；其余按
# 逻辑尺寸匹配（±2pt 容差），ocr（全屏覆盖层）取面积最大的 LinguaRay 窗口。
# awk 解析 listwin.swift 的 `id= w= h=` 字段（BSD awk 兼容写法）。
select_window_id() {
  if [[ "$WINDOW" == "main" ]]; then
    swift scripts/windowid.swift
    return
  fi
  local want_w want_h mode
  if [[ "$WINDOW" == "ocr" ]]; then
    want_w=0; want_h=0; mode="max"
  elif [[ "$WINDOW" == "input" ]]; then
    want_w=720; want_h=440; mode="exact"
  else
    want_w=460; want_h=300; mode="exact"   # popup
  fi
  swift scripts/listwin.swift | awk -v w="$want_w" -v h="$want_h" -v mode="$mode" '
    {
      id = ww = hh = ""
      for (i = 1; i <= NF; i++) {
        if ($i ~ /^id=/) { id = substr($i, 4) }
        else if ($i ~ /^w=/) { ww = substr($i, 3) }
        else if ($i ~ /^h=/) { hh = substr($i, 3) }
      }
      if (id == "" || ww == "" || hh == "") next
      if (mode == "max") {
        area = ww * hh
        if (area > best) { best = area; best_id = id }
      } else if (ww >= w-2 && ww <= w+2 && hh >= h-2 && hh <= h+2) {
        print id; exit
      }
    }
    END { if (mode == "max" && best_id) print best_id }'
}

WIN_ID="$(select_window_id || true)"
if [[ -z "$WIN_ID" ]]; then
  echo "ERROR: 未找到 LinguaRay '$WINDOW' 窗口（autoshow 未生效或尺寸不匹配）" >&2
  swift scripts/listwin.swift >&2 || true
  exit 1
fi

echo "[4/4] 抓取窗口 $WIN_ID → $OUT_DIR/$OUT_FILE"
screencapture -o -l "$WIN_ID" "$OUT_DIR/$OUT_FILE"
echo "完成: $OUT_DIR/$OUT_FILE"
