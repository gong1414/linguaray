# LinguaRay：模型自动发现与首轮修复

本轮落实供应商模型自动查询、模型选择界面及审计中确认的翻译问题。本报告记录本轮实现及本地验证结果。

## 用户可见变化

- 打开已有 LLM 配置、选择本地供应商、填入凭证或修改接口地址后，自动查询该配置的模型接口。输入防抖为 700 ms；模型字段不参与查询前置校验，无须先保存配置。
- 切换供应商、凭证或地址会清空旧结果；较早请求的返回不能覆盖当前结果。可手动刷新，刷新成功优先显示接口返回的模型。
- 显示完整、可搜索的模型列表及查询时间，保留 `anthropic/...`、`accounts/fireworks/models/...` 等完整 ID。选中模型和手动填写模型均可保存。
- 离线目录单独标记为“未验证权限”的参考目录。鉴权失败、限流、超时、接口不支持和其他失败分别提示；401/403 不再被伪装成成功获取模型。
- LLM 的“测试所选模型”发送简短测试文本，检查所选模型是否返回文本；不再只测试能否访问模型列表。自动模型发现本身只执行模型查询，不自动执行推理测试。
- 内部配置 ID 移入高级设置；编辑已有配置时以供应商名称为标题，为模型列表留出空间。

## 供应商与接口

目录由 35 个预设增加到 41 个，其中 LLM 预设由 20 个增加到 26 个。新增如下：

| 预设 | 推理协议 | 默认 API 根地址 |
| --- | --- | --- |
| MiniMax | Anthropic 兼容，分离文本和思考块 | `https://api.minimax.io/anthropic` |
| StepFun（Global） | OpenAI Chat Completions 兼容 | `https://api.stepfun.ai/v1` |
| Mistral | OpenAI Chat Completions 兼容 | `https://api.mistral.ai/v1` |
| Together AI | OpenAI Chat Completions 兼容 | `https://api.together.ai/v1` |
| Fireworks AI | OpenAI Chat Completions 兼容 | `https://api.fireworks.ai/inference/v1` |
| 自定义 OpenAI 兼容服务 | OpenAI Chat Completions 兼容 | 用户填写，凭证可选 |

实际模型发现走 Rust 供应商适配器：OpenAI 兼容 `/models`，Anthropic `/v1/models`（包含分页），Ollama `/api/tags`。支持自定义模型列表地址的适配器保留该能力。新增配置不再把推导出的模型列表地址固化到字段；编辑旧配置的 API 根地址时，仅清除与原预设推导地址一致的旧值，保留独立自定义地址。

**接口返回模型列表不等于已经验证推理权限，也不等于列表内每个模型都适合文本翻译。** 供应商可能不开放模型列表接口、返回公共目录，或要求独立的部署 ID。此时可手动填写模型，并点击测试。没有添加自动下载本地模型的行为。

## 修复与实现

| 原审计项 | 本轮结果 |
| --- | --- |
| F01 流式接收阻塞单线程 Tokio | 同步接收移至阻塞线程池，HTTP 读取任务能够运行；本地 HTTP → Runtime → FFI 回调回归通过 |
| F02 离线目录丢失模型命名空间 | 使用供应商模型文件的完整相对路径；采用标准 TOML 解析器，解析上游归档内链接，跳过并报告失效链接 |
| F03 UTF-8 跨网络包乱码 | 三个适配器共用字节缓冲 SSE/NDJSON 解码器，测试逐字节中文、emoji、CRLF 和无末尾换行 |
| F04 末段丢失、截断被当作成功 | 先发送末段文本再报告结束；异常 EOF 和畸形数据报错；保留停止原因与部分文本；界面同时显示部分文本、失败和重试 |
| F05 只显示前 16 个模型 | 完整列表按需构建并支持搜索；测试第 100 个模型的查找和选择 |
| F06 离线目录掩盖接口失败 | 结构化发现结果将实时模型、参考目录、查询时间和错误码分开；推理测试与模型发现分离 |
| F07 地址和异步结果串用 | 跟随当前 API 根地址，查询使用请求代次，忽略过期模型结果；修改配置后也忽略旧推理测试的反馈 |
| F08 显式语言对被检测阻塞 | 显式语言对直接翻译；自动检测最多等待 3 秒后按现有回退规则继续。HTTP 增加连接、读取和总请求时限 |

离线快照仍固定于 models.dev commit `08324a024a9de60e507e08779f6667fbf8a25001`，运行时只解析一次，避免每次查询重复反序列化。生成器要求 Python 3.11+，较旧 Python 可安装 `tomli`。固定来源重新生成的内容与工作区快照逐字节一致。

运行时发现使用临时供应商对象，测试验证不会写入临时凭证或配置。正常保存仍通过系统安全存储保护秘密字段。UniFFI 参数和返回类型保持不变；`test_provider` 继续返回整数，成功值现在统一为 0。修正该方法的文档注释触发 UniFFI 单个方法 checksum 从 `55082` 变为 `38376`，已同步生成 Dart / Swift 绑定及 `UNIFFI_SURFACE.txt` 基线。其余方法校验值未变，新绑定的原生启动已通过 smoke。

## 验证

- Rust workspace：181 项通过（含真实本地 HTTP 模拟的流式接收、分页、自定义模型地址、鉴权脱敏和所选模型调用）。
- Dart / Flutter：172 项通过；四个 workspace 包静态分析通过。
- `cargo clippy --locked --workspace --all-targets -- -D warnings` 通过。
- 依赖验证、统一格式检查、更新后的 UniFFI 公共接口基线检查、`git diff --check` 通过。
- 模型目录生成器：2 项离线测试通过，固定来源重建可复现。
- Widgetbook 增加实时模型与鉴权失败状态；新增 macOS / Windows 主题各浅色、深色图像，共 8 张，已人工查看 macOS 代表图。Windows 主题图像在 macOS 上生成，不代表 Windows 原生构建通过。
- macOS 原生集成 smoke：1 个完整场景通过，覆盖托盘常驻启动、权限刷新、朗读、本地词典、快捷窗口和全部设置导航；使用独立的应用沙箱测试目录。Flutter runner 曾报告自动前台唤起 `open returned 1`，后续应用启动和窗口操作断言均通过。
- 普通 macOS Debug 构建通过，产物：[LinguaRay.app](/Users/daoyu/Code/projects/islandpot/apps/desktop/flutter/build/macos/Build/Products/Debug/LinguaRay.app)。最终产物是普通应用入口，已覆盖集成测试入口。
- 构建仍提示 `hotkey_manager_macos` 和 `screen_capturer_macos` 尚未支持 Swift Package Manager，当前 CocoaPods 回退构建成功。

## 保留的后续工作和验证边界

- 本轮没有使用真实供应商的付费凭证做推理验收；新增供应商端点与协议以官方文档为依据，实际账户模型权限需通过界面测试确认。
- 取消翻译时真正终止所有上游请求仍是后续工作；本轮提供请求超时、检测等待上限和过期结果隔离。不同于真正取消，这些措施不保证立刻停止已发出的请求。
- 原有所有启用翻译服务并行调用的行为保留；按选中服务调用、费用策略和并发限制待独立设计。
- Azure、Bedrock、Vertex、原生 Gemini / Responses 等不同认证和协议尚未新增。当前通用兼容入口不能替代这些专用适配。
- Windows 原生运行、真实系统权限交互、外部供应商权限和配额不在本机测试结论范围内。

## 参考来源

- [Cherry Studio provider registry](https://github.com/CherryHQ/cherry-studio/blob/main/docs/references/provider-model/provider-registry.md)：区分接口返回 ID 与目录元数据，不以第三方目录替代供应商调用结果。
- [MiniMax Anthropic SDK](https://platform.minimax.io/docs/api-reference/text-anthropic-api)：本轮选择该协议以分离思考内容与文本。
- [StepFun models API](https://platform.stepfun.ai/docs/en/api-reference/models/list)，[固定来源的 StepFun Global 配置](https://github.com/anomalyco/models.dev/blob/08324a024a9de60e507e08779f6667fbf8a25001/providers/stepfun-ai/provider.toml)。
- [Mistral models API](https://docs.mistral.ai/api/endpoint/models)、[Together OpenAI compatibility](https://docs.together.ai/docs/inference/openai-compatibility)、[Fireworks OpenAI compatibility](https://docs.fireworks.ai/tools-sdks/openai-compatibility)。
