# LinguaRay 界面强制规则（Fluent UI / Ueli 壳）

> 2026-08-16 完成源码级候选筛选，决定以 MIT 许可的 Ueli renderer
> 结构和 Microsoft Fluent UI React v9 作为唯一生产界面来源。完整选择证据
> 见 `docs/UI-UPSTREAM-SELECTION.md`。

## 自动门禁规则

1. **唯一组件框架是 Fluent UI。** 生产界面控件只能从
   `@fluentui/react-components` 导入，图标只能来自
   `@fluentui/react-icons`。禁止重新加入 Mantine、lucide-react、shadcn、
   Tailwind UI primitives 或 `packages/ui` 自制组件库。
2. **Ueli 只提供 renderer 壳结构。** 设置导航、紧凑触发窗口和结果分层
   采用 Ueli 的 React/Fluent 组合；Electron bridge、主进程和业务能力不进入
   本项目。来源、固定提交与 MIT 声明保留在 `THIRD_PARTY_NOTICES.md`。
3. **能力和界面严格分开。** `@tauri-apps/*` 只能出现在 `src/bridge/`；
   `view.tsx` 与 `*View.tsx` 不得导入 bridge 或 feature IPC 实现。View 只接收
   状态、callback 和 controller 的 type；Tauri 调用放在 controller/ipc。
4. **禁止自绘普通窗口标题栏。** main、onboarding、input 使用系统原生标题栏；
   只有翻译 popup 与 OCR 捕获浮层允许 `decorations: false`。
5. **窗口权限最小化。** 每个 Tauri capability 必须覆盖且只覆盖相应窗口
   实际使用的 API；设置主窗口不授予不需要的窗口能力。
6. **Debug/Release 身份可区分。** `pnpm dev:app` 使用 debug bundle id；
   本地可安装构建使用稳定 Apple Development 签名，避免 macOS TCC 因每次
   ad-hoc `cdhash` 改变而出现“系统设置显示已授权、进程仍未授权”。

## 代码评审规则

7. 颜色、间距、圆角、阴影、排版优先使用 Fluent `tokens`；页面布局通过
   Fluent `makeStyles` 组合。不得创建第二套品牌 token 或复制 Button/Input/
   Dialog 的视觉与交互状态。
8. 已有 Fluent 控件可表达的能力不得用裸 `<button>`、文本符号、CSS 图形或
   手写 SVG 模拟。原生文件输入等 Fluent 未提供的浏览器能力可以隐藏使用，
   但可见触发器必须是 Fluent `Button`。
9. 单个 View 超过约 600 行按区块拆分；controller 负责能力和状态，View
   负责呈现。窗口入口只做 provider、controller 与路由组合。
10. Storybook 必须渲染真实生产 View，并覆盖 loading、empty、populated、
    error、disabled、长中文、窄窗口和 dark 等状态；不能以简化替身代替生产
    组件。
11. 修改 UI 必须通过 typecheck、lint、unit/axe、Storybook 构建、视觉基线，
    并用稳定签名的真实应用验证 settings、input、popup、OCR 与 onboarding。
12. GPL 项目（Pot、Easydict、Vicinae）的源码、CSS、图标和资产不得进入仓库；
    它们只能用于理解交互。

## 命令速查

- 开发：`pnpm dev:app`
- 稳定签名应用：`pnpm build:local`
- DMG：`pnpm build:dmg`
- 单元与边界门禁：`pnpm test`
- Storybook：`pnpm build-storybook`
- 真实应用截图：`bash scripts/real-app-screenshots.sh [--window …]`
