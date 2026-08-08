# Surface 07: Shortcuts

**Surface ID:** `surface.shortcuts`
**Penpot 页面:** 20 Provider & Settings
**Penpot 画板尺寸:** [待查询]
**生产窗口默认尺寸:** 设置子页
**生产窗口最小尺寸:** 同设置（600×400）
**尺寸冲突说明:** 以生产窗口尺寸为实施标准

## 状态矩阵

| 状态 | 描述 | 组件 |
|---|---|---|
| Default | 动作列表 + 当前组合键 + "修改" | ListRow × N + Button (ghost) |
| Recording | "按下组合键…" + 取消 | Shortcut chip + Button (cancel) |
| Conflict | 红色高亮："与 {other action} 冲突" + "覆盖" / "取消" | Inline error + Button × 2 |
| Registration failed | 警告："此组合无法注册（系统保留）" + 还原 | Inline error (warning) |

## 本地化 copy key

| key | en | zh |
|---|---|---|
| `shortcuts.title` | Keyboard Shortcuts | 键盘快捷键 |
| `shortcuts.action.selection` | Translate Selection | 翻译选区 |
| `shortcuts.action.input` | Translate Input | 输入翻译 |
| `shortcuts.action.ocr` | OCR Translate | OCR 翻译 |
| `shortcuts.action.clipboard` | Translate Clipboard | 翻译剪贴板 |
| `shortcuts.action.change` | Change | 修改 |
| `shortcuts.recording.prompt` | Press a key combo… | 按下组合键… |
| `shortcuts.recording.cancel` | Cancel | 取消 |
| `shortcuts.conflict.title` | Conflict | 冲突 |
| `shortcuts.conflict.message` | Conflicts with {action} | 与 {action} 冲突 |
| `shortcuts.conflict.override` | Override | 覆盖 |
| `shortcuts.registration.failed` | This combo couldn't be registered (system reserved) | 此组合无法注册（系统保留） |
| `shortcuts.action.resetDefaults` | Reset to Defaults | 恢复默认 |
| `shortcuts.action.useDefaults` | Use Defaults | 使用默认 |

## 组件组合

- **列表：** ListRow × N
  - leading：动作图标
  - title：动作名称
  - trailing：Shortcut chip（当前组合键）+ Button (ghost, "修改")
- **录制态：** Shortcut chip 显示"按下组合键…" + Button (cancel)
- **冲突：** Inline error（红色高亮）+ Button (覆盖) + Button (取消)
- **注册失败：** Inline error (warning) + 自动还原原组合
- **底部操作：** Button (secondary, "恢复默认")

## 页面特有约束

- 设置子页，遵循设置窗口尺寸与自适应规则。
- 注册失败不崩溃；自动还原原组合键。
- 冲突检测使用 `shortcut_check_conflict(combo)`。
- 全局热键引擎在 macOS/Windows 一致；同样冲突检测。
