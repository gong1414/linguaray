# LinguaRay 界面强制规则（完整 Ant Design X 体系）

> 2026-08-17 起，旧界面截图、Ueli/Fluent 迁移稿和早期 LinguaRay 布局均不再是
> 设计依据。产品界面以 Ant Design X 2.9.0 官方设计、研发、组件、Markdown、
> SDK、Card、Skill 与 Ultramodern 演示为唯一上游。

## 唯一允许的界面栈

1. `@ant-design/x`：RICH/HUI 语义组件和 `XProvider`。
2. `antd` + `@ant-design/icons`：同体系的通用 GUI 控件和图标。
3. `@ant-design/x-markdown`：所有翻译富文本结果；禁止另造 Markdown 渲染器。
4. `@ant-design/x-sdk`：对话消息、会话、请求和 Provider 状态。
5. `@ant-design/x-card`：A2UI v0.9 动态结果 Surface。
6. `@ant-design/x-skill`：AI 开发指南，版本与运行时保持一致。

禁止重新加入 Fluent、Mantine、Lucide、shadcn、Tailwind UI primitives、Ueli
界面代码或 `packages/ui` 自制控件库。

## 产品模式

1. 输入翻译是官方 Ultramodern **Chat-first 独立工作区**：`Conversations`、
   `Bubble.List`、`Welcome`、`Prompts`、`Suggestion`、`Sender`。
2. 选词弹窗是 **Do-first Hybrid Quick Bar**：保留即时操作，不复制完整聊天页。
3. 设置、权限、服务商配置是 **Do-first GUI**：使用 AntD Layout/Menu/Form/List；
   不得为了“像 AI”而误用 Conversations。
4. loading 必须表达确认阶段的真实步骤；结果必须进入反馈阶段并提供真实动作。
5. 不实现后端不支持的语言选择、深度思考、附件或模型能力开关。

## 数据与能力边界

6. `@tauri-apps/*` 只能出现在 `src/bridge/`。View 不得导入 bridge 或 feature
   IPC；只接收 controller callback。
7. X SDK 的 Provider 只允许实现 `transformParams`、`transformLocalMessage`、
   `transformMessage`。请求统一由 `XRequest` 承载。
8. Tauri IPC 通过 `XRequest.fetch` 适配器接入；API key、认证头和服务商请求仍在
   Rust 能力层，禁止放入 WebView。
9. 多会话必须使用独立 Provider 实例和全局唯一 conversation key。

## A2UI 与 Markdown

10. X Card 新代码只使用 A2UI v0.9。每条命令含 `version: "v0.9"`；先
    `createSurface`，组件是含 `root` 的扁平邻接表，数据由 `updateDataModel` 分离。
11. Catalog 必须在挂载前本地注册；只渲染白名单组件。组件 map 必须稳定。
12. 命令流只能追加。若外部不可变状态整体替换 Surface，必须用新的 React key
    重建 Card，不能让旧消费指针误判。
13. 翻译正文使用 `XMarkdown`，启用 `escapeRawHtml`；外链用
    `openLinksInNewTab`。主题使用官方 light/dark CSS。

## 代码与验证

14. 可见交互不得使用裸 `button/select/textarea`、文本符号、CSS 图形或手写 SVG。
15. 主题由 `XProvider`/Ant token 管理；CSS 只做窗口结构、分栏、滚动、透明层。
16. 普通窗口使用系统标题栏；仅 popup/OCR overlay 允许无装饰窗口。
17. Storybook 渲染生产 View，覆盖 empty/loading/success/partial/error/long/dark。
18. UI 修改必须通过 typecheck、lint、unit、Storybook axe/build、Vite build，并在
    签名的 `LinguaRay Dev.app` 验证真实窗口、焦点、Esc、权限和 IPC。

## X Skill 工作流

开发 AI 界面前必须读取同版本官方技能：

- `x-components`
- `x-markdown`
- `x-card`
- `x-request`
- `x-chat-provider`
- `use-x-chat`

本机注册可运行 `pnpm dlx @ant-design/x-skill@2.9.0`。技能说明不能替代运行时
测试，也不能越过本文件的能力边界。

## 日常命令

- `pnpm storybook`：组件与状态热预览
- `pnpm dev:app`：Tauri WebView 热更新
- `pnpm test && pnpm typecheck && pnpm lint`
- `pnpm build-storybook && pnpm test-storybook`
- `pnpm build:local`：签名开发包/发布前集成验证

不要求测试人员反复安装。绝大多数界面迭代在 Storybook/Vite；系统集成直接运行
构建目录中的 `LinguaRay Dev.app`，不得覆盖 `/Applications/LinguaRay.app`。
