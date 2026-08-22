# 可直接交给其他 AI 的实现提示词

下面从“你正在”开始的内容可以整体复制给另一个编码 AI。不要只复制任务标题；约束、
当前状态和验收条件缺一不可。

---

你正在本地仓库 `/Users/daoyu/Code/projects/islandpot` 中继续 LinguaRay 的
Flutter/Rust 重构。请先完整阅读并遵守：

1. `/Users/daoyu/Code/projects/islandpot/AGENTS.md`
2. `/Users/daoyu/Code/projects/islandpot/docs/ai-handoff/README.md`
3. `/Users/daoyu/Code/projects/islandpot/docs/ai-handoff/CURRENT_STATE.md`
4. `/Users/daoyu/Code/projects/islandpot/docs/ai-handoff/UI_REDESIGN_SPEC.md`
5. `/Users/daoyu/Code/projects/islandpot/docs/ai-handoff/ARCHITECTURE_AND_MIGRATION.md`
6. `/Users/daoyu/Code/projects/islandpot/docs/ai-handoff/ACCEPTANCE.md`
7. `/Users/daoyu/Code/projects/islandpot/docs/ai-handoff/handoff.yaml`

任务目标：继续迁移，但必须把当前仍像旧项目的界面彻底重新设计。不要保留旧工作台、
旧 sidebar、旧欢迎页、旧设置页、旧快捷翻译窗的视觉，也不要把当前 Material 输入翻译
验证稿当成必须照抄的最终设计。把 LinguaRay 设计成一个安静、专业、轻量、隐私优先的
macOS/Windows 桌面翻译工具，而不是聊天应用、网页 dashboard 或旧项目换色版。

技术和架构硬要求：

- 使用固定的 Flutter 3.47.1 / Dart 3.13.1、Flutter 官方 Material 3、Riverpod 3 和
  go_router；不引入第二套 UI 框架、Ant Design、聊天组件库或 WebView UI。
- 依赖方向固定为 `Material view → Riverpod view model → pure Dart application use case
  → port ← adapter → Rust/UniFFI/native plugin`。
- 已完成的 `packages/application` 翻译用例、runtime adapter、translation view model、
  权限/选择/截图/安全存储/快捷键/窗口服务要复用，不得让 widget 重新直接调用 runtime
  或平台 plugin。
- 快捷窗与工作台必须共享翻译 use case/port，不维护第二套翻译实现。
- Provider 密钥只存系统 secure storage；settings、日志、异常、fixture 和 UI state 都不能
  出现明文。
- 权限必须在 app resume、窗口 focus 和每次受保护操作前重新读取；已授权时不能继续提示
  未授权。
- 首次运行必须是工作台内 route，不创建额外原生 onboarding 窗口；工作台和快捷窗同一
  时刻不得叠加两层 surface 或两套标题框。
- 历史、词典、术语库、生词本、TTS、自动更新、外部协议、划词替换、旧数据迁移和高级
  API server 仍然延后并隐藏；不要为了填导航恢复它们。
- 所有文案进入 i18n JSON source，生成文件不手改。
- 当前工作区已有大量未提交迁移成果。禁止 reset、checkout、clean、覆盖未跟踪文件，
  禁止编辑被忽略的旧 React/Tauri leftovers。未经明确要求不要 commit、push 或改远端。

请按以下里程碑实际设计并实现，不要只输出建议：

1. 先核对 `git status` 和现有代码，列出将保留的能力与将替换的视觉层。
2. 先在 Widgetbook 做新的离线纯 UI 目录：工作台 shell、输入翻译、快捷翻译、首次运行、
   设置、provider/service、快捷键和权限。覆盖文档要求的状态、中文/英文、light/dark、
   macOS/Windows viewport。不要初始化 Rust 或发真实网络请求。
3. 完成一套统一的 LinguaRay Material 3 theme，并让 WorkbenchApp/MiniTranslatorApp 根层
   使用它；移除局部新主题嵌在旧主题中的割裂。品牌资产只复用
   `assets/brand/linguaray/`，不要重画 Logo。
4. 用 Material 3 重做窗口 chrome 策略、工作台导航、首次运行和输入翻译 production route。
   输入翻译复用现有 application/adapter/view model；视觉和布局可以完全重做。
5. 将旧 `MiniTranslatorPage` 拆成纯 view、Riverpod view model、application use case/port 和
   adapter/composition 层，再接入新快捷窗 UI。保留动态高度、最大高度、置顶、失焦关闭、
   托盘/光标定位、多屏工作区约束、selection/OCR handoff 和可操作错误。
6. 为 settings/provider/service 增加 application port 和 view model，移除 page 直接访问
   `SettingsStore`/`runtime.settings()`；用 Material 3 重做常规、服务、providers、快捷键、
   权限和 about。保留安全存储、快捷键冲突和配置热更新行为。
7. 每迁移一个 surface 就补 Widgetbook、view model/widget test 和 deliberate golden；完成后
   才删除该 surface 已无引用的旧组件。最后替换 `docs/images/workbench.png`。
8. 运行 `docs/ai-handoff/ACCEPTANCE.md` 中的检查。至少完成本机 macOS 的实际 UI 与 debug
   build，并使用 computer use 实际点击、输入、切换语言、触发翻译和检查窗口行为，不能
   只看测试或静态截图；无法在本机完成的 Windows build、golden 或 system smoke 必须
   明确标为未验证，不得假称通过。

设计必须满足：

- 无聊天主页、无旧米色/深蓝胶囊/荧光绿视觉、无 card 海洋、无大面积渐变或玻璃拟态。
- 工作台在 840×560 无 overflow；快捷窗约 396 宽、动态高度、长结果可滚动。
- empty/loading/streaming/success/multiple providers/partial failure/all failure/no service/
  permission denied/language pack missing/long text 都有可检查状态。
- 错误提示说明发生了什么以及下一步，不能直接展示 Rust exception。
- 完整键盘导航、可见 focus、tooltip、semantics、light/dark 和本地化长文案。
- macOS/Windows 各自只有一种 chrome，不出现额外黑窗、重复边框或两套关闭按钮。

工作方式：保持较小的可验证改动；先设计目录、再 production 接线、最后清理旧 UI。遇到
旧代码时只复用行为，不复用视觉。不要因为任务量大而停在静态 mockup；在安全且范围内的
前提下持续实现到当前里程碑真正通过，并在最终报告中列出文件、截图、测试证据、实际
computer-use 操作结果、未验证平台和剩余工作。

---
