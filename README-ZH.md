<div align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/brand/linguaray/dist/readme/linguaray-readme-dark.svg">
    <source media="(prefers-color-scheme: light)" srcset="assets/brand/linguaray/dist/readme/linguaray-readme-light.svg">
    <img alt="LinguaRay" src="assets/brand/linguaray/dist/readme/linguaray-readme-light.svg" width="480">
  </picture>

  <p><strong>在看到文字的地方，直接完成翻译。</strong></p>
  <p>一款使用 Flutter 与 Rust 构建、注重隐私的 macOS / Windows 桌面翻译工具。</p>

  [![CI](https://github.com/gong1414/linguaray/actions/workflows/ci.yml/badge.svg)](https://github.com/gong1414/linguaray/actions/workflows/ci.yml)
  [![License: MIT](https://img.shields.io/badge/license-MIT-0F766E.svg)](LICENSE)
  [![Flutter](https://img.shields.io/badge/Flutter-3.47.1-02569B?logo=flutter&logoColor=white)](https://flutter.dev/)
  [![Platforms](https://img.shields.io/badge/platform-macOS%20%7C%20Windows-475569)](#平台支持)

  [English](README.md) · **简体中文**
</div>

> [!IMPORTANT]
> LinguaRay 目前处于积极开发的预发布阶段。核心流程和桌面构建已纳入 CI，
> 但尚未发布经过签名的稳定版本。如需体验，请先从源码运行。

## 为什么选择 LinguaRay？

翻译应该出现在你正在使用的工作流里，而不是迫使你切换到另一个浏览器标签页。
LinguaRay 启动后常驻菜单栏，不会自动弹出主界面。翻译动作集中在原生菜单与全局
快捷键中；只有偏好设置是持久窗口，需要翻译时才显示紧凑的快捷翻译窗。

- **文字、划词与截图**：翻译输入或粘贴的文本、其他应用中的选中文字，以及通过
  OCR 识别的屏幕区域。
- **符合桌面使用习惯**：支持菜单栏、可配置快捷键、权限恢复、当前显示器定位和
  DPI 感知窗口。
- **开箱即用，也可深度配置**：内置无需密钥的网页翻译服务，同时提供传统 API、
  OpenAI 兼容接口、本地服务、模型发现等固定供应商预设。
- **隐私优先**：凭据保存在操作系统安全存储中；普通设置与 UI 状态只保存密钥引用。
- **界面与内核分离**：Flutter 负责用户体验，Rust runtime 负责翻译、OCR、服务商
  与持久化设置。

## 当前能力

| 能力 | 状态 |
| --- | --- |
| 输入与剪贴板翻译 | 已实现并有自动化测试 |
| 快捷翻译窗与划词翻译 | 已实现；部分平台需要系统权限 |
| 区域截图与系统 OCR | 已实现 macOS 和 Windows 路径 |
| 全局快捷键、菜单栏与窗口定位 | 已实现 |
| 服务商配置与凭据安全存储 | 已实现 |
| 历史记录、收藏、术语库与生词本 | 已实现 |
| 词典查询与文字朗读 | 在操作系统或所选服务支持时可用 |
| 输入行为与常用语言排序 | 已实现 |
| 本地 API 集成与带校验的更新检查 | 已实现 |
| macOS 与 Windows 桌面构建 | 已由 CI 构建和验证 |
| 签名安装包与稳定版本 | 尚未发布 |

Linux 安装包、旧 Tauri 原型的数据迁移，以及自动用译文替换选中原文明确不在当前
范围内。尚未达到功能测试与平台验收标准的入口会保持隐藏，不以半成品对外展示。

## 平台支持

| 平台 | 最低版本 | 构建状态 |
| --- | --- | --- |
| macOS | 13.0 | CI 支持 |
| Windows | Windows 10 | CI 支持 |

目前不支持 Linux。

## 从源码运行

### 环境要求

- [Flutter 3.47.1](https://docs.flutter.dev/install/archive)，包含 Dart 3.13.1
- 当前 stable [Rust 工具链](https://www.rust-lang.org/tools/install)
- macOS：Xcode 与 CocoaPods
- Windows：Visual Studio 的 **使用 C++ 的桌面开发**工作负载

先确认桌面开发环境：

```bash
flutter doctor
```

然后克隆并运行 LinguaRay：

```bash
git clone https://github.com/gong1414/linguaray.git
cd linguaray
dart pub get
cd apps/desktop/flutter
flutter run -d macos        # Windows 使用：flutter run -d windows
```

日常界面开发直接使用 Flutter hot reload。Widgetbook 提供独立的组件与状态目录：

```bash
cd apps/desktop/flutter
flutter run -d macos -t lib/widgetbook.dart
```

## 架构

```text
Flutter 视图
    ↓ 用户意图 / 不可变状态
Riverpod view model
    ↓
纯 Dart 用例与 port
    ↓ adapter
Rust runtime（UniFFI）+ 类型化桌面平台服务
```

| 路径 | 职责 |
| --- | --- |
| `apps/desktop/flutter` | 桌面宿主、路由、view model、adapter 与平台集成 |
| `packages/application` | 纯 Dart 用例、模型与 port |
| `packages/ui_flutter` | LinguaRay 的 Material 3 设计系统与测试工具 |
| `packages/runtime` | Dart、Rust 与 Swift UniFFI 桥接层 |
| `crates` | 翻译引擎、OCR、服务商配置与共享核心逻辑 |

依赖规则、数据流和存储模型见[架构说明](docs/ARCHITECTURE.md)。

## 开发与测试

仓库是由 Melos 管理的 Dart Pub Workspace。常用检查均从仓库根目录运行：

```bash
dart run melos run analyze
dart run melos run test
dart run melos run dependency_validator
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
```

跨层修改应运行完整检查。UI 修改需要补充 Widgetbook 状态并明确更新 golden；桌面行为
需要在受影响的操作系统上验证。完整流程见[贡献指南](CONTRIBUTING.md)。

## 隐私与安全

LinguaRay 只在你主动发起翻译时，才会把内容发送给本次操作所选择的服务商。服务商
密钥通过操作系统安全存储保存。请勿在公开 Issue 中发布 API 密钥、隐私文本或敏感
截图。

发现安全漏洞时，请按照 [SECURITY.md](SECURITY.md) 私下报告。

## 社区

- 使用 [GitHub Discussions](https://github.com/gong1414/linguaray/discussions)
  提问或讨论设计。
- 使用 [Issues](https://github.com/gong1414/linguaray/issues) 提交可复现的 bug
  和范围明确的功能建议。
- 提交 Pull Request 前，请阅读[贡献指南](CONTRIBUTING.md)和
  [行为准则](CODE_OF_CONDUCT.md)。

## 许可证

LinguaRay 采用 [MIT License](LICENSE)。
