# Surface 15: External API

**Surface ID:** `surface.external-api`
**Penpot 页面:** 50 System
**Penpot 画板尺寸:** [待查询]
**生产窗口默认尺寸:** 设置子页
**生产窗口最小尺寸:** 同设置（600×400）
**尺寸冲突说明:** 以生产窗口尺寸为实施标准

## 状态矩阵

| 状态 | 描述 | 组件 |
|---|---|---|
| Disabled | "外部 API：关闭" + "启用" | Status + Button (enable) |
| Enabling | spinner → 令牌显示一次 → "立即复制 — 您将无法再次查看" | Spinner + 令牌模态 + 提示 |
| Enabled | "外部 API：开启（端口 {port}）" + "重新生成令牌" + "禁用"（此状态不显示/不可复制令牌） | Status + Button × 2 |
| Regenerating | 警告："旧令牌将立即失效" → 新令牌显示一次 | 警告 + 新令牌模态 |

## 本地化 copy key

| key | en | zh |
|---|---|---|
| `externalApi.title` | External API | 外部 API |
| `externalApi.status.disabled` | External API: Off | 外部 API：关闭 |
| `externalApi.status.enabled` | External API: On (port {port}) | 外部 API：开启（端口 {port}） |
| `externalApi.action.enable` | Enable | 启用 |
| `externalApi.action.disable` | Disable | 禁用 |
| `externalApi.action.regenerateToken` | Regenerate Token | 重新生成令牌 |
| `externalApi.tokenWarning` | Copy now — you won't see it again | 立即复制 — 您将无法再次查看 |
| `externalApi.regenerate.warning` | Old token will stop working immediately | 旧令牌将立即失效 |
| `externalApi.port.label` | Port | 端口 |
| `externalApi.portInUse` | Port {port} is in use — choose another | 端口 {port} 已被占用 — 请选择其他端口 |
| `externalApi.endpoint.health` | GET /v1/health | GET /v1/health |
| `externalApi.rateLimit` | 60 requests/minute | 60 次请求/分钟 |

## 组件组合

- **状态显示：** Status badge（关闭/开启 + 端口）
- **启用流程：** Button (enable) → Spinner → 令牌模态（显示一次）+ 复制提示
- **已启用：** 端口显示 + Button (重新生成令牌) + Button (禁用)
- **重新生成：** 警告（旧令牌立即失效）→ 新令牌模态（显示一次）
- **端点信息：** 健康检查端点、速率限制说明
- **端口冲突：** `PortInUse` 状态提示选择其他端口

## 页面特有约束

- 设置子页，遵循设置窗口尺寸与自适应规则。
- 默认关闭；启用时绑定 `127.0.0.1:61742`（端口可配）。
- 令牌：32 随机字节 base64url（无填充），仅创建/重新生成时返回一次；`external_api_status` 永不返回令牌。
- 常量时间比较（`subtle::ConstantTimeEq`）；每请求比较。
- 无 CORS：拒绝任何带 `Origin` 头的请求；`Host` 必须是回环。
- 启用序列崩溃安全：先绑 socket → 写 keystore + prefs → 启动 server；任一失败回滚。
- 端口被占：server 不启动，返回 `PortInUse`，令牌保留，用户改端口重新绑定。
- 速率限制：60 请求/分钟（滑动窗口，按令牌，进程生命周期）。
- 此 Surface 与 Surface 08 (privacy-data) 的外部 API 区重叠；本页聚焦令牌生命周期与端点详情。
