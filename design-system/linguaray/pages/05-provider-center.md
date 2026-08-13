# Surface 05: Provider Center

**Surface ID:** `surface.provider-center`
**Penpot 页面:** 20 Provider & Settings
**Penpot 画板尺寸:** [待查询]
**生产窗口默认尺寸:** 800×600
**生产窗口最小尺寸:** 600×400
**尺寸冲突说明:** 以生产窗口尺寸为实施标准

## 状态矩阵

| 状态 | 描述 | 组件 |
|---|---|---|
| Initial (no providers) | 空状态："添加你的第一个服务商" + 预设建议 | EmptyState + preset 网格 |
| Loading models | 模型下拉中 spinner | Select + Spinner (12px) |
| Model fetch error | 错误 tooltip；仍可手动输入 | Tooltip + TextField |
| Connection testing | 测试按钮上 spinner | Button (loading) |
| Connection OK | 绿色对勾 + 延迟 ms | Status badge (success) |
| Connection failed | 红色 X + 错误消息 | Status badge (destructive) + Inline error |
| Key saved | provider 卡片上 "✓" badge | Status badge (success) |
| Key missing | 警告 badge；"输入密钥"提示 | Status badge (warning) + Button |
| No key required (R11) | `needs_key=false` provider：neutral badge，无 key 输入/保存按钮，显示"无需密钥" | Status badge (neutral) + Text (muted) |
| Duplicate | 新卡片带"(copy)"后缀 | ProviderCard |
| Saving | 保存按钮上 spinner；输入禁用 | Button (loading) + inputs (disabled) |
| Save failed | 错误 toast："保存失败：{reason}" | Toast (destructive) |
| Save conflict | 错误："此服务商已在别处修改。重新加载？" | Inline error + Button |
| Delete confirm | 对话框："删除 {name}？历史引用保留。" | Confirm dialog (destructive) |
| Deleting | 卡片置灰 + spinner；禁用 | ProviderCard (disabled) + Spinner |
| Delete retry | 卡片显示"删除失败 — 重试？" | Inline error + Button |
| Drag-to-reorder | 拖拽手柄；悬停/拖拽时视觉指示 | ProviderCard + drag handle |
| Reorder persist failed | toast："保存顺序失败 — 已还原" | Toast (destructive) |
| Balance loading | 余额位置 spinner | Spinner (12px) |
| Balance unsupported | "—"（此 provider 无余额） | Text (muted) |
| Balance rate-limited | "已限流 — 稍后重试" | Inline error |
| Balance error | "获取余额出错" | Inline error |
| Endpoint invalid | endpoint 字段红边框："必须 HTTPS（或 localhost）" | TextField (error) |
| Model manual entry | 获取失败或不支持时显示文本输入 | TextField |

## 本地化 copy key

| key | en | zh |
|---|---|---|
| `provider.empty.title` | Add your first provider | 添加你的第一个服务商 |
| `provider.empty.cta` | Browse presets | 浏览预设 |
| `provider.action.add` | Add Provider | 添加服务商 |
| `provider.action.duplicate` | Duplicate | 复制 |
| `provider.action.delete` | Delete | 删除 |
| `provider.action.test` | Test | 测试 |
| `provider.action.fetchModels` | Fetch Models | 获取模型 |
| `provider.action.save` | Save | 保存 |
| `provider.key.saved` | Key saved | 密钥已保存 |
| `provider.key.missing` | Enter key | 输入密钥 |
| `provider.key.notRequired` (R11) | No key required | 无需密钥 |
| `provider.connection.ok` | Connected ({latency} ms) | 已连接（{latency} ms） |
| `provider.connection.failed` | Connection failed | 连接失败 |
| `provider.delete.confirm` | Delete {name}? History references are preserved. | 删除 {name}？历史引用会保留。 |
| `provider.delete.failed` | Delete failed — retry? | 删除失败 — 重试？ |
| `provider.save.failed` | Failed to save: {reason} | 保存失败：{reason} |
| `provider.save.conflict` | This provider was modified elsewhere. Reload? | 此服务商已在别处修改。重新加载？ |
| `provider.reorder.failed` | Failed to save order — reverted | 保存顺序失败 — 已还原 |
| `provider.balance.rateLimited` | Rate limited — try later | 已限流 — 稍后重试 |
| `provider.balance.error` | Error fetching balance | 获取余额出错 |
| `provider.endpoint.invalid` | Must be HTTPS (or localhost) | 必须 HTTPS（或 localhost） |

## R11：密钥状态三态模型（needs_key 感知）

`needs_key=false` 的 provider（如本地 Ollama 预设）不使用 API 密钥。此前这类
provider 在 `hasKey=false` 时错误显示 "Key missing"（警告 badge + 密钥输入），
误导用户为不需要密钥的 provider 输入密钥。R11 将密钥状态建模为三态：

| 状态 | 条件 | 行 / 详情面板表现 |
|---|---|---|
| `saved` | `needs_key=true` + 已有密钥 | 行：根据 role 显示 active/available；详情：密钥已保存 badge |
| `missing` | `needs_key=true` + 无密钥 | 行：警告 badge "Key missing"；详情：密钥输入 + 保存按钮 |
| `not-required` | `needs_key=false` | 行：neutral badge（**绝不**显示 key-missing）；详情：显示"无需密钥"文本，**不渲染**密钥输入/保存按钮 |

设计要点：
- `providerStatus` 与 `providerKeyStatus` 均新增 `needsKey` 参数；`!needsKey`
  一律返回 available/neutral（行）或 "not-required"（密钥状态）。
- 详情面板 Key 区外层 `Show` 以 `needs_key` 为门控：为 false 时仅渲染
  "No key required" 文本，隐藏所有密钥输入与保存按钮。
- `handleSaveKey` fail-closed：对 `!needs_key` 或空密钥直接 return，绝不发起
  `provider_set_key` IPC（UI 已不可达，此为纵深防御）。
- 后端 `provider_set_key` / `set_key_blocking` 同样拒绝 `needs_key=false` 与空
  密钥，避免遗留 provider 永不读取的悬空密钥。

## 组件组合

- **设置外壳：** Sidebar（≥700px 全标签 / 600–699px 图标轨）+ 内容区
- **列表区：** ProviderCard × N（按 sort_order），活跃 provider 左边 3px 强调边
  - 名称 + template badge + 密钥状态 + 启用 Switch
  - 拖拽手柄用于重排
- **编辑区/表单：** TextField (name) · TextField (endpoint, HTTPS 校验) · Select (model) / TextField (manual) · TextField (key, monospace)
- **操作行：** Button (test) · Button (fetch models) · Button (duplicate) · Button (destructive, delete)
- **余额/状态：** Status badges (success/warning/destructive)
- **反馈：** Toast (保存/排序失败) · Confirm dialog (删除确认)
- **空状态：** EmptyState + preset 网格

## 页面特有约束

- 设置子页，遵循设置窗口自适应：≥700px 全侧栏；600–699px 图标轨；无汉堡菜单路径。
- 窗口最小 600×400（Tauri 强制）；不可横向溢出。
- 删除状态机崩溃安全：deleting → 移除 keystore → tombstone；崩溃恢复从 keystore 移除重试。
- `secret_ref` 稳定（迁移时 = legacy_id，新建 = "provider/<uuid>"）。
- 多引擎同意：`preferences.parallel_consent_version` 跟踪；加入并行列表需重新确认。
