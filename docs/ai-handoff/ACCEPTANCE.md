# UI 重构验收清单

任何“完成”声明都必须同时说明已完成的里程碑、未完成项、测试命令和人工验证平台。
只展示一张截图、只通过 `flutter analyze` 或只在 macOS 能打开窗口，都不算整个任务完成。

## 设计与 UI

- [ ] 工作台、输入翻译、快捷翻译、首次运行、设置、providers、快捷键、权限均为全新
      Material 3 设计，不再呈现旧项目的 sidebar/card/token 视觉。
- [ ] 根应用使用统一 Material 3 theme；production 页面不再通过局部 Theme 修补视觉割裂。
- [ ] 不存在聊天主页、聊天气泡或 AI conversation 信息架构。
- [ ] 不存在额外 onboarding 原生窗口、重复标题框、两套窗口按钮、黑色空宿主或样式未加载。
- [ ] 工作台在 `840 × 560` 及更大尺寸无 overflow；窄布局按规范折叠。
- [ ] 快捷窗宽约 396、动态高度、长结果滚动、失焦关闭和置顶行为正确。
- [ ] 中文/英文、light/dark、macOS/Windows viewport 均有 Widgetbook 状态。
- [ ] loading、empty、success、streaming、partial failure、error、permission、no service 和
      long content 都能离线检查。
- [ ] 所有可见文案来自 i18n source；无关键按钮因长文案截断。
- [ ] 键盘可操作、focus 可见、semantics/tooltip 完整，错误不只靠颜色表达。

## 架构

- [ ] View 无 `linguaray_runtime`、UniFFI generated type 或平台 plugin import。
- [ ] ViewModel 使用 Riverpod，只依赖 application use case/model。
- [ ] `packages/application` 仍是 pure Dart。
- [ ] 快捷窗和工作台共享翻译 use case/port，没有两套 provider 请求逻辑。
- [ ] settings/provider/service 已通过 application port 和 adapter 访问 Rust；page 不直接调用
      `runtime.settings()`。
- [ ] 平台权限、选择、截图、快捷键、窗口和安全存储停在 adapter/platform 层。
- [ ] normal settings、日志、错误、测试 fixture 和 UI state 不含明文 secret。
- [ ] 旧 UI 文件仅在无引用且测试通过后删除。

## 功能自动化

- [ ] application unit tests：语言检测、自动目标、并行/流式 provider、部分失败、过期请求。
- [ ] view model tests：初始加载、提交、取消/过期、copy effect、error mapping、配置变化。
- [ ] widget tests：输入、清空、语言交换、服务切换、retry、权限操作、快捷键冲突、表单校验。
- [ ] golden：每个核心 surface 的 macOS/Windows、light/dark、中文/英文关键状态。
- [ ] desktop integration：托盘、surface 切换、路由、快捷键触发、划词 handoff、OCR fixture、
      权限刷新和窗口边界。
- [ ] 外部翻译使用本地 stub；OCR 使用固定非敏感图片。

## 必跑命令

从仓库根目录运行：

```bash
dart pub get
dart run melos run analyze
dart run melos run test
dart run melos run dependency_validator
python3 scripts/format.py --check
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
git diff --check
```

UI/平台改动还需要在对应系统运行：

```bash
cd apps/desktop/flutter
flutter run -d macos -t lib/widgetbook.dart   # Windows 使用 -d windows
flutter build macos --debug                   # Windows 使用 flutter build windows --debug
```

若改了 i18n 或 runtime interface，先从根目录运行：

```bash
python3 scripts/codegen.py
```

不得手改生成结果来绕过 codegen。

## macOS 与 Windows 人工 smoke

每个平台分别记录通过/失败/未测：

- [ ] 首次启动在同一个工作台窗口内，完成后不再重复出现。
- [ ] 托盘点击和四个首版快捷键行为正确，冲突有明确反馈。
- [ ] 划词翻译获取文本且操作后原剪贴板文本/图片恢复。
- [ ] 剪贴板输入、手动输入和多 provider 翻译可用。
- [ ] 区域截图可取消；成功后 OCR 文本进入快捷翻译；临时图片被清理。
- [ ] 已授权时不提示未授权；撤销后能检测；从系统设置返回后一次刷新生效。
- [ ] 系统语言包缺失、网络失败、服务未配置都有不同的可执行提示。
- [ ] 快捷窗在主屏、副屏、屏幕四边、Retina/Windows DPI 下不越出工作区。
- [ ] 快捷窗非置顶时失焦关闭，置顶时保持；工作台和快捷窗不重叠成两层 surface。
- [ ] 开机启动、菜单栏显示、语言、主题、默认服务和快捷键在重启后持久化。
- [ ] provider secret 保存后普通设置和日志中只有 reference，无明文。

## 完成后的清理判据

```bash
rg -n "DesignThemeProvider|DesignThemeContext|WorkbenchToolbar|widgets/ui.dart" \
  apps/desktop/flutter/lib
rg -n "appThemeData\(|tokensFor\(" apps/desktop/flutter/lib
```

目标是在已迁移 production surface 中为零；若仍有命中，必须逐项说明为何保留。不要为了
追求零命中删除未迁移功能所需代码，也不要把旧词改名后继续保留同一套实现。

