# Surface 16: Updater

**Surface ID:** `surface.updater`
**Penpot 页面:** 50 System
**Penpot 画板尺寸:** [待查询]
**生产窗口默认尺寸:** 设置子页/弹窗
**生产窗口最小尺寸:** 同设置（600×400）
**尺寸冲突说明:** 以生产窗口尺寸为实施标准

## 状态矩阵

| 状态 | 描述 | 组件 |
|---|---|---|
| Checking | 静默（后台） | — |
| Up to date | "LinguaRay 已是最新（v{version}）" | Status (success) |
| Update available | "v{new_version} 可用" + 更新日志摘要 + "下载" | Status + 更新日志 + Button (download) |
| Downloading | 进度条 | Progress bar |
| Verifying | "正在验证签名…" | Spinner + 文本 |
| Verification failed | 错误："更新签名验证失败 — 已中止更新" | Inline error (destructive) |
| Ready to install | "重启以更新" + "立即重启" / "稍后" | Status + Button × 2 |
| Install failed | 错误："更新安装失败：{reason}" + "手动下载"链接 | Inline error + Link |

## 本地化 copy key

| key | en | zh |
|---|---|---|
| `updater.title` | Updates | 更新 |
| `updater.checking` | Checking for updates… | 正在检查更新… |
| `updater.upToDate` | LinguaRay is up to date (v{version}) | LinguaRay 已是最新（v{version}） |
| `updater.available.title` | v{version} available | v{version} 可用 |
| `updater.available.changelog` | What's new | 更新内容 |
| `updater.action.download` | Download | 下载 |
| `updater.downloading` | Downloading… | 下载中… |
| `updater.verifying` | Verifying signature… | 正在验证签名… |
| `updater.verificationFailed` | Update signature verification failed — update aborted | 更新签名验证失败 — 已中止更新 |
| `updater.readyToInstall` | Restart to update | 重启以更新 |
| `updater.action.restartNow` | Restart Now | 立即重启 |
| `updater.action.later` | Later | 稍后 |
| `updater.installFailed` | Update installation failed: {reason} | 更新安装失败：{reason} |
| `updater.installFailed.downloadManually` | Download manually | 手动下载 |
| `updater.action.checkForUpdates` | Check for Updates | 检查更新 |

## 组件组合

- **检查触发：** Button (检查更新)
- **状态显示：** Status badge（success: 已最新 / info: 可用 / destructive: 失败）
- **更新可用：** 版本号 + 更新日志摘要（Card/文本区）+ Button (下载)
- **下载：** Progress bar
- **验证：** Spinner (12px) + 文本"正在验证签名…"
- **验证失败：** Inline error (destructive)
- **就绪安装：** Status + Button (立即重启) + Button (ghost, 稍后)
- **安装失败：** Inline error + Link (手动下载)

## 页面特有约束

- 设置子页/弹窗，遵循设置窗口尺寸与自适应规则。
- Tauri 更新器使用独立签名密钥。
- manifest 仅在所有平台通过后生成。
- 发布规则：S7 验收通过前不公开标签、不发布 GitHub Release（CI 产物仅供内部测试）。
- 此页也可作为托盘"有可用更新"菜单项触发的弹窗入口（链接 Surface 04）。
