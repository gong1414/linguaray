# Surface 01: Selection Popup

**Surface ID:** `surface.selection-popup`
**Penpot 页面:** 10 Core Translation
**Penpot 画板尺寸:** [待查询]
**生产窗口默认尺寸:** 460×420
**生产窗口最小尺寸:** 200×40 / 600×400
**尺寸冲突说明:** 以生产窗口尺寸为实施标准

## 状态矩阵

| 状态 | 描述 | 组件 |
|---|---|---|
| Initial | 不显示（隐藏） | — |
| Loading | 光标处小卡片："…" spinner | Spinner (12px `Loader2`) + 卡片 |
| Success (single) | 翻译文本 + 引擎标签 | ResultCard（单个） |
| Success (multi) | 标签页/堆叠结果，每个带引擎标签 | ResultCard × N + Tabs/SegmentedControl |
| Partial success | 部分引擎成功 + 失败引擎的错误 badge | ResultCard（成功）+ Inline error badge（失败） |
| Error (network) | 错误卡片："网络错误" | EmptyState + Inline error |
| Error (config) | 错误卡片："缺少 API 密钥" 或 "401 未授权" | EmptyState + Inline error |
| Error (no selection) | 错误卡片："未选中文本" | EmptyState + Inline error |
| Error (no permission) | 错误卡片："请授予辅助功能权限" | EmptyState + Inline error |
| Keystore corrupt | 错误卡片："密钥库不可读" + 设置恢复链接 | Inline error + Link/Button |
| Offline | 传统引擎可用 → 回退结果；否则错误"离线" | ResultCard 或 Inline error |
| Pinned | 弹窗保持显示直到手动关闭；支持 copy/retry/TTS/favorite | IconButton row (Copy/Speak/Pin/Favorite) |

## 本地化 copy key

| key | en | zh |
|---|---|---|
| `selection.loading` | Translating… | 翻译中… |
| `selection.error.network` | Network error | 网络错误 |
| `selection.error.config.key` | API key missing | 缺少 API 密钥 |
| `selection.error.config.auth` | 401 Unauthorized | 401 未授权 |
| `selection.error.noSelection` | No text selected | 未选中文本 |
| `selection.error.noPermission` | Grant Accessibility permission | 请授予辅助功能权限 |
| `selection.error.keystore` | Keystore unreadable | 密钥库不可读 |
| `selection.error.keystore.cta` | Go to settings recovery | 前往设置恢复 |
| `selection.error.offline` | Offline | 离线 |
| `selection.action.copy` | Copy | 复制 |
| `selection.action.speak` | Speak | 朗读 |
| `selection.action.pin` | Pin | 固定 |
| `selection.action.unpin` | Unpin | 取消固定 |
| `selection.action.favorite` | Save to vocabulary | 收藏到生词本 |
| `selection.action.retry` | Retry | 重试 |

## 组件组合

- **卡片容器：** `--color-bg-elevated` + `--radius-lg` + `--shadow-lg`
- **单结果：** ResultCard（source/engine label + 翻译文本 + 操作行）
- **多结果：** SegmentedControl（标签页）+ ResultCard × N，按 provider 排序横向并排
- **操作行：** IconButton (Copy) · IconButton (Speak) · IconButton (Pin/Unpin) · IconButton (Favorite)
- **加载：** Spinner (12px) + visually-hidden "翻译中…" 文本
- **错误：** EmptyState（icon + title）+ Inline error +（可选）恢复链接

## 页面特有约束

- 窗口 z-order：Always-on-top。
- 单结果最大 400×300；展开（多引擎）最大 600×400；展开/收起通过 Tauri `setSize`/`setMaxSize` 切换。
- 不可调整大小（resizable = false）。
- 工作区夹紧（work-area clamping）：四边都夹紧，margin = 8px，原生窗口不超出屏幕工作区。
- 多引擎结果按 provider 排序横向并排显示，2 个 provider = 2 列（每列 ≥ 200px），3+ 超宽时内部横向滚动。
- Pinned 状态无视 blur 仍可见；只有未固定的弹窗在 blur 时隐藏。
- 多结果时弹窗就地展开，不打开竞争窗口。
