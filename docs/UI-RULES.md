# LinguaRay 界面强制规则（Ant Design X）

> 2026-08-17 起，Ant Design X 是 LinguaRay 唯一产品界面框架，Ant Design
> 只为它补充通用桌面控件。选择依据与能力边界见
> `docs/UI-UPSTREAM-SELECTION.md`。

## 自动门禁规则

1. **唯一 UI 栈是 Ant Design X。** AI/翻译交互优先使用 `@ant-design/x`；
   通用控件使用 `antd`，图标使用 `@ant-design/icons`。禁止重新加入 Fluent、
   Mantine、Lucide、shadcn、Tailwind UI primitives 或 `packages/ui` 自制组件库。
2. **翻译不是普通表单。** 输入窗口必须以 X `Sender` 为输入核心，翻译结果
   必须以 X `Bubble` 呈现；空状态、快捷建议、附件或思考过程分别优先采用
   X 的 `Welcome`、`Prompts`、`Attachments`、`ThoughtChain` 等现成语义组件。
3. **能力和界面严格分开。** `@tauri-apps/*` 只能出现在 `src/bridge/`；
   `view.tsx` 与 `*View.tsx` 不得导入 bridge 或 feature IPC。View 只接收状态、
   callback 和 controller type；Tauri 调用放在 controller/ipc。
4. **共享层只组合，不造控件。** `src/ui/x/` 可以组合 Ant Design X/AntD 的
   设置壳、Surface 与 ActionBar，但不得复制 Button、Input、Dialog、Menu、
   Bubble 或 Sender 的实现和状态机。
5. **禁止自绘普通窗口标题栏。** main、onboarding、input 使用系统原生标题栏；
   只有翻译 popup 与 OCR 捕获浮层允许 `decorations: false`。
6. **窗口权限最小化。** 每个 Tauri capability 只覆盖对应窗口实际使用的 API；
   设置主窗口不继承无关权限。
7. **Debug/Release 身份稳定且可区分。** 开发包使用 debug bundle id 和稳定
   Apple Development 签名，避免 macOS TCC 因 ad-hoc `cdhash` 改变而误报权限。

## 代码评审规则

8. 主题、颜色、排版、圆角和控件状态以 `XProvider`/Ant Design token 为准；
   CSS 只负责桌面窗口尺寸、分栏、滚动和透明浮层等结构，不建立第二套 token。
9. 可见交互不得用裸 `<button>`、`<select>`、`<textarea>`、文本符号、CSS 图形
   或手写 SVG 模拟。原生文件输入可以隐藏使用，但可见触发器必须是 AntD 控件。
10. 单个 View 超过约 600 行按区块拆分；controller 管能力和状态，View 管呈现，
    窗口入口只做 provider、controller 与路由组合。
11. Storybook 必须渲染真实生产 View，覆盖 loading、empty、populated、error、
    disabled、长中文、窄窗口和 dark；不能用简化替身代替生产组件。
12. 修改 UI 必须通过 typecheck、lint、unit/axe、Storybook 构建和真实开发包
    验证。日常界面开发使用 Storybook/Vite 热更新；系统窗口、菜单栏、快捷键、
    权限再运行构建目录内的 `LinguaRay Dev.app`，不得反复安装或覆盖
    `/Applications/LinguaRay.app`。
13. GPL 项目（Pot、Easydict、Vicinae）的源码、CSS、图标和资产不得进入仓库；
    它们只能用于理解交互。

## 开发与验证

- 组件和状态预览：`pnpm storybook`
- 桌面热更新：`pnpm dev:app`
- 单元与边界门禁：`pnpm test`
- 静态 Storybook：`pnpm build-storybook`
- 系统集成：直接运行 `src-tauri/target/debug/bundle/macos/LinguaRay Dev.app`
- 发布验收：`pnpm build:local`
- DMG：`pnpm build:dmg`

macOS 桌面应用没有 iOS 式 Xcode Simulator。这里的等价分层是：Storybook/Vite
负责绝大多数 UI 迭代，Tauri Dev/开发 `.app` 负责 WebView 与系统集成，只有
发布候选才做安装包验证。
