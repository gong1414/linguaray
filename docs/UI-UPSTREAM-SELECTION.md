# LinguaRay 上游界面决定：完整采用 Ant Design X

> 决定日期：2026-08-17。源码审计固定于 `ant-design/x` 提交
> `25aad7b9c13abeb165466d53b375d0f2ffe81fa0`（2.9.0）。

## 结论

LinguaRay 不再“参考 Ant Design X 的外观”，而是采用它的完整产品与研发体系。
旧 LinguaRay、Ueli 和 Fluent 页面结构全部失去约束力；只保留 Rust/Tauri 能力、
IPC、领域模型和 controller。

| 官方模块 | LinguaRay 的原生用途 |
| --- | --- |
| 设计规范 | 按 RICH 与 Hybrid UI 划分唤醒、表达、确认、反馈阶段 |
| React 研发 | React 19、XProvider、AntD 6 token、官方组合方式 |
| X Components | Conversations/Bubble/Sender/Welcome/Prompts/Suggestion/ThoughtChain/Actions |
| X Markdown | 安全渲染翻译正文，跟随 light/dark 官方主题 |
| X SDK | XRequest、自定义 Provider、useXChat、useXConversations |
| X Card | A2UI v0.9 Catalog、命令、数据绑定和 Action 回传 |
| X Skill | 六个 2.9.0 官方技能作为 AI 开发规范 |
| Ultramodern | 输入窗口的信息架构、会话侧栏、内容宽度与底部 Sender |

## Hybrid UI 选择

官方规范不是“所有页面都变成聊天”。LinguaRay 使用两种模式：

- 输入翻译：Chat 为主。用户连续提交文本、查看多服务商回复、切换翻译会话。
- 划词弹窗：Do 为主。结果出现后立即复制、朗读、固定、收藏或重试。
- 设置/权限/OCR：传统 GUI 为主，以明确、可预测的操作完成任务。

这避免了旧方案把翻译当普通表单，也避免把设置页错误包装成对话。

## 运行时数据流

```text
Sender
  │ onRequest
  ▼
useXChat ── conversationKey ── useXConversations
  │
  ▼
TranslationChatProvider (仅三个 transform 方法)
  │
  ▼
XRequest(custom fetch contract)
  │
  ▼
InputController → feature IPC → src/bridge → Rust translation capabilities
  │
  ▼
TranslationState
  │
  ├─ XMarkdown：翻译正文
  └─ XCard A2UI v0.9：结构化结果、状态与 Actions
```

`XRequest.fetch` 在这里是安全适配层，不是浏览器直连服务商。API key 始终留在
Rust keystore/能力层，不进入 React 或请求头。

## A2UI 结果 Surface

LinguaRay 注册本地 Catalog，只允许以下白名单组件：结果栈、翻译结果、翻译进度、
翻译错误。每个 Surface 明确发送：

1. `createSurface`
2. `updateComponents`（扁平结构，根节点 id 为 `root`）
3. `updateDataModel`（服务商、正文、状态、原文）
4. Action 从组件回传 `XCard.Box.onAction`，再调用 controller callback

因此动态内容是结构化数据，不是 AI 生成 HTML，也不会执行任意代码。

## X Skill 使用记录

本次重构实际安装并读取了官方 2.9.0：`x-components`、`x-markdown`、`x-card`、
`x-request`、`x-chat-provider`、`use-x-chat`。项目同时固定
`@ant-design/x-skill` 2.9.0，供后续 AI 开发保持相同规则。

技能直接影响的实现包括：Provider 禁止自写 request、每会话独立实例、
Bubble.List 使用稳定 role、XMarkdown 安全默认值、A2UI v0.9 命令顺序和稳定
Catalog/component map。

## 官方来源

- 设计：<https://x.ant.design/docs/spec/introduce-cn>
- React 研发：<https://x.ant.design/docs/react/introduce-cn>
- 组件：<https://x.ant.design/components/introduce-cn/>
- X Markdown：<https://x.ant.design/x-markdowns/introduce-cn>
- X SDK：<https://x.ant.design/x-sdks/introduce-cn>
- X Card：<https://x.ant.design/x-cards/introduce-cn>
- X Skill：<https://x.ant.design/x-skills/introduce-cn>
- Ultramodern：<https://x.ant.design/docs/playground/ultramodern-cn>
- 源码与 MIT 许可：<https://github.com/ant-design/x>
