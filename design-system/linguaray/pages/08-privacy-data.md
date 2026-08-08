# Surface 08: Privacy & Data

**Surface ID:** `surface.privacy-data`
**Penpot 页面:** 20 Provider & Settings
**Penpot 画板尺寸:** [待查询]
**生产窗口默认尺寸:** 设置子页
**生产窗口最小尺寸:** 同设置（600×400）
**尺寸冲突说明:** 以生产窗口尺寸为实施标准

## 状态矩阵

| 状态 | 描述 | 组件 |
|---|---|---|
| History disabled | 开关关闭；解释未存储内容 | Switch (off) + 说明文本 |
| History enabled | 开关开启；保留期选择器；"全部清除"按钮 | Switch (on) + Select + Button (destructive) |
| External API off | 开关关闭；解释 | Switch (off) + 说明文本 |
| External API on | 开关开启；端口显示；"重新生成令牌"（模态中显示一次新令牌）；"禁用"（此视图不可复制令牌） | Switch (on) + 端口文本 + Button × 2 |

## 本地化 copy key

| key | en | zh |
|---|---|---|
| `privacy.title` | Privacy & Data | 隐私与数据 |
| `privacy.history.title` | Translation History | 翻译历史 |
| `privacy.history.enable` | Enable history | 启用历史 |
| `privacy.history.disabled.notice` | When off, nothing is stored. | 关闭时不存储任何内容。 |
| `privacy.history.enabled.retention` | Retention period | 保留期 |
| `privacy.history.retention.30days` | 30 days | 30 天 |
| `privacy.history.retention.90days` | 90 days | 90 天 |
| `privacy.history.clearAll` | Clear All | 全部清除 |
| `privacy.history.clearAll.confirm` | Clear all history? This cannot be undone. | 清除全部历史？此操作不可撤销。 |
| `privacy.externalApi.title` | External API | 外部 API |
| `privacy.externalApi.enable` | Enable external API | 启用外部 API |
| `privacy.externalApi.disabled.notice` | Default off. No local endpoint is exposed. | 默认关闭。不暴露本地端点。 |
| `privacy.externalApi.port` | Port | 端口 |
| `privacy.externalApi.regenerateToken` | Regenerate Token | 重新生成令牌 |
| `privacy.externalApi.disable` | Disable | 禁用 |
| `privacy.externalApi.tokenWarning` | Copy now — you won't see it again | 立即复制 — 您将无法再次查看 |

## 组件组合

- **历史区：**
  - Switch (enable/disable) + 说明文本
  - 启用时：Select (保留期：30/90 天) + Button (destructive, "全部清除")
- **外部 API 区：**
  - Switch (enable/disable) + 说明文本
  - 启用时：端口显示（TextField/只读）+ Button (重新生成令牌) + Button (禁用)
  - 令牌仅显示一次（模态）；此视图不可复制令牌
- **清除确认：** Confirm dialog (destructive)

## 页面特有约束

- 设置子页，遵循设置窗口尺寸与自适应规则。
- 历史加密：AES-256-GCM + 域分离 AAD；默认关闭，需显式同意。
- 保留期默认 30 天；收藏永不过期。
- 外部 API 默认关闭；令牌仅创建/重新生成时返回一次，之后不可读、不可复制。
