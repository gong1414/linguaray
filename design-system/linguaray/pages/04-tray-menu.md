# Surface 04: Tray Menu

**Surface ID:** `surface.tray-menu`
**Penpot 页面:** 10 Core Translation
**Penpot 画板尺寸:** [待查询]
**生产窗口默认尺寸:** 系统托盘
**生产窗口最小尺寸:** N/A
**尺寸冲突说明:** 以生产窗口尺寸为实施标准

## 状态矩阵

| 状态 | 描述 | 组件 |
|---|---|---|
| Normal | LinguaRay 图标；点击 → 菜单 | Tray icon |
| Active translation | 图标轻微脉动 | Tray icon (pulse) |
| Error (general) | 图标红点 | Tray icon (badge) |
| Update available | 图标 badge + 菜单项 | Tray icon (badge) + 菜单项 |

## 本地化 copy key

| key | en | zh |
|---|---|---|
| `tray.menu.translateSelection` | Translate Selection | 翻译选区 |
| `tray.menu.translateInput` | Translate Input | 输入翻译 |
| `tray.menu.translateClipboard` | Translate Clipboard | 翻译剪贴板 |
| `tray.menu.ocrCapture` | OCR Translate | OCR 翻译 |
| `tray.menu.activeProvider` | Active Provider | 当前服务商 |
| `tray.menu.switchProvider` | Switch Provider | 切换服务商 |
| `tray.menu.history` | History | 历史 |
| `tray.menu.settings` | Settings | 设置 |
| `tray.menu.quit` | Quit | 退出 |
| `tray.menu.updateAvailable` | Update Available | 有可用更新 |
| `tray.menu.providerReady` | Ready | 就绪 |
| `tray.menu.providerNotReady` | Not ready (no key) | 未就绪（缺少密钥） |

## 组件组合

- **原生托盘菜单：** macOS `NSStatusItem` / Windows Tauri `SystemTray`
- **快速操作组：** 翻译选区 · 输入翻译 · 翻译剪贴板 · OCR 翻译
- **当前服务商组：** 显示活跃 provider + 快速切换子菜单
- **状态指示：** provider 就绪状态（有/无密钥）
- **导航组：** 历史 · 设置
- **系统组：** 更新（如有）· 退出

## 页面特有约束

- 平台原生菜单（macOS NSStatusItem / Windows SystemTray），菜单项和操作保持一致。
- 应用驻留托盘；启动不强制打开主窗口。
- 快速切换活跃 provider 无需打开设置。
- 启动时不强制打开主窗口（进程生命周期常驻）。
- 此 Surface 为原生系统组件，不使用 Web UI 组件，尺寸由系统托盘规范决定（N/A）。
