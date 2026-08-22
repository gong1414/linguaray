# 架构契约与迁移顺序

## 不可破坏的依赖方向

```text
Material 3 View
      ↓ immutable state / user intent
Riverpod ViewModel
      ↓
Pure Dart Use Case (`packages/application`)
      ↓ abstract port
Desktop Adapter (`apps/desktop/flutter/lib/src/data` or `src/platform`)
      ↓
Rust runtime / UniFFI / native plugin
```

### View

- 只接收不可变 model、primitive、localized labels 和 callback。
- 不导入 `linguaray_runtime`、`nativeapi`、`screen_capturer`、`hotkey_manager` 或安全存储。
- 不读取 singleton，不执行 provider 请求，不判断系统权限。
- 可以独立放入 Widgetbook，并使用 fake state 完成所有视觉状态。

### ViewModel

- 使用 Riverpod 3 管理异步状态、request identity、取消/过期结果和 UI intent。
- 只依赖 application use case/port 暴露的纯 model。
- 不把 UniFFI exception、secret 或 provider 原始响应暴露给 view。
- 导航、clipboard copy 等纯 UI effect 由 composition screen 处理，不进入 Rust core。

### Application

- `packages/application` 保持纯 Dart，无 Flutter、FFI、plugin、文件系统或网络实现依赖。
- 定义 settings、permission、shortcut、selection、capture 等新 use case/port 时，遵循现有
  translation package 的模型；不要把 runtime generated type当领域模型。
- 可恢复错误使用稳定 error code，让多语言 UI 映射；错误 message 不作为业务分支条件。

### Adapter 与平台层

- 所有 Rust/UniFFI 结构和异常在 adapter 中映射。
- `nativeapi`、`hotkey_manager`、`screen_capturer` 和 secure storage 只存在于 adapter/
  platform/service 层。
- 权限在应用激活、窗口聚焦和每次受保护操作前读取，不保留永久 startup cache。
- provider secret 只进系统安全存储和 runtime 内存；settings 只存 secret reference。

## 文件级处理规则

### 保留并扩展

- `packages/application/lib/src/translation/`
- `apps/desktop/flutter/lib/src/data/runtime_translation_repository.dart`
- `apps/desktop/flutter/lib/src/ui/translation/view_models/translation_view_model.dart`
- `apps/desktop/flutter/lib/src/platform/permission_controller.dart`
- `apps/desktop/flutter/lib/src/platform/selection_controller.dart`
- `apps/desktop/flutter/lib/src/platform/capture_controller.dart`
- `apps/desktop/flutter/lib/src/platform/secret_store.dart`
- `apps/desktop/flutter/lib/src/services/shortcut_service/shortcut_service.dart`
- `apps/desktop/flutter/lib/src/services/app_windows.dart`
- `packages/ui_flutter/lib/src/theme/material_theme.dart`

### 视觉重写但保留行为

- `apps/desktop/flutter/lib/src/ui/translation/widgets/translation_workspace_view.dart`
- `apps/desktop/flutter/lib/src/ui/translation/translation_screen.dart`
- `apps/desktop/flutter/lib/src/routes/workbench/index.dart`
- `apps/desktop/flutter/lib/src/routes/workbench/welcome.dart`
- `apps/desktop/flutter/lib/src/routes/settings/`
- `apps/desktop/flutter/lib/src/routes/mini_translator/`
- `apps/desktop/flutter/lib/widgetbook.dart`

### 迁移完成后再删除

- `apps/desktop/flutter/lib/src/widgets/` 中不再被引用的旧视觉组件
- `packages/ui_flutter/lib/src/widgets/` 中对 Material 3 的重复实现
- `apps/desktop/flutter/lib/src/theme/app_theme.dart`
- `packages/ui_flutter/lib/src/theme/tokens.dart`、`themes.dart` 及旧 golden
- 旧 route 中的 `DesignThemeProvider`、`DesignThemeContext`、`WorkbenchToolbar` 依赖

删除前必须运行 `rg` 和全量测试证明无引用。不要删除平台行为、runtime model、生成代码、
品牌资产或尚未迁移页面使用的组件。

## 建议实施顺序

### M0：设计目录，不接真实能力

1. 扩展 Material theme，确定颜色、类型、spacing、shape 和平台字体。
2. 在 Widgetbook 建立工作台 shell、输入翻译、快捷翻译、首次运行、设置、provider 编辑、
   快捷键和权限状态。
3. 同时覆盖 light/dark、中文/英文、macOS/Windows viewport。
4. 由仓库所有者确认方向后再替换 production route；设计确认前不大量接线。

### M1：根主题、窗口 chrome 和工作台 shell

1. `WorkbenchApp` 与 `MiniTranslatorApp` 统一使用 `LinguaRayMaterialTheme`。
2. 移除根层对旧 token provider 的依赖。
3. 用 Material navigation 重做 shell，保留 go_router state 和 window action。
4. 明确每个平台唯一 chrome 策略，验证无额外标题框、黑窗或 surface 叠加。

### M2：首次运行与输入翻译

1. 重做 welcome/onboarding，使用 localization 和实时 permission/shortcut/service state。
2. 完成输入翻译最终设计，但复用现有 use case、adapter 和 view model。
3. 扩展 partial failure、streaming、长文本和服务切换状态。
4. 更新 Widgetbook、widget test 和 macOS/Windows golden。

### M3：快捷翻译

1. 从 `MiniTranslatorPage` 抽出纯 application model/use case 和 Riverpod view model。
2. 将 runtime/history/settings/window 调用移到 adapter 或 composition 层。
3. 实现新 Material 快捷窗；复用相同的翻译 application port，不能维护第二套翻译逻辑。
4. 验证动态高度、置顶、失焦关闭、焦点、长结果、光标/托盘/多显示器定位。

### M4：设置、provider、服务、快捷键、权限

1. 为 settings/runtime 操作建立 application port 和稳定 state model。
2. 页面只通过 Riverpod view model 读写。
3. 重做列表、编辑对话框、validation、saving、error、empty 和 conflict 状态。
4. 保留安全存储引用语义，并增加不会泄露 secret 的测试。

### M5：清理和双端验收

1. 删除已无引用的旧设计组件和旧 theme。
2. 替换 `docs/images/workbench.png` 为最终界面截图。
3. 补齐 Windows golden、desktop integration tests 和双端人工 smoke 记录。
4. 运行所有 Dart、Flutter、Rust、格式和构建检查。

## 编码时的硬性注意事项

- 当前工作区有未提交成果：禁止 `git reset --hard`、`git checkout -- .`、`git clean`
  或其他覆盖性操作。
- 不手改 `*.g.dart`、UniFFI Swift/Dart generated files；修改 source 后运行 codegen。
- 不编辑或参考 `.gitignore` 中的旧 React/Tauri 目录。
- 不添加 path override；workspace 包使用正常 version constraint。
- 不改变 bundle ID、安全存储 scope 和全新 `v2` 数据目录语义。
- 不把 deferred feature 重新放入导航。
- 不用真实外部翻译请求驱动 Widgetbook/golden；使用稳定 fixture/stub。
- 未经明确要求不要 commit、push、force-push 或改远端。

