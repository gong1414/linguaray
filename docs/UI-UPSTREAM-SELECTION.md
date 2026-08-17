# LinguaRay 上游界面框架决定：Ant Design X

> 决定日期：2026-08-17。官方文档、npm 包和 GitHub 源码均已核对；源码审计
> 固定在 `ant-design/x` 提交 `25aad7b9c13abeb165466d53b375d0f2ffe81fa0`。

## 结论

保留现有 Rust/Tauri 能力层、TypeScript controller 和 IPC 协议，只替换 React
View。生产界面使用以下单一体系：

1. `@ant-design/x`：翻译输入、会话结果、欢迎状态和动作区。
2. `antd`：设置导航、表单、列表、弹窗、反馈与主题 token。
3. `@ant-design/icons`：唯一图标来源。

不再从 Ueli 搬壳，也不再使用 Fluent UI。这样依赖关系、交互语义、主题和版本
由同一上游维护，LinguaRay 只保留桌面窗口布局和业务信息架构。

## 为什么选 Ant Design X

Ant Design X 官方将 AI 界面归纳为 RICH：Role、Intention、Conversation、
Hybrid UI。它与 LinguaRay 的实际结构直接对应：

| LinguaRay 场景 | Ant Design X 组件 | 责任边界 |
| --- | --- | --- |
| 输入翻译 | `Sender` | View 负责输入与提交事件；controller 负责翻译 |
| 单/多服务结果 | `Bubble.List` | View 负责结果语义；provider 并发仍在能力层 |
| 空状态 | `Welcome` / `Prompts` | 只传文案和 callback |
| 结果动作 | `Actions` | 复制、朗读、收藏、固定由 callback 注入 |
| 长任务状态 | `Think` / `ThoughtChain` | 未来有真实状态时再接入，不预造组件 |
| OCR/文件输入 | `Attachments` | 未来接真实 OCR controller；不在 View 读文件系统 |
| 设置中心 | AntD `Layout` / `Menu` / `Form` | 配置读写仍由各 feature controller 完成 |

所选版本为 `@ant-design/x` 2.9.0、`antd` 6.6.1、
`@ant-design/icons` 6.3.2；均为 MIT。X 的 React peer requirement 与项目的
React 19 满足关系，且 X 与 AntD 属于同一组件生态，不需要维护两套主题适配。

## 与旧方案的区别

旧的 Fluent + Ueli 组合仍要求 LinguaRay 自己决定并维护翻译窗口的结构；Ueli
本身又是命令启动器而不是 AI/翻译产品。Ant Design X 直接提供 Sender、Bubble、
Actions 等交互原语，因此此次实现不是“照着截图写 CSS”，而是把真实生产 View
替换成上游组件。

开源翻译项目 Pot/Easydict 仍只用于理解产品流程：两者的 GPL 源码、样式、图标
和资产不得复制。Handy、Maccy 等只校准窗口与权限行为，不成为 UI 依赖。

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
                         │ props / callbacks
                         ▼
Ant Design X views
  Sender / Bubble / Actions + AntD settings shell
```

强制约束：

- View 不导入 `src/bridge`、`@tauri-apps/*` 或 feature IPC。
- `src/ui/x` 只组合上游组件，不实现平行 Button/Input/Dialog/Sender/Bubble。
- 普通窗口使用系统标题栏；无装饰只用于 popup/OCR overlay。
- CSS 只处理窗口结构和少量产品布局，主题由 `XProvider` token 控制。
- 依赖、核心组件使用和边界由 `test/ui-freeze.test.ts` 自动检查。

## 验证方式

日常 UI 在 Storybook/Vite 中预览，不要求测试人员反复安装。合并前依次执行
typecheck、lint、unit/axe、Storybook build；需要验证 Tauri WebView、窗口尺寸、
透明浮层、菜单栏和 macOS 权限时，直接运行构建目录内的 `LinguaRay Dev.app`。
正式 `/Applications/LinguaRay.app` 只在发布候选阶段安装。

## 官方来源

- 文档与组件总览：<https://x.ant.design/index-cn>
- 组件说明：<https://x.ant.design/components/overview-cn/>
- 安装与版本关系：<https://x.ant.design/components/introduce-cn/>
- 源码与 MIT 许可：<https://github.com/ant-design/x>
