# Surface 02: Input Window

**Surface ID:** `surface.input-window`
**Penpot 页面:** 10 Core Translation
**Penpot 画板尺寸:** [待查询]
**生产窗口默认尺寸:** 420×280
**生产窗口最小尺寸:** 360×200
**尺寸冲突说明:** 以生产窗口尺寸为实施标准

## 状态矩阵

| 状态 | 描述 | 组件 |
|---|---|---|
| Initial | 空文本区，焦点就绪 | TextArea（focus） |
| Loading | 翻译按钮 → "…"；文本区只读 | Button (loading) + TextArea (disabled) |
| Success | 翻译显示在输入下方 | TextArea（输入）+ ResultCard（结果） |
| Error | 错误消息显示在输入下方 | TextArea（输入）+ Inline error |
| Offline | 错误"离线"（或配置回退时显示回退结果） | Inline error 或 ResultCard |

## 本地化 copy key

| key | en | zh |
|---|---|---|
| `input.title` | Translate | 翻译 |
| `input.placeholder` | Type text to translate… | 输入要翻译的文本… |
| `input.action.translate` | Translate | 翻译 |
| `input.action.clear` | Clear | 清空 |
| `input.result.label` | Translation | 翻译结果 |
| `input.error.offline` | Offline | 离线 |

## 组件组合

- **窗口标题：** `--text-lg` + `--color-fg`
- **输入区：** TextArea（label = "Translate"，placeholder，focus 时 2px ring）
- **操作行（右对齐）：** Button (secondary, "Clear") · Button (primary, "Translate")
- **结果区：** ResultCard（成功）或 Inline error（失败），位于输入下方
- **加载：** 翻译 Button (loading) + TextArea disabled

## 页面特有约束

- 窗口 z-order：Always-on-top。
- 可调整大小（resizable = true）。
- 自动保存（autosave）输入内容，避免丢失工作。
- 持久化桌面 shell；Enter 触发翻译。
