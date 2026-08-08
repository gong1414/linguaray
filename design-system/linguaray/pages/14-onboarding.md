# Surface 14: Onboarding

**Surface ID:** `surface.onboarding`
**Penpot 页面:** 50 System
**Penpot 画板尺寸:** [待查询]
**生产窗口默认尺寸:** 600×400
**生产窗口最小尺寸:** 600×400
**尺寸冲突说明:** 以生产窗口尺寸为实施标准

## 状态矩阵

> **1 个欢迎页 + 5 个步骤 = 6 个状态**
> （welcome / step-1-accessibility / step-2-provider / step-3-history / step-4-shortcuts / step-5-complete）

| 状态 | 描述 | 组件 |
|---|---|---|
| Welcome | "欢迎使用 LinguaRay" + 简介 + "开始使用" | Hero + Button (primary) |
| Step-1-accessibility | 辅助功能授权（macOS）：解释 + "打开系统设置" + "跳过" | 说明 + Button × 2 |
| Step-2-provider | 添加首个服务商：预设网格 → 选择 → 输入密钥 → "测试" | Preset 网格 + 表单 + Button (test) |
| Step-3-history | 历史启用："启用翻译历史？" + 隐私说明 + "启用" / "跳过" | 隐私说明 + Button × 2 |
| Step-4-shortcuts | 快捷键设置：显示默认快捷键；"自定义" 或 "使用默认" | 快捷键列表 + Button × 2 |
| Step-5-complete | "设置完成！" + "打开设置" 或 "最小化到托盘" | Hero + Button × 2 |

## 本地化 copy key

| key | en | zh |
|---|---|---|
| `onboarding.welcome.title` | Welcome to LinguaRay | 欢迎使用 LinguaRay |
| `onboarding.welcome.description` | A privacy-first translation tool for your menu bar. | 一款隐私优先的菜单栏翻译工具。 |
| `onboarding.welcome.cta` | Get started | 开始使用 |
| `onboarding.accessibility.title` | Grant Accessibility | 授予辅助功能权限 |
| `onboarding.accessibility.description` | LinguaRay needs Accessibility to read selected text. | LinguaRay 需要辅助功能权限以读取选中文本。 |
| `onboarding.accessibility.openSettings` | Open System Settings | 打开系统设置 |
| `onboarding.accessibility.skip` | Skip | 跳过 |
| `onboarding.provider.title` | Add Your First Provider | 添加您的第一个服务商 |
| `onboarding.provider.description` | Pick a preset, enter your key, and test it. | 选择一个预设，输入密钥，然后测试。 |
| `onboarding.provider.test` | Test | 测试 |
| `onboarding.history.title` | Enable Translation History? | 启用翻译历史？ |
| `onboarding.history.description` | History is encrypted and stored locally only. | 历史经过加密，仅本地存储。 |
| `onboarding.history.enable` | Enable | 启用 |
| `onboarding.history.skip` | Skip | 跳过 |
| `onboarding.shortcuts.title` | Keyboard Shortcuts | 键盘快捷键 |
| `onboarding.shortcuts.description` | Use the defaults or customize them later. | 使用默认值或稍后自定义。 |
| `onboarding.shortcuts.customize` | Customize | 自定义 |
| `onboarding.shortcuts.useDefaults` | Use Defaults | 使用默认 |
| `onboarding.complete.title` | You're all set! | 设置完成！ |
| `onboarding.complete.openSettings` | Open settings | 打开设置 |
| `onboarding.complete.minimizeToTray` | Minimize to tray | 最小化到托盘 |

## 组件组合

- **窗口：** 600×400 单列流程，不可小于 600×400
- **步骤导轨（宽桌面）：** 步骤指示器（step rail）显示当前进度
- **Welcome：** Hero 标题（`--text-xl`）+ 简介（`--text-sm`）+ Button (primary, "开始使用")
- **Step-1：** 说明卡 + Button (打开系统设置) + Button (ghost, 跳过)
- **Step-2：** 预设 provider 网格（Card）→ 表单（TextField endpoint/key + Select model）+ Button (test)
- **Step-3：** 隐私说明 + Button (启用) + Button (ghost, 跳过)
- **Step-4：** 快捷键列表（ListRow × N + Shortcut chip）+ Button (自定义) + Button (使用默认)
- **Step-5：** Hero + Button (打开设置) + Button (最小化到托盘)
- **导航：** 步骤间前进/后退；右对齐主操作

## 页面特有约束

- 目标窗口 600×400（与设置最小相同）；不可调整到 600×400 以下。
- 单列流程；宽桌面尺寸时显示步骤导轨（step rail）。
- 主操作右对齐。
- 这是首次启动流程；完成后应用驻留托盘。
- macOS 才有辅助功能授权步骤（Step-1）；Windows 无对应步骤（可跳过或调整为其他必要设置）。
