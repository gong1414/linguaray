# Bob 实机查看与 LinguaRay 功能差距

日期：2026-09-05。LinguaRay 源码基线：`47b0f10`。

## 产品形态

用户明确要求：菜单栏常驻，启动不显示窗口、不出现 Dock 图标，按需打开翻译、OCR 和设置。设计重点应放在随叫随用的小窗；设置页用于配置。本轮仅查看和对照，没有修改业务代码或 Bob 配置。

## 查看范围与证据边界

- 通过原生 UI 实际打开正在运行的 Bob，关于页显示 **1.20.0 [255] Pro**。
- 实际查看翻译设置、OCR 设置、通用设置、插件列表、功能增强、集成、关于以及应用菜单；读取设置项与操作说明，没有读取翻译历史、收藏内容或服务密钥。
- 同时查看当前 Flutter 构建的 LinguaRay 翻译设置，并检查当前生产入口、窗口控制、触发器、OCR、翻译编排与集成源码。未使用已退役的原型。
- Bob 的输入快捷键 `Option+A` 在本次自动化操作后没有显示新的可观察窗口；不能据此判定 Bob 快捷键失效。系统菜单栏 UI 获取超时。因此没有完成菜单栏图标操作、全局快捷键及真实翻译请求的端到端验收，也没有独立验证 Bob 在所有窗口状态下的 Dock 行为。
- 下文分别标记「实机设置」与「官方文档」。设置项存在不等于已执行其功能或验证识别、翻译质量。

## 已有能力

LinguaRay 已有以下产品入口和实现，不应重复列为缺失：

- 常驻启动、菜单栏菜单、开机启动设置；划词、截图、输入、剪贴板文本翻译，以及保留内容的显示翻译窗口入口。
- 截图 OCR、静默截图 OCR、文件 OCR、剪贴板图片 OCR、连续追加识别、OCR 自动复制。
- 多服务并行翻译、服务结果切换、语言目标规则、常用语言、复制与双击复制、快捷翻译窗口置顶和失焦隐藏。
- 历史、收藏、术语表、生词本、词典查询和系统朗读。
- 提供商模型自动发现、模型搜索、手动 ID 与所选模型测试。当前目录及前轮实现记录为 41 个预设，其中 26 个为 LLM；不代表已逐个进行真实账户验证。
- 自定义代理、本地 API、URL Scheme；仓库内已有 PopClip、SnipDo、Raycast 集成模板。Raycast 发布物仍为源码包，不能等同于已进入其商店或在本机安装验收。

证据入口：[菜单栏动作](../../apps/desktop/flutter/lib/src/routes/app_tray_controller.dart)、[OCR 控制器](../../apps/desktop/flutter/lib/src/platform/ocr_controller.dart)、[设置模型](../../packages/application/lib/src/settings/models.dart)、[集成文档](../INTEGRATIONS.md)、[模型发现实施记录](2026-09-05-implementation.md)。本页相对链接在下文以仓库根路径补充定位。

## 优先差距

| 优先级 | Bob 的能力或用户要求 | LinguaRay 当前情况 | 建议 |
| --- | --- | --- | --- |
| 第一批 | 用户要求安静常驻、无 Dock | 普通启动隐藏窗口；打开设置会切换到带 Dock 的普通应用策略；发现更新会自动打开设置页 | 明确统一菜单栏应用策略；更新使用菜单提示，避免自动弹窗。保留可达的设置及退出入口 |
| 第一批 | 小窗操作完整；Bob 实机设置有位置、最大高度、输入区状态、字体控制；官方文档支持拖动宽度 | 快捷窗默认宽 720，原生 `isResizable: false`；高度和屏幕边界会自动适配，但缺少位置/高度偏好、折叠原文和字体缩放。OCR 失焦即隐藏且没有钉住入口 | 默认紧凑、宽度可调，双栏阅读作为展开模式；补 OCR 置顶及关闭、重试、收藏、缩放等快捷键 |
| 第一批 | Bob 实机设置：静默划词翻译、静默输入框翻译、点击译文替换原文 | 只有读取选区和展示结果，没有替换原文的动作及目标编辑控件跟踪 | 先做用户点击后的替换，再做静默快捷键；区分原目标、当前焦点、失败回退与撤销 |
| 第一批 | Bob 官方文档：偶尔使用的服务可以钉住，点击才调用 | 一次提交遍历所有启用服务并行调用；结果切换只是选择展示对象。取消订阅未贯通到上游网络请求 | 默认服务与手动调用服务分开；补单服务重试及取消传播，并明确取消不能撤销已完成的推理 |
| 第二批 | Bob 实机设置：剪贴板翻译接受文本或图片；拖图片到菜单栏可触发 OCR | 剪贴板翻译仅读文本；图片走独立 OCR 入口；没有菜单栏图片拖入处理 | 按剪贴板类型分流到文本翻译或图片识别后翻译；增加拖入图片入口 |
| 第二批 | Bob 实机设置有二维码识别；官方文档有多图选取、智能分段 | 文件入口使用单选 `openFile`；连续 OCR 已有，但没有多文件队列、二维码识别开关、段落策略选择 | 先补批量队列和拖拽，再做段落还原及二维码；不能把连续追加误当成批量选图 |
| 第二批 | Bob 实机设置：去换行、去注释符号、修复英文断词，自动复制首个译文/播放原文，按命名格式复制 | 翻译偏好主要为提交方式与双击复制；缺少这些自动化设置与命名复制动作 | 做成明确可选项，保留原始文本；增加驼峰、蛇形等复制方式 |
| 后续专项 | Bob 实机设置与官方文档：原图翻译，译文直接显示在截图中 | 截图转文本后打开普通翻译窗；没有图片覆盖译文的展示流程 | 单独设计图片预览、区域定位、排版与导出；已有 OCR 框坐标类型可复用，但不代表此功能已完成 |
| 后续专项 | Bob 实机插件列表：文本翻译、文本识别、语音合成插件及安装/更新入口 | 提供商目录可以扩展，但没有第三方服务插件的安装、执行与更新机制；系统朗读已有，缺少同类 TTS 服务管理 | 按实际供应商需求决定插件协议；不为增加品牌数量立即引入任意脚本执行 |

## 当前代码定位

- 常驻启动：`apps/desktop/flutter/lib/src/services/app_windows.dart:155`；默认入口 `apps/desktop/flutter/lib/src/routes/app_host.dart:64`。
- 设置导致 Dock 出现：`apps/desktop/flutter/lib/src/services/dock_icon_controller.dart:40`，条件为「设置可见或菜单栏图标关闭」。这是一项明确实现策略，与用户本次要求需要对齐，不能描述为尚未实现常驻。
- 更新自动弹窗：`apps/desktop/flutter/lib/src/routes/app_host.dart:74`，检查结果为 available 时直接打开更新设置。
- 快捷窗口不可拖动调整大小：`apps/desktop/flutter/lib/src/services/app_windows.dart:224`；自动高度与显示器边界：`apps/desktop/flutter/lib/src/ui/quick_translate/quick_translate_window_coordinator.dart:66`。
- 原文/译文布局及现有提交快捷键：`apps/desktop/flutter/lib/src/ui/quick_translate/widgets/quick_translate_view.dart:144`。未找到该窗口针对重试、收藏、置顶、关闭及字体缩放的完整快捷键映射；不能将 macOS 常规菜单项等同于这些跨平台小窗操作已验证。
- OCR 失焦隐藏：`apps/desktop/flutter/lib/src/ui/ocr/ocr_screen.dart:30`；当前视图没有置顶回调。
- 当前选区仅读取：`apps/desktop/flutter/lib/src/platform/selection_controller.dart:13`；全部触发动作见 `apps/desktop/flutter/lib/src/platform/trigger_controller.dart:62`。
- 剪贴板翻译仅读文本：`apps/desktop/flutter/lib/src/platform/trigger_controller.dart:111`。
- 文件 OCR 单选：`apps/desktop/flutter/lib/src/platform/capture_controller.dart:93`；连续追加：`apps/desktop/flutter/lib/src/platform/ocr_controller.dart:134`。
- 全服务并行：`packages/application/lib/src/translation/translate_text.dart:88`；Dart 流未传递上游取消句柄：`apps/desktop/flutter/lib/src/services/llm_stream.dart:26`。
- 原图翻译的现有数据基础：`crates/core/src/model/translation.rs:29` 定义识别框与文本；当前截图翻译流程在 `apps/desktop/flutter/lib/src/platform/trigger_controller.dart` 中转为文本请求。
- 外部集成：`docs/INTEGRATIONS.md`、`integrations/`、`apps/desktop/flutter/lib/src/platform/external_action_controller.dart`。

## 界面调整判断

这是基于用户常驻工具定位的建议，尚未作为实现提交：

1. 日常入口是菜单栏与快捷键，设置页不充当启动首页。
2. 快捷翻译默认窄窗、紧凑工具栏、可折叠原文；长文或比较时再展开双栏。保留明亮、轻盈和不使用绿色系的偏好。
3. 结果旁优先放复制、替换、朗读、收藏；服务使用状态和手动调用入口应就近可见。
4. OCR 保留连续识别，补钉住、多图进度和段落整理；设置可从小窗直达。
5. 设置页的配色、字体和布局继续服务于可读性；本次对照不足以声称任何新方案已经通过用户视觉验收。

## 官方补充来源

- [Bob 翻译指南](https://bobtranslate.com/guide/quickstart/translate.html)：静默替换、原图翻译、按需调用服务、窗口尺寸与内部快捷键。
- [Bob OCR 指南](https://bobtranslate.com/guide/quickstart/ocr.html)：多图选择、连续识别、二维码、段落处理与窗口操作。

本轮没有重新运行测试、修改应用配置或提交代码；交付的是基于实机设置与当前源码的差距清单。
