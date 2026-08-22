# LinguaRay UI 重构交接包

这组文档用于把当前仓库交给另一个 AI 继续设计和实现。它不是产品官网
文案，也不是最终发行说明。执行者必须先读仓库根目录的 `AGENTS.md`，再按
下面顺序阅读：

1. [`CURRENT_STATE.md`](CURRENT_STATE.md)：当前真实完成度、旧 UI 残留和已知风险。
2. [`UI_REDESIGN_SPEC.md`](UI_REDESIGN_SPEC.md)：全新界面的产品目标、信息架构和状态要求。
3. [`ARCHITECTURE_AND_MIGRATION.md`](ARCHITECTURE_AND_MIGRATION.md)：不可破坏的分层规则和迁移顺序。
4. [`ACCEPTANCE.md`](ACCEPTANCE.md)：可以判定完成的测试与人工验收条件。
5. [`IMPLEMENTATION_PROMPT.md`](IMPLEMENTATION_PROMPT.md)：可直接交给编码 AI 的完整提示词。
6. [`handoff.yaml`](handoff.yaml)：给工具或 AI 快速定位“保留、替换、延后”范围的清单。

## 当前基线的关键事实

- 当前工作区包含尚未提交的 Flutter/Rust 迁移成果。不得 `reset`、`checkout`
  或覆盖这些改动，也不得把未跟踪文件当作可以删除的临时文件。
- 输入翻译的 application/use case、adapter、Riverpod view model 已建立；它们是
  新架构基线，不应退回到 widget 直接调用 Rust、UniFFI 或平台插件。
- 现有可见界面不是设计参考。主窗口外壳、欢迎页、设置页、快捷翻译窗仍来自
  旧项目；当前输入翻译页也只是架构验证版，可以重新设计。
- `docs/images/workbench.png` 和已有 legacy golden 只记录当前状态，不是新设计目标。
- 新界面只以 Flutter 官方 Material 3 为基础，不再叠加另一套 UI 框架，不做聊天主页。
- 日常设计验证通过 Widgetbook、golden 和 `flutter run` 完成，不要求反复安装 release 包。
- 所有操作仅在本地完成；除非仓库所有者再次明确要求，否则不要提交、推送、
  重写历史或改远端。

## 交付方式

先在 Widgetbook 中完成可检查的纯 UI 状态，再接入 Riverpod view model，最后接入
application port 和平台 adapter。每迁移完一个 surface，就删除它对旧设计系统的
依赖并补齐测试；不要一次性删除仍被其他页面使用的旧组件。

