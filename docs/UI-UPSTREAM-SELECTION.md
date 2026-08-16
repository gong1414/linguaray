# LinguaRay 上游界面壳筛选与迁移决定

> 调研日期：2026-08-16。本文记录的是源码级筛选，不是截图风格参考。
> 所有候选均已 shallow clone 到 `/tmp/linguaray-ui-upstreams-20260816`
> 并固定到下表所列提交进行检查。

## 结论

LinguaRay 不直接 fork Pot，也不把业务能力搬进另一个应用。保留现有
Rust/Tauri 能力层和 TypeScript controller，将界面层切换为下面的组合：

1. **唯一组件框架：Microsoft Fluent UI React v9。** 它是 MIT 许可、
   React 19 可用、完整主题化并以 WCAG 2.1 AA 为基础，不再维护一套
   LinguaRay 自制 Button/Input/Card/Switch。
2. **主要桌面壳参考：Ueli。** Ueli 是 MIT、React 19、Fluent UI v9，
   已同时实现快捷键唤起的紧凑窗口和完整设置中心。复用其纯 React
   设置分组、导航和搜索/结果窗口结构；Electron bridge 代码不进入本项目。
3. **触发与状态参考：Handy。** Handy 是 MIT、Tauri 2，覆盖全局快捷键、
   独立浮层、设置、首次引导和 macOS 权限回查。只借鉴窗口生命周期和
   状态机；它的 Tailwind 自制组件不复用。
4. **翻译交互参考：Pot。** Pot 的输入区、语言栏和多服务结果分组最贴合
   LinguaRay，但项目为 GPL-3.0，且 Tauri 1 前端直接调用密集，因此只作为
   交互参考，禁止复制源码或样式。
5. **macOS 行为校准：Maccy/Pesty。** 两者为 MIT 原生菜单栏工具，用来
   校准焦点、键盘导航、浮窗尺寸和设置行为；Swift/SwiftUI 代码不移植。

这不是“混搭设计”：最终生产代码只出现 Fluent UI 一套组件。其他项目
只回答窗口如何触发、信息如何分层、权限何时复查等产品问题。

## 候选源码审计

| 项目 | 固定提交 | 许可/技术 | 与 LinguaRay 的重合 | 结论 |
| --- | --- | --- | --- | --- |
| [Pot](https://github.com/pot-app/pot-desktop) | `594d32ede96a` | GPL-3.0；Tauri 1 + React 18 + NextUI/Tailwind | 划词、输入、OCR、多服务结果、跟随鼠标浮窗 | 交互参考；不得复制。206 个前端文件中 94 个直接引用 Tauri，能力/UI 耦合不符合目标 |
| [Easydict](https://github.com/tisfeng/Easydict) | `f23ae47be795` | GPL-3.0；Swift/AppKit/SwiftUI | macOS 划词、OCR、权限、菜单栏 | macOS 行为参考；不得复制且不能跨平台复用 |
| [Ueli](https://github.com/oliverschwendener/ueli) | `f04ebdd82df7` | MIT；Electron + React 19 + Fluent UI v9 | 快捷键触发窗口、搜索/输入、结果列表、设置导航、主题 | **主壳来源**。225 个 renderer 文件、114 个直接使用 Fluent UI、无 CSS 文件；只适配纯 UI，隔离 Electron bridge |
| [Handy](https://github.com/cjpais/Handy) | `98a4d80cce8a` | MIT；Tauri 2 + React 18 + Tailwind | 快捷键、独立状态浮层、权限、引导、设置、托盘 | 窗口/状态参考。自带 18 个 UI primitives、646 行 CSS，不作为组件来源 |
| [Maccy](https://github.com/p0deje/Maccy) | `d994f91f11e4` | MIT；SwiftUI/AppKit | 菜单栏、全局快捷键、搜索列表、设置 | 交互校准；原生实现不可跨平台直接移植 |
| [Pesty](https://github.com/momenbasel/pesty) | `70fd6b2c47d4` | MIT；Swift 6 | 横向触发条、快捷键录制、权限设置 | 交互校准；项目较小且仅 macOS |
| [Headroom](https://github.com/allandecastro/headroom) | `80a612e5b0ca` | MIT；Tauri 2 + React 18 + Tailwind | 菜单栏 popover、设置、引导 | 不采用。只有 9 stars，且自建 Button/Toggle/Slider 等组件，重复当前风险 |
| [Tauri Template](https://github.com/dannysmith/tauri-template) | `437a18b9b639` | MIT；Tauri 2 + React 19 + Radix/shadcn | 主窗口、quick pane、设置、命令面板 | 构建参考，不作为 UI 来源；仓库内维护 38 个复制型 UI primitives |
| [CC Switch](https://github.com/farion1231/cc-switch) | `4080a8e95c1c` | MIT；Tauri 2 + React + Radix/shadcn | 服务商、设置、状态、更新 | 业务页参考；自有 UI primitives 数量大，且不是快捷触发型壳 |
| [PicGo](https://github.com/Molunerfinn/PicGo) | `45fd078e4ef6` | MIT；Electron + Vue | 托盘、设置、任务结果 | 技术栈不兼容，不移植 |
| [Twinkle Tray](https://github.com/xanderfrangos/twinkle-tray) | `7f31cb4f6630` | MIT；Electron + React + SCSS | 托盘触发面板、完整设置 | Windows-only 且大量自定义 SCSS，只参考面板行为 |
| [Clippy](https://github.com/yarasaa/Clippy) | `f613604d9f78` | README 声称 MIT；SwiftUI | 快捷键浮窗、OCR、权限、设置、引导 | clone 中缺少独立 LICENSE 文件；许可补齐前不复制 |
| [Tauri macOS menubar example](https://github.com/ahkohd/tauri-macos-menubar-app-example) | `c7a96468c44b` | MIT；Tauri 2 + React 19 | tray anchor、popover window lifecycle | 只复用必要的 Tauri 窗口生命周期实现，不用其演示 UI |
| [Vicinae](https://github.com/vicinaehq/vicinae) | `2cd77352150e`（API） | GPL-3.0；C++/Qt + React extensions | Raycast 型命令面板 | 交互参考；不得复制 |
| [Flow Launcher](https://github.com/Flow-Launcher/Flow.Launcher) | `5018101c3211`（API） | MIT；WPF | 快捷键命令窗口、插件结果 | 技术栈不可直接复用，只校准键盘交互 |

## 选择依据

权重按 LinguaRay 的真实风险设置，而不是按截图好看排序：

| 维度 | 权重 | Fluent UI + Ueli | Pot | Handy | Tauri Template |
| --- | ---: | ---: | ---: | ---: | ---: |
| 许可允许 MIT 项目直接复用 | 20 | 20 | 0 | 20 | 20 |
| React/Tauri renderer 可移植性 | 20 | 17 | 11 | 18 | 20 |
| 快捷触发窗口 + 设置中心覆盖 | 20 | 19 | 20 | 18 | 15 |
| 使用外部成熟组件而非自建控件 | 20 | 20 | 16 | 5 | 9 |
| 可访问性、主题和键盘基础 | 10 | 10 | 6 | 5 | 8 |
| 活跃度和已验证桌面规模 | 10 | 9 | 9 | 8 | 6 |
| **总分** | **100** | **95** | **62** | **74** | **78** |

## 能力与界面边界

```text
Rust capabilities
  selection / OCR / providers / history / shortcuts / updater / permissions
                         │
                         ▼
src/bridge (唯一 Tauri API 边界)
                         │
                         ▼
feature controllers + domain state
                         │ props/events only
                         ▼
Fluent UI views
  settings shell / trigger window / result cards / onboarding / overlay
```

强制规则：

- `view.tsx` 和 `*View.tsx` 不能导入 `src/bridge`、`@tauri-apps/*` 或
  controller 实现；只能接收 serializable state、callback 和 controller type。
- 生产界面控件只从 `@fluentui/react-components` 导入，图标只从
  `@fluentui/react-icons` 导入。
- 仅布局可以使用 CSS module；颜色、排版、间距、阴影、圆角全部使用
  Fluent tokens，不创建第二套 token。
- 原生标题栏仍归 Tauri/OS；只有翻译 popup 与 OCR overlay 可无装饰。
- GPL 项目的源码、CSS、图标和资产不进入仓库。
- 若复制 Ueli 的实质代码，必须保留 `THIRD_PARTY_NOTICES.md` 中的 MIT
  copyright 与固定提交来源。

## 迁移顺序与完成证据

1. FluentProvider、亮/暗主题和 Ueli 风格设置壳。
2. 输入翻译与结果 popup，采用紧凑的 header/content/footer 结构。
3. 设置页统一成 Fluent `Card`/`Field`/`Switch`/`Dropdown`/`Dialog`。
4. 引导与 OCR overlay；对照 Handy 的权限回查和窗口可见性规则。
5. 删除 Mantine 和 lucide-react，更新规则与第三方声明。
6. 必须通过 typecheck、lint、unit、Storybook axe、视觉基线和真实签名应用
   的快捷键/菜单栏/输入/OCR/设置全流程，才算迁移完成。

### 2026-08-16 完成记录

- 生产界面已全部迁移到 Fluent UI v9；Mantine、Lucide 与旧 CSS module
  已从生产依赖和视图代码中删除。
- popup 视图不再直接导入 bridge/IPC；复制、朗读、收藏和打开设置均由
  controller 注入，能力层和展示层边界由 `test/ui-freeze.test.ts` 强制检查。
- `pnpm typecheck`、`pnpm lint`、`pnpm test`（29 个测试文件、233 项测试）、
  `pnpm build` 与 `pnpm build-storybook` 全部通过。
- 使用稳定的 Apple Development 身份构建并验证
  `/Applications/LinguaRay.app`。已实际检查设置导航、现有辅助功能权限回查、
  输入翻译窗口、圆角透明结果 popup、全屏 OCR overlay 和 onboarding 权限页；
  未再出现额外普通 OCR 窗口、未授权误报、裸 HTML 控件或页脚裁切。
- 为避免 WKWebView 中预置样式表干扰 Griffel 运行时样式桶，窗口级 reset
  由 `src/app/windowDocument.ts` 在挂载前写入元素 style，不在 HTML 中注入
  额外 `<style>` 标签。

## 明确排除

- 不 fork Pot 后再把 LinguaRay 能力塞进去：GPL 许可、Tauri 1 和前端能力
  耦合会让架构倒退。
- 不采用 shadcn/Tailwind 源码组件集合：这些组件最终由本仓库维护，仍会
  回到“每个 AI 都能随手改控件”的问题。
- 不为 macOS 与 Windows 分别维护 SwiftUI/WinUI 两套设置页：能力层可以
  复用，但两套 UI 会把测试与迭代成本翻倍。
- 不复制整套 Electron 主进程；Ueli 只提供 renderer 界面结构，窗口与权限
  继续由当前 Tauri/Rust 层负责。
