# 现有功能的架构与目录审查

审查基线：`df3fd44`。本次检查跟踪中的 Flutter、Dart application、Rust runtime/engine、原生宿主、测试和工具目录；不包含忽略的旧原型目录。以下审查正文记录迁移前的问题和建议；原文件路径及行号保留为历史证据。文末记录本次实施进度，现行目录以 `docs/ARCHITECTURE.md` 为准。

**结论**：保留 Flutter + 纯 Dart application + Rust + UniFFI 的总体结构，优先整理应用内部的功能边界、窗口生命周期、设置状态和持久化策略。现有分包足以支持当前产品规模；第一阶段没有必要增加新的 Pub package 或 Rust crate。

已有基础可以继续使用：纯 Dart application 包、数据适配器、共享 Material 主题、单原生宿主、翻译异步事件轮询、统一供应商目录及模型发现、原生凭据存储、更新签名校验和两平台 CI。

对应用内非本地化 Dart 文件的静态 import/export/part 图检查未发现循环依赖。主要问题是职责聚集和部分边界没有落实。生成代码的长度不作为拆分依据。

**1. 优先统一本地文件提交策略**

- `packages/runtime/rust/src/domain/history.rs:314`、`vocabulary.rs:265`：先写临时文件，再删除原文件，最后重命名。删除与重命名之间存在中断窗口。
- `packages/runtime/rust/src/domain/glossary.rs:708`：直接覆盖术语库文件，与历史、生词本的策略不同。
- 建议在当前 runtime crate 中提取 `storage/`，统一序列化、校验、平台适配的替换提交、损坏隔离和恢复策略。领域模块保留各自业务规则，避免抽象成一个承包所有 CRUD 的万能仓库。
- 验收：写入失败、替换失败、中断残留、损坏文件及备份恢复；必须在 Windows 和 macOS 分别测试。先保持 JSON 格式及数据目录兼容，不把数据库迁移混进本轮。
- 这是基于代码的风险判断，本次未注入崩溃来复现数据丢失。

**2. 拆分设置契约和设置缓存的副作用**

- `packages/application/lib/src/settings/ports.dart:3` 的 `WorkspaceSettingsRepository` 有 42 个方法，覆盖偏好、翻译规则、供应商、服务、模型发现、网络、API 服务及关于信息。
- `apps/desktop/flutter/lib/src/ui/settings/view_models/settings_view_model.dart` 同时定义 General、Services、Providers、About 四类 ViewModel。
- `services/settings_store.dart:178` 的读取还承担默认值初始化、登录项应用和回写；`:222` 应用原生外观；`:255` 应用 API 服务设置。文件内多个读取失败路径保留缓存但不暴露错误。
- 应用内直接导入 `settings_store.dart` 的文件有 18 个，导入 `services/runtime.dart` 的有 20 个；它们是值得优先收口的依赖集中点。
- 建议按 Preferences、TranslationPreferences、Provider、Service、Integration 拆窄接口；将纯表单校验移入 application 对应模块。`data/provider_draft_validation.dart` 目前只依赖 application，却被 ViewModel 反向导入。
- 缓存只负责快照、变更订阅和错误状态；登录项、原生外观、API 服务启动由应用生命周期协调器负责。按设置分区通知，避免每个变更都触发全部监听方的工作。
- 现有仓库门面可作为迁移期间的适配入口，完成全部内部调用方迁移后移除。该包 `publish_to: none`，无需机械地安排公开发版弃用周期；仍要核对工作区全部消费者。
- 验收：保存与事件回读顺序、加载失败、快速连续切换、外部设置变更、窗口重开后的状态。

**3. 把桌面工作流与原生能力分开**

- `platform/trigger_controller.dart` 同时做选区读取、截图/OCR、错误分流、窗口切换及取消后的窗口恢复。这是应用编排，不能仅凭位于 `platform/` 就视为平台适配器。
- `services/app_windows.dart` 同时拥有窗口句柄、路由入口、当前界面和展示策略；`ui/quick_translate/quick_translate_window_coordinator.dart` 则拥有原生事件与动态尺寸策略。
- `ui/quick_translate/quick_translate_screen.dart:31` 的 515 行中还包含朗读、词典、生词保存、原文替换、权限监听和窗口交互；`:51` 向界面暴露 `nativeapi.Window`。
- `ui/ocr/ocr_screen.dart:29` 直接订阅原生失焦事件；`:64` 直接设置置顶。`ui/settings/glossary_settings_screen.dart:150` 直接打开文件选择器并处理导入导出。
- 建议把原生窗口、选区、截图、文件对话框等实现放到 `platform/`；将命令分发、恢复策略、窗口状态机归入 `app/`；功能状态和用户操作归入各 feature 的 controller/ViewModel。窗口接口暴露业务所需的操作，避免把原生 Window 对象传回 Widget。
- 保持一个原生宿主和 settings/quick/OCR 的互斥界面。移动文件不改变菜单栏驻留、焦点、自动尺寸和取消行为。
- 验收：截图取消恢复原窗口、失焦关闭、置顶、翻译取消、跨屏工作区约束、460 内容像素及设置/翻译/OCR 往返切换。

**4. 合并手动与后台更新的状态所有权**

- `platform/startup_update_controller.dart:27` 持有 `ValueNotifier<UpdateState?>`，`:66` 自行创建 GitHub repository。
- `ui/updates/updates_screen.dart:17` 又定义 UpdatesViewModel；监听后台结果，并在手动检查后写回后台 notifier。
- 建议一个 UpdateCoordinator 负责检查去重、下载、安装交接和当前状态；生命周期定时器只发送检查命令，页面只观察同一份状态。网络仓库和平台安装器保持分离，签名、哈希校验保持现有顺序。
- 验收：手动检查与自动检查同时发生、下载期间自动检查完成、安装就绪后窗口重开、下载失败重试、退出时资源释放。本次不声称已经复现这些并发场景的缺陷。

**5. Rust 先整理内部职责，再考虑移动 crate**

- `packages/runtime/rust` 目前包含 FFI、运行时状态、业务数据、文件存储、备份和 API 服务。它的职责超过“绑定桥接”，目录说明需要与现实一致。
- `runtime.rs:654` 起的多个 `include!` 是为了保留 UniFFI 导出模块身份。直接改成子模块或挪到新 crate 会影响绑定校验，不能当作普通文件移动。
- 建议先在当前 crate 内形成清晰的领域逻辑、存储和运行时服务模块，导出方法保留原位置并委托内部服务。只有绑定层已经足够薄、确实需要独立复用时，再考虑把实现迁往 `crates/runtime`。
- `crates/core/src/capability.rs:4` 依赖 reqwest，并包含 HTTP 响应分类与脱敏。后续可让 core 保留能力契约和错误模型，把 HTTP 转换放到 engine/transport；这是次于存储与应用编排的工作。
- `crates/engine/src/provider/traditional/system/macos.rs` 为 1448 行，可按 OCR、翻译、检测及系统桥接拆内部模块；不按行数拆生成文件，也不把每个供应商拆成独立 crate。
- 验收：UniFFI 表面校验、完整 codegen、旧设置/备份读取、Rust 默认与无默认 feature 检查、原生二进制和绑定同时交付。

**6. 目录整理采用功能优先，桌面能力集中**

推荐 Flutter 应用内部目标结构：

```text
apps/desktop/flutter/lib/src/
  app/                       # 入口装配、依赖注入、生命周期
    navigation/              # 设置路由和导航模型
    windows/                 # 单宿主、界面切换、尺寸与焦点策略
    commands/                # 快捷键/菜单/API/协议命令的编排
  features/
    translation/             # 快捷翻译、会话、词典/朗读交互
    ocr/                     # OCR 状态、操作和界面
    providers/               # 配置、动态模型发现、连接测试
    services/                # 翻译/OCR/词典服务组合与排序
    library/
      history/
      glossary/
      vocabulary/
    preferences/             # 外观、语言、输入偏好及设置页面
    updates/                 # 更新协调器、界面、网络仓库适配
    integrations/            # 本地 API、外部工具配置页面
    backup/                  # 导入导出流程
  platform/                  # 原生能力契约和各平台实现
    windows/
    capture/
    selection/
    shortcuts/
    permissions/
    credentials/
    speech/
    network/
    files/
  shared/                    # 设置页框架、状态提示、共用标签
  i18n/                      # 保持现有生成入口
  catalog/                   # 仅 Widgetbook 使用的预览和 fixtures
```

每个 feature 按需要放 `screen`、`view_model/controller`、`view/widgets` 和 `data`，不强制创建空的三层目录。跨功能的纯模型、契约、业务用例继续位于 `packages/application`；平台特有窗口协议留在桌面应用。`packages/ui_flutter` 保持主题和通用视觉组件，不接管业务状态。

当前顶层 `apps/desktop/flutter`、`packages/application`、`packages/runtime`、`packages/ui_flutter`、`crates`、`integrations` 和品牌资产根目录可以保留。优先消解 app 内含义重叠的 `services/`、`platform/`、`controllers/`，而不是先重命名顶层。

同一功能的迁移示例：

| 当前分散位置 | 目标职责位置 |
| --- | --- |
| provider model controller、provider settings ViewModel、provider dialog、runtime provider adapter | `features/providers/`，模型和端口在 application/providers |
| `platform/ocr_controller.dart`、`ui/ocr/` | `features/ocr/`；截图和剪贴板实现留 platform |
| `ui/settings/glossary*`、`data/runtime_glossary_repository.dart` | `features/library/glossary/` |
| `platform/startup_update_controller.dart`、`ui/updates/`、`data/github_update_repository.dart` | `features/updates/`；启动调度留 app |
| `routes/app_host.dart`、`app_tray_controller.dart`、部分 app_windows/trigger 逻辑 | `app/` 下的生命周期、命令和窗口协调模块 |

测试逐步对应 feature 目录；168 张基线图集中保留以便两平台审查。脚本目前规模不大，优先保留稳定入口，内部辅助实现可按 codegen/release/checks 分类。文档将现行架构、历史审计和迁移计划分清，旧截图是审计证据，不是产品资产；忽略的旧原型目录不参与本轮目录设计。

macOS 的 `MacAppPresentationPlugin.swift` 同文件包含 Presentation、SystemProxy、Speech、Protocol；Windows 的 `flutter_window.cpp` 也集中注册语音、协议与代理。可按能力拆文件，并保持 channel 名称及消息格式兼容。

**实施顺序与完成标准**

1. 先写入模块依赖规则和迁移清单，补充架构边界检查。现有 reachability 只检查文件可达性与 catalog 泄漏，不会拦截界面直接 import nativeapi/file_selector、ViewModel import data 等跨层关系。建立明确边界及限时例外表，逐阶段清零。
2. 独立处理存储可靠性，再按“供应商与服务 → 偏好设置 → 更新 → 资料库 → 翻译/OCR/窗口编排”迁移；每次移动一个完整功能，不留下两套入口和状态源。
3. 每个阶段把机械文件移动与行为修改分开提交。运行相应纯 Dart/Widget 测试；视觉不变时基线应原样通过，不能通过批量刷新掩盖迁移回归。
4. 窗口或原生能力变化增加两平台集成验证；Rust/FFI 变化运行完整 Rust、UniFFI 和代码生成检查。持续保留从旧安装到新构建的设置与备份兼容验证。
5. 最后再评估 runtime crate 位置、依赖升级、持久化 schema 清理；这几项不是第一阶段目录整理的前置条件。

**2026-09-05 实施进度**

已完成：

- 设置、历史、生词本、术语库和备份导出统一使用 `storage.rs`，避免先删旧文件。新增中途写入失败、替换失败、进程退出、残留临时文件和备份失败保留旧文件测试；现有损坏格式及备份恢复测试继续执行。单文件提交不是跨资料库事务。
- 99 个桌面源文件完成按功能迁移，生产入口、Widgetbook、测试、集成测试及脚本引用同步更新。旧路径和转发门面删除，FFI 与本地化生成文件未修改。
- 设置仓库拆为六个端口，删除原 42 方法接口及 196 行运行时转发门面；服务与供应商适配器分离，四类 ViewModel 和三类系统设置页面各归所属功能。
- 供应商校验和预设选择移入纯 Dart application；展示标签与 Rust 目录映射分离，页面不再导入数据适配器。
- 更新合并为一个驻留协调器；页面和菜单栏共享状态，定时器只发检查命令。测试覆盖重入、并发、下载中检查、页面重开、下载/安装失败重试及销毁后返回。
- 快捷翻译/OCR 的原生窗口操作收进协调器；历史记录通过应用命令打开翻译；术语库文件对话框及读写从 Widget 提取到平台实现与功能控制器。
- 新增依赖边界检查，无迁移例外；CI 与发布验证同时执行。Windows CI 增加完整 runtime 测试，覆盖文件替换语义。

已完成 SettingsStore 的副作用拆分与分区通知：缓存只保留快照、错误和 section Listenable；登录项、原生外观和本地 API 由 `SettingsEffectsCoordinator` 在生命周期中应用。协调器串行合并并发调用，并在系统操作期间设置再次变化时继续应用最新快照；系统拒绝登录项变更时只回写一次真实状态。设置页、托盘、快捷键和主题只订阅相关分区。

已完成资料库 CRUD 状态提取：历史记录、术语库和生词本的加载、筛选、选择和写入都在各自 ViewModel 中；页面只负责展示和确认对话框。持久化仍在 Rust。

已按能力拆分原生宿主：macOS 的 Speech / Protocol / SystemProxy 从 `MacAppPresentationPlugin.swift` 独立成文件；Windows 的语音、协议和系统代理从 `flutter_window.cpp` 抽到对应 host 源文件。channel 名称和消息格式保持兼容。

已把 HTTP 响应分类和脱敏从 `linguaray-core` 挪到 `crates/engine/src/common/http.rs`。core 只保留能力契约和按状态码构造的错误模型，不再依赖 reqwest。

当前目录重整不以移动 crate、改变持久化格式或增加包数量为前提。

本次验证记录见最终交付；历史基线 CI 的通过状态不代替重构后验证。
