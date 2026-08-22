# 当前状态与未完成清单

本文以 2026-08-22 的本地工作区为准。状态分为：

- **已建立**：代码路径和自动化测试已经存在，迁移时应复用。
- **部分完成**：能力可运行或有实现，但 UI、分层或双端验收仍不完整。
- **未完成**：首版范围内仍需要实现或验证。
- **延后**：不进入这次 UI 重构首版，不应显示半成品入口。

## 已建立且必须保护

| 能力 | 状态 | 代码位置 | 迁移要求 |
| --- | --- | --- | --- |
| 纯 Dart 翻译用例与 port | 已建立 | `packages/application/lib/src/translation/` | 保留纯 Dart；禁止增加 Flutter、FFI、插件或具体 provider 依赖 |
| Rust 翻译 runtime 和 UniFFI | 已建立 | `crates/`、`packages/runtime/` | UI 重构不得复制或重写翻译引擎 |
| runtime 翻译 adapter | 已建立 | `apps/desktop/flutter/lib/src/data/runtime_translation_repository.dart` | FFI 类型和异常必须停在 adapter 边界 |
| 输入翻译 Riverpod 状态 | 已建立 | `apps/desktop/flutter/lib/src/ui/translation/view_models/translation_view_model.dart` | 可调整 UI state，但不能让 view 直接访问 runtime |
| 输入翻译纯 view | 部分完成 | `apps/desktop/flutter/lib/src/ui/translation/widgets/translation_workspace_view.dart` | 架构可复用，视觉与布局可以完全重做 |
| Material 3 品牌主题 | 部分完成 | `packages/ui_flutter/lib/src/theme/material_theme.dart` | 作为唯一新主题继续完善，并最终接管两个应用 surface 的根主题 |
| go_router 与 ProviderScope | 已建立 | `apps/desktop/flutter/lib/main.dart`、`lib/src/routes/` | 保留路由职责；页面只做组合，不塞业务逻辑 |
| 安全存储引用 | 已建立 | `lib/src/platform/secret_store.dart` | 明文密钥不得进入普通 settings、日志、异常或 UI state |
| 权限实时刷新 | 已建立 | `lib/src/platform/permission_controller.dart` | 保留“激活时、聚焦时、操作前刷新”的语义 |
| 全局快捷键注册 | 已建立 | `lib/src/services/shortcut_service/shortcut_service.dart` | 已包含注销、冲突、无效组合和配置热更新，重做 UI 时复用 |
| 窗口定位和 surface 互斥 | 已建立 | `lib/src/services/app_windows.dart` | 保留靠近鼠标、多显示器工作区约束、工作台/快捷窗不叠加 |

## 仍是旧项目 UI 的范围

以下代码可以提供行为参考，但不应作为新视觉稿或组件 API 的模板：

| Surface | 当前问题 | 必须完成的迁移 |
| --- | --- | --- |
| 工作台外壳与侧栏 | 使用 `Workbench`、`SidebarGroup`、`NavigationItem` 和旧 token；截图仍呈现旧项目布局 | 用 Material 3 重新设计导航、标题区域和内容容器；统一根主题 |
| 欢迎/首次启动页 | 旧 `BrandLogo`、`Button`、自定义 capability card；文案直接写在 Dart 中 | 改成工作台内的首次运行流程，使用本地化资源和可测试状态；不得创建额外原生 onboarding 窗口 |
| 输入翻译页 | 内部已是 Material 3，但仍嵌在旧 toolbar/shell 中；当前布局只是验证稿 | 重新设计完整工作流；保留 view/view model/use case 边界，不要求保留当前 card 布局 |
| 快捷翻译窗 | 单个大型 StatefulWidget 直接读取 settings/runtime/history，并使用旧自定义控件 | 拆成 application 用例、Riverpod view model、纯 Material view 和平台组合层；保留动态高度、失焦关闭、置顶与光标附近定位 |
| 设置外壳 | 旧三栏 rail、旧 toolbar、旧 design token | 重新设计清晰的桌面设置导航；窄窗口不得溢出 |
| 常规/权限设置 | 页面直接监听 `SettingsStore`，使用旧 `PreferenceRow`、`Switch`、`NativeSelect` | 增加 application settings port/view model，再以 Material 组件呈现 |
| 服务与 provider 设置 | 页面直接调用 runtime，弹窗和表单为旧组件 | 保留 Rust 配置与安全存储行为，重做列表、编辑、验证、空白/错误/保存中状态 |
| 快捷键设置 | 注册服务可复用，但录制控件、冲突反馈和页面布局是旧 UI | 新 Material 页面必须显示注册状态、冲突原因、恢复默认和键盘可访问性 |
| About | 旧设置页面 | 用 Material 3 重做；保留版权、第三方声明、版本和 MIT 信息 |

旧 UI 依赖的主要入口包括：

- `apps/desktop/flutter/lib/src/widgets/`
- `packages/ui_flutter/lib/src/widgets/`
- `apps/desktop/flutter/lib/src/theme/app_theme.dart`
- `packages/ui_flutter/lib/src/theme/tokens.dart`
- route 中的 `widgets/ui.dart`、`DesignThemeContext`、`WorkbenchToolbar` 导入

迁移时按页面逐步替换。只有 `rg` 证明不再引用后，才删除旧文件；不得先整目录删除。

## 首版功能完成度

| 功能 | 实现状态 | 验收缺口 |
| --- | --- | --- |
| 输入/粘贴翻译 | 部分完成 | macOS 实际 UI 已跑通到系统翻译调用；系统语言包缺失已映射为可操作错误。新 UI、完整 provider stub integration 和 Windows 验证仍需补齐 |
| 多翻译服务结果 | 部分完成 | application use case 支持并行和流式快照；当前只在 unit/widget/golden 中覆盖，需接入最终 UI 和实际 provider 配置 |
| 托盘与快捷窗开关 | 部分完成 | 行为代码已存在；需要新快捷窗 UI 和 macOS/Windows smoke |
| 全局快捷键 | 部分完成 | 注册、注销、冲突检测、热更新及 unit test 已有；需要新设置 UI 和双端系统 smoke |
| 划词翻译 | 部分完成 | Rust 已在 macOS/Windows 模拟复制、轮询并恢复文本或图片剪贴板；需要操作级错误 UI、恢复失败提示和双端实际验证 |
| 剪贴板输入 | 部分完成 | trigger 已可读取剪贴板并打开快捷窗；需迁移到新 view model 并测试非文本、空内容和隐私提示 |
| 截图 OCR | 部分完成 | Flutter `screen_capturer` 已负责区域框选并交给 Rust OCR；Windows 旧 Rust capture API 仍是 unsupported，因此新代码不得绕回它；缺 Windows 实机、DPI、多显示器和取消流程验证 |
| 权限 | 部分完成 | macOS 辅助功能/录屏权限会在 resume、focus 和操作前刷新；需要新权限页面、授予/撤销/返回设置完整 smoke |
| 服务/provider 配置 | 部分完成 | Rust 设置、安全密钥引用和旧 UI 已存在；新分层、新 UI、校验错误与 loading/empty 状态仍需完成 |
| 快捷窗动态尺寸与定位 | 部分完成 | 代码及位置 unit test 已有；需新 UI 下重新测长文本、多结果、Retina、Windows DPI 和多屏边缘 |
| macOS debug 构建 | 已建立 | 当前本地通过；每个迁移里程碑仍需重跑 |
| Windows debug/release 构建 | 未完成 | 本轮未在 Windows 构建或运行，新翻译页 Windows golden 也未生成 |
| 官方 desktop integration tests | 未完成 | 当前没有覆盖首版闭环的 `integration_test` 套件 |
| 双端人工系统 smoke | 未完成 | 托盘、快捷键、划词、OCR、权限、多屏、失焦和置顶未形成双端签字记录 |

## 已知技术风险

1. `hotkey_manager_macos` 与 `screen_capturer_macos` 当前构建会提示不支持
   Swift Package Manager。现阶段 CocoaPods 构建可通过，但要记录依赖升级或替换计划。
2. 根 `WorkbenchApp`/`MiniTranslatorApp` 仍使用旧 `appThemeData` 和
   `DesignThemeProvider`，输入翻译页只在局部套用新 Material 主题，导致视觉割裂。
3. 快捷翻译窗仍把 runtime、窗口、历史、权限和视觉状态集中在一个 StatefulWidget；
   直接改颜色或尺寸不算迁移完成。
4. 设置页面仍直接调用 `SettingsStore` 或 `runtime.settings()`；它们尚未满足
   `view → view model → use case → port ← adapter`。
5. 当前仓库有大量未提交改动和新增文件。执行者必须基于当前工作区继续，不能假设
   `HEAD` 代表最新成果。
6. `docs/images/workbench.png` 展示旧视觉；完成新 UI 后必须替换，否则 README 会继续
   对外展示错误界面。

## 本轮明确延后的功能

以下功能保持隐藏，不得为了填满导航而重新启用：

- 历史记录 UI（底层 store 可保留）
- 词典、术语库、生词本
- TTS
- 自动更新
- 外部协议调用
- 划词替换
- 旧 Tauri 数据迁移
- 高级 API server 页面
- 聊天式翻译主页或 `translation_chat_dialog`

