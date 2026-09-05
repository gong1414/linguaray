# DeepSeek 真实功能验证

基线：`4aebca9`。日期：2026-09-05。使用用户授权的测试 Key，在原生 macOS Flutter 应用中验证。测试使用独立数据目录，输入 Key 不编译进程序；供应商配置、钥匙串测试条目及临时 Key 文件已清理。

## 实测结果

| 项目 | 结果 | 证据 |
| --- | --- | --- |
| DeepSeek 模型接口 | 通过 | `GET https://api.deepseek.com/v1/models` 返回 HTTP 200 和 3 个 ID |
| 填入 Key 自动发现 | 通过 | 实际供应商编辑窗口显示与接口一致的实时列表，未保存草稿时不写普通配置 |
| 手动刷新 | 通过 | 重新查询成功，查询时间更新 |
| 错误 Key 与恢复 | 通过 | 错误凭据返回认证失败，实时列表清空；恢复正确 Key 后重新获取成功 |
| 保存和重新打开 | 修复后通过 | Key 写入 macOS 登录钥匙串；重新打开编辑器，输入框留空，仍能利用已存凭据获取模型 |
| Flash 英译中 | 修复后通过 | `会议上午9点开始。请带上项目报告。` |
| 切换 Pro 后中译英 | 通过 | 不重启应用、不重填 Key，保存模型修改后返回 `Please submit the test report before Friday.` |
| 模型增删同步 | 通过受控场景 | 同一发现接口先返回 old/kept，刷新后返回 kept/new；新增进入、旧 ID 移除。此项使用本地假凭据，不向本地接口发送真实 Key |
| 删除配置与凭据 | 修复后通过 | 临时供应商从运行时删除，随后读取对应钥匙串条目为 null |
| 软件版本检查 | 未完成升级验证 | 实际更新页面返回检查失败；当前网络的 GitHub 匿名 API 额度耗尽，返回 HTTP 403 |

本次真实返回的模型 ID：

```text
deepseek-v4-flash
deepseek-v4-flash-vision-exp
deepseek-v4-pro
```

这三个 ID 来自该 Key 的实时接口响应。没有把内置目录中的 `deepseek-chat`、`deepseek-reasoner` 当作已发现的可用模型。视觉模型只验证到列表中存在，未执行图像请求。

英文输入：`The meeting starts at 9 a.m. Please bring the project report.`
中文输入：`请在周五前提交测试报告。`

## 发现并修复的问题

### 1. 供应商无法保存真实 Key

实际保存流程失败。检查锁定依赖 `cnativeapi 0.1.4` 的 `secure_storage_macos.mm` 与 `secure_storage_windows.cpp` 后确认，其实现仍为 stub：Set/Remove 返回 false，Get 返回默认值。原来的内存替身测试没有覆盖这个平台实现。

改用 `flutter_secure_storage 10.3.1`，读、写和删除逐层等待完成；macOS 使用登录钥匙串，保留现有应用的沙箱设置，不增加共享访问组。Windows 接入该插件的系统安全存储实现，但 Windows 原生运行尚未验收。选择 10.3.1 是因为 11.x 要求 win32 6，与当前截图依赖的版本范围冲突；未通过覆盖依赖强行安装。

普通配置继续保存不含 Key 的引用。原依赖无法成功保存这些平台的凭据，因此没有可从旧实现迁移的成功写入记录。

### 2. 流式译文显示 JSON 包装

真实模型曾返回并显示 `{"translations":[{"text":"..."}]}`。原因是流式接口复用了要求 JSON 的普通翻译提示词。

增加流式专用纯文本提示词，沿用语言、格式与术语约束；普通结构化翻译接口继续保持 JSON 契约。Rust 的本地 HTTP 测试检查实际发出的提示词；真实中英互译断言译文不再以 JSON 包装开头。

### 3. 凭据删除失败

macOS 登录钥匙串下，插件的批量读取返回 `errSecParam (-50)`。删除供应商改为根据已保存配置中的秘密字段逐项删除，完成后再删除配置，不再批量读取其他供应商的凭据。原生测试验证写入、重新读取及删除后的空值。

## 软件更新现状

通过已登录的 GitHub CLI 只读查询，最新 release 为 [v0.6.0](https://github.com/gong1414/linguaray/releases/tag/v0.6.0)，其资产只有：

- `LinguaRay-macos-unsigned.zip`
- `LinguaRay-windows-x64-unsigned.zip`
- `SHA256SUMS-macos.txt`
- `SHA256SUMS-windows.txt`

当前更新器要求 `LinguaRay-macos.dmg` 或 `LinguaRay-windows-x64.exe`，以及 `SHA256SUMS.txt`，并执行签名验证。当前 release 不满足这一交付格式。没有发布新版本、下载或安装未签名包；不能把此次模型刷新测试说成软件自动升级已经通过。

## 验证与复现

- 原生 DeepSeek 集成测试：通过，包含编辑窗口、实际 API、系统凭据存储、翻译窗口和模型变更。
- Dart/Flutter workspace：190 项通过；安全存储、模型发现及编辑器相关 13 项也单独通过。
- Rust workspace：182 项通过；静态分析、Clippy、依赖验证、格式及 UniFFI 基线检查通过。
- 常规原生 smoke 增加真实系统凭据存储往返测试，避免再次只通过内存替身判定存储可用。
- 日志位于本机 `/tmp/linguaray-deepseek-*.log`；已扫描修改文件、测试日志及独立运行配置，未发现真实 Key。

真实测试默认跳过，只有显式提供 Key 文件路径才运行。不要把 Key 作为 `--dart-define` 的值；该值会编译进入程序。路径须在应用沙箱可读取的位置，运行目录须与用户配置隔离。

```sh
cd apps/desktop/flutter
flutter test integration_test/deepseek_live_test.dart -d macos \
  --dart-define=LINGUARAY_RUNTIME_DATA_DIR=/absolute/isolated/runtime-directory \
  --dart-define=LINGUARAY_DEEPSEEK_KEY_FILE=/absolute/private/key-file
```

运行后删除 Key 文件。测试会删除本轮临时供应商及对应安全存储条目。

协议参考：[DeepSeek Chat Completions](https://api-docs.deepseek.com/api/create-chat-completion/)，存储依赖：[flutter_secure_storage 10.3.1](https://pub.dev/packages/flutter_secure_storage/versions/10.3.1)。
