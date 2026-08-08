# Surface 12: OCR Overlay

**Surface ID:** `surface.ocr-overlay`
**Penpot 页面:** 40 OCR & Media
**Penpot 画板尺寸:** [待查询]
**生产窗口默认尺寸:** 全屏/显示器
**生产窗口最小尺寸:** N/A
**尺寸冲突说明:** 以生产窗口尺寸为实施标准

## 状态矩阵

| 状态 | 描述 | 组件 |
|---|---|---|
| Initial | 变暗屏幕 + 十字准星 | 全屏遮罩 + crosshair |
| Selecting | 亮色矩形跟随光标 | 选区矩形 |
| Capturing | 闪光 + spinner | 闪光 + Spinner |
| OCR processing | 选区处小 spinner | Spinner (small) |
| Success | 选区处翻译 popup | Translation popup（→ Surface 01） |
| Error (no text) | "未识别到文本" | Inline error |
| Error (permission) | macOS："请授予屏幕录制权限"；Windows："捕获不可用"（受保护内容/不支持会话/远程桌面） | Inline error + 权限引导 |
| Cancelled | 遮罩关闭（Esc / 右键） | — |

## 本地化 copy key

| key | en | zh |
|---|---|---|
| `ocr.overlay.hint` | Drag to select a region · Esc to cancel | 拖拽选择区域 · Esc 取消 |
| `ocr.error.noText` | No text recognized | 未识别到文本 |
| `ocr.error.permission.macos` | Grant Screen Recording permission | 请授予屏幕录制权限 |
| `ocr.error.permission.macosCta` | Open System Settings | 打开系统设置 |
| `ocr.error.permission.windows` | Capture unavailable | 捕获不可用 |
| `ocr.error.protectedContent` | Protected content cannot be captured | 受保护内容无法捕获 |
| `ocr.processing` | Recognizing text… | 正在识别文本… |

## 组件组合

- **遮罩：** 全屏透明遮罩（每显示器一个），`--color-bg-overlay` 变暗
- **十字准星：** crosshair 跟随光标
- **选区矩形：** 亮色边框跟随拖拽
- **捕获反馈：** 闪光 + Spinner
- **处理反馈：** 选区处小 Spinner + 文本"正在识别文本…"
- **错误：** Inline error + 权限引导（macOS 打开系统设置）

## 页面特有约束

- 全屏透明遮罩，每显示器一个；无窗口 chrome、标题栏、任务栏条目。
- 不可调整大小（resizable = false）；z-order：Above all。
- 用户拖拽选择矩形；`Esc` 或右键立即取消。
- 选择后遮罩隐藏，捕获继续。
- OCR 仅在选择时占用活跃显示器。
- macOS：ScreenCaptureKit + Vision；需屏幕录制权限（TCC）。
- Windows：DXGI Desktop Duplication + Windows.Media.Ocr；透明覆盖窗每显示器一个；无用户授权对话框（DXGI 不需要）；DRM/远程桌面/不支持驱动返回明确错误。
- 跨显示器选择：每显示器帧单独捕获裁剪后按物理像素坐标拼接；输出 BGRA8/sRGB。
