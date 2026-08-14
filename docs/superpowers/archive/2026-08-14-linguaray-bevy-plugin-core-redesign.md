> **SUPERSEDED** — 2026-08-14. Canonical kernel spec is now [2026-08-14-linguaray-plugin-core-design.md](../specs/2026-08-14-linguaray-plugin-core-design.md). Archived-on: 2026-08-14 · reason: superseded by linguaray-plugin-core-design.

# LinguaRay — Cordis 启发的 Tauri 插件内核重构设计（rev-2）

**Status:** Draft（待评审）

**Date:** 2026-08-14

**Decision:** 不采用 Bevy 作为应用内核；采用 **Tauri Host Plugins + Rust Capability Supervisor**

**Compatibility:** 本文取代同路径的 rev-1 Bevy 草案；保留文件名仅为维持评审链接稳定。

> 本设计基于对 DeepSeek Harness 官方仓库、其 vendored Cordis v4 源码、官方教程及
> Cordis 论文的直接核验。目标不是复制它的 TypeScript/Agent 形态，而是把真正有价值的
> **可逆副作用、响应式依赖、声明式组合、能力接缝**落实到 LinguaRay 的 Rust/Tauri
> 桌面应用中。

---

## 0. 结论

DeepSeek Harness 值得借鉴，但 rev-1 把它过度简化成了
`Plugin + Resource + Event + DI`，并错误地认为 Bevy 已经提供 Cordis 的核心语义。

官方实现真正依赖的是：

1. 每个插件实例都有独立的 **Fiber 生命周期**；
2. 每项注册都是带逆操作的 **可逆 Effect**，卸载时 LIFO 回收；
3. 插件用 `inject` 声明依赖；服务出现、消失或被替换时，消费者会自动
   `PENDING → ACTIVE → UNLOADING → ACTIVE`；
4. 配置树用稳定 ID 做增量 reconciliation，而不是重启整个应用；
5. 事件分发模式是公共契约，特别是可包装/否决默认行为的 waterfall；
6. Service Definition、Provider、Consumer 三者形成可替换的 capability seam。

Bevy 的 `Plugin` 是构建期配置接口，不能运行时卸载；`Plugin::cleanup` 发生在启动阶段，
不是退出 disposer；Bevy 0.19 的 `App` 是 `!Send + !Sync`；Bevy 的
Message/Event 也不提供 Cordis 的依赖撤回、异步 disposer、waterfall 或配置 reconciliation。
如果在 Bevy 上补齐这些能力，最终仍需自建 Cordis 式运行时，同时额外承担 ECS 调度器成本。

**最终选择：**

- Tauri 继续作为唯一 OS 宿主、事件循环、IPC 和权限系统；
- 用 Tauri 自带 Plugin API 拆分 host-facing 能力（命令、setup、event、on_drop、permissions）；
- 用一个小型 Rust `CapabilitySupervisor` 管理运行时依赖、Fiber 状态和 Effect 回收；
- 业务调用走显式异步 Service trait，不引入第二套 frame/update 调度器；
- 现有功能按 strangler pattern 渐进迁移，R4–R7 不以重构完成为前置条件。

---

## 1. 研究依据与边界

### 1.1 核验的官方材料

- [DeepSeek Harness 官方仓库](https://github.com/deepseek-ai/deepseek-harness)
- [DeepSeek Harness Architecture](https://github.com/deepseek-ai/deepseek-harness/blob/master/docs/architecture.md)
- [Cordis Primer](https://github.com/deepseek-ai/deepseek-harness/blob/master/docs/cordis-primer.md)
- [Lifecycle and Effects](https://github.com/deepseek-ai/deepseek-harness/blob/master/docs/cordis-tutorial/02-lifecycle-and-effects.md)
- [Services and inject](https://github.com/deepseek-ai/deepseek-harness/blob/master/docs/cordis-tutorial/03-services.md)
- [Typed event modes](https://github.com/deepseek-ai/deepseek-harness/blob/master/docs/cordis-tutorial/04-events.md)
- [Composition and HMR](https://github.com/deepseek-ai/deepseek-harness/blob/master/docs/cordis-tutorial/06-composition-and-hmr.md)
- [Cordis v4 source](https://github.com/deepseek-ai/deepseek-harness/tree/master/vendor/cordis/src)
- [A Programming Paradigm for Spatiotemporal Composability](https://github.com/cordiverse/paper)
- [Tauri Plugin Development](https://v2.tauri.app/develop/plugins/)

DeepSeek Harness 当前明确处于 **Developer Preview**，官方提示会发生兼容性破坏。
因此本文借鉴的是可验证的架构语义，不追随其包名、配置格式或不稳定 API。

### 1.2 Cordis 的核心不是普通 DI 容器

Cordis 论文把组合性拆成两个正交维度：

- **Temporal composability：**组件撤销时，它在整个生命周期产生的上下文变更都有逆操作；
- **Spatial composability：**组件声明所需上下文，服务绑定变化后运行时重新判断其依赖是否满足。

源码中的对应物是：

- `Fiber`：一个插件实例及其 `PENDING/LOADING/ACTIVE/UNLOADING/FAILED/DISPOSED` 状态；
- `ctx.effect()`：执行 effect、收集 disposer、逆序且至多一次地恢复；
- `ctx.provide()`：服务注册本身也是 effect；
- `inject`：依赖不满足时不启动，provider 撤回时先停 dependent；
- `fiber.inertia`：一次异步 load/unload 转换完成前不启动另一次转换；
- loader：按 entry stable ID 增量 reconcile 配置树。

这些才是本项目应该移植的部分。

### 1.3 本项目不复制的部分

LinguaRay v1 不需要：

- 从 npm/文件系统加载不可信第三方代码；
- YAML 插件树与 HMR；
- Agent session realm/preset 隔离；
- 任意字符串服务和事件名；
- 为了 ECS 而把历史行、结果卡等短期 UI 数据建成 Entity/Component；
- 在桌面主进程中再运行一套 frame loop。

未来若开放第三方插件，应重新评估进程隔离/WASM，而不是让未知代码进入主进程内核。

---

## 2. LinguaRay 的真实需求

本次重构只服务以下问题：

1. `src-tauri/src/lib.rs` 同时承担 43 个 command、setup、迁移、热键、托盘和领域编排；
2. `Session` / `AppState` 手工聚合基础设施，依赖边界不清晰；
3. keystore/database readiness 改变时，History、Provider、External API 等能力需要一致停用；
4. 热键、托盘、监听器、后台任务的注册与注销分散，容易泄漏或乱序；
5. Provider runtime profile 与协议 driver 混在多条路径中；
6. R4–R6 会继续加入 History/Vocabulary/Dictionary/OCR/TTS/External API/Updater，
   继续堆入 `lib.rs` 会扩大回归面；
7. Tauri 的 command 权限必须继续静态可审计，不能被动态插件绕过。

### 2.1 成功标准

- 新增一个内建能力只需：一个 domain module、可选 Tauri Host Plugin、一个 composition entry；
- 禁用/恢复 capability 能自动撤销/重建副作用；
- required service 消失时，dependent 先停，再撤 provider；
- partial startup failure 会完整回滚；
- 不改变现有 latest-wins、local-sacred、fallback、keystore fail-closed 与权限行为；
- 不强迫所有 command 经过单线程全局锁；
- macOS/Windows 均可编译、测试和逐阶段 ship。

---

## 3. 目标架构

```text
┌─ Tauri Host ─────────────────────────────────────────────────────┐
│ OS event loop · WebView · IPC · capabilities · main-thread APIs │
│                                                                 │
│ Builder                                                          │
│   .plugin(translation::host_plugin())                             │
│   .plugin(providers::host_plugin())                               │
│   .plugin(shortcuts::host_plugin())                               │
│   .plugin(history::host_plugin()) ...                             │
└──────────────────────────┬───────────────────────────────────────┘
                           │ static commands / HostEffect adapters
                           ▼
┌─ CapabilitySupervisor（控制面）──────────────────────────────────┐
│ stable plugin id · dependency graph · Fiber state                │
│ typed service slots · activation epoch · cancellation            │
│ EffectScope(LIFO async disposal) · config reconciliation          │
└──────────────────────────┬───────────────────────────────────────┘
                           │ resolves a typed ServiceLease per call
                           ▼
┌─ Domain services（数据面）───────────────────────────────────────┐
│ Translation · Provider · History · Dictionary · OCR · TTS ...    │
│ async trait methods on Tauri tokio runtime                        │
│ DB/native blocking work remains in spawn_blocking                │
└──────────────────────────┬───────────────────────────────────────┘
                           │ typed facts / explicit interceptors
                           ▼
┌─ Observers / UI effects ─────────────────────────────────────────┐
│ popup · tray · history persistence · telemetry · frontend events │
└──────────────────────────────────────────────────────────────────┘
```

关键点：`CapabilitySupervisor` 只串行化 **生命周期转换**，不串行化所有业务请求。
翻译 HTTP、DB 查询等仍在 Tauri 的 tokio/blocking pool 中并发运行。

---

## 4. 两层插件模型

### 4.1 Tauri Host Plugin

面向 OS/WebView 的模块使用 Tauri 原生 Plugin API：

- `setup`：安装 managed state、启动 host adapter；
- `invoke_handler`：模块自己的静态 command 列表；
- `on_event`：Tauri `RunEvent`；
- `on_drop`：进程 teardown 的幂等兜底；
- plugin permissions：命令默认不可访问，按 capability 文件授权。

每个模块可以导出：

```rust
pub fn host_plugin<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("linguaray-history")
        .invoke_handler(tauri::generate_handler![
            commands::search,
            commands::favorite,
            commands::delete,
            commands::export,
        ])
        .setup(setup)
        .on_drop(on_drop)
        .build()
}
```

这解决 rev-1 中“不可能由运行时 Vec 汇总 `invoke_handler!`”的问题：
commands 仍是编译期宏输入，但由各 Tauri Plugin 自己拥有，并获得命名空间和权限文件。

迁移期保留现有未命名空间的 application command façade；前端与权限逐域迁移后再删除 façade，
禁止一次性改完全部 IPC 名称。

### 4.2 Capability Plugin

领域能力由 supervisor 管理，其接口只描述运行时依赖与激活：

```rust
pub trait CapabilityPlugin: Send + Sync + 'static {
    fn descriptor(&self) -> PluginDescriptor;

    fn activate<'a>(
        &'a self,
        ctx: ActivationContext,
    ) -> futures::future::BoxFuture<'a, Result<(), PluginError>>;
}

pub struct PluginDescriptor {
    pub id: PluginId,
    pub required: &'static [ServiceId],
    pub optional: &'static [ServiceId],
    pub provides: &'static [ServiceId],
}
```

`activate()` 通过 `ctx.provide()` 和 `ctx.install_effect()` 登记输出。它不返回裸线程、监听器
或 OS handle；所有长期副作用必须进入当前 Fiber 的 `EffectScope`。

### 4.3 为什么不是“每个文件都是插件”

插件边界应对应可替换或可独立停用的 capability，不是代码目录：

- `history/crypto.rs` 是 History 内部实现，不是插件；
- `wire.rs` 中协议编码器是 EngineDriver 的实现，不是插件；
- 纯函数不需要生命周期；
- 一个 package 可以同时包含 Service Definition、Provider 或 Consumer 中的一种或多种角色。

---

## 5. Fiber 与响应式依赖

### 5.1 状态机

```text
Disabled ──enable──▶ Pending ──deps ready──▶ Starting ──ok──▶ Active
   ▲                   ▲                         │             │
   │                   │                         └─error──▶ Failed
   │                   │                                       │
   └────disable────────┴──────── Stopping ◀──deps lost/config──┘
```

每个 Fiber 保存：

- stable `PluginId`；
- config fingerprint；
- required/optional service keys；
- provider activation IDs 的 committed snapshot；
- 单调 `ActivationEpoch`；
- cancellation token；
- `EffectScope`；
- 当前 transition future/诊断错误。

### 5.2 依赖规则

1. required services 全部存在才允许 `activate()`；
2. optional service 缺失不阻止启动，只在使用点返回 `None`；
3. provider 被撤回/替换时，先标记其 binding unavailable；
4. 停止所有 dependent，并等待其 disposer 完成；
5. 再停止 provider；
6. 新 provider ACTIVE 后重新激活 dependent；
7. transition 期间的新配置只更新 target，当前 transition 完成后再 reconcile；
8. 不允许两个 lifecycle transition 并发修改同一 Fiber。

这对应 Cordis 的 reactive coeffect，而不是一次性的启动顺序检查。

### 5.3 Readiness 映射

现有状态可映射为服务可用性：

| 当前状态 | Supervisor binding |
|---|---|
| DB 可用 | `DatabaseService` provided |
| DB recovery | binding withdrawn；依赖 DB 的 capability 进入 Pending |
| Keystore healthy | `SecretsService` provided |
| Keystore corrupt/reset | binding withdrawn；History/Provider key/External API 自动停用 |
| HTTP client 构建成功 | `HttpTransport` provided |
| HTTP client 构建失败 | Remote engine providers Pending；local drivers 仍可工作 |

现有 `DataReadiness` 继续作为兼容 façade，直到所有消费者迁移后再删除，不能同时维护两套权威状态。

---

## 6. 可逆 Effect

### 6.1 接口语义

```rust
ctx.install_effect("global shortcut", || async {
    let registration = host.register_shortcut(combo).await?;
    Ok(async move {
        registration.unregister().await
    })
}).await?;
```

必须满足：

- setup 成功后立即登记 disposer；
- 一个 activation 的 disposer 按 LIFO 执行；
- disposer 至多执行一次；
- async setup 中途失败会回滚已经登记的部分；
- unload 会先取消新业务调用，再等待/取消 in-flight jobs，最后释放 effects；
- child capability 的 scope 是 parent effect，parent 停止时递归停止 child；
- cleanup error 被聚合并记录，但不能让剩余 disposer 被跳过。

### 6.2 哪些属于 Effect

属于：

- 全局热键注册；
- Tauri/frontend event listener；
- 托盘菜单/图标控制器；
- popup/window listener；
- background task、timer、watcher、local HTTP listener；
- service publication；
- event/interceptor registration。

不属于：

- 用户显式保存的 Provider、History、Vocabulary 等持久数据；
- schema migration；
- 用户导出的文件。

持久数据有独立事务/恢复契约，不能因为插件被停用就自动删除。

### 6.3 Tauri 退出

动态 disable/recovery 必须等待 supervisor 正常 unload。进程退出采用：

1. 产品自身的 Quit 路径先异步调用 `supervisor.shutdown()`；
2. supervisor 停止接收新 activation/request；
3. 按依赖逆序停 Fiber；
4. Tauri Plugin `on_drop` 只做幂等 best-effort 兜底；
5. 禁止在 tokio runtime 内嵌 `block_on` 等待 cleanup。

操作系统强杀进程不保证 async cleanup；安全性不能依赖退出 disposer。

---

## 7. Service 与调用并发

### 7.1 类型化 Service Slot

Supervisor 内部可以用 `TypeId + stable name + realm` 存异构值，但业务层必须通过
泛型 `ServiceKey<T>` 获取，不能使用任意字符串和 `Any` downcast 散落在业务代码。

```rust
pub static TRANSLATION: ServiceKey<dyn TranslationService> =
    ServiceKey::new("translation");
```

一个 binding 包含 provider Fiber ID 与 ActivationEpoch。消费者激活时提交 binding snapshot；
provider 被同值替换也视为新 binding，避免旧 `Arc` 被误认为仍有效。

### 7.2 控制面与数据面分离

Supervisor actor 只处理：

- enable/disable/config update；
- service provide/withdraw；
- start/stop/reconcile；
- diagnostics/shutdown。

业务请求不经过全局 actor 排队：

1. command 从 `KernelHandle` 获取当前 `ServiceLease<T>`；
2. lease 校验 binding ACTIVE；
3. async trait method 在 Tauri tokio 上运行；
4. DB/原生同步操作继续使用 `spawn_blocking`；
5. 完成提交前同时校验 operation generation 与 activation epoch。

这样既保留响应式依赖，又不把所有翻译和查询串行化。

### 7.3 In-flight 规则

- feature disable：拒绝新 lease，取消可取消任务，等待有状态 mutation 到安全点；
- provider 替换：旧 activation completion 不得写入新 activation 的 UI/runtime state；
- translation latest-wins：继续使用现有 generation token，不能用 plugin epoch 替代；
- DB recovery：继续使用 `data_gate` 保护 archive/reset 与数据访问；Supervisor 不取代数据库锁纪律；
- timeout/cancellation 是每个 Service 方法的契约，不由通用 event bus猜测。

---

## 8. 事件、拦截器与持久事实

DeepSeek Harness 明确区分 durable session events、live agent events 与 capability events。
LinguaRay 也必须区分：

### 8.1 直接能力调用

请求/响应优先用 Service 方法：

- `TranslationService::translate(request)`；
- `HistoryService::search(filter)`；
- `DictionaryService::lookup(word)`；
- `SpeechService::speak(request)`。

不要为了“事件驱动”把每个 RPC 拆成 request/result event。

### 8.2 Typed Signal

已经发生、允许多个观察者响应的事实使用 typed signal：

- `TranslationStarted`；
- `TranslationCompleted`；
- `ProviderChanged`；
- `HistoryChanged`。

Signal 明确：是否 await、listener error 是否聚合、是否允许并行。History persistence、Tray、Popup
可以订阅 translation facts，但业务成功的判定不能因非关键 observer 失败而改变，除非契约明确如此。

### 8.3 Typed Interceptor Chain

只有确实需要“插件可包装/否决默认行为”时才提供 waterfall 等价物，例如：

- translation request policy；
- external API authorization；
- provider fallback policy。

每条 chain 是独立 Rust 类型，显式定义 `next()`、短路值、错误和顺序；不实现一个允许任意字符串事件
混用 `emit/serial/waterfall` 的万能总线。

### 8.4 Durable facts

需要跨重启恢复的事实必须先进入 DB/加密历史，Tauri frontend event 不能作为权威来源。
Popup、Tray 与 UI projection 都可从 durable data 或稳定 runtime snapshot重建。

---

## 9. Provider 与 Engine 的正确模型

rev-1 把 Provider 与 Engine 合并成 `Vec<Arc<dyn Engine>>`，混淆了运行时数据和代码策略。

### 9.1 三层模型

```text
ProviderProfile（DB 数据）
  uuid / name / endpoint / model / enabled / needs_key / version / protocol
                              │ protocol
                              ▼
EngineDriverRegistry（代码策略）
  openai-compatible / anthropic / google-translate / local-xxx
                              │ plan/execute
                              ▼
EngineInvocation（不可变调用快照）
  operation_id / provider_uuid / version / endpoint / model / locality / key lease
```

### 9.2 接口

```rust
pub trait EngineDriver: Send + Sync {
    fn protocol(&self) -> ProtocolKind;
    fn validate(&self, profile: &ProviderProfile) -> Result<(), ConfigError>;
    fn build_request(&self, input: DriverInput<'_>) -> Result<HttpRequestPlan, DriverError>;
    fn parse_response(&self, response: HttpResponse) -> Result<Translation, DriverError>;
}
```

- Provider CRUD 只改变 profile，不重装 plugin；
- protocol driver 由 capability plugin 注册；
- HTTP transport 是独立 Service，统一执行 no-redirect、timeout、limits；
- local engine 可实现专用 async `execute` seam，而不是伪装 HTTP；
- keystore key 只在 invocation 边界短期借用，不进入 registry；
- `TraditionalEngine` 与 `wire::ApiKind` 渐进适配到 Driver，不做一次性改写。

---

## 10. 声明式组合

LinguaRay 的内建组合是编译期可信列表，不需要 YAML：

```rust
fn builtin_capabilities() -> Vec<Arc<dyn CapabilityPlugin>> {
    vec![
        Arc::new(DatabaseCapability::new()),
        Arc::new(KeystoreCapability::new()),
        Arc::new(HttpTransportCapability::new()),
        Arc::new(EngineDriversCapability::new()),
        Arc::new(ProvidersCapability::new()),
        Arc::new(TranslationCapability::new()),
        Arc::new(HistoryCapability::new()),
        Arc::new(DictionaryCapability::new()),
        Arc::new(ShortcutsCapability::new()),
        Arc::new(TrayCapability::new()),
    ]
}
```

Tauri host composition同样是显式列表。增加一行是可审计的，不追求“零修改核心文件”这种表面目标；
真正目标是新增能力不需要修改其他能力的内部实现。

运行时配置只控制 stable plugin entry 的 enabled/config，不加载新 Rust 代码。Reconcile 规则：

- stable ID 不变且 config fingerprint 不变：保留 Fiber；
- config 改变：重启该 Fiber 及因 binding 改变而受影响的 dependent；
- disabled：完整 unload；
- re-enable：依赖满足后重新 activate；
- 平台不支持：状态为 Disabled/Unavailable，不能伪装 Active。

---

## 11. 模块目标

| Capability | Service provider | Consumer / facts | Host effects |
|---|---|---|---|
| Database | `DatabaseService` | Providers/History/Vocab/Dictionary | 无 |
| Keystore | `SecretsService` | Provider keys/History/External API | 无 |
| HTTP | `HttpTransport` | Remote drivers/Anki/Updater | 无 |
| Engine Drivers | `EngineDriverRegistry` | Translation | 无 |
| Providers | `ProviderService` | Translation/Settings | commands |
| Translation | `TranslationService` | Popup/Tray/History | commands, frontend emit |
| Selection | `SelectionService` | Translation | AX/native capture |
| Clipboard | `ClipboardService` | Translation | native clipboard |
| Shortcuts | `ShortcutService` | Translation/OCR/Input | global registrations |
| History | `HistoryService` | History UI/Vocabulary | commands |
| Dictionary | `DictionaryService` | Dictionary UI | commands/file dialog |
| Popup | `PopupService` | translation facts | window/listener |
| Tray | `TrayService` | translation/provider/update facts | menu/icon/timer |
| OCR | `OcrService` | Translation | capture/permission |
| TTS | `SpeechService` | Result actions | audio/native voice |
| External API | `ExternalApiService` | Translation | loopback listener/token |
| Updater | `UpdateService` | Tray/Settings | updater lifecycle |

`settings.rs` 可以是配置 repository，不必强行成为所有配置的万能 Service。

---

## 12. 渐进迁移路线

### K0 — 可行性 Spike（不改生产行为）

在独立分支实现最小 supervisor，仅使用 fake plugins：

- Fiber 状态机；
- required/optional service slot；
- provider withdrawal；
- async LIFO EffectScope；
- config reconcile；
- diagnostics snapshot；
- shutdown。

再建立一个内部 Tauri test plugin，证明：

- plugin 自有 command handler 可用；
-权限默认拒绝、显式 capability 后允许；
- `on_drop` 可作为幂等兜底；
- 不改变现有 application commands。

K0 不迁移 `Session/AppState`，不重命名 IPC，不接真实热键/DB/keystore。

### K1 — `lib.rs` 结构拆分（无 supervisor 行为）

- commands 按领域移动到模块；
- setup helpers 按 host capability 移动；
- `lib.rs` 只保留 application builder 与兼容 façade；
- 所有现有测试数字和权限不变。

### K2 — Supervisor 上线，但只管理测试 capability

- supervisor 成为 Tauri managed state；
- diagnostics 可见；
- production capability 尚不依赖它；
- 完成 macOS/Windows concurrency与shutdown gate。

### K3 — 迁移一个真正有可逆副作用的 capability

优先选择 `Shortcuts`：它已有 revision、rollback、startup restore 和真实 OS registrar 测试。

- global shortcut registrations 全部进入 EffectScope；
- disable/enable/rebind/recovery 行为保持；
- 旧 command façade 与新 plugin command parity；
- 失败可立即回退到现有实现。

### K4 — Readiness 服务化

- Database/Keystore/HttpTransport 改为 typed bindings；
- Provider/History 等按依赖进入 Pending/Active；
- `DataReadiness` 兼容 façade 由新状态投影，禁止双向写；
- recovery 并发矩阵全部通过后才删除旧 ownership。

### K5 — Provider/Engine Driver seam

- 先为现有 `wire.rs`/Google engine 写 adapter；
- profiles 仍来自原 DB；
- parity 覆盖所有 protocol、local-sacred、fallback classification；
- 无行为变化后再删旧 registry。

### K6 — Translation observers 与剩余 capability

- Translation facts 接 Popup/Tray/History；
- 逐个迁移 Dictionary/OCR/TTS/External API/Updater；
- 每个 PR 只迁一个 capability；
- 最后删除兼容 command façade 与巨石 setup。

### 与 R4–R7 的关系

- K0–K2 可与产品路线并行，但不得扩大当前阶段 gate；
- R4–R7 新功能不等待 K6；
- K2 完成后新增 capability 可按新边界编写，但仍需独立产品阶段审核；
- 任何内核迁移失败都不得阻止当前 `main` 按旧路径 ship。

---

## 13. 验证红线

### 13.1 Supervisor 单元/并发测试

- dependent 在 provider 前注册：保持 Pending，provider 出现后启动；
- provider 撤回：dependent disposer 完成后才执行 provider disposer；
- provider 同值替换：activation ID 改变，dependent 重启；
- partial activation failure：已安装 effects 全部逆序回滚；
- disposer panic/error：其他 disposer 仍运行并聚合错误；
- concurrent enable/disable/config update：最终状态等于最后 desired config；
- disable during async activation：旧 activation 不得提交 Active；
- child effects 随 parent 递归回收；
- shutdown 幂等；
- failed capability 不影响不依赖它的 capability。

### 13.2 数据面并发测试

- unload 后新 lease 被拒绝；
- old activation late completion 不写新 activation；
- latest-wins generation 检查点与现状一致；
- DB/data_gate/keystore 锁序不变；
- translation parallel order/fallback 至多一次；
- observer error 不错误地把成功 translation 变失败。

### 13.3 Tauri/权限测试

- 每个 plugin command 默认 deny；
- main/popup/input window capability 隔离；
- application compatibility command wire shape 不变；
- Tauri Plugin `setup/on_event/on_drop` 顺序测试；
- macOS main-thread API 只由 HostEffect adapter 执行；
- Windows target compile；
- 禁止 runtime command registration 绕过 `generate_handler!`/permissions。

### 13.4 行为 parity

必须保留：

- fallback error taxonomy；
- local-sacred；
- latest-wins；
- B5 stable order；
- B6 bounded session fallback；
- provider optimistic lock与consent；
- keystore/DB recovery fail-closed；
- history explicit consent/encryption；
- popup sizing/clamping；
- shortcut revision/rollback。

---

## 14. K0 Go / No-Go Gate

只有全部满足才允许 K1/K2：

1. supervisor 核心不依赖 Bevy或另一套 runtime；
2. 生命周期测试无 skipped/flaky；
3. loom 或等价压力测试未发现重复 dispose/乱序 commit；
4. Tauri test plugin commands/permissions 在 macOS 与 Windows 构建通过；
5. 1000 次 config churn 后 Fiber/effect/task 数回到基线；
6. 代码量、复杂度和诊断能力优于继续手写 `AppState`；
7. K0 对生产 binary、启动时间和 idle RSS 无实质影响；
8. 代码审查明确批准后，才写 K1 实施计划。

No-Go 时保留本设计中的模块拆分与 capability seam 原则，放弃 runtime supervisor；
使用静态 Tauri Plugins + 显式 service traits 仍可解决大部分巨石问题。

---

## 15. 开放决策

1. K0 supervisor 放在 `src-tauri/src/kernel/`，还是独立 workspace crate `crates/kernel/`？
   建议独立 crate，确保不依赖 Tauri，并可做纯 Rust 并发测试。
2. 第一批 runtime enable/disable 只覆盖 Shortcuts/External API，还是所有 capability？
   建议仅覆盖真实有运行时副作用的能力；纯 repository 不必支持热卸载。
3. config reconciliation 是否持久化为通用表？
   v1 建议继续使用现有 typed settings/preferences，不建通用插件 YAML/JSON。
4. 是否需要 realm/isolation？
   v1 不需要；多配置档案或第三方插件出现后另立设计。
5. 第三方插件信任模型？
   本轮不做。未来默认进程隔离/WASM，不允许任意 native dylib 进入主进程。

---

## 16. 审核结论

- **批准方向：** Cordis 的 capability seam、Fiber、reversible effect、reactive dependency、
  declarative reconciliation；Tauri 原生 plugin commands/lifecycle/permissions。
- **拒绝方向：** Bevy 作为第二应用内核、`Arc<Mutex<bevy::App>>`、运行时汇总
  `invoke_handler!`、把 Provider profile 与 Engine driver 合并、一次性迁移全部 AppState。
- **下一步：** 先对本文做架构审核；批准后单独写 K0 TDD 计划。本文不授权开始 K0、
  不修改生产代码、不影响 R4 的 S3 gate。
