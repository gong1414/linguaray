# 菜单栏常驻与快捷翻译：第一批实施

基线：`47b0f10`。依据用户确认的菜单栏常驻、小窗优先、明亮轻盈且不使用绿色系的方向，以及 [Bob 功能对照](2026-09-05-bob-feature-comparison.md)。

## 本批行为

- macOS 的翻译、OCR、设置都维持 accessory 策略。打开设置不再出现 Dock 图标；重新打开正在运行的应用仍能显示设置。
- 启动检查到更新后，只在菜单栏菜单增加更新入口，不主动弹出设置。
- 快捷翻译默认 460 宽，可拖动调宽；高度仍随内容适配并受当前显示器工作区约束。更多菜单可切换 760 宽双栏阅读、折叠原文与缩放字号。标题可以拖动窗口。
- 原文清空后自动恢复输入区。关闭与停止有明确入口；macOS 使用 Command，Windows 使用 Ctrl，支持关闭、重试、收藏、置顶及字号调整。OCR 增加置顶与关闭/置顶快捷键。
- 初始优先选择设置中的默认翻译服务，仅请求当前服务。点击其他服务时才调用；相同原文和语言条件下可切回已完成的结果。按需比较沿用第一次解析出的目标语言；新原文或语言变化不复用旧结果。
- 重试只请求当前服务。停止或关闭未钉住的翻译窗口，会取消当前翻译；已收到的部分文字保留，并显示停止状态。
- macOS 划词时若能取得可写的原选区，完整译文旁显示「替换原文」。点击后重新检查辅助功能权限、原控件、完整原文、选区与捕获时间，直接写回保存的控件并核对结果，不操作剪贴板，也不向当前焦点模拟粘贴。
- 原文/选区变化、目标失效或超过 5 分钟时拒绝替换；不支持写回的应用保留复制流程。只有完整翻译结果允许替换，失败或停止后的部分译文不会显示替换操作。

## 原生验证中发现并修复的崩溃

新增「本地 HTTP → Rust → UniFFI → Dart」流式场景后，发现旧 `StreamCallback` 路径会在真实桌面进程中触发 SIGABRT。崩溃栈包含 `DLRT_GetFfiCallbackMetadata` 与 Rust `StreamCallback::on_chunk`；生成器使用 `Pointer.fromFunction`，而运行时从后台线程调用它。

[Dart 官方文档](https://api.dart.dev/dart-ffi/Pointer/fromFunction.html)明确限制这种回调的调用线程。异步事件接口沿用现有 UniFFI Future 生成机制，其完成通知使用支持跨线程的 [NativeCallable.listener](https://api.dart.dev/dart-ffi/NativeCallable/NativeCallable.listener.html)。没有手改生成文件。

新的生产链路：

```text
当前服务 → Rust start_translation → TranslationTask
                                  ├─ next()：异步读取 TranslationEvent
                                  └─ cancel()：唤醒读取端与请求任务
```

Rust 在等待响应头和接收正文时都能响应取消；阻塞接收工作通过短时接收等待退出。普通翻译服务也通过同一任务接口返回结果。Dart Stream 取消逐层传递到该任务。

公共接口为增量扩展：增加 `start_translation`、`TranslationTask.next/cancel` 和 `TranslationEvent`，保留已有方法的校验值。旧回调方法保留给兼容的原生客户端，Dart 生产代码已经移除其调用。接口源码、Dart/Swift/header 绑定与 `UNIFFI_SURFACE.txt` 一起更新。

## 验证

- Dart/Flutter：190 项通过，包含按需调用、完成结果复用、新原文隔离、停止并保留部分文本、折叠原文、停止按钮、Escape 关闭及窗口布局。
- Rust workspace：182 项通过。本地 HTTP 用例覆盖等待响应头时取消、正文部分返回后取消；两种情况均确认客户端在服务器读超时前关闭连接。
- 静态分析、Clippy、依赖验证、UniFFI 基线检查及格式检查按最终工作区执行。
- Widgetbook 现有预览接入新的窗口操作与替换入口，增加多服务按需调用状态；刷新 macOS、Windows 各 34 张浅色/深色主题图，人工查看紧凑空态、结果和多服务代表图。
- macOS 原生 smoke 覆盖真实中文流式返回、取消待完成的异步事件读取、常驻启动、设置无 Dock、460 宽小窗、手动展开双栏、快捷键注册与全部设置路由。使用独立运行数据目录。
- 修正测试夹具的目录唯一性：生词本测试原来仅使用时间戳，并发测试可能撞目录；增加进程 ID 和原子序号。业务存储逻辑未变。

本地验证日志统一为 `/tmp/linguaray-resident-*.log`。

## 范围与待做事项

- 本批提供用户点击后的原文替换。静默划词翻译、静默输入框翻译尚未接入，不会在后台自动改写用户输入。
- 原文替换目前只接入 macOS，并依赖目标应用暴露可写的辅助功能选区。尚未对微信、浏览器、Word 等常用应用逐一验收；其他平台不显示不可用的替换按钮。
- 不支持写回时仍可复制；不使用模拟粘贴扩大兼容面。未承诺所有应用都能撤销辅助功能写回。
- Windows 主题与共享逻辑已测试，Windows 原生运行仍需在 Windows 验收。原图翻译、多图/二维码 OCR、拖图片到菜单栏、第三方插件及静默替换均不在本批实现中。
- 取消结束客户端翻译任务；已经由提供商处理的内容无法撤回。语言检测仍沿用原有 3 秒等待上限，未新增其独立取消接口。

## 关键位置

- `apps/desktop/flutter/lib/src/services/dock_icon_controller.dart`、`routes/app_host.dart`、`routes/app_tray_controller.dart`：常驻与更新入口。
- `apps/desktop/flutter/lib/src/ui/quick_translate/`、`ui/ocr/`：紧凑布局及窗口操作。
- `apps/desktop/flutter/lib/src/ui/translation/view_models/translation_view_model.dart`、`packages/application/lib/src/translation/translate_text.dart`：单服务调用、结果复用和取消传播。
- `packages/runtime/rust/src/runtime/llm_api.rs`、`apps/desktop/flutter/lib/src/services/llm_stream.dart`：异步事件桥接与请求取消。
- `apps/desktop/flutter/macos/Runner/Plugins/SelectionReplacementPlugin.swift`、`apps/desktop/flutter/lib/src/platform/selection_replacement_controller.dart`：原选区捕获、校验与写回。
