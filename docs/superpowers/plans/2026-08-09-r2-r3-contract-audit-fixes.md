# R2/R3a Contract Audit Fixes Implementation Plan (rev-23 — fix extract_function_body lifetime E0106 + reconfirm SYNC core/wrapper + 33 tests + exact tooltip + no .bak)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Each task is strict TDD: write the failing test (RED) → implement (GREEN) → commit.

**Load-bearing principle (rev-9):** every Rust function signature, type, and API call in this plan is verified against the current crate; every TSX block is valid TSX against the current imports. There is NO `/* ... */`, NO `// ... existing`, NO pseudocode, NO "re-read and adapt", NO "adjust its signature", NO "verify ... add if missing" instructions. Where the existing component is edited in place (not rewritten), the plan uses precise `diff`-style instructions (rev-8-1 / rev-9-2) — NOT full-file copies with placeholder ellipses. Where a copy-paste code block reproduces a whole function, that function is complete and compilable as written.

**Rev-9 changes** (four P1 corrections over rev-8, each verified against the current source):

- **rev-9-1 (capture_and_translate gate+spawn_blocking — replaces rev-8's non-compiling gate handling):** the rev-8 `capture_and_translate` body wrapped `app_state.data_gate.read()` in a `match ... { Ok(g) => g, Err(_) => ... }`, but `data_gate: parking_lot::RwLock<()>` and `parking_lot::RwLock::read()` returns `RwLockReadGuard` DIRECTLY (NOT a `Result`) — that `match` on `Ok/Err` does not compile. Additionally, holding the gate guard across `run_translate_session(...).await` is a locking antipattern (the guard must be released after the db Arc is cloned, before the async session). **rev-9-1 mirrors the VERIFIED `translate_clipboard` (lib.rs:329-416) pattern exactly:** (1) `client`/`keystore` are acquired from `Session` (not under the gate); (2) the db Arc is acquired via a `spawn_blocking` that takes the `data_gate.read()` guard + `require_ready_gated(&app_arc, &_gate)` inside the blocking closure and returns `Arc<Database>` — the guard is DROPPED when `spawn_blocking` returns (so the async session never holds the gate); (3) the spawn result is matched on the OUTER `Result<Result<Arc<Database>, String>, JoinError>` shape (`Ok(Ok(db))` / `Ok(Err(msg))` / `Err(e)`); (4) `run_translate_session(&db, ...)` takes `&Arc<Database>` (verified fn signature at lib.rs:492-505). `state.gen.is_latest(gen)` is re-checked after `spawn_blocking().await` returns (before the session) and after the session resolves (before the emit). The helper NO LONGER attempts to keep the gate guard alive across `.await`.
- **rev-9-2 (SettingsShell TRUE controlled component — replaces rev-8-1's read-once initializer):** rev-8-1 changed the signal initializer to `createSignal(props.activePage ?? props.initialSection ?? "provider-center")` — but this reads `props.activePage` ONLY at first render; when the parent later passes a new `activePage` (e.g. a `navigate` event arrives), the shell's internal `active()` does NOT update (SolidJS `createSignal` initializer runs once). The tray-action `navigate` Vitest test asserted `data-page` follows the event, which would have FAILED against the real (un-mocked) shell. **rev-9-2 rewrites the active state as a derivation:** `const [internalActive, setInternalActive] = createSignal<SettingsSection>(props.initialSection ?? "provider-center"); const active = () => props.activePage ?? internalActive();` and `handleClick` calls `setInternalActive(id)` ONLY when `props.activePage === undefined` (uncontrolled mode) before `props.onNavigate?.(id)`. The diff-instruction edits 2 + 3 in A4 Step 7 are updated accordingly; the shell now passes the controlled-component test `activePage` changes sync `data-page` + sidebar highlight (added to `test/SettingsShell.test.tsx` in C5 Step 1).
- **rev-9-3 (refresh_tray returns `tauri::Result<()>` — replaces rev-8's `()` + contradiction):** the rev-8 `refresh_tray` was declared `pub fn refresh_tray(app: &tauri::AppHandle)` returning `()` (the body logged internally), but rev-8 Step 9b's `refresh_tray_if_available` wrote `if let Err(e) = refresh_tray(app)` — `()` has no `Err`, that line does not compile. rev-8 Step 9b even hedged with "adjust its signature to return `tauri::Result<()>` if it currently returns `()`" (a non-precise instruction — banned by rev-9's load-bearing principle). **rev-9-3 fixes the signature at the source:** `pub fn refresh_tray(app: &tauri::AppHandle) -> tauri::Result<()>` returns `Ok(())` on the `Some(tray)` happy path (after `tray.set_menu` + `tray.set_tooltip`) and returns `build_tray(app)` on the `None`-tray (first-build) branch (`build_tray` itself returns `tauri::Result<()>`, so this composes). `refresh_tray_if_available(app)` is `fn` returning `()`, calling `if let Err(e) = refresh_tray(app) { log::warn!(...) }` — compiles as written. The 8 provider mutation commands call `refresh_tray_if_available(&app_handle)` on their success path.
- **rev-9-4 (frozen design scope governance — replaces rev-8's "scope-reduced" framing):** the plan body framed Surface 04 as "REDUCED, fully-executable" + "Surface 04 scope-reduced" titles, which read as a design downgrade against the FROZEN pages/04 contract. **rev-9-4 reframes WITHOUT changing the frozen docs:** (a) the "Surface 04 scope-reduced" title in the A4 heading is REMOVED. (b) All non-precise instructions ("adjust its signature", "verify ... add if missing", "or wherever nav() is matched") are DELETED and replaced with exact diffs. **rev-10 supersedes the rev-9-4 narrative claims** (the "SAME surface", "wired but dormant", and "NOT a design downgrade" assertions): rev-10 replaces them with a per-state Surface status implementation table and a "Range decisions (pending user approval)" section that gives each unimplemented state an explicit A/B option. The frozen pages/04 red-dot requirement is NOT met this stage and is NOT modified — implementing or deferring it requires user approval.

**rev-11 changes (user-approved scope resolution — 3 A-paths implemented this stage, 2 B-paths deferred):**

- **rev-11-1 (Tray Error red-dot + Active pulse — user-approved A-path, NEW Task A5).** rev-10 left the Tray Error red-dot, Active pulse, and Update badge ALL unimplemented under "pending user approval". **rev-11 resolves the approval:** Error red-dot (A-path) and Active-translation pulse (A-path) are BOTH implemented this stage via a pure-Rust tray visual-state controller (`set_tray_visual_state`, new Task A5); Update badge (B-path) remains deferred to R5/R6 per user approval. The controller is a priority state machine `Error > Update > Active > Normal` living entirely in `src-tauri/` (NOT routed through the Web frontend — the translate/clipboard/command flows call it directly). The Error red-dot icon is generated PROGRAMMATICALLY at build time (a `build.rs` build-script + `image` build-dependency writes a 32×32 solid-red PNG into `OUT_DIR`, embedded at compile time via `include_bytes!(env!("OUT_DIR"))` — NO external PNG file path assumption, NO design-asset dependency). `TrayIcon::set_icon(&self, icon: Option<Image<'_>>) -> crate::Result<()>` and `TrayIcon::set_tooltip<S: AsRef<str>>(&self, tooltip: Option<S>) -> crate::Result<()>` are the verified Tauri 2 tray APIs; `Image::from_bytes(bytes: &[u8]) -> crate::Result<Image>` decodes the embedded PNG. Task A4's `capture_and_translate` and `translate_clipboard` are extended (rev-11-3, "modify Task A4") to call `set_tray_visual_state(app, ActiveTranslation)` on start and `set_tray_visual_state(app, Normal|Error)` on completion/failure. The `TrayVisualState::UpdateAvailable` arm is RETAINED in the enum (so the priority ordering is testable) but is never called this stage — annotated "never called this stage; deferred to R5/R6 per user-approved scope decision".
- **rev-11-2 (Connection latency — user-approved A-path, MODIFY Task C3c).** rev-10/P1-8 froze `provider_test_connection` at `{ ok, message }` with NO `latency_ms`. **rev-11 resolves the approval:** latency is now implemented this stage (A-path). `ConnectionResult` (lib.rs:1449) gains `latency_ms: Option<u32>`; `provider_test_connection` (lib.rs:1507) wraps the HTTP GET probe in `Instant::now()`/`elapsed()` timing (only set on the reachable/Ok path; `None` on early-return failures). The frontend (C3c) renders `{message} · {latency}ms` when `latency_ms` is present. The Global Constraints "No invented backend contracts (P1-8)" line and the P1-8 summary line are amended: `latency_ms` is now a real additive field (the "no latency" wording is superseded); `provider_get_balance` STILL does not exist and the Balance UI is STILL a static placeholder (B-path deferred to R4/S3).
- **rev-11-3 (Modify Task A4 for tray pulse wiring).** A4's `capture_and_translate` and `translate_clipboard` rewrite (already in the plan) is extended: on entry call `set_tray_visual_state(&app, TrayVisualState::ActiveTranslation)`, and on every terminal branch (`popup::result`/`popup::multi_result`/`popup::error`/early `return Ok(())`) call `set_tray_visual_state(&app, TrayVisualState::Normal)` (success/no-op) or `set_tray_visual_state(&app, TrayVisualState::Error)` (error). A4 Step 9's switch-provider failure path ALSO sets `Error` then reverts to `Normal` after a short delay is NOT introduced — the switch-failure tooltip stays, and A4 calls `set_tray_visual_state(&app, TrayVisualState::Error)` on switch failure and `Normal` on switch success so the red dot surfaces the failure.
- **rev-11-4 (B-paths explicitly deferred).** Update badge and Balance states are recorded as "deferred per user-approved scope decision" (Update badge → R5/R6; Balance states → R4/S3). The "Range decisions (待用户批准)" section is replaced by a "审核快照（rev-11 用户已批准）" table; the per-state implementation matrix now reflects the A/B resolutions. The phrase "待用户批准" is removed everywhere it appeared; "pending user approval" in A4/D3 is replaced with "per user-approved scope decision (rev-11)".
- **rev-11-5 (Scope-goal wording fix).** The Goal line "Close every verified contract gap" is amended to "Close A-path contract gaps this stage; B-path items (Update badge, Balance states) are deferred per user-approved scope decision". The P1-5 summary line "Tray fully executable" is amended to "Tray Normal/Active/Error states implemented; Update badge deferred". The Task A4 heading "Surface 04 fully executable, tray fully executable" is amended to "Surface 04 Normal/Active/Error executable; Update badge deferred".

**rev-12 changes (定点修订 — A5 Tray 状态机 4 个 P1 + 3 个 P2, 全部针对 Task A5; rev-9-1/2/3 与其他任务保持不变):**

- **rev-12-1 (P1-1 Active pulse 必须真实图标动画 — 替换 rev-11 的 "仅 tooltip 变化").** rev-11 的 `TrayVisualState::ActiveTranslation` arm 注释明确写着 "the icon is left at the app default" + "A live icon pulse animation is out of scope (platform-specific drawing)"，并且只调用 `tray.set_tooltip(Some("Translating…"))`。这并未真正满足 frozen pages/04 的 "Active translation (pulse)" 契约（pulse 意为可见的脉动，不仅是 tooltip 文字）。**rev-12 实现真正的 icon 帧切换脉动：** build.rs（Step 4）现在生成**两个** tray PNG — `tray-error-32.png`（红点 overlay，见 rev-12-2）和 `tray-active-32.png`（app 默认图标的 dimmed 变体，整体亮度降到 ~60%）。`TrayStateController`（见 rev-12-3）在进入 `ActiveTranslation` 时启动一个后台 timer（`tokio::task::spawn` + `tokio::time::interval`，每 800ms），循环调用 `tray.set_icon(normal)` → `tray.set_icon(dimmed)` → …；离开 `ActiveTranslation`（最后一个翻译完成）时 `timer_handle.abort()` 并恢复 `normal` 图标。timer handle 存储在 `TrayStateController::pulse_timer: Option<tokio::task::JoinHandle<()>>`。Tauri tray 在 macOS 上不支持 opacity 动画，因此脉动必须是真实 icon 字节切换（已核实：`TrayIcon::set_icon(&self, icon: Option<Image<'_>>)` 接受新 `Image` 并立即替换）。tooltip "Translating…" 仍设置（作为辅助信号），但脉动的**可见**信号是图标本身的 dimmed/normal 切换。
- **rev-12-2 (P1-2 红点 overlay 必须叠加在底图上 — 替换 rev-11 的 32×32 纯红方块).** rev-11 的 `build_tray_error_icon` 用 `for px in img.pixels_mut() { *px = Rgba(red); }` 把整张 32×32 全填红色，生成的是一个纯红方块而非 "red-dot overlay"。这显然不是 pages/04 "red-dot on icon" 的意图（红点应叠加在 app 图标上）。**rev-12 改为真正的 overlay 合成：** build.rs (1) `image::open("src-tauri/icons/32x32.png")` 加载 app 默认底图（已核实存在于仓库 `src-tauri/icons/32x32.png`，974 bytes）；(2) 在右上角画一个小圆点（直径 ~10px，圆心约 (26, 6)，颜色 `#DC2626` = `[220, 38, 38, 255]` — 冻结 danger 色，**不是** rev-11 的 `#E5484D`/`(229,72,77)`）；(3) 使用 `image::RgbaImage` + 手动 `put_pixel` 循环绘制圆点（不依赖额外 drawing crate，圆点用 `dx*dx + dy*dy <= r*r` 判定）；(4) `img.save(out_dir.join("tray-error-32.png"))`。红点颜色 `#DC2626` 是本修订的硬性约束（用户指定）。Step 4 的测试断言：加载生成的 PNG，验证大部分像素与底图一致（只有右上角 badge 区域 ~10px 圆内变化）。
- **rev-12-3 (P1-3 状态控制器改为真正的 reducer — 替换 rev-11 的直接覆盖 `set_tray_visual_state`).** rev-11 的 `set_tray_visual_state(app, state)` 是一个直接覆盖函数：它不读当前状态、不比较优先级、不知道有几个并发翻译。这导致两个并发翻译先后完成时，第一个完成会把状态拉回 `Normal`，而第二个仍在翻译中（视觉上 pulse 消失）；并且 `Error` 会被随后的 `Normal`（成功路径）覆盖。**rev-12 引入 `TrayStateController` reducer：** `pub struct TrayStateController { active_translations: u32, has_error: bool, pulse_timer: Option<tokio::task::JoinHandle<()>> }`，方法 `begin_translation(&mut self, app)` → `active_translations += 1` → `recompute(app)`；`end_translation(&mut self, app)` → `active_translations = active_translations.saturating_sub(1)` → `recompute(app)`；`set_error(&mut self, app, has_error: bool)` → 更新 flag → `recompute(app)`；`recompute(&self, app)` 从状态集计算最高优先级 → `if has_error { Error } else if active_translations > 0 { ActiveTranslation } else { Normal }`（`UpdateAvailable` 永不在此阶段激活，延期至 R5/R6）。pulse timer 在 `recompute` 进入 `ActiveTranslation` 时启动（若尚未运行），离开时 `abort()` 并恢复 normal 图标。控制器存储在 `Arc<tokio::sync::Mutex<TrayStateController>>`，挂在 `AppState`（或 `Session`）上；`begin_translation`/`end_translation`/`set_error` 通过 `#[tauri::command]` 包装或直接在 `capture_and_translate`/`translate_clipboard`/switch-provider 中 `controller.lock().await.<method>(app)` 调用。Step 2 的测试新增并发场景：两个翻译先后完成 → 最后才回 `Normal`；`Error` 不被 `end_translation` 清除；`Error` 清除后恢复到 `Active`（若仍有翻译）或 `Normal`；pulse timer 在离开 `Active` 后停止。
- **rev-12-4 (P1-4 模块可见性修正 — `pub mod tray_state`).** rev-11 的 Step 6 用 `mod tray_state;`（私有）+ `pub use tray_state::{...}` 重导出，但 Step 2 的集成测试写的是 `use linguaray_lib::tray_state::{tray_state_priority, TrayVisualState};`（按模块路径导入），私有模块下该路径不可见，测试无法编译。**rev-12 改为 `pub mod tray_state;`**（最简单、与测试路径一致），保留 `pub use tray_state::{...}` 作为 crate-root 便捷重导出（二者共存，不冲突）。rev-12-3 新增的 `TrayStateController` 也在重导出列表中。
- **rev-12-5 (P2 latency 测试验证实际 Instant 结果 + 饱和转换).** rev-11 的 `connection_latency.rs` 只断言 `ConnectionResult` 序列化 shape（`latency_ms` 字段存在），未验证 `provider_test_connection` 真的用 `Instant` 计时；且 `as_millis() as u32` 在 >49 天的 `Duration` 下会截断（虽然实际不会发生，但 clippy `cast_possible_truncation` 可能告警）。**rev-12：** (a) `connection_latency.rs` 新增一个测试，构造一个 mock/探测路径验证 `latency_ms` 反映实际 `Instant::now()`/`elapsed()` 结果（而非硬编码常量）；(b) C3c Step 3 的 `start.elapsed().as_millis() as u32` 改为 `u32::try_from(start.elapsed().as_millis()).unwrap_or(u32::MAX)`（饱和转换，clippy-clean）。
- **rev-12-6 (P2 tray 状态文字本地化).** rev-11 的 `set_tray_visual_state` 硬编码 tooltip 字符串（`"LinguaRay"`、`"Translating…"`、`"LinguaRay — Error"`），未走本地化。**rev-12：** tooltip 文字从 settings 读取 `locale`（`"en" | "zh"`），通过一个 `tray_tooltip_text(state, locale) -> &'static str` 查表函数返回本地化字符串（en: "Translating…" / zh: "翻译中…"；en: "LinguaRay — Error" / zh: "LinguaRay — 错误"；Normal 仍是 "LinguaRay"）。`TrayStateController::recompute` 在调用 `set_tooltip` 时传入当前 locale（locale 从 `AppState`/settings 读取，与 KeystoreRecovery 的 `SETTINGS_COPY[locale]` 模式一致）。

**rev-13 changes (定点修订 — A5 Tray 状态机 5 个 P1 + housekeeping, 全部针对 Task A5; rev-9-1/2/3、rev-11/rev-12 的非 A5 内容、其他任务保持不变):**

- **rev-13-1 (P1-1 控制器归属 + 调用点命名).** rev-12 对 `tray` 字段挂在 `Session` 还是 `AppState` 含糊（rev-12-3 写 "挂在 `AppState`（或 `Session`）上"），调用点混用 `state.tray`（Session 风格）与 `app_state.tray`。**rev-13 定点：** `tray: Arc<tokio::sync::Mutex<TrayStateController>>` 字段挂在 **`AppState`**（已核实 `AppState` lib.rs:99-106 无 tray 字段；`Session` lib.rs:70-74 也无），所有调用统一用 **`app_state.tray`**（`capture_and_translate` 的 `app_state: &Arc<AppState>` 参数名）。Switch Provider 分支在 `spawn_blocking` 前先 `app_state.inner().clone()`（`tauri::State` 非 `Send`，不能 move 进闭包）。
- **rev-13-2 (P1-2 RAII guard 保证 begin/end 配对).** rev-12 把 `begin_translation` 放在 `capture_and_translate` 函数顶部，靠手工给十余个提前 return 分支补 `end_translation` —— 漏一个就泄漏 Active 计数。**rev-13 引入 `TranslationGuard` RAII guard：** preflight（文字捕获 + anchor 构建）完成后才 `begin_translation`，guard 的 `Drop` 在作用域退出时自动 `end_translation`（覆盖所有 return/`?`/panic 路径），保证 begin/end 恰好配对。`translate_clipboard` 同样：剪贴板读取成功后才 begin。成功分支调 `guard.mark_success()`（清除上一代 error）；失败分支调 `app_state.tray.lock().await.record_error(gen)`。
- **rev-13-3 (P1-3 error 带 generation — 替换 rev-12 的 sticky bool).** rev-12 的 `has_error: bool` 只能靠 `set_error(false)`（switch-provider 成功）清除 —— 用户 Retry 成功也不会清红点（成功路径只调 `end_translation`）。**rev-13 改为 `error_gen: Option<u64>`：** 翻译失败 `record_error(gen)`；新翻译 `begin_translation(new_gen)` 若 `error_gen < new_gen` 则清除（新代际取代旧 error）；成功（`mark_success`）清除当前/上一代 error；switch 成功 `clear_error()` 无条件清除。Retry 成功（新代际）红点消失。
- **rev-13-4 (P1-4 timer epoch 串行化).** rev-12 的 pulse timer 在 `abort()` 后已调度的 tick 仍可能写 icon，覆盖刚进入的 `Error`。**rev-13 加 `visual_epoch: u64`：** 每次状态转换先 `visual_epoch += 1`（使旧 timer 失效）再 `abort()`；timer 每 tick 先检查自己的 epoch 是否仍为当前，否则退出（不写 icon）。通过 `Arc<AtomicU64>` epoch-flag 避免 timer 持有 mutex 跨 `sleep`。
- **rev-13-5 (P1-5 测试注入 renderer + tokio time).** rev-12 的测试直接调 `set_pulsing(true/false)`，不测真实 timer。**rev-13 抽象 `trait TrayRenderer { set_icon/set_tooltip }`：** 生产实现 `TrayIconRenderer` 包 `TrayIcon`；测试用 `RecordingRenderer` 记录所有调用；用 `tokio::time::pause()` + `advance()` 控制定时器。测试：Active 后 mock 收到 alternating frames；第二个 begin 不产生第二个 timer；最后一个 end 后 timer 停止；Error 后无 active frame；旧 epoch tick 被拒绝。
- **rev-13 housekeeping.** (a) tokio features 明确写 `["macros", "rt-multi-thread", "time", "sync"]`（已核实 Cargo.toml:102 缺 time/sync）；(b) 列出全部 5 个 AppState 构造点（lib.rs:2513/2597/2620 + recovery.rs:42/248），生产用 `new(app)`、测试用 `with_renderer(RecordingRenderer, En)`；(c) locale 用系统 `LANG`（`detect_system_locale()`），不依赖 `Settings`（已核实 Settings 无 locale 字段）—— 删除 rev-12 的 `read_locale(state)` helper；(d) `build.rs` 删除未用的 `imageops` import；(e) `image` 加为 `[dev-dependencies]` 供像素差异测试；(f) 红点像素测试加载生成的 PNG，断言底图大部分像素不变 + badge 区域有 `#DC2626`；(g) latency 测试测纯 `Duration → u32` 饱和函数 + 静态保证 probe 调用该函数（无真实 timing flakiness —— rev-12-5 的本意保留，rev-13 仅澄清实现位置在 C3c，不属 A5）。

**rev-14 changes (定点修订 — A5 Tray 状态机 6 个 P1 + 4 个 P2, 全部针对 Task A5; 核心改变：从异步 `tokio::sync::Mutex` 改为同步 `parking_lot::Mutex` + RenderGate 同一锁内 epoch + render; rev-9-1/2/3、rev-11/rev-12/rev-13 的非 A5 内容、其他任务保持不变):**

- **rev-14-1 (P1-1 同步 parking_lot::Mutex 替换 tokio::sync::Mutex).** rev-13 的 `TranslationGuard::drop` 用 `tauri::async_runtime::spawn(async move { controller.lock().await.end_translation(gen).await; })` —— Drop 是同步的，spawn 一个异步任务来 end，但这个 spawn 的 future 是分离的 (detached)，spawn 返回时计数仍为 1（RAII 的"作用域退出即结束"保证被破坏）。**rev-14 改为同步 `parking_lot::Mutex`：** `tray: Arc<parking_lot::Mutex<TrayStateController>>`（已核实 `parking_lot = "0.12"` 在 `Cargo.toml:53`，生产依赖）。`begin_translation`/`end_translation`/`record_error(gen)`/`mark_success(gen)`/`clear_error_for_gen(gen)` 全部是同步方法（无 `async`、无 `.await`）。`TranslationGuard::drop` 中同步 `controller.lock().end_translation(gen); if succeeded { controller.lock().clear_error_for_gen(gen); } controller.lock().recompute();` —— Drop 直接在调用线程完成全部工作，真正实现 RAII 保证。timer 仍通过 Tauri runtime spawn，但 timer 内部获取锁是同步的（`std::thread::spawn` + `controller.lock()`）。
- **rev-14-2 (P1-2 只在状态真正变化时 bump epoch).** rev-13 的 `recompute` 每次 begin/end 都 `visual_epoch += 1` —— 两个并发翻译 Active → Active（计数从 1→2）也 bump epoch，导致旧 timer 被杀掉再启动（pulse 抖动）。**rev-14 控制器保存 `current_state: TrayVisualState`：** `recompute` 先计算 `new_state`，只有 `new_state != current_state` 时才 (1) `visual_epoch += 1`；(2) 停止旧 timer（若有，`std::thread::JoinHandle` 的 `join()` 或 drop）；(3) 创建新 timer（如果进入 `Active`）；(4) 更新 `current_state`；(5) `render()` 写入新 icon。Active → Active（计数从 1→2）不 bump epoch、不杀 timer。
- **rev-14-3 (P1-3 RenderGate 串行写屏障).** rev-13 的 epoch 检查（在 timer 内）和 icon 写入（`renderer.set_icon(...)`）不在同一把锁内 —— `abort()` 非阻塞，已调度的 tick 仍可能在锁释放后写 icon，覆盖刚进入的 Error。**rev-14 把所有 icon 写入收敛到同一个同步 `render(&mut self)` 方法（RenderGate），在 `parking_lot::Mutex` guard 内调用：** `render(&mut self)` 根据 `self.current_state` 调用 `renderer.set_icon_normal()`/`set_icon_dimmed()`（Active 交替帧 `self.pulse_frame = !self.pulse_frame`）/`set_icon_error_dot()` + `renderer.set_tooltip(&self.tooltip_text(locale))`。timer 每次 `controller.lock()` 拿到 guard 后，先检查 `my_epoch == controller.visual_epoch`，匹配才 `controller.render()`（render 内部读 `current_state`，AtomicRender）。静态状态转换（Error/Normal）也在锁内 bump epoch + render。epoch 检查与 icon 写入现在原子地发生在同一把锁内 —— 无竞态。
- **rev-14-4 (P1-4 测试编译错误).** rev-13 的测试用 `#[tokio::test]` + `c.begin_translation(1).await` —— 但 rev-14 方法都是同步的，`.await` 不编译。**rev-14：** 测试改为 `#[test]` + `c.begin_translation(1);`（无 `.await`）。`guard.mark_success();`（同步，无 `.await`）。`#[tokio::test(start_paused = true)]` 改为 `#[test]` + 显式控制 timer（使用极小 tick interval + `thread::sleep` 等待，或注入 `RecordingRenderer` 后断言 controller 内部状态而不依赖真实 timer timing）。dev-dependencies tokio 加 `test-util` feature（`tokio = { version = "1", features = ["macros", "rt-multi-thread", "test-util"] }`）—— 为 `#[tokio::test(start_paused = true)]` 保留可能；但 rev-14 timer 改为 `std::thread` 后，时间控制测试改用同步断言。
- **rev-14-5 (P1-5 Switch Provider Session 获取).** rev-13 的 Step 10 假设 `state` 在 `handle_tray_menu_event` 作用域内，但已核实签名 `fn handle_tray_menu_event(app: &tauri::AppHandle, event: MenuEvent)` —— 只有 `app`，无 `state`/`session` 参数。**rev-14 在 switch-provider 分支明确写出 Session 获取：** `let session = app.state::<Arc<Session>>().inner().clone(); let gen = session.gen.next();`（`app.state::<T>()` 是 `tauri::Manager` 方法，返回 `tauri::State<'_, T>`，`.inner()` 返回 `&T`，`.clone()` 得到 `Arc<Session>`）。`app_state` 同样通过 `app.state::<Arc<AppState>>().inner().clone()` 获取。
- **rev-14-6 (P1-6 清理 rev-12/rev-13 活动指令).** 全文搜索 `has_error`（非 `error_gen`）、`tokio::sync::Mutex`（tray 相关）、`state.tray`（非 `app_state.tray`），全部替换为 rev-14 最终模型。A4 Surface 说明第 1471 行的 `state.tray.lock().await.set_error(&app, true, locale)` 改为 `app_state.tray.lock().record_error(gen)` / `clear_error_for_gen(gen)`。File Structure、P1-5 summary、Self-Review P1-5 同步更新为同步 parking_lot 模型。
- **rev-14 P2 修正.** (a) **locale：** 加 `sys-locale = "0.3"` 到 `Cargo.toml` `[dependencies]`，`detect_system_locale()` 使用 `sys_locale::get_locale()`（跨平台，不读 `LANG` 环境变量 —— LANG 在 Windows 上不存在；rev-13 的 `std::env::var("LANG")` 仅 Unix 有效）。(b) **像素测试缺失产物失败：** 不 `return`（跳过），而是 `panic!("build.rs output not found: {error_png}")` —— 跳过会让 build.rs 回归静默通过。(c) **Debug derive：** `TrayStateController` 不 derive `Debug`（含 `Arc<dyn TrayRenderer>`，`dyn Trait` 不自动实现 `Debug`）；或给 `TrayRenderer` trait 加 `Debug` bound。rev-14 选择前者（不 derive Debug）。(d) **RecordingRenderer cfg：** rev-14 当时写 `#[cfg(test)]` 导出 RecordingRenderer，**rev-15 P1-2 更正为 `#[cfg(any(test, feature = "xproc-test-helper"))]`** —— 集成测试 crate（`src-tauri/tests/`）是独立的 crate，它编译 lib 时 `cfg(test)` **不会**被启用（`cfg(test)` 只在编译 lib 自己的 unit-test 目标时启用），因此集成测试看不到 `#[cfg(test)]` 的项；本计划的 verification 命令统一带 `--features xproc-test-helper`，故 `#[cfg(any(test, feature = "xproc-test-helper"))]` 在 `cargo test --features xproc-test-helper` 下对集成测试可见，且在 `cargo build`（无 feature）下不编译 RecordingRenderer。

**rev-15 changes (定点修订 — A5 Tray 状态机 4 个 P1 + 杂项, 全部针对 Task A5; 保留 rev-14 的同步 Mutex / state 门控 epoch / sys-locale / 像素强断言方向; rev-9-1/2/3、rev-11/rev-12/rev-13/rev-14 的非 A5 内容、其他任务保持不变):**

- **rev-15-1 (P1-1 Pulse 线程必须可退出 — channel 信号 + join, 替换 rev-14 的 stop_timer+join 死锁).** rev-14 的 pulse timer 是一个无限 `loop { sleep; render }`，`recompute` 在离开 `Active` 时调 `stop_timer()` → `handle.join()`。问题：`join()` 等待线程退出，但线程**永不退出**（无限 loop），`join()` 永远不返回 —— 这是一处会挂死整个 app 的死锁。**rev-15 引入 `PulseWorker`：** 一个持有 `stop_tx: std::sync::mpsc::Sender<()>` + `handle: Option<std::thread::JoinHandle<()>>` 的小结构。`PulseWorker::start(renderer, interval)` 用 `mpsc::channel()` 创建信号通道，worker 线程在 `stop_rx.recv_timeout(interval)` 上等待 —— 收到 `Ok(())`（停止信号）或 `Err(Disconnected)` 立即 `return`；`Err(Timeout)` 才执行一次帧切换（`dimmed = !dimmed; renderer.set_icon_dimmed/normal()`）。`PulseWorker::stop(&mut self)` 先 `stop_tx.send(())`（唤醒线程），再 `handle.take().join()`（线程此时已从 `recv_timeout` 返回，`join` 立即完成 —— 不死锁）。`impl Drop for PulseWorker { fn drop(&mut self) { self.stop(); } }`。`TrayStateController` 持有 `pulse_worker: Option<PulseWorker>`（**不是** `Option<std::thread::JoinHandle<()>>`）：进入 `Active` 时 `pulse_worker = Some(PulseWorker::start(renderer.clone(), interval))`；离开 `Active` 时 `pulse_worker.take()` —— `take()` 触发旧 `PulseWorker` 的 `Drop`，`Drop` 调用 `stop()`（send + join）。`TrayStateController::drop` 也 `drop` 残留的 `pulse_worker`（自动停止）。**不需要 epoch check 在 timer 内** —— 因为 timer 持有独立的 `Arc<dyn TrayRenderer>` 引用，且 `stop()` 是同步的 channel 信号 + join（join 是真正的同步屏障：在 `send` 之后线程必然在下一次 `recv_timeout` 边界返回，`join` 等待到该返回点）。rev-14 的 `stop_timer()` 死锁问题由此彻底解决。
- **rev-15-2 (P1-2 RecordingRenderer cfg 可见性 — `#[cfg(any(test, feature = "xproc-test-helper"))]`).** rev-14 把 RecordingRenderer 标为 `#[cfg(test)]`，但集成测试 `src-tauri/tests/tray_state.rs` 是一个**独立 crate**，它以普通依赖方式依赖 `linguaray_lib`；编译 lib 供集成测试使用时 `cfg(test)` **不启用**（`cfg(test)` 只在编译 lib 自身的 unit-test 目标时启用），所以集成测试 `use linguaray_lib::tray_state::RecordingRenderer;` 会报 `unresolved import`。无条件 `pub` 则在 `cargo build`（无 feature）下也编译 RecordingRenderer，把测试 mock 链进生产二进制。**rev-15 改为 `#[cfg(any(test, feature = "xproc-test-helper"))]`**（struct + 所有 `impl` 块 + `RenderedIcon`/相关测试 helper 全部在同一 cfg 门下）。`lib.rs` 的 `pub use tray_state::{...RecordingRenderer...}` 也加同一 cfg。本计划的 verification 命令统一带 `--features xproc-test-helper`，因此集成测试可见；`cargo build`（无 feature）下 RecordingRenderer 完全不编译。`Cargo.toml [features] xproc-test-helper = []` 已存在（已核实）。
- **rev-15-3 (P1-3 Switch Provider 不推进翻译 generation — `record_error()` / `clear_error()` 无 gen 参数).** rev-14 的 switch-provider 路径调 `session.gen.next()` 分配一个翻译 generation 给 tray —— 但 `GenerationToken::next()` 返回 `fetch_add(1) + 1`（已核实 concurrency.rs），**推进后旧 gen 变 stale**。这意味着：用户在翻译进行中（gen=5）点 Switch Provider → switch 分配 gen=6 → 正在进行的 gen=5 翻译立即变 stale（`is_latest(5)` 返回 false）→ 其结果被丢弃，RAII guard 的 Drop 也用 end_translation 而非 mark_success 清理。这是 switch 与翻译互踩的竞态。**rev-15 修正：Switch Provider 完全不碰翻译 generation。** `TrayStateController` 的 `record_error()` / `clear_error()` 改为**无 gen 参数**（switch 不是翻译流程，tray 错误状态用独立的 sticky `has_error: bool` 标记，不与翻译 `error_gen` 混用）。switch 失败 → `controller.lock().record_error()`；switch 成功 → `controller.lock().clear_error()`。翻译流程仍用 `error_gen: Option<u64>`（`TranslationGuard` 持 gen）。回归测试 `switch_does_not_bump_translation_generation`：在 `session.gen.next()` 分配 gen=1 之后，调一个 mock switch 路径，断言 `session.gen.is_latest(1)` 仍为 true（switch 不调 `gen.next()`）。
- **rev-15-4 (P1-4 清除两套矛盾的 Timer 架构 — 只保留 PulseWorker(channel) 模型).** rev-14 正文同时存在两套叙述：(a) rev-15-1 之前的 "timer 持有独立 `Arc<dyn TrayRenderer>`，stop 是同步 channel 信号"，(b) rev-14 的 "timer 内 epoch check + RenderGate（`my_epoch == visual_epoch` 后 `tick_render()`）" + "stop_timer() join 是 RenderGate 屏障"。这两套模型矛盾：rev-14 说 timer 在 tick 时 `controller.lock()` 拿 guard 后检查 epoch，但 rev-14 的 `spawn_pulse_timer` 实际上**不持** controller lock（它只捕获 `renderer` + `locale`，每 tick 直接调 `renderer.set_icon_*`）—— 即 rev-14 正文与 rev-14 代码块不一致。**rev-15 只保留一套：PulseWorker（channel 退出）+ 独立 `Arc<dyn TrayRenderer>`。** 删除所有 "epoch check 在 timer 内"、"tick_render()"、"thread self-exits"、"`AtomicRender`"、"RenderGate (epoch check + render in the SAME lock)" 的叙述。`stale_epoch_tick_does_not_clobber_error` 测试改为：进入 `Active` → pulse 开始；`record_error()` → `recompute` 切到 `Error` → `pulse_worker.take()`（`Drop` 调用 `stop()`：send 信号 + join 完成）→ 验证 `RecordingRenderer` 最后收到的帧**不是** dimmed（pulse 已停止，没有 stale tick 能在 join 之后写入）。删除 `visual_epoch` 字段（不再需要 —— `PulseWorker::stop()` 的 send + join 是真正的同步屏障）。`current_state: TrayVisualState` 字段保留（rev-14 P1-2 的"只在 `new_state != current_state` 时切换 pulse"逻辑仍需要它，避免 Active→Active 重启 worker）。
- **rev-15 杂项.** (a) **Cargo.lock** 加入 A5 提交清单（已核实 `src-tauri/Cargo.lock` 是 git 跟踪文件 —— 新增 `sys-locale = "0.3"` 依赖会改 Cargo.lock）。(b) **测试数** 改为实际枚举：**27**（不是 21；本修订初稿估计的 23 也偏低 —— 权威数字是 Step 2 代码块内 `^#\[test\]$` 的实际计数，27）：6 priority + **6 reducer concurrency**（5 counter/error-driven + `switch_flow_has_error_is_independent_of_translation_error_gen`）+ 2 RAII guard + 2 generation-aware error + 4 renderer + pulse-worker-lifecycle（`active_emits_alternating_frames_on_the_renderer`、`second_begin_does_not_churn_the_worker`、`last_finish_stops_the_worker`、`error_produces_no_active_pulse_frame`）+ **2 pulse-worker channel-quit**（新增 `stop_signal_joins_the_worker`、`drop_stops_the_worker`）+ 1 worker-stop-barrier（`leaving_active_stops_the_worker_no_stale_frames` —— 从 rev-14 的 `stale_epoch_tick_does_not_clobber_error` 改名，断言改为 PulseWorker.stop 后无 dimmed 帧）+ 2 localization + 1 pixel-diff + **1 switch-does-not-bump-generation**（`switch_does_not_bump_translation_generation`，新增，验证 switch 不调 `gen.next()`）。(c) **`finish_translation(success)` 合并：** 把 `end_translation + (if success { clear_error }) + recompute` 合并为一次 `finish_translation(&mut self, gen: u64, success: bool)` 原子操作。`TranslationGuard::drop` 调 `finish_translation(self.gen, self.success)`。翻译失败分支先调 `record_error(gen)`，guard Drop 调 `finish_translation(gen, false)`（不减 error，仅 `end_translation + recompute`）；翻译成功分支 `guard.mark_success()`，guard Drop 调 `finish_translation(gen, true)`（`end_translation + clear_error + recompute`）。**switch 路径不走 guard**，switch 失败直接 `record_error()`（无 gen，无计数，设置 sticky `has_error`），switch 成功直接 `clear_error()`（无 gen，清 `has_error`）。(d) **timer 测试用确定性同步：** 不用 `thread::sleep(20ms)`，而是 channel 验证 `stop_tx.send()` 后 `join()` 返回 + `RecordingRenderer` 帧数验证（pulse-worker-lifecycle 测试仍可用极小 interval + 小 sleep 观察 frames，但 channel-quit 测试不依赖 timing）。

**rev-16 changes (定点修订 — A5 Tray 状态机 3 个 P1 + 5 个 P2, 全部针对 Task A5; 保留 rev-15 的 PulseWorker(channel-quit) / cfg(any(test, feature)) / Cargo.lock / 27 tests 基线方向, 在其上叠加 gen 保护 + switch revision + 无函数重载 + 无 thread::sleep; rev-9-1/2/3、rev-11~rev-15 的非 A5 内容、其他任务保持不变):**

- **rev-16-1 (P1-1 Rust 不支持函数重载 — 重命名 record_error/clear_error 的两个重载).** rev-15 在 `impl TrayStateController` 中同时定义了 `record_error(&mut self, gen: u64)` (翻译流) 和 `record_error(&mut self)` (switch 流, 无 gen) —— 两个同名方法。**Rust 不支持按参数重载 (function overloading)，这无法编译** (`E0592: duplicate definitions`)。rev-16 把它们重命名为独立方法名（不再有重载）：
  - **`record_translation_error(&mut self, gen: u64)`** — 翻译失败时调用（原 `record_error(gen)`，翻译流，gen-tagged `error_gen`）。
  - **`begin_switch(&mut self) -> u64`** / **`finish_switch(&mut self, rev: u64, success: bool)`** — switch 流的 begin/finish 对（用 `switch_revision`/`switch_error_rev` —— 见 rev-16-3）。
  全部调用点、测试、接口表、覆盖矩阵同步更新。`finish_translation` / `begin_translation` 名字保持不变（它们在原计划中本来就无重载）。**rev-17-4: `record_switch_error()` / `clear_switch_error()` 被删除** —— `finish_switch(rev, false)` / `finish_switch(rev, true)` 已完全替代它们且带有 stale revision 保护。**rev-17 P2-3: `clear_translation_error(gen)` 被删除** —— 它从未被调用（`finish_translation(gen, true)` 已在内部合并 clear 逻辑）。
- **rev-16-2 (P1-2 旧翻译成功清除新翻译错误 — gen 保护).** rev-15 的 `finish_translation(&mut self, gen: u64, success: bool)` 在 `success` 时无条件 `self.error_gen = None` —— 但如果 gen=1 的旧翻译（已 stale）迟到成功，会清掉 gen=2 新翻译刚记录的错误。rev-16 加代际保护：
  ```rust
  pub fn finish_translation(&mut self, gen: u64, success: bool) {
      self.active_translations = self.active_translations.saturating_sub(1);
      if success {
          // 只清除 <= 当前 gen 的错误（旧 gen 成功不清新 gen 错误）
          if self.error_gen.is_some_and(|eg| eg <= gen) {
              self.error_gen = None;
          }
      }
      self.recompute();
  }
  ```
  同时 `record_translation_error` 也需要防止旧 gen 迟到错误覆盖新错误：
  ```rust
  pub fn record_translation_error(&mut self, gen: u64) {
      // 只在新错误 gen >= 已有错误 gen 时更新（防止旧 gen 迟到覆盖）
      if self.error_gen.is_none_or(|eg| gen >= eg) {
          self.error_gen = Some(gen);
      }
      self.recompute();
  }
  ```
  `begin_translation(gen)` 清除旧 error 的条件保持 `error_gen < gen`（严格小于，已存在）。新增 2 个测试：`older_success_does_not_clear_newer_error`（gen1 成功不清 gen2 错误）+ `older_error_does_not_replace_newer_error`（gen1 迟到错误不覆盖 gen2 错误）。
- **rev-16-3 (P1-3 Switch sticky bool 并发乱序 — 独立 switch revision).** rev-15 的 switch 错误状态用 `has_error: bool`（无 revision）—— 两个并发 switch 完成乱序会错误清除/设置（switch A 失败设 true，switch B 成功清 false，但 B 先到，A 后到 → 最终 Error，而最新用户意图是 B 成功）。rev-16 改用独立 switch revision（**不复用** Session.gen —— 翻译 gen 与 switch 完全解耦，rev-15 P1-3 的方向保持）：
  ```rust
  pub struct TrayStateController {
      // ... 翻译相关字段 (active_translations / error_gen) 不变 ...
      switch_revision: u64,          // 当前 switch 的 revision（单调递增）
      switch_error_rev: Option<u64>, // 哪个 revision 的 switch 产生了错误
      // has_error: bool  ← 删除（被 switch_revision/switch_error_rev 取代）
      current_state: TrayVisualState,
      pulse_worker: Option<PulseWorker>,
      tick_interval: Duration,
      renderer: Arc<dyn TrayRenderer>,
      locale: Locale,
  }

  pub fn begin_switch(&mut self) -> u64 {
      self.switch_revision += 1;
      self.switch_revision
  }

  pub fn finish_switch(&mut self, rev: u64, success: bool) {
      // 只有最新 revision 的 switch 能更新状态
      if rev != self.switch_revision {
          return; // stale switch result, ignore
      }
      if success {
          self.switch_error_rev = None;
      } else {
          self.switch_error_rev = Some(rev);
      }
      self.recompute();
  }
  ```
  (**rev-17-4: `record_switch_error()` / `clear_switch_error()` 已删除** —— `finish_switch(rev, false)` / `finish_switch(rev, true)` 已完全替代它们且带有 stale revision 保护。) `recompute_pure` 中 Error 判断改为：`Error if error_gen.is_some() || switch_error_rev.is_some()`（替换 rev-15 的 `has_error`）。访问器 `has_error()` 改为 `switch_error_rev()` 返回 `Option<u64>`。Switch Provider 调用点改为 `begin_switch()` → `finish_switch(rev, success)` 模式（Step 10）。新增 2 个测试：`two_concurrent_switches_second_wins`（switch A 成功 + switch B 失败 → 最终 Error）+ `stale_switch_result_ignored`（旧 revision 的迟到结果被忽略）。
- **rev-16 P2 修正.**
  1. **删除测试中的 `thread::sleep`：** rev-15 的 PulseWorker-lifecycle 测试用极小 interval (2ms) + `thread::sleep(20ms)` 观察帧 —— 这在 CI 慢机器上仍可能 flaky。rev-16 改用 `mpsc::channel` 确定性同步：PulseWorker 内部每次 tick 后向一个 notification channel `send(())`，测试 `recv_timeout` 等待确定性帧数（不是 sleep 等待固定时间）；停止屏障测试 (`stop_signal_joins_the_worker` / `drop_stops_the_worker` / `leaving_active_stops_the_worker_no_stale_frames`) 本就不用 sleep（rev-15 已确认），rev-16 保持。PulseWorker 的 notification channel 是测试注入的（构造时传入 `Option<Sender<()>>`，prod 传 `None`）。
  2. **`switch_does_not_bump_translation_generation` 测试增强：** 加入结构测试 —— grep 源码确认 switch handler 不含 `gen.next`（或 `session.gen`）；或测试提取后的真实 switch helper。rev-16 在 Step 10 提取一个 `pub fn handle_switch_provider(app, uuid) -> Result<()>` helper（测试可见），Step 2 的测试调用它并断言 `token.is_latest(g1)` 不变，同时加一个 `#[test]` 用 `include_str!` 读 `lib.rs` 源码断言 switch arm 不含 `.gen.next()`（结构性回归保护）。
  3. **删除测试中未使用的 `RenderedIcon`/`TrayRenderer` import：** rev-15 的测试 `use linguaray_lib::tray_state::{ ... RenderedIcon, TrayRenderer, ... }` —— 但测试只构造 `RecordingRenderer`（不需 `TrayRenderer` trait 名），且 `RenderedIcon` 通过 `RecordingRenderer::calls()` 返回值的方法（`.is_dimmed()` 等）访问，不需要直接命名类型。rev-16 删除这两个未使用 import（clippy `unused_imports`）。
  4. **增加默认构建验证步骤：** rev-15 的 A5 验证只跑 `cargo build --features xproc-test-helper` —— 但 `#[cfg(any(test, feature = "xproc-test-helper"))]` 的 re-export 在**无 feature** 下必须仍编译。rev-16 在 Step 11 (A5 验证) 加 `cargo build --manifest-path src-tauri/Cargo.toml`（不带 feature）验证 cfg-gated re-export 不在生产构建中泄漏 `RecordingRenderer`。
  5. **测试文件头修正：** Step 2 的测试 doc-comment 写 "reducer concurrency (5 tests)" —— 实际是 6 tests（含 `switch_flow_has_error_is_independent_of_translation_error_gen`）。rev-16 修正 "5 tests" → "6 tests"（与 rev-15 杂项 (b) 的枚举一致）。

**rev-17 changes (定点修订 — A5 Tray 状态机 4 个 P1 + 4 个 P2, 全部针对 Task A5; 修复用户审核笔记中的问题; 保留 rev-16 的全部架构方向: PulseWorker channel / parking_lot::Mutex / switch_revision / gen 保护; rev-9-1/2/3、rev-11~rev-16 的非 A5 内容、其他任务保持不变):**

- **rev-17-1 (P1-1 handle_switch_provider 改为 async) — SUPERSEDED by rev-18-1（仅作历史记录，活动代码不采用 async）.** rev-16 的 `handle_switch_provider` 是 `pub fn handle_switch_provider(app: &tauri::AppHandle, uuid: &str) -> Result<(), String>`（同步），但它的函数体内部调 DB 操作（`spawn_blocking` + `set_active_primary_core`），`spawn_blocking(...).await` 需要 `async fn`。同步 `pub fn` 内出现 `.await` 不编译。**rev-17 改为 `pub async fn`**，签名变为 `pub async fn handle_switch_provider(app: tauri::AppHandle, app_state: Arc<AppState>, uuid: String)`，并相应更新调用方 `handle_tray_menu_event` 的 `tray.switch-<uuid>` arm 用 `tauri::async_runtime::spawn(async move { handle_switch_provider(app2, app_state, uuid).await; })` 异步执行（菜单事件回调本身不是 async，必须 spawn）。函数体不再 `.await` 一个 `spawn_blocking` 块（去掉 `pub fn` 的 `.await`），直接在 async 上下文中 `set_active_primary_core(&app_state, &uuid).await`，然后 `c.finish_switch(rev, result.is_ok())`。

  > **rev-20-3 (历史标注):** rev-17-1 的 async 模型基于"`set_active_primary_core` 是 async"的错误前提 —— 实际 `set_active_primary_core` 是 SYNC fn（A4 Step 9: body 是 spawn_blocking 的同步 payload），async fn 不能 `.await` 一个 sync fn。**rev-18-1 推翻此修订**，改回 SYNC `pub fn` + core/wrapper 两层 + `spawn_blocking` 卸载。**下列 rev-17-1 代码块仅为历史 changelog 记录，活动代码（A5 Step 10 + A4 Step 9 tray arm）不再使用 async `handle_switch_provider` / `spawn(async move {...})` 模式 —— 见 rev-18-1 的 SYNC core + wrapper 代码。**
  ```rust
  // rev-20-3: SUPERSEDED by rev-18-1 — 以下 async 签名与 spawn(async move) 调用不再使用。
  // 保留仅为说明 rev-17 → rev-18 的演进。活动代码见 rev-18-1 的 core + wrapper 两层模型。
  // rev-17 (SUPERSEDED): pub async fn handle_switch_provider(app: tauri::AppHandle, app_state: Arc<AppState>, uuid: String) {
  //     let rev = { let mut c = app_state.tray.lock(); c.begin_switch() };
  //     let result = set_active_primary_core(&app_state, &uuid).await;  // ← 错误前提：set_active_primary_core 是 SYNC
  //     let mut c = app_state.tray.lock();
  //     c.finish_switch(rev, result.is_ok());
  //     refresh_tray_if_available(&app);
  // }
  //
  // rev-17 (SUPERSEDED) handle_tray_menu_event 中 switch arm：
  // "tray.switch-provider" => {
  //     let app2 = app.clone();
  //     let app_state = app.state::<Arc<AppState>>().inner().clone();
  //     let uuid = /* 从子菜单 id 解析 */;
  //     tauri::async_runtime::spawn(async move {             // ← rev-18-1 改为 spawn_blocking
  //         handle_switch_provider(app2, app_state, uuid).await;  // ← rev-18-1: 无 .await，SYNC wrapper
  //     });
  // }
  ```
  ~~所有 `pub fn handle_switch_provider` / `handle_switch_provider(app: &tauri::AppHandle, uuid: &str) -> Result<(), String>` 的提法同步改为 `pub async fn handle_switch_provider(app: tauri::AppHandle, app_state: Arc<AppState>, uuid: String)`。~~ （**rev-18-1 推翻：** 活动代码改为 SYNC core `pub fn handle_switch_provider_core(app_state, uuid)` + SYNC wrapper `pub fn handle_switch_provider(app, app_state, uuid)`，见 rev-18-1。）
- **rev-17-2 (P1-2 PulseWorker notify 改为 PulseEvent 枚举).** rev-16 的 `notify: Option<mpsc::Sender<()>>` 只发空信号 `()`，测试无法在 `recv` 端区分这是一个 `Tick` 帧还是一个 `Stopped`（worker 退出）事件。`last_finish_stops_the_worker` / `leaving_active_stops_the_worker_no_stale_frames` 测试断言 `notify_rx.recv_timeout()` 返回 `Err(Disconnected)` 来判断 worker 已死 —— 但这依赖"Sender 被 drop"的副作用，而不是 worker 显式发出 `Stopped` 事件，不够清晰。**rev-17 引入 `PulseEvent` 枚举：**
  ```rust
  pub enum PulseEvent {
      Tick,    // 一个 pulse 帧完成（toggle + render 后发出）
      Stopped, // worker 即将退出（收到 stop signal 或 sender disconnected）
  }
  pub struct PulseWorker {
      stop_tx: mpsc::Sender<()>,
      handle: Option<std::thread::JoinHandle<()>>,
      // rev-19-3: notify field REMOVED from struct — moved into worker thread closure
  }
  ```
  worker 循环中：`Err(Timeout)` → toggle + render 后 `tx.send(PulseEvent::Tick)`（不用 expect/unwrap，`send` 可能失败但不应 panic）；`Ok(())` 或 `Err(Disconnected)` → `tx.send(PulseEvent::Stopped)` 后 `return`。测试用 `recv_timeout` 收到 `PulseEvent::Tick` 计帧、收到 `PulseEvent::Stopped` 判定 worker 已死。controller 字段 `notify_tx: Option<mpsc::Sender<PulseEvent>>`。`PulseWorker::start` 的 `notify` 参数被 moved 进 worker 线程闭包（不存储在 struct 上，避免 dead_code）。
- **rev-17-3 (P1-3 record_translation_error 的 latest_translation_gen 保护).** rev-16 的 `record_translation_error(gen)` 保护是 `gen >= error_gen`（`error_gen.is_none_or(|eg| gen >= eg)`）。但这个保护不够 —— 场景：gen1 begin → error_gen=None；gen2 begin → begin_translation 清旧 error（gen1 没有 error，不清），error_gen 仍 None；gen1 迟到 error → record_translation_error(1) → `1 >= None` 为 true → error_gen=Some(1)；但 gen2 才是最新的，gen1 的错误不应显示。**rev-17 给控制器加 `latest_translation_gen: u64` 字段**（最近一次 begin_translation 的 gen）：
  ```rust
  pub struct TrayStateController {
      // ... 原有字段 ...
      latest_translation_gen: u64,  // rev-17-3: 最近一次 begin_translation 的 gen
  }
  pub fn begin_translation(&mut self, gen: u64) {
      if gen > self.latest_translation_gen {
          self.latest_translation_gen = gen;
      }
      if self.error_gen.map_or(false, |e| e < gen) {
          self.error_gen = None;
      }
      self.active_translations = self.active_translations.saturating_add(1);
      self.recompute();
  }
  pub fn record_translation_error(&mut self, gen: u64) {
      // 只有当前 gen 或更新 gen 的错误才记录；旧 gen (< latest_translation_gen) 的迟到错误被忽略
      if gen >= self.latest_translation_gen && self.error_gen.is_none_or(|eg| gen >= eg) {
          self.error_gen = Some(gen);
      }
      self.recompute();
  }
  ```
  新增测试 `stale_gen_error_ignored_after_newer_begin`：gen1 begin → gen2 begin → gen1 迟到 error → error_gen 仍 None。
- **rev-17-4 (P1-4 删除 record_switch_error/clear_switch_error — finish_switch 已替代).** rev-16 同时有 `record_switch_error()` / `clear_switch_error()` 和 `begin_switch()` / `finish_switch(rev, success)`。后者已经完全替代了前者的功能（`finish_switch(rev, false)` = `record_switch_error`，`finish_switch(rev, true)` = `clear_switch_error`，且 `finish_switch` 还有 stale revision 保护，是更完整的 API）。rev-16 在 Step 10 的注释里也提到 "handler uses begin_switch+finish_switch, the lower-level record_switch_error/clear_switch_error are for direct calls"，但实际没有任何调用点用这两个低层方法 —— 它们是死代码。**rev-17 删除 `record_switch_error(&mut self)` 和 `clear_switch_error(&mut self)` 两个方法**。全部测试 / 接口 / 调用点同步改为 `finish_switch`。Step 2 测试 `switch_flow_error_is_independent_of_translation_error_gen` 中 `c.record_switch_error()` / `c.clear_switch_error()` 改为 `c.finish_switch(rev, false)` / `c.finish_switch(rev, true)`。
- **rev-17 P2 修正.**
  1. **functional switch test（SUPERSEDED by rev-18-3 — 见下）：** rev-16 的 `switch_handler_does_not_call_gen_next` 测试只是手动模拟 controller 交互（`begin_switch` + `finish_switch`），不调真实的 `handle_switch_provider`。rev-17 改为测试真实的 `handle_switch_provider`（rev-17-1 改为 async 后可直接 `await`）：构造一个 `Arc<AppState>`（带 mock DB 或用 `set_active_primary_core` 的测试入口），调用 `handle_switch_provider(app, app_state, uuid).await`，断言 `token.is_latest(g1)` 不变（switch 不碰 translation gen）。rev-17 也保留 rev-16 的结构性 `include_str!` grep 测试。**（rev-18-3 SUPERSEDED：活动测试改为 `#[test]`（非 `#[tokio::test]`）+ 调用 SYNC `handle_switch_provider_core(&app_state, &uuid)` — 无 `.await`、无 AppHandle — 对真实 temp DB，见 rev-18-3。）**
  2. **Step 11 测试数 31 → 32：** rev-16 的 Step 11 文本写 "the **31** `tray_state` tests pass"，但 Step 2/Step 7 实际枚举是 32。rev-17 修正为 32。
  3. **clear_translation_error 删除：** rev-16-1 列出 `clear_translation_error(&mut self, gen: u64)` 作为一个方法，但它从未被调用（`finish_translation` 已合并 clear 逻辑）。rev-17 从方法列表、接口表、覆盖矩阵中删除它。
  4. **PulseWorker send+join 确定性测试：** rev-16 的 `stop_signal_joins_the_worker` / `drop_stops_the_worker` 测试只断言 `stop()` / `drop` 返回（不挂死）。rev-17 增强为：测试验证 `PulseEvent::Stopped` 被发出后 `join()` 返回（确定性）—— 即在 `notify` channel 上 `recv_timeout` 收到 `PulseEvent::Stopped`，再断言 worker handle 已 join（确定性而非"测试完成即通过"）。

**rev-18 changes (定点修订 — A5 Tray 状态机核心: handle_switch_provider 拆为 core + wrapper 两层; 保留 rev-17 的架构方向: PulseWorker channel / parking_lot::Mutex / switch_revision / gen 保护 / latest_translation_gen / PulseEvent 枚举; rev-9-1/2/3、rev-11~rev-17 的非 A5 内容、其他任务保持不变):**

- **rev-18-1 (P1-1 handle_switch_provider 拆为 core + wrapper 两层).** rev-17-1 把 `handle_switch_provider` 改成 `pub async fn`（与 `set_active_primary_core` 同步签名冲突），而 rev-17-1 之前所有版本都试图用**一个函数**同时承担两件事：(a) DB + tray controller 操作（纯逻辑，可测试），(b) AppHandle 的 tray 视觉刷新（`refresh_tray_if_available`，需要 Tauri 运行时）。测试这一单函数就必须构造 `tauri::AppHandle`（`tauri::test::mock_app` / `build_test_app_handle`）—— 但当前 `Cargo.toml` 无 tauri test feature，集成测试 crate 无法可靠拿到 mock AppHandle。**rev-18 把它分成两层：**
  - **`handle_switch_provider_core(app_state: &Arc<AppState>, uuid: &str) -> Result<(), String>`** —— 纯同步核心：只操作 DB（`set_active_primary_core`）+ tray controller（`begin_switch` / `finish_switch`）。**不接触 `AppHandle`**（不调 `refresh_tray_if_available`、不碰 icon/tooltip/menu）。功能测试可直接调用此函数，**不需要 mock AppHandle / `tauri::test::mock_app` / `build_test_app_handle`**。
  - **`handle_switch_provider(app: &tauri::AppHandle, app_state: &Arc<AppState>, uuid: &str) -> Result<(), String>`** —— wrapper：调用 core + 用 AppHandle 做 tray 视觉刷新（`refresh_tray_if_available`）。tray handler（`handle_tray_menu_event` 的 `tray.switch-<uuid>` arm）通过 `tauri::async_runtime::spawn_blocking` 调用此 wrapper。
  ```rust
  /// 纯同步核心：只操作 DB + tray controller。
  /// 不接触 AppHandle（不需要 refresh_tray / tooltip / icon）。
  /// 测试可以直接调用此函数（不需要 mock AppHandle）。
  pub fn handle_switch_provider_core(app_state: &Arc<AppState>, uuid: &str) -> Result<(), String> {
      let rev = {
          let mut c = app_state.tray.lock();
          c.begin_switch()
      };
      // set_active_primary_core 签名不变: (Arc<AppState>, String) — owned Arc + owned String，
      // 因为它内部 spawn_blocking 需要 owned（见 A4 Step 9）。
      let result = set_active_primary_core(app_state.clone(), uuid.to_string());
      let mut c = app_state.tray.lock();
      c.finish_switch(rev, result.is_ok());
      result.map(|_| ()).map_err(|e| e.to_string())
  }

  /// 包装函数：调用 core + 用 AppHandle 做 tray 视觉刷新。
  /// tray handler 通过 spawn_blocking 调用此函数。
  pub fn handle_switch_provider(
      app: &tauri::AppHandle,
      app_state: &Arc<AppState>,
      uuid: &str,
  ) -> Result<(), String> {
      let result = handle_switch_provider_core(app_state, uuid);
      // 用 AppHandle 刷新 tray（menu/icon/tooltip）
      let _ = refresh_tray_if_available(app);
      if result.is_err() {
          log::warn!("switch provider failed: {:?}", result);
      }
      result
  }
  ```
  tray handler 中（`tray.switch-<uuid>` arm），**spawn_blocking 调 wrapper（不是 async spawn）**：
  ```rust
  "tray.switch-<uuid>" => {
      let app_clone = app.clone();
      let app_state = app.state::<Arc<AppState>>().inner().clone();
      let uuid = /* 从子菜单 id 解析 */;
      tauri::async_runtime::spawn_blocking(move || {
          let _ = handle_switch_provider(&app_clone, &app_state, &uuid);
      });
  }
  ```
  - **硬性约束：** (1) core 函数纯同步、无 AppHandle；(2) wrapper 函数用 AppHandle 做 refresh；(3) 测试调 core（不需要 mock AppHandle）；(4) `set_active_primary_core` 签名不变（owned `Arc<AppState>` + owned `String`，见 A4 Step 9）；(5) `spawn_blocking` 调 wrapper（不是 async spawn）。
  - 所有 rev-17-1 引入的 `pub async fn handle_switch_provider` / `handle_switch_provider(...).await` / `.await set_active_primary_core` 提法全部删除，替换为 core + wrapper 两层模型。
- **rev-18-2 (P1-2 controller_with_notify 构造函数初始化 notify_tx 字段).** rev-17-2 给 `TrayStateController` 加了 `notify_tx: Option<mpsc::Sender<PulseEvent>>` 字段，但 `with_renderer_interval_and_notify` 之外的构造路径（`new`、`with_renderer`、`with_renderer_and_interval`）必须全部传 `None`，否则缺字段编译失败。rev-18 确认 `with_renderer_interval_and_notify` 的 `Self { ... notify_tx, ... }` 已正确初始化（Step 5 代码块已含 `notify_tx,`），且三个委托构造都传 `None`：`new` → `with_renderer_interval_and_notify(..., None)`；`with_renderer` → `with_renderer_interval_and_notify(..., None)`；`with_renderer_and_interval` → `with_renderer_interval_and_notify(..., None)`。无字段缺失。
- **rev-18-3 (P1-3 功能测试调用 handle_switch_provider_core —— 不需要 AppHandle/mock).** rev-17 P2-1 的 `switch_handler_does_not_call_gen_next` 测试构造了 `build_test_app_state()` / `build_test_app_handle()`，但这两个 helper 在计划中并未定义，且 `tauri::test::mock_app` 在当前 `Cargo.toml` 无 tauri test feature 下不可用。**rev-18-1 的 core + wrapper 分离彻底解决此问题：** 功能测试调用 `handle_switch_provider_core(&app_state, &uuid)`（**不传 AppHandle**），用现有的 `tests/recovery.rs::Harness` fixture 模式（临时 DB + AppState 构建，见 L42-54）—— `tempfile::tempdir()` + `Database::open(&db_path)` + `db_providers::create(...)` 插入一个真实 provider + `AppState { ... }` 字面量含 `tray` 字段。断言：(1) DB 中 `primary_uuid` 被正确更新（`db_providers::read_active_selection` 读出 `primary == Some(uuid)`）；(2) tray controller 状态正确（成功 → `switch_error_rev() == None` + `current_state() == Normal`；失败路径 —— 不存在的 uuid —— → `switch_error_rev() == Some(rev)` + `current_state() == Error`）；(3) 翻译 generation 不变（`token.is_latest(g1)` 仍 true，switch 不碰 `GenerationToken`）；(4) 不使用 mock controller，**不构造 AppHandle**，不依赖 `tauri::test::mock_app`。`set_active_primary_core` 的调用方式为 `set_active_primary_core(app_state.clone(), uuid.to_string())`（owned 参数，因为内部 spawn_blocking 需要 owned）。
- **rev-18-4 (P1-4 PulseWorker notify_for_thread clone 避免 dead_code).** rev-17-2 的 `PulseWorker::start` 中 `let notify_for_thread = notify.clone();` 在 worker 闭包内被 move，每个 tick `tx.send(PulseEvent::Tick)`，退出时 `tx.send(PulseEvent::Stopped)`。若 prod `notify` 是 `None`，worker 闭包内 `if let Some(tx) = notify_for_thread.as_ref()` 分支不执行，但 `notify_for_thread` 变量本身已 `move` 进闭包（在闭包内被 `.as_ref()` 引用），不产生 dead_code warning。rev-18 确认此点（`notify_for_thread` 在闭包体中被读取，clippy 不报 dead_code）。
- **rev-18-5 (P1-5 测试确定性改进).** rev-17 的 3 个 PulseWorker 测试用 `let _ = recv_timeout` 忽略结果，不够确定性。rev-18 改为：
  - `stop_signal_joins_the_worker`: 先 `recv_timeout` 收到一个 `PulseEvent::Tick`（用小 interval 确认 worker 在运行），然后 `worker.stop()`，再 `recv_timeout` 收到 `PulseEvent::Stopped`（match，不是 `let _`）。
  - `second_begin_does_not_churn_the_worker`: 用 worker 的 start count 或 controller 内部 id 验证（不是帧数比较）—— 通过断言 `c.is_pulsing()` 保持 true + 第二次 `begin_translation` 后 `current_state()` 仍是 `ActiveTranslation`（recompute 不切换 worker）；帧数观察保留但用 `match`/`expect` 而非 `let _`。
  - `drop_stops_the_worker`: 断言收到 `PulseEvent::Stopped`（match，**不是** `Disconnected` —— Stopped 是显式信号，Disconnected 是 Sender drop 的副作用，前者确定性更高）。
- **rev-18-6 (P1-6 删除所有 rev-16 残留的 record_switch_error/clear_switch_error 引用).** rev-17-4 删除了方法定义，活动代码中已无引用。rev-18 确认 `record_switch_error`/`clear_switch_error` 在活动代码（Step 5 模块定义、Step 10 调用点、Step 2 测试、接口表、覆盖矩阵）中 **0 处**（只在历史 changelog 中作为"已删除"说明，这是正确的 changelog 用途，保留）。

**rev-18 P2 修正.**
1. **33 测试数确认：** Step 2 代码块中 grep `^#\[test\]$` + `^#\[tokio::test\]` 的实际计数是 33（rev-17 枚举正确）。rev-18 保留 33（rev-18-3 重写 functional test 调 core 仍在原位，不改总数）。
2. **stop/join 和 recv/join 的确定性序列：** 3 个 PulseWorker 测试（rev-18-5）用 `match recv_timeout { Ok(PulseEvent::Tick) => ... | Ok(PulseEvent::Stopped) => ... | other => panic!(...) }`（不是 `let _ = recv_timeout`）。
3. **`let _` 改为 `expect` 或 `match`：** rev-18-3 重写的 functional test 中所有 `recv_timeout`/DB 查询用 `match`/`expect`/`unwrap_or_else(panic!)`（不忽略结果）。
4. **grep 确认 `handle_switch_provider` 不含 async pattern：** Step 2 的结构性 `include_str!` 测试（`switch_arm_source_has_no_gen_next_call`）扩展断言 switch arm 不含 `.await`、不含 `spawn(async move`、不含 `pub async fn handle_switch_provider`（rev-18-1 同步化 + core/wrapper 分离回归保护）。注意：arm 内含 `spawn_blocking(move || handle_switch_provider(...))`（wrapper）或 `handle_switch_provider_core(...)`（core）是预期的（同步 spawn_blocking 调 wrapper 是正确模式；core 是纯同步可测试入口）。
5. **fixture 0 sync-core await 残留：** rev-18-3 的 fixture + `handle_switch_provider_core` 调用路径中无 `.await`（同步 fn），无 `#[tokio::test]`（改为 `#[test]`），**无 AppHandle 构造**（core 不需要）。
6. **recv_timeout 确定性验证：** 所有 `recv_timeout` 用 50ms 超时 + `match` 显式分支（rev-18-5）。

**rev-19 changes (定点修订 — A5 Tray 状态机 5 个修正; 保留 rev-18 的全部架构方向: core+wrapper 两层 / spawn_blocking / PulseEvent / parking_lot::Mutex / switch_revision / gen 保护 / latest_translation_gen / recv_timeout match 分支; 不修改冻结设计文档; rev-9-1/2/3、rev-11~rev-18 的非 A5 内容、其他任务保持不变):**

> **rev-19 是定点修订。** 全部 5 个 P1 修正 + 3 个 P2 修正针对 A5 的测试 fixture、`PulseWorker` 字段、no-churn 断言、switch 失败 tooltip、`tray.switch-<uuid>` 子菜单。rev-18 的 core+wrapper 两层架构、SYNC `set_active_primary_core`、`spawn_blocking` offload、`PulseEvent` 枚举、`parking_lot::Mutex`、`switch_revision`/`switch_error_rev`、gen 保护、`recv_timeout` `match` 分支等全部保留不变。

**已核实事实驱动 rev-19（全部对当前源码核实，非推测）：**
- `Database::open(path)` (`db/mod.rs:93`) — 只 `Connection::open(path)` + 设置 pragma（`foreign_keys`/`busy_timeout`/`journal_mode`/`synchronous`），**不创建任何表**。直接在 open 后调 `db_providers::create` 会报 "no such table: providers"。
- `schema::create_all_tables(conn: &Connection) -> Result<(), DbError>` (`db/schema.rs:30`) 和 `schema::seed_singletons(conn: &Connection) -> Result<(), DbError>` (`db/schema.rs:165`) 是**独立函数**，需要显式调用。
- 现有测试 `tests/provider_crud.rs:21-34` 的 `fresh_db()` 模式（**已核实**）：
  ```rust
  fn fresh_db() -> (tempfile::TempDir, Database) {
      let dir = tempdir().unwrap();
      let db_path = dir.path().join("test.db");
      let db = Database::open(&db_path).unwrap();
      db.with_conn(|conn| {
          let tx = conn.transaction()?;
          schema::create_all_tables(&tx)?;
          schema::seed_singletons(&tx)?;
          tx.commit()?;
          Ok(())
      }).unwrap();
      (dir, db)
  }
  ```
- `db_providers::create(conn: &mut Connection, template_id: &str, name: &str, endpoint: &str, model: Option<&str>) -> Result<ProviderProfile, DbError>` (`db/providers.rs:357`) — 第一个参数是 `&mut Connection`（不是 `&Arc<Database>`），在 `db.with_conn(|conn| db_providers::create(conn, ...))` 闭包内调用。
- `db_providers::read_active_selection(conn: &Connection) -> Result<ActiveSelection, DbError>` (`db/providers.rs:664`) — 读 `primary: Option<String>`。

- **rev-19-1 (P1-1 helper controller_with_notify 必须传 Some(notify_tx)).** rev-18 的 `controller_with_notify` helper（Step 2 代码块）已经正确传 `notify_tx`（一个 owned `Sender`，不是 `Option`）给 `with_renderer_interval_and_notify(..., notify_tx)`。但 `with_renderer_interval_and_notify` 的第 4 个参数签名是 `notify_tx: Option<mpsc::Sender<PulseEvent>>`，传一个裸 `Sender` 不编译（类型不匹配 `Sender` vs `Option<Sender>`）。**rev-19 修正：** helper 显式构造 `let controller = TrayStateController::with_renderer_interval_and_notify(renderer, Locale::En, Duration::from_millis(2), Some(notify_tx));`（`Some(notify_tx)`，不是裸 `notify_tx`）。同时 Step 2 中所有直接调用 `PulseWorker::start(..., Some(notify_tx))` 的地方保持 `Some(notify_tx)`（已经是 `Option` 形式，正确）。**rev-18-2 仍成立：** `new`/`with_renderer`/`with_renderer_and_interval` 三个委托构造都传 `None`（默认无 notify）。无字段缺失。
- **rev-19-2 (P1-2 DB fixture 必须先 create_all_tables + seed_singletons).** rev-18-3 的功能测试 `switch_handler_does_not_call_gen_next` 用 `Database::open(&db_path)` 打开 DB 后**直接** `db_providers::create(...)` —— 但 `Database::open` 不创建表（已核实 `db/mod.rs:93`），所以 `create` 会报 `no such table: providers`，测试在第一条断言前就 panic。**rev-19 修正：** fixture 改用现有 `tests/provider_crud.rs:21-34` 的 `fresh_db()` 模式 —— `Database::open` 之后在 `db.with_conn(|conn| { let tx = conn.transaction()?; schema::create_all_tables(&tx)?; schema::seed_singletons(&tx)?; tx.commit()?; Ok(()) })` 先建表 + 种子，**然后**才 `db_providers::create`。Step 2 代码块（section 10 的 fixture）完整改写为 `fresh_db` 模式 + 在同一个 DB 上 `create` 一个真实 provider。`db_providers::create` 的调用方式改为 `db.with_conn(|conn| db_providers::create(conn, "custom", "Test Provider", "http://localhost:11434", None))`（`&mut Connection` 第一参数，已核实 `db/providers.rs:357`）。
- **rev-19-3 (P1-3 PulseWorker struct 不持有 notify 字段).** rev-18 的 `PulseWorker` struct 持有 `notify: Option<std::sync::mpsc::Sender<PulseEvent>>` 字段（rev-17-2 引入），worker 线程持有一个 clone。但 worker 退出后线程的 Sender 被 drop，而 struct 自身的 `notify` 字段从未被读取（`stop()`/`Drop` 只用 `stop_tx` + `handle`）—— clippy `dead_code` 在 prod（`notify = None`）路径下会告警 "field `notify` is never read"。**rev-19 修正（推荐方案）：** `PulseWorker` struct **不持有 `notify` 字段**：
  ```rust
  pub struct PulseWorker {
      stop_tx: std::sync::mpsc::Sender<()>,
      handle: Option<std::thread::JoinHandle<()>>,
  }
  ```
  notify Sender **只** move 进 worker 线程闭包（worker 内部 clone 后 `send`），struct 只持有 `stop_tx` + `handle`。worker 退出时线程自然 drop 自己的 notify Sender（test 的 receiver 收到 `Disconnected` 或先收到显式 `PulseEvent::Stopped`）。`PulseWorker::start` 签名不变（仍接受 `notify: Option<Sender<PulseEvent>>` 参数），只是不再存到 struct 字段；`start` 内 `let notify_for_thread = notify;`（不 clone，直接 move 进闭包 —— struct 不持有，所以无需 clone）。`stop()`/`Drop` 不变（只用 `stop_tx` + `handle`）。**rev-18-4 仍成立但简化：** 不再有 struct 字段被忽略的问题（struct 根本没有 notify 字段）。Step 5 模块代码 + Interfaces (produces) PulseWorker 代码块同步更新。
- **rev-19-4 (P1-4 worker_start_count 验证 no-churn).** rev-18-5 的 `second_begin_does_not_churn_the_worker` 用 `is_pulsing()` + `current_state()` + 帧数比较验证 Active→Active 不换 worker，但 `is_pulsing()` 只能证明 "有一个 worker 在跑"，不能区分 "同一个 worker" vs "换了一个新 worker"；帧数比较（`frames_after > frames_before`）是 timing-sensitive（CI flake 风险）。**rev-19 修正：** 给 `TrayStateController` 加一个 `worker_start_count: u32` 字段（每次 `recompute` 在 `new_state == ActiveTranslation` 分支里 `PulseWorker::start` 之后 `+= 1`），暴露 `pub fn worker_start_count(&self) -> u32` 访问器。测试断言第二次 `begin_translation` **不增加** `worker_start_count`（同一个 worker）：
  ```rust
  c.begin_translation(1);
  let count_after_first = c.worker_start_count();
  assert_eq!(count_after_first, 1, "first begin started exactly one worker");
  c.begin_translation(2); // Active → Active, recompute 不进 new_state==Active 分支
  assert_eq!(c.worker_start_count(), count_after_first, "second begin did NOT churn the worker");
  ```
  `worker_start_count` 是单调递增计数器（只增不减），用于测试断言 no-churn。Step 5 的 struct 字段表 + `recompute` 实现 + 访问器 + Step 2 的 `second_begin_does_not_churn_the_worker` 测试同步更新。`controller_with_notify` helper 构造的 controller `worker_start_count` 初始为 0。
- **rev-19-5 (P1-5 wrapper 失败时设置 tray tooltip) — tooltip 内容被 rev-21-2 SUPERSEDED（顺序模型保留，前缀由 rev-21-2 添加）.** rev-18 的 wrapper `handle_switch_provider` 在失败时只 `log::warn!`，没有设置 tray tooltip —— 用户在 switch 失败时看不到任何文字反馈（只有 controller 的 red dot 视觉）。**rev-19 修正：** wrapper 失败时设置 tray tooltip 为错误信息（**rev-21-2 SUPERSEDED：** tooltip 内容从原始 `msg` 改为带前缀的 `format!("Switch failed: {msg}")`；下列 rev-19-5 代码块的顺序模型 —— refresh 在前、tooltip 在后 —— 仍为活动模型，但 tooltip 文本需加 `"Switch failed: "` 前缀，活动代码见 A5 Step 10 rev-21-2 代码块）：
  ```rust
  // rev-21-2: SUPERSEDED the tooltip TEXT — the order (refresh → tooltip) is the
  // active model, but the tooltip is now format!("Switch failed: {msg}"), NOT the
  // raw msg. The block below is the rev-19-5 historical form (raw msg) — see A5
  // Step 10 for the rev-21-2 active code.
  pub fn handle_switch_provider(
      app: &tauri::AppHandle,
      app_state: &Arc<AppState>,
      uuid: &str,
  ) -> Result<(), String> {
      let result = handle_switch_provider_core(app_state, uuid);
      if let Err(ref msg) = result {
          // rev-19-5: switch 失败时设置 tray tooltip 为错误信息（用户可见反馈）。
          // DB rollback 时 primary 不变（set_active_primary_core 的 tx 保证）。
          // rev-21-2: tooltip 改为 format!("Switch failed: {msg}")（带前缀）。
          if let Some(tray) = app.tray_by_id("main-tray") {
              let _ = tray.set_tooltip(Some(msg.as_str())); // rev-21-2: SUPERSEDED → format!("Switch failed: {msg}")
          }
      }
      // 成功或失败都 refresh_tray（成功：新 primary 显示在状态项；失败：tooltip 已设）
      let _ = refresh_tray_if_available(app);
      if result.is_err() {
          log::warn!("switch provider failed: {:?}", result);
      }
      result
  }
  ```
  `tauri::Manager::tray_by_id` 已在 A4 `refresh_tray` 中使用（`main-tray` id）；`TrayIcon::set_tooltip<S: AsRef<str>>(&self, Option<S>) -> crate::Result<()>` 是已核实的 Tauri 2 API（rev-11 verified API facts）。`refresh_tray_if_available` 在 tooltip 设置**之后**调用（成功路径刷新菜单显示新 primary；失败路径 tooltip 已设，refresh 重建菜单但 tooltip 已被显式 set 覆盖错误信息 —— 注意 `refresh_tray` 内部也会 `set_tooltip`，所以失败路径的 tooltip 必须在 refresh **之前** set，或 refresh 失败时不覆盖。实际顺序：失败时先 set tooltip，再 refresh；若 refresh 成功会重置 tooltip 为 `read_primary_status` —— **因此 rev-19-5 的 tooltip 设置放在 refresh 之后**，确保失败 tooltip 不被 refresh 覆盖）。**最终顺序：** core → 若 err 且 tray 存在则 refresh（重建菜单）→ 若 err 则 set tooltip（覆盖 refresh 的 tooltip）→ log。Step 10 代码块按此最终顺序更新（**rev-21-2：tooltip 文本为 `format!("Switch failed: {msg}")`**）。

**rev-19 P2 修正.**
1. **tray.switch-<uuid> 子菜单动态生成：** A4 Step 9 的 `build_switch_provider_submenu` 已经从 db 读取 enabled providers 并为每个创建 `MenuItem::with_id(app, &format!("tray.switch-{uuid}"), &name, true, None::<&str>)?`（已核实计划代码块 L2344-2354）；`handle_tray_menu_event` 中 `if let Some(uuid) = id.strip_prefix("tray.switch-")` 提取 uuid（已核实 L2461）。**rev-19 确认 A4 Step 9 的子菜单是动态生成的**（不是固定单项），并明确：`read_enabled_providers(app)` 从 db 读 `(uuid, name)` 列表（filter `enabled == true`），每个 provider 一个 menu item；`refresh_tray_if_available` 在 provider mutation 后重建整个菜单（含子菜单），所以新建/删除/重命名 provider 后子菜单即时更新。A4 Step 9 无需改动，**rev-19 只在 A5 引用此行为时明确说明**（避免与 A5 的 switch handler 产生歧义）。
2. **brace-balanced code blocks：** rev-19 修订的所有 Rust 代码块（`fresh_db` fixture、`PulseWorker` struct、`worker_start_count` 字段 + 访问器 + `recompute`、`handle_switch_provider` wrapper）的 `{`/`}` 均正确闭合，markdown 围栏（```）配对。
3. **33 tests 确认：** rev-19 不新增/删除测试（只改 fixture + 断言 + struct 字段），测试数仍为 33（rev-17/rev-18 枚举正确）。`second_begin_does_not_churn_the_worker` 的断言从帧数比较改为 `worker_start_count` 比较，测试本身不增减。

**rev-20 changes (定点修订 — 4 个文档一致性修正 + 测试数审计; 保留 rev-19 的全部架构方向: core+wrapper 两层 / spawn_blocking / PulseEvent / parking_lot::Mutex / switch_revision / gen 保护 / latest_translation_gen / PulseWorker 无 notify 字段 / worker_start_count / fresh_db fixture / 33 tests; 不修改冻结设计文档; rev-9-1/2/3、rev-11~rev-19 的非 A5 内容、其他任务保持不变):**

> **rev-20 是定点修订。** 4 个修正全部针对文档/活动代码的一致性（File Structure 描述、grep 测试源码窗口、历史 changelog 的活动引用标注、A4 switch arm 与 A5 wrapper 模型对齐）。rev-19 的全部架构（core+wrapper / spawn_blocking / PulseEvent / parking_lot::Mutex / switch_revision / gen 保护 / PulseWorker struct 无 notify 字段 / worker_start_count / fresh_db fixture / 33 tests）原封不动。

- **rev-20-1 (P1-1 File Structure PulseWorker struct 删除 notify 字段).** rev-19-3 删除了 Step 5 实际代码中 `PulseWorker` struct 的 `notify` 字段（notify Sender 只 move 进 worker 线程闭包），但 File Structure（L490 接口描述）仍写为 `pub struct PulseWorker { stop_tx, handle, notify: Option<Sender<PulseEvent>> }` —— 描述与实际代码不一致。**rev-20 修正：** File Structure 的 `PulseWorker` 描述改为 `pub struct PulseWorker { stop_tx: std::sync::mpsc::Sender<()>, handle: Option<std::thread::JoinHandle<()>> }`（无 notify 字段），并标注 "rev-19-3: notify moved into worker thread closure — the struct NO LONGER has a `notify` field, avoiding `dead_code` when prod passes `notify = None`"。现在全文所有位置（Step 5 代码、Step 11 预期、File Structure 接口描述、rev-19-3 changelog、`lib.rs` re-export）一致：`PulseWorker` struct 只有 `stop_tx` + `handle`。
- **rev-20-2 (P1-2 删除 take(4096) 截断 — grep 断言用完整源码窗口).** Step 2 的 `switch_arm_source_has_no_gen_next_call` 结构性测试原先用 `src[switch_start..].chars().take(4096).collect()` 取一个 4096 字符窗口做 grep 断言 —— 但截断源码窗口可能导致 grep 断言在大型文件中假通过/假失败（如果 switch handler 被拆分/注释/重排到 >4KB 之外，断言会漏掉回归）。**rev-20 修正：** 删除 `.chars().take(4096)`，直接对从 switch arm 开始的完整源码切片断言：`let window: &str = &src[switch_start..];`（`&str` borrow，无 `String` collect，无 cap）。所有 grep 断言（`.gen.next()` / `session.gen` / `.await` / `spawn(async move` / `pub async fn handle_switch_provider`）对完整窗口生效。Step 11 标题同步加注 "rev-20-2 the structural grep test uses the FULL source window — no `take(4096)` cap"。测试数不变（仍为 33 —— 此修订不改测试数，只改测试内部的窗口处理）。
- **rev-20-3 (P1-3 标注 rev-17-1 async handle_switch_provider 为 SUPERSEDED).** rev-17 changelog（L130/L132/L155）保留了 `pub async fn handle_switch_provider(...)` 定义代码块（标注为历史 rev-17 引入），但虽标注为历史，仍有活动引用的可能误导 —— 读者可能误以为 async 是当前模型。**rev-20 修正：** (a) rev-17-1 标题加 "— SUPERSEDED by rev-18-1（仅作历史记录，活动代码不采用 async）"；(b) rev-17-1 代码块（`pub async fn handle_switch_provider` + `spawn(async move { ... .await })` arm）改为 Rust 注释格式（`// rev-20-3: SUPERSEDED by rev-18-1` + `// rev-17 (SUPERSEDED): pub async fn handle_switch_provider(...)`），明确这是历史记录、活动代码见 rev-18-1 的 core+wrapper；(c) rev-17-1 的"所有提法改为 async"那句加删除线 + "rev-18-1 推翻"标注；(d) rev-17 P2-1（functional switch test 用 `.await`）加 "SUPERSEDED by rev-18-3" 标注；(e) `tray_state.rs` 模块 doc-comment 中 rev-17 P1-1 条目加 "— SUPERSEDED by rev-18-1" + 说明 active 模型是 SYNC core+wrapper。**活动代码中 `pub async fn handle_switch_provider` 0 处**（只在历史标注/changelog 中，且全部带 SUPERSEDED 标记）。
- **rev-20-4 (P1-4 A4 switch arm 对齐 A5 spawn_blocking wrapper 模型).** A4 Step 9 的 `handle_tray_menu_event` switch arm 代码块原先用 `tauri::async_runtime::spawn(async move { ... spawn_blocking(...).await ... })`（嵌套 spawn + `.await`），与 A5 rev-18-1 的 SYNC wrapper 模型（`spawn_blocking(move || handle_switch_provider(...))`）矛盾。rev-18-1 的 changelog 文字已说明"A5 Step 10 supersedes this for the tray-state wiring"，但 A4 的代码块本身仍显示旧模式，读者可能按 A4 代码实现而得到与 A5 矛盾的结果。**rev-20 修正：** A4 Step 9 switch arm 代码块改为 A5 的 SYNC wrapper 模型 —— `tauri::async_runtime::spawn_blocking(move || { let _ = handle_switch_provider(&app_clone, &app_state, &uuid_owned); });`（无 `spawn(async move)`、无 `.await`、wrapper 是 A5 Step 10 定义的 sole entry，内部调 core + `refresh_tray_if_available` + 失败 tooltip）。代码块内注释明确指出 wrapper 的职责（core: DB + tray controller SYNC；wrapper: refresh + tooltip），并指向 A5 Step 10。A4 与 A5 现在一致：switch arm = `spawn_blocking` 调 SYNC wrapper。

**rev-20 测试数审计 (33 确认).** 全文活动测试数声明（File Structure L508 "Test count = 33"、Step 7 L5399 "PASS (33 tests)"、Step 11 L5581 "the **33** `tray_state` tests pass"、A5 task context L2993 "Test count = 33"、A5 Files L3001 "Test count = **33**"、Stage A checklist L5634 "33 tests"、test-design notes L4214 "Test count = 33"、模块 doc-comment "P2-2 bringing the authoritative count to 33" / "test count stays 33"）全部为 **33**。历史 changelog 中提及的 31/32（rev-15 的 27、rev-16 的 32、rev-17 P2-2 的 "31→32"）是演进过程记录，保留为历史。rev-20 不新增/删除测试（只改窗口处理 + 文档一致性），测试数仍为 33。

**rev-20 不变量 (必须遵守).**
- `PulseWorker` struct 在所有位置（Step 5 代码、Step 11、File Structure、changelog、re-export）只有 `stop_tx` + `handle`（无 `notify` 字段）。
- grep 测试不用 `take(4096)`（对完整源码窗口断言）。
- `async handle_switch_provider` / `spawn(async move { handle_switch_provider(...).await })` 在活动代码中 **0 处**（只在历史 changelog 中带 SUPERSEDED 标记）。
- 测试数 **33**（活动代码全部一致）。
- 不修改冻结设计文档（MASTER.md / handoff-manifest.md / pages/04 / pages/05）。
- A4 Step 9 switch arm 与 A5 Step 10 wrapper 一致：`spawn_blocking` 调 SYNC `handle_switch_provider`（无 `.await`、无嵌套 `spawn(async move)`）。

**rev-21 changes (定点修订 — 极小定点修复: 2 个 P1 + 3 个 P2 一致性确认; 保留 rev-20 的全部架构方向 + 不变量; 不修改冻结设计文档; rev-9-1/2/3、rev-11~rev-20 的非 A5 内容、其他任务保持不变):**

> **rev-21 是极小定点修订。** 修复用户审核笔记中的 5 个问题，全部针对活动代码的精度与一致性。rev-20 的全部架构（PulseWorker struct 无 notify 字段 / 无 take(4096) / async handle_switch_provider SUPERSEDED / A4 switch arm 对齐 A5 spawn_blocking wrapper / 33 tests）原封不动。

- **rev-21-1 (P1-1 grep 测试断言失败信息截断为前 500 字符 — 避免 `&src[switch_start..]` 输出整个文件).** rev-20-2 把 `switch_arm_source_has_no_gen_next_call` 的 grep 窗口从 `.chars().take(4096)` 改为 `&src[switch_start..]`（从 switch arm 到 EOF 的完整源码切片）—— 这让断言更精确（无截断假通过/假失败）。但断言失败信息格式为 `"... (found in:\n{window})"`，当 `window = &src[switch_start..]`（可能数千行）时，一次失败会把**整个 lib.rs 尾部**（从 switch arm 到 EOF）打印到测试输出 —— 不仅使 CI 日志爆炸，还可能在 handler 跨函数延伸或大括号不匹配时把无关代码当成 "found in" 证据误导调试。**rev-21 修正：** 保留 `let window: &str = &src[switch_start..];`（窗口本身不变 —— 仍是完整源码切片，断言的精确性来自对完整窗口的 grep），但所有 grep 断言的失败信息从 `(found in:\n{window})` 改为 `(first 500 chars of switch arm window: {preview})`，其中 `let preview = &window[..window.len().min(500)];`（500 字符足以定位 `tray.switch-` arm + `spawn_blocking` 调用，又不会把整个文件尾部塞进日志）。窗口截断只影响**失败时的诊断输出**，不影响断言的匹配范围（断言仍对完整 `window` grep）。Step 2 代码块的所有 5 个 grep 断言（`.gen.next()` / `session.gen` / `.await` / `spawn(async move` / `pub async fn handle_switch_provider`）同步更新失败信息格式。Step 11 标题同步加注 "rev-21-1 grep assertion failure messages truncated to first 500 chars of the switch-arm window"。测试数不变（仍为 33 —— 此修订不改测试数，只改失败时的诊断输出）。
- **rev-21-2 (P1-2 switch-provider wrapper tooltip 必须带 "Switch failed: " 前缀).** rev-19-5 的 wrapper 在失败分支设 `tray.set_tooltip(Some(msg.as_str()))` —— 直接用 `set_active_primary_core` 返回的原始错误 `msg`，**没有** `"Switch failed: "` 前缀。这导致用户在托盘 tooltip 上看到的是原始 DB 错误（如 `"provider not found"`），而非预期的格式化失败提示 `"Switch failed: provider not found"`（A4 Step 9 的 `handle_tray_menu_event` 契约明确意图是 `"Switch failed: <msg>"`，见 L1793 / L5581）。**rev-21 修正：** wrapper 失败分支改为带前缀的格式化 tooltip：
  ```rust
  if let Err(ref msg) = result {
      let tooltip = format!("Switch failed: {msg}");
      if let Some(tray) = app.tray_by_id("main-tray") {
          let _ = tray.set_tooltip(Some(&tooltip));
      }
      log::warn!("switch provider failed: {msg}");
  }
  ```
  `"Switch failed: "` 前缀是本修订的硬性约束（用户指定）。`log::warn!` 仍用原始 `msg`（日志不需要用户可见的前缀）；只有 tooltip 用格式化后的 `tooltip`。顺序保持 rev-19-5：refresh → set tooltip（前缀化的 tooltip 在 refresh 之后设置，不被 refresh 的 `set_tooltip` 覆盖）。A4 Surface 说明（L1793）中 "the switch-provider failure tooltip (A4's intent was `"Switch failed: <msg>"`)**rev-19-5 (A5 Step 10 wrapper) sets the raw error `msg` as the tooltip**" 同步修正为 "**rev-21-2 sets the prefixed tooltip `Switch failed: {msg}`**"。File Structure / 接口描述 / rev-19-5 changelog 中所有 "sets the raw error `msg` as the tooltip" / `tray.set_tooltip(Some(msg.as_str()))` 提法同步改为带前缀版本。
- **rev-21 P2 一致性确认 (3 项).**
  1. **P2-1 测试数 33 确认（非 32）：** rev-20 测试数审计已确认活动代码全部为 33。rev-21 复核：全文 grep `32` 出现在历史 changelog（rev-16/rev-17 的 "32 tests" 是演进记录，保留为历史）+ A5 Step 11 标题 "32 → 33" 的修正记录（L5586）—— **活动代码（File Structure L508 / Step 7 L5399 / Step 11 L5600 / A5 task context L2993 / A5 Files L3001 / Stage A checklist L5634 / test-design notes L4214）全部为 33**，无遗漏。rev-21 不改测试数（仍为 33），只在 Step 11 标题加注 "rev-21-3 test count reconfirmed as 33 (not 32)"。
  2. **P2-2 无 `.bak` 文件：** 已在外部删除（仓库中不存在 `*.bak` 文件）。rev-21 在 rev-21 不变量中记录 "仓库无 `.bak` 文件（已外部删除）"。本计划无 `.bak` 引用。
  3. **P2-3 无 `async handle_switch_provider` 在活动代码中：** rev-20-3 已标注 rev-17-1 的 async `handle_switch_provider` 为 SUPERSEDED，全部活动代码（A5 Step 10 core+wrapper / A4 Step 9 switch arm / File Structure / 接口表）为 SYNC `pub fn`。rev-21 复核：`pub async fn handle_switch_provider` 仅出现在历史 changelog（rev-17-1 代码块，带 SUPERSEDED + 注释标记），活动代码 0 处。rev-21 不改此（保持 rev-20-3 状态），只在 rev-21 不变量中再次确认。

**rev-21 不变量 (必须遵守).**
- grep 测试的**断言**对完整 `window = &src[switch_start..]` 生效（窗口不截断）；但**失败信息**只打印 `window` 的前 500 字符（`&window[..window.len().min(500)]`）。
- switch-provider wrapper 失败分支的 tooltip 必须带 `"Switch failed: "` 前缀（`format!("Switch failed: {msg}")`）；`log::warn!` 用原始 `msg`（无前缀）。
- 测试数 **33**（活动代码全部一致，无 32 残留）。
- `pub async fn handle_switch_provider` 在活动代码中 **0 处**（只在历史 changelog 中带 SUPERSEDED 标记）。
- 仓库无 `.bak` 文件（已外部删除）；本计划无 `.bak` 引用。
- 不修改冻结设计文档（MASTER.md / handoff-manifest.md / pages/04 / pages/05）。

**rev-22 changes (定点修订 — 极小定点修复: 4 个 P1, 全部针对 grep 结构性测试的精度与 UTF-8 安全; 保留 rev-21 的全部架构方向 + 不变量 + `"Switch failed: "` tooltip 前缀 + 33 tests + 无 .bak + 无 async; 不修改冻结设计文档; rev-9-1/2/3、rev-11~rev-21 的非 A5 grep-test 内容、其他任务保持不变):**

> **rev-22 是极小定点修订。** 修复 4 个 P1 问题，全部聚焦 `switch_arm_source_has_no_gen_next_call` 结构性 grep 测试（A5 Step 2）：(1) UTF-8 边界 panic、(2) grep 窗口不够精确、(3) core/wrapper 同步确认缺乏结构性断言、(4) 测试数 33 再次确认。rev-21 的全部架构（grep 窗口 = 完整 `&src[switch_start..]` / `"Switch failed: "` 前缀 tooltip / 33 tests / 无 .bak / 无 async handle_switch_provider）原封不动；rev-22 只改**该 grep 测试函数体内部**的 preview 截取方式与断言结构（窗口从「到 EOF 的整段」收窄为「大括号匹配提取的三个精确函数体」）。

- **rev-22-1 (P1-1 `&window[..window.len().min(500)]` 在 UTF-8 边界 panic — 改为 `chars().take(500)` 收集).** rev-21-1 的 preview 用 `let preview: &str = &window[..window.len().min(500)];` —— 这是**字节**切片。如果第 500 字节正好落在多字节 UTF-8 字符（如中文注释、中文 tooltip `"翻译中…"`、`"Switch failed: "` 后跟非 ASCII 的 provider 名称）中间，`&window[..500]` 会 panic `"byte index 500 is not a char boundary"`。该测试对 `include_str!("../src/lib.rs")` 的源码切片取 preview —— lib.rs 含中文注释（本计划多处要求中文注释），500 字节边界落在中文字符中间的概率非零，会导致一次本应通过的测试 panic 退出（而非断言失败）。**rev-22 修正：** preview 改为 `let preview: String = window.chars().take(500).collect();`（`chars()` 按 UTF-8 scalar value 迭代，`take(500)` 取前 500 个**字符**，`collect::<String>()` 重新编码为合法 UTF-8 —— 永不切断多字节字符，永不 panic）。preview 类型从 `&str` 变为 `String`（因为 `chars().take()` 的迭代器输出不保证连续字节，`collect` 是必须的；断言失败信息用 `{preview}` 格式化，`String` 实现了 `Display`，无需改动格式字符串）。失败信息仍为 `(first 500 chars of switch arm window: {preview})` 的语义（"chars" 而非 "bytes"）—— rev-22 将失败信息措辞从 "first 500 chars" 保持不变（"chars" 本就是字符语义，现在名实相符）。**不影响断言匹配范围**（断言仍对完整函数体断言；preview 只用于失败诊断输出）。

- **rev-22-2 (P1-2 `&src[switch_start..]` 到 EOF 不够精确 — 改为 `extract_function_body` 大括号匹配提取精确函数体).** rev-21-1 的 grep 窗口 `let window: &str = &src[switch_start..];` 从 `"tray.switch-"`（或 `handle_switch_provider`）首次出现处到文件**末尾** —— 可能跨越多个无关函数（switch arm 后面若紧跟其他菜单 arm、其他 `pub fn`、其他 `impl` 块，全部被纳入 window）。这导致：(a) 一个不相关的后续函数引入 `.await` / `spawn(async move` / `.gen.next()` 会**假失败**该测试；(b) 断言失败时 preview（即便 UTF-8 安全）可能指向无关函数，误导调试。**rev-22 修正：** 新增 `extract_function_body` 辅助函数，用大括号深度匹配提取单个函数从签名到闭花括号的精确源码片段：
  ```rust
  /// rev-22-2: extract a function body by its exact signature prefix. Walks the
  /// source from the signature's opening `{` tracking brace depth until back to
  /// 0; returns the slice from the signature start to (and including) the
  /// matching `}`. Panics if the signature or its `{` are not found.
  fn extract_function_body<'a>(src: &'a str, signature: &str) -> &'a str {
      let start = src.find(signature)
          .unwrap_or_else(|| panic!("expected `{signature}` in lib.rs"));
      let brace_offset = src[start..].find('{')
          .unwrap_or_else(|| panic!("expected `{{` after `{signature}`"));
      let brace_start = start + brace_offset;
      let mut depth = 0i32;
      let mut end = brace_start + 1; // default: include at least the opening brace
      for (i, ch) in src[brace_start..].char_indices() {
          match ch {
              '{' => depth += 1,
              '}' => {
                  depth -= 1;
                  if depth == 0 {
                      end = brace_start + i + 1;
                      break;
                  }
              }
              _ => {}
          }
      }
      assert!(depth == 0, "unbalanced braces in `{signature}` body");
      &src[start..end]
  }
  ```
  然后该 grep 测试对**三个精确提取的函数体**各自断言（替换原先对单一 `window` 的 grep）。`switch arm` 断言并入 `handler_body`（`handle_tray_menu_event` 的函数体覆盖 `tray.switch-` arm + `spawn_blocking` 调用）：
  ```rust
  let handler_body = extract_function_body(src, "fn handle_tray_menu_event(");
  let core_body    = extract_function_body(src, "pub fn handle_switch_provider_core(");
  let wrapper_body = extract_function_body(src, "pub fn handle_switch_provider(");
  // preview for diagnostics (rev-22-1: UTF-8-safe, 500 CHARS not bytes)
  let handler_preview: String = handler_body.chars().take(500).collect();
  ```
  大括号匹配是按字符迭代（`char_indices`，UTF-8 安全），不切断多字节字符；`{` / `}` 是 ASCII，不受 UTF-8 边界影响；字符串/字符字面量内的花括号（如 `format!("{{}}")`）可能干扰深度计数，但本仓库 lib.rs 的 switch-provider 函数体不含会让深度错乱的花括号字面量（已核实 handle_tray_menu_event / handle_switch_provider / handle_switch_provider_core 函数体无带花括号的字符串字面量）。

- **rev-22-3 (P1-3 确认 SYNC core + SYNC wrapper — 两个函数体各自断言不含 `.await`).** rev-21 / rev-20-3 仅在一个宽泛 `window` 上断言「无 `.await`」—— 但 rev-22-2 把窗口收窄为三个精确函数体后，断言可以更**精确地定位**到每个函数。**rev-22 修正：** 在 `core_body` 与 `wrapper_body` 上各自断言：
  ```rust
  // core_body: SYNC — no async pattern anywhere in its body
  assert!(
      !core_body.contains(".await"),
      "rev-22-3: handle_switch_provider_core must be SYNC (set_active_primary_core is SYNC) — no `.await` in its body (first 500 chars: {core_body.chars().take(500).collect::<String>()})"
  );
  assert!(
      !core_body.contains("session.gen") && !core_body.contains(".gen.next()") && !core_body.contains(".gen .next()"),
      "rev-22-3: handle_switch_provider_core must NOT acquire the translation GenerationToken (switch is decoupled from translation gen — rev-15 P1-3 / rev-16-1)"
  );
  // wrapper_body: SYNC `pub fn`, no async spawn, no .await
  assert!(
      !wrapper_body.contains("pub async fn"),
      "rev-22-3: handle_switch_provider must be `pub fn` (SYNC), not `pub async fn` (rev-18-1)"
  );
  assert!(
      !wrapper_body.contains(".await") && !wrapper_body.contains("spawn(async move"),
      "rev-22-3: handle_switch_provider wrapper must NOT `.await` or spawn an async task (rev-18-1 SYNC model)"
  );
  // handler_body (the tray.switch- arm): the arm dispatches via spawn_blocking(SYNC),
  // not spawn(async move { .await }); and the switch arm must not touch the translation gen.
  assert!(
      !handler_body.contains(".gen.next()") && !handler_body.contains(".gen .next()") && !handler_body.contains("session.gen"),
      "rev-22-3: the tray.switch- arm in handle_tray_menu_event must NOT call `.gen.next()` / acquire the translation GenerationToken (rev-16 P1-3 / rev-18-1)"
  );
  assert!(
      !handler_body.contains("spawn(async move"),
      "rev-22-3: the tray.switch- arm must NOT spawn(async move { ... .await }) — it uses spawn_blocking for a SYNC fn (rev-18-1)"
  );
  ```
  `core_body` 的 `session.gen` / `.gen.next()` 断言 + `wrapper_body` 的 `pub async fn` / `.await` / `spawn(async move` 断言 + `handler_body`（switch arm）的 `.gen.next()` / `session.gen` / `spawn(async move` 断言 —— 三者**各自独立**，任一函数体回归（重引入 gen / async / async spawn）都会被精确指名失败（不再是一个笼统的 "switch arm window"）。rev-21-2 的 `"Switch failed: "` tooltip 前缀（A5 Step 10 wrapper 失败分支）不在该 grep 测试断言范围内（wrapper 的 tooltip 代码是可编译的正确代码，不是被禁止的模式）；rev-22-3 不引入 tooltip 前缀的结构断言（tooltip 由 A4/A5 的功能与文档约束，不在此结构性 grep 测试）。

- **rev-22-4 (P1-4 测试数 33 确认 — 32 仅在历史 changelog).** rev-22 不新增/删除测试：`switch_arm_source_has_no_gen_next_call` 测试本身**保留**（只是重写函数体：新增 `extract_function_body` helper 是该测试函数**内部的局部 fn**，不增加 `#[test]` 计数；preview 改写、断言重写均不增减测试函数）。活动代码的测试数仍为 **33**（rev-20/rev-21 已审计）。`extract_function_body` 是 `switch_arm_source_has_no_gen_next_call` 函数体内的局部 helper（`fn extract_function_body(...)`，非 `#[test]`），不计入 `^#[test]$` grep 计数 —— rev-22 不改测试数。Step 11 标题加注 "rev-22-4 test count reconfirmed as 33 (extract_function_body is a local helper inside the grep test, not a new #[test])"。

- **rev-22 杂项 (tooltip / .bak / 空字符串 tooltip — 一致性确认, 不改代码).**
  1. **tooltip 前缀确认：** `format!("Switch failed: {msg}")` 在 wrapper 失败分支（rev-21-2 已落地，A5 Step 10 wrapper）—— rev-22 复核仍为活动代码（A5 Step 10 第 5600 行 `let tooltip = format!("Switch failed: {msg}");`）。`log::warn!("switch provider failed: {msg}")` 用原始 `msg`（无前缀），与 rev-21-2 一致。rev-22 不改此。
  2. **`.bak` 删除确认：** rev-21 已确认仓库无 `.bak` 文件；rev-22 复核仍无（工作树中 `find . -name '*.bak'` 为空）。本计划无 `.bak` 引用。
  3. **空字符串 tooltip 可接受性确认：** 若 `msg` 为空字符串（`set_active_primary_core` 返回 `Err("".to_string())` —— 实际不会发生，但理论可能），tooltip 为 `"Switch failed: "`（前缀 + 空 msg），**不是**空字符串 —— 前缀始终存在，用户至少看到 `"Switch failed: "` 的文字反馈（加上红点视觉信号）。rev-22 确认这是可接受行为（前缀是硬性约束，保证非空可读 tooltip）；不引入 `if msg.is_empty() { ... }` 分支（YAGNI —— 实际无空 msg 来源）。
  4. **`""` 引号确认：** 复核 A5 Step 10 wrapper 代码无意外的空字符串或引号问题（`format!("Switch failed: {msg}")` 是唯一的 tooltip 构造点；`log::warn!` 用原始 msg；无 `"Switch failed: "` 与 `"Switch failed: {msg}"` 混用）。

**rev-22 不变量 (必须遵守).**
- grep 测试的 preview 用 `window.chars().take(500).collect::<String>()`（UTF-8 安全，500 **字符** 非 500 字节，永不 panic）。
- grep 测试用 `extract_function_body(src, signature)` 大括号深度匹配提取**精确函数体**（不再用 `&src[switch_start..]` 到 EOF 的宽泛窗口）；对 `core_body` / `wrapper_body` / `handler_body`（switch arm）**三个函数体各自独立断言**。
- `extract_function_body` 是 `switch_arm_source_has_no_gen_next_call` 测试函数体内的**局部 helper fn**（非 `#[test]`，不计入测试数）。
- `handle_switch_provider_core` 与 `handle_switch_provider` 都是 SYNC `pub fn`（无 `.await`、无 `pub async fn`、无 `spawn(async move`）—— 在各自函数体上结构性断言。
- switch arm（`handle_tray_menu_event` 的 `tray.switch-` 分支）不含 `.gen.next()` / `session.gen` / `spawn(async move`。
- switch-provider wrapper 失败分支的 tooltip 必须带 `"Switch failed: "` 前缀（`format!("Switch failed: {msg}")`）；`log::warn!` 用原始 `msg`（无前缀）—— rev-21-2 不变量保留。
- 测试数 **33**（活动代码全部一致，无 32 残留；`extract_function_body` 局部 helper 不增计数）。
- `pub async fn handle_switch_provider` 在活动代码中 **0 处**（只在历史 changelog 中带 SUPERSEDED 标记）—— rev-20-3 不变量保留。
- 仓库无 `.bak` 文件（已外部删除）；本计划无 `.bak` 引用。
- 不修改冻结设计文档（MASTER.md / handoff-manifest.md / pages/04 / pages/05）。

**Contract documents (rev-8-9 governance, revised rev-12):** the four design documents (MASTER.md, handoff-manifest.md, pages/04-tray-menu.md, pages/05-provider-center.md) are FROZEN — this plan does NOT modify them. The design-doc differences are recorded in D3's retroactive table (within THIS plan only) and in the "审核快照（rev-12 用户已批准）" Surface status table at the top of this plan. Per rev-12, the frozen pages/04 red-dot requirement IS implemented this stage (`TrayStateController` `Error` state overlays a build-time-composited red-dot-on-base-icon PNG via A5); the frozen pages/04 Active-pulse requirement IS implemented this stage (`ActiveTranslation` state drives a real icon frame-switch pulse via a background timer + build-time-generated dimmed icon, NOT just a tooltip change); the frozen pages/04 Update-badge requirement is deferred to R5/R6 per user-approved scope decision (rev-11/rev-12). Any further scope reduction (OCR/History deferral, Balance placeholder, Update-badge deferral) is already user-approved (rev-11); a design-doc edit would require a separate proposal.

---

## Surface 审核快照（rev-12 用户已批准）

### Surface 04 (Tray Menu) — pages/04-tray-menu.md 状态矩阵

| 状态 | 决策 | 实现状态 | 任务 |
|---|---|---|---|
| Normal | — | ✅ 已实现 | A4 |
| Active translation (pulse) | **A（已批准，本阶段实现）** | ✅ 本阶段实现（rev-12：真实 icon 帧切换脉动 — 后台 timer 在 normal/dimmed 图标间切换，非仅 tooltip） | A5 + A4 接入 |
| Error (general, red-dot) | **A（已批准，本阶段实现）** | ✅ 本阶段实现（rev-12：红点叠加在 app 底图上 — build.rs 合成 `32x32.png` + 右上角 `#DC2626` 圆点，非纯红方块） | A5 + A4 接入 |
| Update available (badge) | **B（已批准延期至 R5/R6）** | ⏸️ 延期（依赖 R5/R6 updater） | — |

### Surface 05 (Provider Center) — pages/05-provider-center.md 状态矩阵

| 状态 | 决策 | 实现状态 | 任务 |
|---|---|---|---|
| Connection OK + latency | **A（已批准，本阶段实现）** | ✅ 本阶段实现（后端 latency_ms + 前端显示） | C3c（rev-11） |
| Balance loading/unsupported/rate-limited/error | **B（已批准延期至 R4/S3）** | ⏸️ 延期（后端无 balance IPC；前端显示 "Not available"） | — |

### 用户已批准的范围决策（rev-12 冻结）

以下决策在 rev-11 中由用户批准、在 rev-12 中细化实现方式，**不再待批准**：

1. **Tray Error 红点**：选项 A — 本阶段实现（Task A5 rev-12：`TrayStateController` `Error` 状态 + build.rs 合成红点 overlay PNG — 加载 `src-tauri/icons/32x32.png` 底图，在右上角画直径 ~10px 的 `#DC2626` 圆点，**不是**纯红方块）。
2. **Tray Active pulse**：选项 A — 本阶段实现（Task A5 rev-12：`TrayStateController` `ActiveTranslation` 状态驱动**真实 icon 帧切换脉动** — 后台 timer 每 800ms 在 normal 与 dimmed 图标间切换 + tooltip "Translating…"，**不是**仅 tooltip 变化）。
3. **Tray Update badge**：选项 B — 批准延期至 R5/R6（updater 后端不存在）。`TrayVisualState::UpdateAvailable` arm 在 enum 中保留（使优先级排序可测试）但本阶段**从不调用**。
4. **Connection latency**：选项 A — 本阶段实现（Task C3c rev-11/rev-12：`ConnectionResult.latency_ms: Option<u32>` + HTTP probe `Instant` 计时 + 前端显示 `{latency}ms`；rev-12 饱和转换 `u32::try_from(...).unwrap_or(u32::MAX)` + 测试验证实际计时）。
5. **Balance states**：选项 B — 批准延期至 R4/S3（balance IPC 后端不存在；前端保持 "Not available" 静态占位）。

以上决策已批准，冻结设计文档（pages/04、pages/05）仍不修改（A-path 的实现使 frozen 文档的要求在本阶段得到满足，而非通过修改文档来缩小范围）。

**Historical changelogs removed (rev-10).** The rev-5, rev-6, rev-7, and rev-8 changelog blocks have been removed from the plan body — the cumulative corrections they describe are folded into the current task steps and the "Verified code facts" below; the per-revision narrative is preserved in git history. Only the rev-9 corrections (rev-9-1/2/3 technical fixes, retained verbatim) and rev-9-4/rev-10 governance notes remain.

---

**Verified code facts (rev-7 basis):**
- **Monitor API (Tauri 2.11.5, verified at `tauri-2.11.5/src/window/mod.rs:78-104`):** `Monitor::work_area(&self) -> &PhysicalRect<i32, u32>` returns the real OS-reported usable area (accounts for menu bar / dock / task bar — NOT `Option`, NOT derived). `Monitor::scale_factor(&self) -> f64` returns `f64` directly (NOT `Result`). `Monitor::position()` + `Monitor::size()` return the physical rect. `PhysicalRect<P, S>` (`tauri-runtime-2.11.3/src/dpi.rs:28`) is `{ position: PhysicalPosition<P>, size: PhysicalSize<S> }`, so `PhysicalRect<i32, u32>` exposes `position.x: i32`, `position.y: i32`, `size.width: u32`, `size.height: u32`. `AppHandle::monitor_from_point(&self, x: f64, y: f64) -> tauri::Result<Option<Monitor>>` (f64, NOT i32). `WebviewWindow::scale_factor(&self) -> tauri::Result<f64>` (returns `Result` — the popup-window fallback uses `unwrap_or(1.0)`). There is NO `system_bar_deduction()` approximation.
- **SettingsShell (verified against the current `src/features/settings/SettingsShell.tsx`):** Props = `{ initialSection?: SettingsSection; onNavigate?: (section: SettingsSection) => void; children: JSX.Element }`. `SettingsSection = "provider-center" | "keystore-recovery" | "shortcuts" | "privacy"`. State (current, pre-edit): `const [active, setActive] = createSignal<SettingsSection>(props.initialSection ?? "provider-center")`. Root: `<div class="settings-shell" data-layout={wide() ? "full" : "rail"}>`. The component ALREADY renders `WindowChrome` + `SidebarItem`s + `Tooltip` (for disabled + rail items) + a `matchMedia("(min-width: 700px)")` responsive signal + lazy `getCurrentWindow().close()`/`minimize()` handlers. `navItems` has `provider-center` + `keystore-recovery` enabled and `shortcuts` + `privacy` disabled (`disabled: true` on the `SidebarItem`). **rev-9-2 (load-bearing):** a `createSignal` initializer runs ONCE — so `createSignal(props.activePage ?? ...)` would NOT track a parent-supplied `activePage` change. The plan (A4 Step 7 edit 2) therefore makes `active` a DERIVATION `() => props.activePage ?? internalActive()`, not a read-once signal, so the controlled `activePage` prop drives `active()` reactively.
- **InputPanel (verified against the current `src/InputPanel.tsx`):** calls `invoke("translate_session", ...)` and owns `text`/`state`/`idle`/`hasResult` signals. `TranslationState` discriminant union (verified at `src/features/translation/types.ts:65`): `loading | single-success{text,engine} | multi-success{results} | partial{results} | error{sub,message} | offline | no-selection | no-permission | keystore-corrupt`. `ResultEntry` (types.ts:51) = `{ uuid: string; engine: string; text?: string; errorText?: string; ok: boolean }` — `engine` REQUIRED, failure field is `errorText` (NOT `error`). There is NO `idle` kind. Uses `decodeSessionResult`, `ResultCard` (`outcome` + `engineLabel` + `text` + `errorText`), `InlineError`.
- **KeystoreRecovery (verified against the current `src/features/settings/KeystoreRecovery.tsx`):** `KsState = "healthy" | "corrupt" | "archived"`. `ToastEntry = { id, variant: "info"|"success"|"warning"|"destructive", message }`. Signals: `state`, `reason`, `resetOpen`, `busy` (`"archive"|"reset"|null`), `toasts`. Handlers: `handleArchive` (invoke `archive_keystore`), `handleResetConfirm` (invoke `reset_keystore`). UI: a destructive `Banner` (state=corrupt) with `title`+`description`+`action` (an `<span>` holding Archive + Reset `<Button>`s), an info `Banner` (state=archived), a destructive `Confirm` (`resetOpen`, initial focus Cancel, `triggerRef={resetTriggerRef}`), and a `Toast` stack (`aria-live="polite"`). `resetTriggerRef` restores focus on Confirm close. Copy from `SETTINGS_COPY[locale].keystore`.
- **Tray + provider commands (verified against the current `src-tauri/src/lib.rs`):** there are EIGHT provider mutation commands — `provider_create(state, template_id, name, endpoint, model)`, `provider_update(state, uuid, patch)`, `provider_delete(state, uuid)`, `provider_reorder(state, uuids)`, `provider_toggle(state, uuid, enabled)`, `provider_set_active(state, primary, parallel, fallback)`, `provider_duplicate(state, uuid)` (lib.rs:1135), `provider_confirm_and_set_active(state, primary, parallel, fallback, expected_scope)` (lib.rs:1377) — ALL take `state: tauri::State<'_, Arc<AppState>>` and NONE take an `AppHandle`. Each clones `let app = state.inner().clone();` (an `Arc<AppState>`) and runs the write inside `tauri::async_runtime::spawn_blocking(move || { ... })`. `provider_set_active` writes via `set_active_slots` (lib.rs:1711) or `set_active_slots_keep_consent` (lib.rs:1892); `SetActiveOutcome` (lib.rs:1672) = `Written | NeedsConsent{actual_scope}`. `provider_confirm_and_set_active` returns `Result<i64, ProviderCommandError>` (lib.rs:1383) — the typed error carries `StaleScope{actual_scope}` (lib.rs:1635) so the frontend can re-prompt; it writes via `write_consented_selection` + the `ConfirmActiveOutcome` (lib.rs:1681) = `Written{version} | StaleScope{actual_scope}`. `TrayIcon::set_menu<M: ContextMenu>(&self, menu: Option<M>) -> crate::Result<()>` takes `Option<M>` (`tauri-2.11.5/src/tray/mod.rs:512`) → call is `tray.set_menu(Some(menu))`. `tauri::Manager::tray_by_id(&self, id: &str) -> Option<TrayIcon>` updates an existing tray in place (the tray id is `"main-tray"`). `Database::with_conn<T>(&self, f: impl FnOnce(&mut Connection) -> Result<T, DbError>) -> Result<T, DbError>` (db/mod.rs:109) — the closure returns `Result<T, DbError>` (enum at db/mod.rs:39, aliased `DbErr` at lib.rs:1606), NOT `rusqlite::Error`.
- **ui-lab paths (verified):** `apps/ui-lab/src/App.tsx` → `src/` is `../../../src/...` (three `../`). The `@linguaray/ui-lab` package (port 1421) already has a Playwright config, a `?nav=`/`?theme=`/`?state=` router, fixture pages under `apps/ui-lab/src/pages/`, and the visual script `test:visual`. There is NO `src/ui-lab/`. **rev-8-2 (verified):** the `@app` alias is ALREADY configured — `apps/ui-lab/vite.config.ts:16` (`"@app": fileURLToPath(new URL("../../src", import.meta.url))`) + `apps/ui-lab/vitest.config.ts:19` (same) + `apps/ui-lab/tsconfig.json:21` (`"@app/*": ["../../src/*"]`). All ui-lab fixture imports use `@app/...` (NOT relative `../../../src/...`).
- **on_hotkey / translate pipeline (verified, retained):** `on_hotkey` (lib.rs) allocates `gen = state.gen.next()` synchronously, then under `state.gen.selection_lock()` reads cursor+selection, checks `state.gen.is_latest(gen)` after capture, and on success calls the session. `translate_with_fallback_ref` (service.rs) converts a local `FallbackEligible` to `LocalNoFallback` and a non-local `FallbackEligible` with `fallback=None` to `LocalNoFallback`. `is_local` matches `localhost | 127.0.0.1 | ::1 | 0.0.0.0`. `GenerationToken`: `next() -> u64`, `is_latest(gen: u64) -> bool`, `selection_lock() -> MutexGuard`. Test pattern (test/ProviderCenter.test.tsx): `vi.hoisted` + `invokeMock` + `routeInvoke`.
- **Banner (verified at `packages/ui/src/components/Banner.tsx:7`):** `BannerProps = { variant, title, description?, action?, onDismiss?, dismissLabel?, class? }` — NO `icon`, NO `children`.

---

**Goal:** Close A-path contract gaps surfaced by the R2/R3a audit this stage; B-path items (Update badge, Balance states) are deferred per user-approved scope decision (rev-11) so the four shipped surfaces (selection popup, input window, system tray, settings shell) match their design contracts: theme bootstrap on every entry, the Alt+Space hotkey drives the multi-engine session with generation-token staleness guards, the popup clamps to the work area (in the cursor's monitor's scale) in unified units and never leaks `secret_ref`, the input window persists drafts, the settings shell reads active selection at cold-start, the tray reflects every provider mutation and shows Normal/Active/Error states, and the parallel path returns stable order with bounded, local-sacred-aware fallback that can actually trigger.

**P1 work items (summary; full detail in the stages below):**
- **P1-1 (capture_and_translate + generation token + multi-monitor scale):** the helper takes a `gen: u64` token and checks `state.gen.is_latest(gen)` at every await boundary; `on_hotkey` passes its synchronously-allocated `gen` through. `build_popup_anchor` resolves the cursor's monitor via `monitor_from_point` and uses THAT monitor's `scale_factor()` for the work-area + cursor conversion (rev-7-1).
- **P1-2 (geometry unified units):** `PopupAnchor { cursor_logical, work_area, scale_factor }` is the single source of geometry; `set_popup_mode` recomputes both size and position per mode.
- **P1-3 (Retry always has the source):** loading + error + result + multi payloads carry `source_text`; the popup controller saves `lastSource` on all of them; a new session clears stale `lastSource`; Retry only appears when `lastSource` is non-empty; clipboard translate saves the raw clipboard text.
- **P1-4 (fallback eligibility can hit):** `translate_primary_only` runs the primary only and preserves the original `Error`; `eligible_for_session_fallback` scans for a non-local `FallbackEligible` with a local-sacred-aware mixed rule; fallback fires once via `translate_with_fallback_ref`.
- **P1-5 (Tray Normal/Active/Error states implemented; Update badge deferred):** `set_active_primary_core` is the extracted sync core; full `SubmenuBuilder`/`MenuItem::with_id` menu; status item reads the primary name from db; the tray refreshes after EVERY provider mutation (rev-7-8 per-command `refresh_tray_if_available`); `navigate` event drives `SettingsShell` via the controlled `activePage` prop; **rev-15 (Task A5):** a pure-Rust `TrayStateController` reducer drives the Normal/Active/Error icon+tooltip states with concurrency-safe counting — all methods SYNC (`active_translations: u32` counter + `error_gen: Option<u64>` generation-tagged error (translation flow only) + `latest_translation_gen: u64` (rev-17-3 — newest begin_translation gen, gates `record_translation_error`) + `switch_revision: u64`/`switch_error_rev: Option<u64>` (switch-provider flow, **rev-16-3: replaces rev-15's sticky `has_error: bool` with revision-tagged switch errors to avoid concurrent-switch completion reordering**) + `current_state: TrayVisualState` + `recompute()` resolves `Error > Active > Normal` and only switches the `PulseWorker` when `new_state != current_state`; `UpdateAvailable` retained in enum but never activated this stage — deferred to R5/R6). The controller is held in `Arc<parking_lot::Mutex<TrayStateController>>` on `AppState` (rev-14/rev-15: synchronous `parking_lot::Mutex`, NOT `tokio::sync::Mutex` — so `TranslationGuard::drop` runs `finish_translation` SYNCHRONOUSLY on the calling thread, restoring the true RAII guarantee). `Error` overlays a build-time-**composited** red-dot-on-base-icon PNG (`src-tauri/icons/32x32.png` + top-right `#DC2626` ~10px dot — NOT a solid-red square). `ActiveTranslation` drives a **real icon frame-switch pulse** via a `PulseWorker` — **rev-15 P1-1 / rev-17-2: a `std::thread` background worker every 800ms toggles the renderer between dimmed/normal, and exits via an `mpsc` channel signal (`PulseWorker::stop` does `stop_tx.send(())` + `handle.join()` — the worker's `recv_timeout` returns on the signal so `join` completes; NO infinite-loop + join deadlock). The worker emits `PulseEvent::Tick` per frame + `PulseEvent::Stopped` before exit (rev-17-2 — was `send(())`). Leaving `Active` drops the worker (`Option::take()` → `PulseWorker::drop` → `stop()`).** `capture_and_translate` + `translate_clipboard` use the `TranslationGuard` (gen-tagged `finish_translation`); **rev-16 P1-1 / rev-17-4: NO function overloading + NO dead switch mutators** — the controller methods are `record_translation_error(gen)` (translation, rev-17-3 `latest_translation_gen`-guarded) / `begin_switch()`/`finish_switch(rev, success)` (switch), each with a distinct name (Rust does not support overloading; **rev-17-4: `record_switch_error()`/`clear_switch_error()` DELETED — `finish_switch` is the sole switch mutator**). **rev-16 P1-3 / rev-18-1: switch-provider does NOT touch the translation `GenerationToken`** — switch uses `begin_switch()` → `finish_switch(rev, success)` (a monotonic switch revision independent of `Session.gen`); stale switch results (`rev != switch_revision`) are ignored. **rev-18-1: `handle_switch_provider` is `pub fn` (SYNC)** (rev-17-1's `async` was based on the wrong premise that `set_active_primary_core` was async — it is SYNC; rev-18-1 reverts to the SYNC `pub fn`); the tray.switch arm runs it via `tauri::async_runtime::spawn_blocking` (offload the SYNC SQLite I/O — NOT `spawn(async move { ... .await })`). The translation flow's `error_gen` and the switch flow's `switch_error_rev` are independent flags the reducer ORs.
- **P1-6 (permissions + clipboard plugin):** `Cargo.toml` gets `tauri-plugin-clipboard-manager = "2"` before capabilities reference it; `.plugin(...)` registration; `build.rs` + capability JSONs list every new command; an integration test asserts the capability set.
- **P1-7 (tests use the verified fixture pattern):** every new frontend test uses `vi.hoisted` + `invokeMock` + `routeInvoke`; the default mock includes `provider_get_active_selection`.
- **P1-8 (no invented backend contracts; rev-11 adds latency as an approved additive field; rev-12 hardens the conversion):** `provider_test_connection` returns `{ ok, message, latency_ms? }` (rev-11: `latency_ms: Option<u32>` is now a real additive field, set on the reachable path via `Instant::now()`/`elapsed()`, `None` on early-return failures; **rev-12:** the `as_millis() as u32` truncation is replaced with `u32::try_from(elapsed.as_millis()).unwrap_or(u32::MAX)` saturation, and `connection_latency.rs` gains a test asserting `latency_ms` reflects the actual `Instant` probe result rather than a hardcoded constant); `provider_get_balance` still does not exist (Balance UI is a static placeholder; Balance states deferred to R4/S3 per user-approved scope decision).
- **P1-9 (C5/C6 real keyboard tests):** Playwright against a `?nav=settings-keyboard` ui-lab route; disabled items are `aria-disabled="true"` + `tabindex={0}` (rev-7-6 CSS `:focus` locator).
- **P1-10 (D5 screenshots cover real surfaces):** each surface has a stable fixture/route; the InputPanel + Keystore fixtures reuse the production `InputPanelView`/`KeystoreRecoveryView` (rev-7-3/rev-7-4 complete; rev-7-7 correct field names); 600/699/700/800 × light/dark; tray = manual capture.
- **P2 (document fixes):** dark canvas token `#020617`; `theme-color` meta activation disables the non-current scheme. Task count (rev-12): 22 `### Task` + 8 `#### Task` = 30 headings; excluding the C3 umbrella = 29 executable tasks (rev-12 keeps the same task count as rev-11 — the A5 fixes are within-task refinements, not new tasks; rev-12-5/rev-12-6 add tests/steps to existing C3c/A5 tasks).

**Architecture:** Four sequential checkpoints (A/B/C/D). Stage A re-grounds the main path, theme, popup geometry, tray wiring, and permissions. Stage B fulfills the Surface 01-04 contracts (popup operations, input autosave, parallel ordering, fallback bounds). Stage C completes Surface 05-06 and the settings shell. Stage D cleans aliases, fixes the test runner, captures visual baselines across all real surfaces, and runs the final verification sweep. Each stage is a hard checkpoint — stop after the final task in each stage and run that stage's verification block before continuing.

**Tech Stack:** Rust 1.77 + Tauri 2 + `tauri::Manager`/`Emitter`/`WebviewWindow` (backend), SolidJS 1.9 + `@tauri-apps/api` 2 + Vitest 4 + `@solidjs/testing-library` + Playwright (visual baselines) (frontend), `@linguaray/ui` workspace package (token CSS + components).

---

## Global Constraints

These apply to every task. Each task's requirements implicitly include this section.

- **Semantic tokens only.** Production `src/` MUST consume colors/spacing/radius/shadow via CSS variables (`--color-*`, `--core-space-*`, `--radius-*`, `--shadow-*`) defined by `@linguaray/ui/styles`. No hardcoded hex may be introduced anywhere under `src/`. The existing `test/no-hardcoded-hex.test.ts` scan covers `src/**/*.css|tsx|ts`; new and edited files MUST pass. Hex inside a `var(--token, #fallback)` fallback slot is the only permitted form. The `<meta name="theme-color">` values in `index.html` are meta-tag content, not `src/` CSS, and are exempt. **P2:** the dark canvas token value is `#020617` (rev-4 corrects the rev-3 `#0B1120`); the light token is `#F8FAFC`. These two values are the only allowed theme-color contents.
- **theme-color meta activation (rev-5-6).** `index.html` ships BOTH `<meta name="theme-color" media="(prefers-color-scheme: light)" content="#F8FAFC">` and `<meta name="theme-color" media="(prefers-color-scheme: dark)" content="#020617">`. After `initTheme()` resolves the scheme, the CURRENT meta's `media` is set to `"all"` (so it ALWAYS applies — even when the user FORCED a theme that disagrees with the OS preference) and its `content` is re-asserted to the resolved token. The NON-current meta is set to `media="disabled"` so it never overrides. This avoids the OS-chrome flicker AND the rev-4 bug where OS Light + forced Dark left the dark meta at `media="(prefers-color-scheme: dark)"` (no match) + the light meta disabled → no meta applied.
- **Old space aliases are forbidden in new `src/` files.** `--space-1`/`--space-2`/`--space-3` (the bare numeric aliases) are legacy. New code uses the formal semantic tokens: `--space-xs` (=2px), `--space-sm` (=4px), `--space-md` (=8px), `--space-lg` (=12px), `--space-xl` (=16px), `--space-2xl` (=24px) — all defined in `packages/ui/src/styles/tokens.css:202-207` as aliases over `--core-space-*`. Task D1 sweeps the existing occurrences; a guard test (added in D1) blocks regressions.
- **Backend IPC contract is additive.** New Tauri commands MUST be registered in BOTH the `invoke_handler!` list at `src-tauri/src/lib.rs` AND the `tauri_build` commands list at `src-tauri/build.rs` (the latter generates the permission manifest / ACL). Existing command signatures, event names (`popup-state`, `popup-multi-result`, `tray-action`, `navigate`), and wire shapes (`ProviderProfile`, `TranslateSessionResult`, `TranslationOutcomeSerialized`) are the source of truth — do NOT alter them EXCEPT for the additive `source_text` field on `Payload`/`PopupMultiPayload`/`PopupStatePayload` (B4, P1-3), which is optional and backward-compatible. **A dedicated task (A4) covers the permission layer; every backend command added in any task MUST also land its `build.rs` entry, its permission TOML, and its capabilities allow-entry inside that task, not deferred.** Permission identifiers are `allow-<command-name>` (kebab-case), auto-generated by `tauri_build`. **P1-6:** an integration test (`src-tauri/tests/capabilities.rs`) asserts the capability set parses and contains every required permission — not just a grep.
- **Sanctioned dependency: clipboard plugin (frozen, no fallback).** `tauri-plugin-clipboard-manager` (Cargo) + `@tauri-apps/plugin-clipboard-manager` (npm) are the ONLY sanctioned additions. They are REQUIRED — the `navigator.clipboard` fallback is deleted (P1-6). **P1-6 ordering:** the Cargo dep + `.plugin(...)` registration land in A4 (BEFORE any capability references the plugin), and the npm dep lands in B4 Step 0. There is no "keep navigator.clipboard" branch.
- **No invented backend contracts (P1-8, rev-11 latency amendment, rev-12 saturation hardening).** C3 adds ZERO new backend commands. `provider_test_connection` returns `{ ok: bool, message: String, latency_ms: Option<u32> }` — the `latency_ms` field is a rev-11 user-approved additive field (set on the reachable HTTP-200 path via `Instant::now()`/`elapsed()`; `None` on early-return failures) and the UI displays `message` plus `{latency}ms` when `latency_ms` is present. **rev-12:** the `as_millis() as u32` cast is replaced with `u32::try_from(elapsed.as_millis()).unwrap_or(u32::MAX)` (saturation; clippy-clean for `cast_possible_truncation`), and `connection_latency.rs` gains a test asserting `latency_ms` reflects the actual `Instant` probe result. `provider_get_balance` does NOT exist; the Balance UI shows a static "Balance check not yet available" string and invokes nothing (Balance states deferred to R4/S3 per user-approved scope decision, rev-11).
- **Stage checkpoints.** Stages A, B, C, D each end with a "Stage X Verification" block. Do not start the next stage until that block passes. Commit after every step as written.
- **TDD ordering.** Every task writes the failing test first, runs it to confirm RED, implements, runs to confirm GREEN, then commits. "RED" means the test demonstrably fails for the intended reason before the implementation exists — never accept "it might already pass" as a RED. A vacuous pass (e.g. a menu that never opens so the assertion is never reached) is NOT a valid RED — fix the selector before counting it.
- **No `*ForTest` naming.** Helpers exposed for tests use the natural public name. Rust test-only surface uses `#[cfg(test)]` inline modules or `tests/` integration tests. The two pre-existing `_for_test` items are KEPT (renaming would expand the blast radius); no NEW `*ForTest` names are introduced.
- **Every code block is compilable (rev-4 load-bearing, refined rev-9).** Every Rust function signature, type, and API call is verified against the current crate; every TSX block is valid TSX against the current imports. There is NO `/* ... */`, NO `// ... existing`, NO "adapt to the API", NO "adjust its signature", NO "verify ... add if missing", NO "or wherever X is matched", NO pseudocode anywhere in this plan. Every Rust block is valid Rust against the current crate; every TSX block is valid TSX against the current imports. Where a whole helper is reproduced it is COMPLETE and compilable as written; where existing code is edited in place, the plan uses a precise `diff`-style instruction (rev-8-1 / rev-9-2).
- **Test code uses the verified `vi.hoisted + invokeMock + routeInvoke` pattern.** Every frontend test in this plan that mocks `invoke` uses the `vi.hoisted` + `invokeMock` + `routeInvoke` shape already in `test/ProviderCenter.test.tsx`. No bare `vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn(...) }))` at file scope without the hoisted indirection, no `mockImplementation` that throws "unexpected invoke" for routes the default mock should satisfy.
- **Verification commands (load-bearing).** Every `cargo` command in this plan carries `--features xproc-test-helper`:
  - Build: `cargo build --manifest-path src-tauri/Cargo.toml --features xproc-test-helper`
  - Test: `cargo test --manifest-path src-tauri/Cargo.toml --features xproc-test-helper`
  - Clippy: `cargo clippy --manifest-path src-tauri/Cargo.toml --features xproc-test-helper --all-targets -- -D warnings`
  - The frontend harness is `pnpm test` (root vitest) once D2 fixes `test:src`/`test:all`. `git diff --check` runs before every commit (whitespace/conflict-marker guard).
- **Commits use explicit file lists.** No `git add -A`, no `git add .` anywhere in the plan. Every commit lists the exact files, INCLUDING autogenerated permission TOMLs. Task D4 Step 0 adds a guard that verifies `git diff --cached --name-only` excludes `.mimosa/`, `dist/`, and `test-results/` before each commit, using `grep -E '(^|/)(\.mimosa|dist|test-results)(/|$)'`.
- **Task order.** Stage A runs A1 → A3 → A2 → A4 → **A5 (rev-11/rev-12)**. A3 (geometry) must land before A2 (hotkey) because A2's `on_hotkey` rewrite depends on `set_popup_mode`/`show_at_sized`/`compute_popup_geometry_logical`. A3 is self-contained: it ships a source-aware loading emitter so A2 can depend on A3 without waiting on B4. A4 (permissions + tray) lands last so every command the frontend tests invoke is already authorized. **A5 (rev-11/rev-12: tray visual-state controller — `TrayStateController` reducer)** lands AFTER A4 because A4 establishes the `main-tray` id + `build_tray` + the executable Switch Provider submenu that A5's controller looks up via `app.tray_by_id("main-tray")`; A5 also extends A4's `capture_and_translate`/`translate_clipboard` rewrite + A4's switch-provider handler with `begin_translation`/`end_translation`/`set_error` calls on the shared `Mutex<TrayStateController>`.
- **Task count = 29 executable (rev-12).** A(5) + B(6) + C(2 + 8 + 3 = 13) + D(4 + 1 screenshot = 5). The count is 22 `### Task` headings + 8 `#### Task` sub-headings = 30 total task headings, minus the C3 umbrella (`### Task C3`, which is not itself executable) = 29 executable tasks: A1, A2, A3, A4, **A5 (rev-11/rev-12)**, B1, B2, B3, B4, B5, B6, C1, C2, C3a, C3b, C3c, C3d, C3e, C3f, C3g, C3h, C4, C5, C6, D1, D2, D3, D4, D5. rev-12 does NOT change the task count — the 4 P1 fixes + 2 P2 fixes are refinements WITHIN Task A5 (and one P2 touch-up in C3c), not new tasks.

---

## File Structure

Map of every file this plan touches. New files are marked **(new)**; modified files cite the function/region changed.

**Frontend (`src/` + root):**

- **(new)** `src/theme.ts` — `initTheme()` bootstrap; resolves scheme, motion, locale; manages BOTH `theme-color` metas (disables the non-current one, P2).
- `src/popup-entry.tsx` — call `initTheme()` before `render`.
- `src/input-entry.tsx` — call `initTheme()` before `render`.
- `src/index.tsx` — call `initTheme()` + add `import "@linguaray/ui/styles"`.
- `index.html` — fix default title/favicon; theme-color meta with BOTH `media="(prefers-color-scheme: light)"` (`#F8FAFC`) and dark (`#020617`) variants (P2).
- `src/App.tsx` — add the `tray-action` listener + a `navigate` listener; `translate-selection` calls `translateSelection`; `switch-provider` opens Settings on the provider page.
- `src/Popup.tsx` — friendly engine labels (no `secret_ref`); Copy uses Tauri clipboard with Copied feedback; Retry calls `ctrl.retrySelection()` which re-translates the saved SOURCE text; settings/recovery CTAs; TTS/Favorite `aria-disabled` (focusable) not native `disabled`.
- `src/InputPanel.tsx` — multi-engine rendering with friendly engine labels; autosave/restore; Clear purges draft; auto-focus; disabled while loading.
- `src/Popup.css` — replace `--space-1/2/3` aliases with `--space-sm/md/lg` (D1).
- `src/App.css` — no change (already token-clean).
- `src/features/translation/popupController.ts` — load provider name map on mount; expose `engineLabel(uuid)`; save the SOURCE text from loading/error/result/multi payloads (P1-3); `retrySelection()` re-translates the saved source; new session clears `lastSource`.
- `src/features/translation/inputController.ts` **(new)** — InputPanel's friendly-label map + `engineLabel`, mirroring the popup controller.
- `src/features/translation/types.ts` — extend `PopupStatePayload` + `PopupMultiPayload` with optional `source_text?: string`; add `selection.action.*` copy keys.
- `src/features/translation/copy.ts` — add the new copy strings (zh + en).
- `src/features/translation/selection-ipc.ts` **(new)** — `translateSelection(sourceText)` + `translateClipboard()` distinct frontend entries.
- `src/features/settings/ProviderCenter.tsx` — drop the `google`/`deepl` entries from `PRESETS`; read active selection on cold load; fail-closed on read error; connection test shows `message` + `{latency}ms` when `latency_ms` is present (rev-11); balance shows "not yet available" placeholder (deferred to R4/S3 per user-approved scope decision).
- `src/features/settings/SettingsShell.tsx` — **rev-8-1 + rev-9-2: precise `diff`-style edits to the EXISTING component** (Props gains optional controlled `activePage?: SettingsSection`; rev-9-2 makes `active` a DERIVATION `() => props.activePage ?? internalActive()` — NOT a read-once `createSignal` initializer — so a parent-supplied `activePage` updates reactively; `handleClick` writes the internal signal ONLY when `props.activePage === undefined`; `renderItem` passes `ariaLabel = item.disabled ? \`${item.label} — ${t.nav.placeholderHint}\` : item.label`; root gains `data-testid="shell"` + `data-page={active()}`; disabled items go through the updated `SidebarItem` for `aria-disabled` + `tabindex={0}`). `WindowChrome`, `Tooltip`, the `matchMedia` responsive rail, and the `close`/`minimize` handlers are all KEPT. The existing `SettingsSection` is the single union (no new `SettingsPage` type).
- `src/features/settings/provider-ipc.ts` — add `providerGetActiveSelection()` wrapper.
- `src/features/settings/provider-types.ts` — add `ActiveSelectionFE` mirror type.
- **(new)** `apps/ui-lab/src/pages/InputPanel.tsx` + `KeystoreRecovery.tsx` + `apps/ui-lab/e2e/surfaces.visual.spec.ts` + `apps/ui-lab/e2e/keyboard.spec.ts` (D5/C5) — extend the EXISTING `apps/ui-lab` workspace package (port 1421) with InputPanel + Keystore fixtures + a Playwright visual baseline suite + the keyboard spec. **rev-7-3/rev-7-4/rev-8-2:** the fixtures import the production `InputPanelView`/`KeystoreRecoveryView` via the `@app` alias (configured at `apps/ui-lab/{vite,vitest}.config.ts` + `tsconfig.json` → `../../src`); the keyboard spec uses the `.settings-shell__nav .sidebar-item:focus` locator (rev-8-4). The popup fixtures already exist as `apps/ui-lab/src/pages/SelectionPopup.tsx`.
- **(new)** `test/theme.test.ts` — `initTheme` sets `data-theme`/`data-motion`/`lang` + theme-color meta activation.
- **(new)** `test/popupGeometry.test.ts` — pure-function geometry sanity (logical→physical, work_area clamping, margin 8px).
- **(new)** `test/tray-action.test.tsx` — App listens + dispatches; OCR disabled; `translate-selection` calls `translateSelection`; `switch-provider` emits `navigate`; menu-item count.
- `test/Popup.test.tsx` — extend with friendly-label, Copy-feedback, Retry-uses-saved-SOURCE-text, settings-nav, recovery-CTA, TTS/Favorite-aria-disabled.
- `test/InputPanel.test.tsx` — extend with multi/partial/all-failed, friendly-label, autosave/restore, Clear-purges, auto-focus, disabled-while-loading.
- `test/ProviderCenter.test.tsx` — extend with cold-start active-selection read (success + fail-closed), 4-preset-only, plus all C3 sub-task tests; the default mock includes `provider_get_active_selection`.
- `test/SettingsShell.test.tsx` — extend with rail-mode accessible-name (rev-8-5: driven by `installMatchMedia(false)`, NOT `window.innerWidth`), disabled aria-label (rev-8-5: asserts the REAL `/Coming in R3b/` copy, NOT `/coming later/`), controlled activePage via navigate event, a11y status + Re-check + focus re-check.
- `test/no-hardcoded-hex.test.ts` — add the alias-ban guard (D1).
- **(new)** `test/no-space-alias.test.ts` — D1 guard that new `src/` files do not use `--space-[0-9]`.
- **(new)** `apps/ui-lab/e2e/surfaces.visual.spec.ts` — Playwright `toHaveScreenshot` at 600/699/700/800 × light/dark across real surfaces (D5).

**Backend (`src-tauri/`):**

- `src-tauri/src/lib.rs`:
  - `run_translate_session` (lib.rs:492) — resolve `to: ""` to `settings::load(app).target_language` at the TOP of the fn.
  - `on_hotkey` (lib.rs:1982) — replace the capture+translate block with a call to the new shared `capture_and_translate` helper, passing the synchronously-allocated `gen`.
  - **(new)** `async fn capture_and_translate(app: &AppHandle, state: &Arc<Session>, app_state: &Arc<AppState>, supplied_text: Option<String>, x: f64, y: f64, gen: u64) -> ()` — extracted from `on_hotkey` (the selection_lock + capture_selection + settings + session + decision + per-state emit block). Shared by hotkey, tray `translate-selection`, and Retry. Checks `state.gen.is_latest(gen)` at every await boundary (rev-9-1: including after the `spawn_blocking` that acquires the db Arc; the gate guard is taken + dropped INSIDE the blocking closure, mirroring `translate_clipboard`).
  - **(new)** `#[tauri::command] async fn translate_selection_ipc(app, state, app_state, text: Option<String>) -> Result<(), ()>` — Selection path. `text = Some(t)` (Retry) skips capture and uses the saved SOURCE; `text = None` (tray) runs fresh capture via `capture_and_translate`. DISTINCT from `translate_clipboard` (never reads the clipboard).
  - **(new)** `#[tauri::command] async fn provider_get_active_selection(state) -> Result<ActiveSelection, String>`.
  - **(new)** `#[tauri::command] fn open_settings_window(app, section: Option<String>) -> Result<(), String>` — emits `navigate` with the section so `SettingsShell` can switch pages.
  - **(new)** `fn set_active_primary_core(app_state: Arc<AppState>, uuid: String) -> Result<SetActiveResult, String>` — the sync core of `provider_set_active`, reused by the tray (P1-5).
  - Register `translate_selection_ipc`, `provider_get_active_selection`, `open_settings_window` in BOTH the `invoke_handler!` list AND `build.rs`.
  - `build_tray` (lib.rs:2157) — full Switch Provider submenu (P1-5); OCR/History disabled "Coming later"; status item reads the primary provider name; refresh hook after provider mutations.
  - **(new, rev-14 / rev-15 / rev-16 / rev-17 / Task A5; sync parking_lot controller + PulseWorker(channel-quit) + std::thread timer)** `src-tauri/src/tray_state.rs` — pure-Rust tray visual-state controller. **rev-16/rev-17 model:** a `pub struct TrayStateController { active_translations: u32, error_gen: Option<u64>, latest_translation_gen: u64 (rev-17-3), switch_revision: u64, switch_error_rev: Option<u64>, current_state: TrayVisualState, pulse_worker: Option<PulseWorker>, tick_interval: Duration, renderer: Arc<dyn TrayRenderer>, notify_tx: Option<mpsc::Sender<PulseEvent>> (rev-17-2), locale: Locale }` reducer (does NOT derive `Debug` — holds `Arc<dyn TrayRenderer>`; **rev-16-3: `has_error: bool` REPLACED by `switch_revision: u64` + `switch_error_rev: Option<u64>`** so concurrent switch completions are ordered by revision, not a race-prone bool; **rev-17-3: adds `latest_translation_gen: u64`** so a stale OLDER gen's late error is ignored after a newer gen began); **rev-15/rev-17-2/rev-19-3: `PulseWorker` is a `pub struct PulseWorker { stop_tx: std::sync::mpsc::Sender<()>, handle: Option<std::thread::JoinHandle<()>> }` (rev-19-3: notify moved into worker thread closure — the struct NO LONGER has a `notify` field, avoiding `dead_code` when prod passes `notify = None`; rev-17-2: the notify Sender passed to `PulseWorker::start` carries `PulseEvent`, was `()`) whose worker thread loops on `stop_rx.recv_timeout(interval)` — `Ok(())`/`Err(Disconnected)` → emit `PulseEvent::Stopped` + return (exit), `Err(Timeout)` → toggle `dimmed` + `renderer.set_icon_dimmed/normal()` + `notify.send(PulseEvent::Tick)` (rev-16 P2-1 / rev-17-2: per-tick notification so tests can deterministically wait for N frames via `recv_timeout` instead of `thread::sleep`). `PulseWorker::stop(&mut self)` = `stop_tx.send(())` + `handle.take().join()` (the worker returns from `recv_timeout` on the signal so `join` completes — NO infinite-loop + join deadlock). `impl Drop for PulseWorker { fn drop(&mut self) { self.stop(); } }`.** `TrayStateController` holds `pulse_worker: Option<PulseWorker>`: entering `Active` → `pulse_worker = Some(PulseWorker::start(renderer.clone(), interval, notify))`; leaving `Active` → `pulse_worker.take()` (Drop → stop). `recompute()` resolves `new_state` and ONLY when `new_state != current_state` swaps the worker + updates `current_state` + `render()` (`UpdateAvailable` retained in the `TrayVisualState` enum for priority ordering but NEVER activated this stage — deferred to R5/R6). SYNC methods (rev-16-1: NO overloading — distinct names; rev-17-4: NO dead switch mutators): translation-flow `begin_translation(gen)` (rev-17-3: bumps `latest_translation_gen`)/`finish_translation(gen, success)` (gen-guarded clears, rev-16-2)/`record_translation_error(gen)` (gen-guarded set, rev-16-2 + `latest_translation_gen` guard rev-17-3); switch-flow `begin_switch()`/`finish_switch(rev, success)` (rev-16-3: NO gen arg, uses `switch_revision`; **rev-17-4: `record_switch_error()`/`clear_switch_error()` DELETED — finish_switch is the sole switch mutator**). `pub fn tray_state_priority(state) -> u8` (pure, testable: `Normal`=0 < `ActiveTranslation`=1 < `UpdateAvailable`=2 < `Error`=3). `Error` overlays a build-time-**composited** red-dot-on-base-icon PNG (loaded `src-tauri/icons/32x32.png` + top-right ~10px `#DC2626` dot — NOT a solid-red square) via `tauri::image::Image::from_bytes(include_bytes!(concat!(env!("OUT_DIR"), "/tray-error-32.png")))`; `ActiveTranslation` drives a **real icon frame-switch pulse** via `PulseWorker` (worker holds an independent `Arc<dyn TrayRenderer>`; rev-15 P1-4: the ONLY timer model in the plan — no `visual_epoch`, no `tick_render()`, no in-timer controller lock); PLUS a localized tooltip (`"Translating…"`/`"翻译中…"` via `tray_tooltip_text(state, locale)`); `Normal` restores `app.default_window_icon()` + drops the worker (→ `PulseWorker::drop` → `stop()`). The controller is stored in `Arc<parking_lot::Mutex<TrayStateController>>` on `AppState`.
  - **(rev-11 / Task C3c; rev-12 saturation)** `ConnectionResult` (lib.rs:1449) — gains `pub latency_ms: Option<u32>` (serde-serialized); `provider_test_connection` (lib.rs:1507) wraps the reachable-path `client.get(...).send().await` in `let start = std::time::Instant::now();` ... **rev-12:** `let latency_ms = Some(u32::try_from(start.elapsed().as_millis()).unwrap_or(u32::MAX));` (saturation, clippy-clean; was `as u32` in rev-11) — set ONLY on the `Ok(resp)` arm; `None` on the early-return failure arms and the `Err(e)` transport-failure arm.
- `src-tauri/src/popup.rs`:
  - `Payload` (popup.rs:40) — add `source_text: Option<&str>`. Update every emit site; `loading`/`error` pass `Some(&source)` when known (P1-3) else `None`; `result`/`multi_result` pass `Some(&source)`.
  - **(new)** `pub fn compute_popup_geometry_logical(cursor_x, cursor_y, mode, anchor: &PopupAnchor) -> (i32, i32, u32, u32)` — clamps in LOGICAL pixels using `anchor.work_area` + `anchor.scale_factor`, returns PHYSICAL pixels.
  - **(new)** `pub enum PopupMode { Loading, Single, Multi, Error }` with `size_logical() -> (u32, u32)` returning 200×40 / 400×300 / 600×400 / 400×300.
  - **(new)** `pub struct PopupAnchor { cursor_logical: (f64, f64), work_area: PhysicalRect, scale_factor: f64 }` (P1-2).
  - **(new)** `pub fn show_at_sized(app, x, y, width, height)` — `set_max_size` FIRST, then `set_size`, then `set_position`, then emit loading, then `show`/`set_focus`.
  - **(new)** `pub fn set_popup_mode(app, mode, anchor: &PopupAnchor)` — recomputes BOTH size and position via `compute_popup_geometry_logical`, then `set_max_size` → `set_size` → `set_position`.
- `src-tauri/src/service.rs`:
  - **(new)** `pub async fn translate_primary_only(client, keystore, preset, input) -> Result<Translation, Error>` — runs primary only, preserves the original `Error` (no `LocalNoFallback` conversion, P1-4).
  - **(new)** `pub fn eligible_for_session_fallback(outcomes: &[TranslationOutcome]) -> bool` — pure decision (P1-4).
  - `translate_parallel` (service.rs:190) — tag every entry with its input index; emit in strict input order; use `translate_primary_only`; session-level fallback via `eligible_for_session_fallback` + a single `translate_with_fallback_ref`.
- `src-tauri/src/error.rs` — no change (classification verified: `FallbackEligible`/`Config`/`Keystore`/`LocalNoFallback` already exist).
- `src-tauri/src/db/providers.rs` — `ActiveSelection` (line 649) gains `serde::Serialize` (B3).
- `src-tauri/tests/translate_parallel.rs` — strict-order-with-pre-failure test + fallback-call-count + local-sacred/error-class matrix tests (fixed mock URLs on `lvh.me`, P1-4).
- `src-tauri/tests/popup_geometry.rs` **(new)** — geometry/clamping tests (logical→physical, work_area, margin).
- `src-tauri/tests/hotkey_session.rs` **(new)** — central `to` resolution + decision routing + source-structure grep assertion.
- `src-tauri/tests/capabilities.rs` **(new)** — capability-set integration test (P1-6).
- `src-tauri/tests/tray_state.rs` **(new, rev-14 / rev-15 / rev-16 / rev-17 / rev-18 / rev-19 / Task A5; sync tests)** — pure-Rust tests for the tray visual-state controller: `TrayVisualState` priority ordering (`Error > Update > Active > Normal`); **rev-14/rev-15/rev-16/rev-17/rev-18/rev-19** `TrayStateController` reducer concurrency tests — two `begin_translation` then one `finish_translation` keeps `Active`; second `finish_translation` → `Normal`; **rev-16-1: `record_translation_error(gen)`** (renamed from `record_error(gen)`) overrides `Active`/`Normal` and is NOT cleared by `finish_translation(false)`; **rev-16-3 / rev-17-4: switch-flow `begin_switch()`/`finish_switch(rev, success)`** (no gen arg, revision-tagged; record_switch_error/clear_switch_error DELETED) are independent of the translation `error_gen`; **rev-16-2 / rev-17-3: gen guards** — `older_success_does_not_clear_newer_error` + `older_error_does_not_replace_newer_error` + `stale_gen_error_ignored_after_newer_begin` (rev-17-3 NEW — `latest_translation_gen` guard); **rev-16-3: switch revision ordering** — `two_concurrent_switches_second_wins` + `stale_switch_result_ignored`; `is_pulsing()` accessor asserts the `PulseWorker` is `Some` only while `Active`; **rev-19-4: `worker_start_count()` accessor asserts a second `begin` while Active does NOT churn the worker (the count stays at 1 — deterministic, replaces rev-18-5's timing-sensitive frame-count comparison)**; generation-aware error clearing (a newer-gen Retry success clears the prior red dot); **rev-15: current_state-gated worker swap** (a second `begin` while Active does NOT churn the worker — no restart); **rev-15 PulseWorker channel-quit** (`stop()` sends + joins — the worker returns from `recv_timeout` and `join` completes; `PulseWorker::drop` stops the worker); **rev-15: no stale tick after leaving Active** (after `record_translation_error(1)` switches to `Error` + drops the worker, the `RecordingRenderer` receives no further dimmed frames — verified via the channel-quit barrier emitting `PulseEvent::Stopped` (rev-17-2), NOT an epoch check); **rev-16 P2-1 / rev-17-2: PulseWorker-lifecycle tests use a `notify` channel carrying `PulseEvent` for deterministic frame waiting (NO `thread::sleep`)**; **rev-17 P2-4 / rev-18-5: channel-quit tests `match` the `recv_timeout` result against `PulseEvent::Tick`/`Stopped` then join (deterministic — `drop_stops_the_worker` asserts `Stopped`, NOT `Disconnected`)**; **rev-15 P1-3 + rev-16 P2-2 + rev-18-3 + rev-19-2: switch does NOT bump the translation generation** (rev-18-3: `#[test]`, calls the REAL SYNC core `handle_switch_provider_core(&app_state, &uuid)` — NO AppHandle — against a REAL temp DB + inserted provider — leaves `session.gen.is_latest(prior_gen) == true`; asserts `read_active_selection().primary == Some(uuid)` + tray `switch_error_rev() == None` + `current_state() == Normal` on success; rev-19-2: the fixture uses the `fresh_db` pattern — `Database::open` + `schema::create_all_tables` + `schema::seed_singletons` inside a transaction FIRST, THEN `db_providers::create`); **rev-16 P2-2 / rev-18 P2-4 / rev-19 P2-1: structural test** (`switch_arm_source_has_no_gen_next_call` reads `lib.rs` via `include_str!` and asserts the switch arm contains no `.gen.next()` AND no `.await`/`spawn(async move`/`pub async fn handle_switch_provider`; rev-19 P2-1: ALSO asserts `build_switch_provider_submenu` + the `tray.switch-{uuid}` format exist — the dynamic per-provider submenu, NOT a single fixed MenuItem); localization; red-dot pixel-diff (overlay, not solid square — `panic!` if PNG missing). **Test count = 33** (rev-19: unchanged from rev-18 — the functional switch test FIXTURE is rewritten to `fresh_db` + the no-churn assertion to `worker_start_count`, no test added/removed; rev-18: unchanged from rev-17 — the functional switch test was rewritten in-place from `#[tokio::test]`/async to `#[test]`/SYNC against a real DB; ALL `#[test]`, 0 `#[tokio::test]`; see Step 2's `#[test]` enumeration for the authoritative count).
- `test/Popup.test.tsx` — the Retry test asserts `translate_selection_ipc` is called with the saved SOURCE, and `translate_clipboard` is NOT called.
- `src-tauri/tauri.conf.json` — main window `visible: false` + `minWidth`/`minHeight` 600×400.

**Build / permissions / capabilities:**

- `src-tauri/build.rs` — add `translate_session`, `translate_selection_ipc`, `provider_get_active_selection`, `open_settings_window` to the commands list. **(rev-11 / Task A5; rev-12 overlay + dimmed)** the existing `build.rs` build script is ALSO extended with a block that writes TWO tray PNGs into `OUT_DIR` using the `image` build-dependency: (1) `tray-error-32.png` — **rev-12: a red-dot OVERLAY composited on the base icon** (`image::open("src-tauri/icons/32x32.png")` loads the 32×32 base, then a ~10px-diameter `#DC2626` = `[220, 38, 38, 255]` dot is drawn at the top-right via a manual `put_pixel` circle test `dx*dx + dy*dy <= r*r` — NOT a solid-red square as in rev-11); (2) `tray-active-32.png` — **rev-12 new** a dimmed variant of the base icon (each pixel's RGB scaled to ~60% brightness, alpha unchanged) for the pulse frame-swap. The runtime embeds both via `include_bytes!(concat!(env!("OUT_DIR"), "/tray-error-32.png"))` and `.../tray-active-32.png"`. The base `src-tauri/icons/32x32.png` IS read from the repo (it already ships as the app default icon) — this is build-time PROGRAMMATIC COMPOSITING, not checking in a new design asset.
- `src-tauri/permissions/autogenerated/` — verify `translate_session.toml`, `translate_selection_ipc.toml`, `provider_get_active_selection.toml`, `open_settings_window.toml` appear after a build AND are listed in the `git add` of the task that introduces the command.
- `src-tauri/capabilities/input.json` — add `allow-translate-session` + `allow-provider-list` (P1-6).
- `src-tauri/capabilities/popup.json` — add `allow-provider-list`, `allow-provider-get-active-selection`, `allow-translate-selection-ipc`, `allow-open-settings-window`, `clipboard-manager:allow-write-text`.
- `src-tauri/capabilities/main.json` — add `allow-translate-session`, `allow-translate-selection-ipc`, `allow-provider-get-active-selection`, `allow-open-settings-window`.
- `src-tauri/Cargo.toml` — add `tauri-plugin-clipboard-manager = "2"` (A4, BEFORE capabilities reference it, P1-6). **(rev-14 / rev-15 / Task A5)** add `image = { version = "0.25", default-features = false, features = ["png"] }` under `[build-dependencies]` (used by `build.rs` to composite the red-dot overlay + generate the dimmed pulse icon; NOT a runtime dep) AND under `[dev-dependencies]` (the pixel-diff test loads the generated PNG). **rev-14 runtime deps (Task A5):** `parking_lot = "0.12"` is ALREADY a production dep (verified `Cargo.toml:53`) — the controller's `tray: Arc<parking_lot::Mutex<TrayStateController>>` needs no new runtime dep. `sys-locale = "0.3"` is added to `[dependencies]` (`detect_system_locale()` uses `sys_locale::get_locale()` — cross-platform, NOT `std::env::var("LANG")` which is Unix-only). **rev-14 dev-deps:** `tokio = { version = "1", features = ["macros", "rt-multi-thread", "test-util"] }` (the current line at `Cargo.toml:102` lacks `test-util` — needed for `#[tokio::test(start_paused = true)]` should any test retain it; rev-14 moves the timer to `std::thread` so `time`/`sync` are NOT added to the runtime tokio). **rev-15 (Task A5):** the `[features] xproc-test-helper = []` line ALREADY EXISTS (verified) — rev-15 P1-2 gates `RecordingRenderer` + the `lib.rs` re-export behind `#[cfg(any(test, feature = "xproc-test-helper"))]` so the integration test crate (compiled under `--features xproc-test-helper`) sees it and `cargo build` (no feature) does not compile the mock. **rev-15 (Cargo.lock):** `src-tauri/Cargo.lock` IS git-tracked (verified `git ls-files`) — adding `sys-locale = "0.3"` updates the lock, so `src-tauri/Cargo.lock` is added to the A5 commit (Step 12).
- `package.json` — add `@tauri-apps/plugin-clipboard-manager`; fix `test:src`; add `test:all`; add Playwright devDep + scripts (D5).

---

## Stage A: Main Path, Theme, Geometry, and Permissions Foundation

Checkpoint goal: every entry bootstraps theme identically; the popup clamps to the work area at the right size per mode with Retina-safe coordinate conversion (logical→physical) unified through a `PopupAnchor` and `set_max_size` before `set_size`; Alt+Space hits the multi-engine session via a shared `capture_and_translate` helper (with generation-token staleness guards) and sizes the popup per state; the tray's translation/clipboard/switch/settings actions reach a listener in `App.tsx` and drive `SettingsShell.activePage`; the new backend commands are permission-authorized (and the clipboard plugin is installed) before Stage B uses them.

**Stage A task order: A1 → A3 → A2 → A4 → A5 (rev-11/rev-12).**

### Task A1: Theme bootstrap (`src/theme.ts`)

**Files:**
- Create: `src/theme.ts`
- Modify: `src/popup-entry.tsx`, `src/input-entry.tsx`, `src/index.tsx`, `index.html` (theme-color meta variants)
- Test: `test/theme.test.ts`

**Interfaces:**
- Produces: `export function initTheme(): void` — reads `localStorage.getItem("linguaray.theme")` (`"light"` | `"dark"` | null), falls back to `window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light"`, then sets `document.documentElement.dataset.theme`, `document.documentElement.dataset.motion` (`"reduced"` when `window.matchMedia("(prefers-reduced-motion: reduce)").matches`, else `"full"`), and `document.documentElement.lang` (from `src/i18n.ts`'s `detectLocale()`). It ALSO manages the two `<meta name="theme-color">` elements (P2): the meta whose `media` matches the resolved scheme keeps its media and gets its `content` re-asserted to the resolved token; the other meta is disabled by setting `media="disabled"`.

- [x] **Step 1: Write the failing test**

Create `test/theme.test.ts`:

```ts
import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { initTheme } from "../src/theme";

describe("initTheme", () => {
  const original = { ...document.documentElement.dataset };

  beforeEach(() => {
    document.documentElement.removeAttribute("data-theme");
    document.documentElement.removeAttribute("data-motion");
    document.documentElement.removeAttribute("lang");
    localStorage.clear();
    document.querySelectorAll("meta[name=theme-color]").forEach((m) => m.remove());
  });

  afterEach(() => {
    for (const k of Object.keys(document.documentElement.dataset)) {
      delete document.documentElement.dataset[k];
    }
    for (const [k, v] of Object.entries(original)) {
      if (v !== undefined) document.documentElement.dataset[k] = v;
    }
    document.querySelectorAll("meta[name=theme-color]").forEach((m) => m.remove());
  });

  it("sets data-theme from localStorage when present", () => {
    localStorage.setItem("linguaray.theme", "dark");
    initTheme();
    expect(document.documentElement.dataset.theme).toBe("dark");
  });

  it("falls back to prefers-color-scheme when localStorage is unset", () => {
    vi.spyOn(window, "matchMedia").mockImplementation((q) => ({
      matches: q.includes("dark"),
      media: q,
      addEventListener: () => {},
      removeEventListener: () => {},
      addListener: () => {},
      removeListener: () => {},
      onchange: null,
      dispatchEvent: () => true,
    } as unknown as MediaQueryList));
    initTheme();
    expect(document.documentElement.dataset.theme).toBe("dark");
    vi.restoreAllMocks();
  });

  it("sets data-motion=reduced when prefers-reduced-motion matches", () => {
    vi.spyOn(window, "matchMedia").mockImplementation((q) => ({
      matches: q.includes("reduced-motion"),
      media: q,
      addEventListener: () => {},
      removeEventListener: () => {},
      addListener: () => {},
      removeListener: () => {},
      onchange: null,
      dispatchEvent: () => true,
    } as unknown as MediaQueryList));
    initTheme();
    expect(document.documentElement.dataset.motion).toBe("reduced");
    vi.restoreAllMocks();
  });

  it("sets data-motion=full when reduced-motion does not match", () => {
    vi.spyOn(window, "matchMedia").mockImplementation(() => ({
      matches: false,
      media: "",
      addEventListener: () => {},
      removeEventListener: () => {},
      addListener: () => {},
      removeListener: () => {},
      onchange: null,
      dispatchEvent: () => true,
    } as unknown as MediaQueryList));
    initTheme();
    expect(document.documentElement.dataset.motion).toBe("full");
    vi.restoreAllMocks();
  });

  it("sets lang to the detected locale", () => {
    localStorage.setItem("linguaray.locale", "zh");
    initTheme();
    expect(document.documentElement.lang).toBe("zh");
  });

  it("activates the resolved-scheme theme-color meta with media=all and disables the other (rev-5-6)", () => {
    const light = document.createElement("meta");
    light.setAttribute("name", "theme-color");
    light.setAttribute("media", "(prefers-color-scheme: light)");
    light.setAttribute("content", "#F8FAFC");
    const dark = document.createElement("meta");
    dark.setAttribute("name", "theme-color");
    dark.setAttribute("media", "(prefers-color-scheme: dark)");
    dark.setAttribute("content", "#020617");
    document.head.append(light, dark);

    localStorage.setItem("linguaray.theme", "dark");
    initTheme();

    const metas = document.querySelectorAll<HTMLMetaElement>("meta[name=theme-color]");
    expect(metas.length).toBe(2);
    const activeDark = Array.from(metas).find((m) => m.getAttribute("content") === "#020617");
    expect(activeDark, "dark theme-color meta must exist").toBeTruthy();
    // rev-5-6: the current meta gets media="all" (always wins), NOT
    // prefers-color-scheme:dark (which would lose when OS prefers light).
    expect(activeDark!.getAttribute("media")).toBe("all");
    const disabled = Array.from(metas).find((m) => m.getAttribute("media") === "disabled");
    expect(disabled, "non-current scheme meta must be disabled (media=disabled)").toBeTruthy();
  });

  it("rev-5-6: a FORCED theme wins over the OS preference (user Dark while OS Light)", () => {
    // OS prefers light; user forced dark. The rev-4 form left the dark meta at
    // media="(prefers-color-scheme: dark)" (no match) + light meta disabled → no
    // meta applied. rev-5-6 sets the current (dark) meta to media="all".
    vi.spyOn(window, "matchMedia").mockImplementation((q) => ({
      matches: q.includes("light"), // OS prefers light
      media: q,
      addEventListener: () => {},
      removeEventListener: () => {},
      addListener: () => {},
      removeListener: () => {},
      onchange: null,
      dispatchEvent: () => true,
    } as unknown as MediaQueryList));

    const light = document.createElement("meta");
    light.setAttribute("name", "theme-color");
    light.setAttribute("media", "(prefers-color-scheme: light)");
    light.setAttribute("content", "#F8FAFC");
    const dark = document.createElement("meta");
    dark.setAttribute("name", "theme-color");
    dark.setAttribute("media", "(prefers-color-scheme: dark)");
    dark.setAttribute("content", "#020617");
    document.head.append(light, dark);

    localStorage.setItem("linguaray.theme", "dark"); // forced dark
    initTheme();

    const metas = document.querySelectorAll<HTMLMetaElement>("meta[name=theme-color]");
    const activeDark = Array.from(metas).find((m) => m.getAttribute("content") === "#020617");
    expect(activeDark, "forced-dark meta must be the active one").toBeTruthy();
    expect(activeDark!.getAttribute("media")).toBe("all");
    const activeLight = Array.from(metas).find((m) => m.getAttribute("content") === "#F8FAFC");
    expect(activeLight!.getAttribute("media")).toBe("disabled");
    vi.restoreAllMocks();
  });
});
```

- [x] **Step 2: Run test to verify it fails**

Run: `pnpm vitest run test/theme.test.ts`
Expected: FAIL — `Cannot find module '../src/theme'`.

- [x] **Step 3: Write the implementation**

Create `src/theme.ts`:

```ts
import { detectLocale } from "./i18n";

const LIGHT_THEME_COLOR = "#F8FAFC";
const DARK_THEME_COLOR = "#020617";

/**
 * Read once at first paint: theme, motion preference, and locale. Sets three
 * attributes on documentElement so @linguaray/ui token CSS ([data-theme=...]
 * blocks in tokens.css) and base.css ([data-motion=reduced]) resolve BEFORE the
 * first component renders, avoiding a flash of unstyled/wrong-theme content.
 *
 * P2: keeps BOTH theme-color metas in the DOM but DISABLES the non-current one
 * (media="disabled") so the browser chrome honors only the resolved scheme. The
 * current meta keeps its prefers-color-scheme media and gets its content
 * re-asserted to the resolved token.
 *
 * Safe to call in any entry (popup/input/settings) and in jsdom tests.
 */
export function initTheme(): void {
  const root = document.documentElement;

  let theme: "light" | "dark";
  const stored =
    typeof localStorage !== "undefined" ? localStorage.getItem("linguaray.theme") : null;
  if (stored === "light" || stored === "dark") {
    theme = stored;
  } else if (
    typeof window !== "undefined" &&
    typeof window.matchMedia === "function" &&
    window.matchMedia("(prefers-color-scheme: dark)").matches
  ) {
    theme = "dark";
  } else {
    theme = "light";
  }
  root.dataset.theme = theme;

  const reduced =
    typeof window !== "undefined" &&
    typeof window.matchMedia === "function" &&
    window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  root.dataset.motion = reduced ? "reduced" : "full";

  root.lang = detectLocale();

  if (typeof document !== "undefined") {
    syncThemeColorMetas(theme);
  }
}

/**
 * rev-5-6: activate the resolved-scheme meta, disable the other.
 *
 * The CURRENT meta gets `media="all"` (always wins) — NOT
 * `(prefers-color-scheme: <current>)`. The rev-4 form kept the prefers media,
 * which breaks when the user FORCES a theme that disagrees with the OS: OS
 * Light + forced Dark → the Dark meta keeps `media="(prefers-color-scheme: dark)"`
 * (does NOT match the OS Light preference) and the Light meta gets
 * `media="disabled"`, so NO meta wins and the OS chrome falls back to the
 * browser default. With `media="all"` the current meta always applies, and the
 * other meta is `media="disabled"` so it never overrides.
 */
function syncThemeColorMetas(theme: "light" | "dark"): void {
  const metas = document.querySelectorAll<HTMLMetaElement>('meta[name="theme-color"]');
  const currentColor = theme === "dark" ? DARK_THEME_COLOR : LIGHT_THEME_COLOR;
  if (metas.length === 0) {
    const m = document.createElement("meta");
    m.setAttribute("name", "theme-color");
    m.setAttribute("media", "all");
    m.setAttribute("content", currentColor);
    document.head.appendChild(m);
    return;
  }
  const currentKeyword = theme; // "light" | "dark"
  for (const m of Array.from(metas)) {
    const media = m.getAttribute("media") ?? "";
    // The meta that SHIPS with the resolved scheme's keyword is the current one.
    // (index.html ships media="(prefers-color-scheme: light)" and "...dark".)
    const isCurrent = media.includes(currentKeyword);
    if (isCurrent) {
      // rev-5-6: force the current meta to ALWAYS apply (media="all"), so a
      // forced theme wins over the OS preference. Re-assert the resolved color.
      m.setAttribute("media", "all");
      m.setAttribute("content", currentColor);
    } else {
      // Disable the non-current scheme meta so the OS chrome uses only the current.
      m.setAttribute("media", "disabled");
    }
  }
}
```

- [x] **Step 4: Run test to verify it passes**

Run: `pnpm vitest run test/theme.test.ts`
Expected: PASS (6 tests).

- [x] **Step 5: Wire initTheme into the three entries**

Edit `src/popup-entry.tsx`:

```tsx
import { render } from "solid-js/web";
import "@linguaray/ui/styles";
import { initTheme } from "./theme";
import Popup from "./Popup";
initTheme();
render(() => <Popup />, document.getElementById("root")!);
```

Edit `src/input-entry.tsx` identically (with `InputPanel` in place of `Popup`).

Edit `src/index.tsx`:

```tsx
/* @refresh reload */
import { render } from "solid-js/web";
import "@linguaray/ui/styles";
import { initTheme } from "./theme";
import App from "./App";
initTheme();
render(() => <App />, document.getElementById("root") as HTMLElement);
```

- [x] **Step 6: Run full vitest to confirm no regression**

Run: `pnpm test`
Expected: PASS (the existing suites + the 6 new tests).

- [x] **Step 7: Commit**

```bash
git diff --check
git add src/theme.ts src/popup-entry.tsx src/input-entry.tsx src/index.tsx test/theme.test.ts
git commit -m "feat(theme): add initTheme() bootstrap called from all three entries + theme-color meta activation (P2)"
```

---

### Task A3: Popup native sizing + work-area clamping (Retina-safe, unified units via PopupAnchor)

> **Ordering:** A3 runs BEFORE A2 so A2's `on_hotkey` rewrite can call `set_popup_mode`/`show_at_sized`/`compute_popup_geometry_logical`. A3 is self-contained: it ships a source-aware loading emitter so A2 can depend on A3 without waiting on B4.

**Files:**
- Modify: `src-tauri/src/popup.rs` — add `PopupAnchor`, `PopupMode` (+ `Error` variant), `compute_popup_geometry_logical`, `show_at_sized`, `set_popup_mode`, `loading_with_source`.
- Test: `src-tauri/tests/popup_geometry.rs` **(new)** + `test/popupGeometry.test.ts` **(new)** (TS sanity check).

**Interfaces (P1-2):**
- Produces:
  - `pub enum PopupMode { Loading, Single, Multi, Error }` with `size_logical() -> (u32, u32)` returning LOGICAL targets: Loading 200×40, Single 400×300, Multi 600×400, Error 400×300.
  - `pub struct PopupAnchor { cursor_logical: (f64, f64), work_area: LogicalWorkArea, scale_factor: f64 }` (P1-2). `cursor_logical` and `work_area` are BOTH logical (CSS) px. `work_area` is filled from `Monitor::work_area()` (rev-6-1: the REAL `PhysicalRect<i32, u32>`) divided by `scale_factor`. `scale_factor` converts logical→physical.
  - `pub fn compute_popup_geometry_logical(cursor_x, cursor_y, mode, anchor: &PopupAnchor) -> (i32, i32, u32, u32)` — clamps the popup inside `anchor.work_area` (8px margin) in LOGICAL pixels, then converts to PHYSICAL (`* anchor.scale_factor`). Returns `(x, y, width, height)` PHYSICAL.
  - `pub fn show_at_sized(app, x, y, width, height) -> Result<(), String>` — `set_max_size` FIRST, THEN `set_size`, THEN `set_position`, emit loading, `show`, `set_focus`. All PHYSICAL pixels.
  - `pub fn set_popup_mode(app, mode, anchor: &PopupAnchor) -> Result<(), String>` — recomputes BOTH size AND position via `compute_popup_geometry_logical(anchor)`, then `set_max_size` → `set_size` → `set_position`.
  - `pub fn loading_with_source(app, anchor: &PopupAnchor, source_text: Option<&str>) -> Result<(), String>` — shows the popup sized for Loading mode, emits a loading `Payload` carrying `source_text` (P1-3: the loading payload carries the source so Retry is available even before the result arrives).

- [x] **Step 1: Write the failing test**

Create `src-tauri/tests/popup_geometry.rs`:

```rust
//! Task A3: popup geometry clamping. Pure-function tests — no Tauri runtime.
//! P1-2 + rev-6-1: PopupAnchor unifies units. cursor_logical + work_area are
//! both CSS px (logical); scale_factor converts to physical so the Tauri window
//! API gets the right numbers on Retina. `work_area` is filled from the REAL
//! `Monitor::work_area()` (PhysicalRect<i32, u32>) at the call site.
use linguaray_lib::popup::{
    compute_popup_geometry_logical, PopupAnchor, PopupMode,
};
use tauri::PhysicalRect;

const MARGIN: i32 = 8;

fn anchor_at(cx: f64, cy: f64, w: f64, h: f64, sf: f64) -> PopupAnchor {
    PopupAnchor {
        cursor_logical: (cx, cy),
        work_area: PhysicalRect { left: 0.0, top: 0.0, right: w, bottom: h },
        scale_factor: sf,
    }
}

#[test]
fn loading_mode_logical_is_200x40() {
    let a = anchor_at(100.0, 100.0, 1920.0, 1080.0, 1.0);
    let (_, _, w, h) = compute_popup_geometry_logical(PopupMode::Loading, &a);
    assert_eq!((w, h), (200, 40));
}

#[test]
fn single_mode_logical_is_400x300() {
    let a = anchor_at(100.0, 100.0, 1920.0, 1080.0, 1.0);
    let (_, _, w, h) = compute_popup_geometry_logical(PopupMode::Single, &a);
    assert_eq!((w, h), (400, 300));
}

#[test]
fn multi_mode_logical_is_600x400() {
    let a = anchor_at(100.0, 100.0, 1920.0, 1080.0, 1.0);
    let (_, _, w, h) = compute_popup_geometry_logical(PopupMode::Multi, &a);
    assert_eq!((w, h), (600, 400));
}

#[test]
fn error_mode_logical_is_400x300() {
    let a = anchor_at(100.0, 100.0, 1920.0, 1080.0, 1.0);
    let (_, _, w, h) = compute_popup_geometry_logical(PopupMode::Error, &a);
    assert_eq!((w, h), (400, 300));
}

#[test]
fn retina_doubles_physical_size() {
    let a = anchor_at(100.0, 100.0, 1920.0, 1080.0, 2.0);
    let (_, _, w, h) = compute_popup_geometry_logical(PopupMode::Single, &a);
    assert_eq!((w, h), (800, 600));
}

#[test]
fn clamps_right_edge_to_work_area_minus_margin() {
    let a = anchor_at(990.0, 100.0, 1000.0, 800.0, 1.0);
    let (x, _, w, _) = compute_popup_geometry_logical(PopupMode::Single, &a);
    assert!(x + w as i32 <= 1000 - MARGIN, "x={x} w={w} overflowed right edge");
}

#[test]
fn clamps_bottom_edge_to_work_area_minus_margin() {
    let a = anchor_at(100.0, 790.0, 1000.0, 800.0, 1.0);
    let (_, y, _, h) = compute_popup_geometry_logical(PopupMode::Single, &a);
    assert!(y + h as i32 <= 800 - MARGIN, "y={y} h={h} overflowed bottom edge");
}

#[test]
fn clamps_left_edge_to_work_area_plus_margin() {
    let a = anchor_at(0.0, 100.0, 1000.0, 800.0, 1.0);
    let (x, _, _, _) = compute_popup_geometry_logical(PopupMode::Single, &a);
    assert!(x >= MARGIN, "x={x} underflowed left edge");
}

#[test]
fn clamps_top_edge_to_work_area_plus_margin() {
    let a = anchor_at(100.0, 0.0, 1000.0, 800.0, 1.0);
    let (_, y, _, _) = compute_popup_geometry_logical(PopupMode::Single, &a);
    assert!(y >= MARGIN, "y={y} underflowed top edge");
}

#[test]
fn retina_clamps_in_logical_then_scales_position() {
    // 2x display, cursor at logical (990,100) inside a 1000x800 logical work area.
    // Clamp in logical, then physical position is 2x. Physical width is 800.
    let a = anchor_at(990.0, 100.0, 1000.0, 800.0, 2.0);
    let (x, _, w, _) = compute_popup_geometry_logical(PopupMode::Single, &a);
    assert_eq!(w, 800);
    assert!(x <= 1184, "x={x} overflowed physical right edge after logical clamp");
}
```

Create `test/popupGeometry.test.ts` (TS sanity check that the mode sizes match the contract):

```ts
import { describe, it, expect } from "vitest";

const LOGICAL_SIZES = {
  loading: { w: 200, h: 40 },
  single: { w: 400, h: 300 },
  multi: { w: 600, h: 400 },
  error: { w: 400, h: 300 },
};

describe("popup geometry contract (frontend mirror)", () => {
  it("loading is 200x40 logical", () => {
    expect(LOGICAL_SIZES.loading).toEqual({ w: 200, h: 40 });
  });
  it("single is 400x300 logical", () => {
    expect(LOGICAL_SIZES.single).toEqual({ w: 400, h: 300 });
  });
  it("multi is 600x400 logical", () => {
    expect(LOGICAL_SIZES.multi).toEqual({ w: 600, h: 400 });
  });
  it("error matches single (400x300)", () => {
    expect(LOGICAL_SIZES.error).toEqual(LOGICAL_SIZES.single);
  });
});
```

- [x] **Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --features xproc-test-helper --test popup_geometry`
Expected: FAIL — `cannot find type PopupMode / PopupAnchor in crate`.

- [x] **Step 3: Implement the geometry types + functions**

Add to `src-tauri/src/popup.rs` (after the existing `POPUP_MULTI_EVENT` const, before the `#[cfg(test)]` block). This requires the `PhysicalRect` type — Tauri 2 exposes `tauri::PhysicalRect`; if the precise generic form differs in the installed patch, the struct below uses a plain local rect so the pure math is testable without the Tauri runtime. **The `PopupAnchor.work_area` field uses the local `LogicalWorkArea` (CSS px) so the pure function compiles in `tests/` without a Tauri link.** The conversion from `tauri::PhysicalRect` happens at the call site (A2).

```rust
// ─── A3: native sizing + work-area clamping (P1-2 unified units) ─────────

/// A monitor work area in LOGICAL (CSS) pixels. Callers fill this from
/// `Monitor::work_area()` (rev-6-1: the REAL `PhysicalRect<i32, u32>` returned
/// by Tauri 2.11.5) divided by scale_factor.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct LogicalWorkArea {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

impl LogicalWorkArea {
    pub fn width(&self) -> f64 {
        self.right - self.left
    }
    pub fn height(&self) -> f64 {
        self.bottom - self.top
    }
}

/// P1-2: the single source of popup geometry. `cursor_logical` and `work_area`
/// are BOTH logical (CSS) px; `scale_factor` converts to physical. Every mode
/// change recomputes size AND position from this anchor.
#[derive(Debug, Clone, Copy)]
pub struct PopupAnchor {
    pub cursor_logical: (f64, f64),
    pub work_area: LogicalWorkArea,
    pub scale_factor: f64,
}

/// The four popup sizes the UI requests. Matched 1:1 to the loading /
/// single-success / multi-result / error UI states. Dimensions are LOGICAL.
pub enum PopupMode {
    Loading,
    Single,
    Multi,
    Error,
}

impl PopupMode {
    /// Logical (CSS) (width, height). Callers convert to physical via scale_factor.
    pub fn size_logical(&self) -> (u32, u32) {
        match self {
            PopupMode::Loading => (200, 40),
            PopupMode::Single => (400, 300),
            PopupMode::Multi => (600, 400),
            PopupMode::Error => (400, 300),
        }
    }
}

/// Margin between the popup and any work-area edge (logical px).
const CLAMP_MARGIN: f64 = 8.0;

/// Pure geometry: pick the popup's (x, y, width, height) PHYSICAL pixels for a
/// given mode + anchor. Clamps in LOGICAL space, then multiplies by scale_factor.
/// Pure on purpose — fully testable without a Tauri runtime. (P1-2)
pub fn compute_popup_geometry_logical(
    mode: PopupMode,
    anchor: &PopupAnchor,
) -> (i32, i32, u32, u32) {
    let (lw, lh) = mode.size_logical();
    let (lwf, lhf) = (lw as f64, lh as f64);
    let (cx, cy) = anchor.cursor_logical;

    let right_limit = anchor.work_area.right - CLAMP_MARGIN - lwf;
    let left_limit = anchor.work_area.left + CLAMP_MARGIN;
    let x_logical = cx.clamp(left_limit, right_limit.max(left_limit));

    let bottom_limit = anchor.work_area.bottom - CLAMP_MARGIN - lhf;
    let top_limit = anchor.work_area.top + CLAMP_MARGIN;
    let y_logical = cy.clamp(top_limit, bottom_limit.max(top_limit));

    let sf = anchor.scale_factor;
    let phys = |v: f64| -> i32 { (v * sf).round() as i32 };
    let phys_u = |v: f64| -> u32 {
        let p = phys(v);
        if p < 0 { 0 } else { p as u32 }
    };
    (phys(x_logical), phys(y_logical), phys_u(lwf), phys_u(lhf))
}

/// Show the popup at an explicit PHYSICAL (x, y, width, height). Order is
/// load-bearing (P1-2): set_max_size FIRST, then set_size, then set_position.
pub fn show_at_sized(
    app: &tauri::AppHandle,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> Result<(), String> {
    let win = window(app)?;
    let size = tauri::PhysicalSize { width, height };
    win.set_max_size(Some(size)).map_err(|e| e.to_string())?;
    win.set_size(size).map_err(|e| e.to_string())?;
    win.set_position(tauri::PhysicalPosition { x, y })
        .map_err(|e| e.to_string())?;
    win.emit(
        "popup-state",
        Payload { status: "loading", text: "", engine: "", source_text: None },
    )
    .map_err(|e| e.to_string())?;
    win.show().map_err(|e| e.to_string())?;
    win.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}

/// Show loading popup sized for the given anchor, carrying the source text so
/// the frontend can save it for Retry (P1-3). A3 ships this so A2 can depend on
/// A3 without waiting on B4.
pub fn loading_with_source(
    app: &tauri::AppHandle,
    anchor: &PopupAnchor,
    source_text: Option<&str>,
) -> Result<(), String> {
    let (x, y, w, h) = compute_popup_geometry_logical(PopupMode::Loading, anchor);
    let win = window(app)?;
    let size = tauri::PhysicalSize { width: w, height: h };
    win.set_max_size(Some(size)).map_err(|e| e.to_string())?;
    win.set_size(size).map_err(|e| e.to_string())?;
    win.set_position(tauri::PhysicalPosition { x, y })
        .map_err(|e| e.to_string())?;
    win.emit(
        "popup-state",
        Payload { status: "loading", text: "", engine: "", source_text },
    )
    .map_err(|e| e.to_string())?;
    win.show().map_err(|e| e.to_string())?;
    win.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}

/// Resize + reposition the popup for a UI mode. Recomputes BOTH size AND
/// position from the anchor (P1-2: a mode change is a geometry change).
pub fn set_popup_mode(
    app: &tauri::AppHandle,
    mode: PopupMode,
    anchor: &PopupAnchor,
) -> Result<(), String> {
    let (x, y, w, h) = compute_popup_geometry_logical(mode, anchor);
    let win = window(app)?;
    let size = tauri::PhysicalSize { width: w, height: h };
    win.set_max_size(Some(size)).map_err(|e| e.to_string())?;
    win.set_size(size).map_err(|e| e.to_string())?;
    win.set_position(tauri::PhysicalPosition { x, y })
        .map_err(|e| e.to_string())?;
    Ok(())
}
```

The `Payload` struct also needs the `source_text` field now (A3 introduces it; B4 adds the result emitters that populate it). Update popup.rs line 40:

```rust
#[derive(Clone, serde::Serialize)]
struct Payload<'a> {
    status: &'a str,
    text: &'a str,
    engine: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_text: Option<&'a str>,
}
```

Update the existing emit sites (`show_at`, `result`, `error`) to pass `source_text: None`. The inline `#[cfg(test)]` block in popup.rs that constructs `Payload` must also add `source_text: None`.

- [x] **Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --features xproc-test-helper --test popup_geometry`
Run: `pnpm vitest run test/popupGeometry.test.ts`
Expected: PASS (10 Rust tests + 4 TS tests).

- [x] **Step 5: Verify the crate still compiles + existing popup tests pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --features xproc-test-helper --lib popup::`
Expected: the existing `popup.rs` inline tests pass (they now pass `source_text: None`).

- [x] **Step 6: Commit**

```bash
git diff --check
git add src-tauri/src/popup.rs src-tauri/tests/popup_geometry.rs test/popupGeometry.test.ts
git commit -m "feat(popup): PopupAnchor + logical->physical geometry clamping + source-aware loading emitter (P1-2)"
```

**Monitor/work_area derivation note (load-bearing for A2):** Tauri 2.11.5's `Monitor` exposes `work_area() -> &PhysicalRect<i32, u32>` (the REAL usable area), plus `position()` + `size()` + `scale_factor()` (`scale_factor` returns `f64` directly). To build a `PopupAnchor`, `build_popup_anchor` resolves the cursor's monitor via `monitor_from_point` and divides THAT monitor's `work_area()` by the monitor's `scale_factor` to get `LogicalWorkArea`, and divides the physical cursor (`cursor::position()`) by the SAME factor to get `cursor_logical`. (The popup window's `scale_factor()` is only the `None`-monitor fallback.) A2 derives the anchor centrally in `capture_and_translate` via `build_popup_anchor`. Do NOT mix units — both fields of `PopupAnchor` are logical, and the factor used must be the TARGET MONITOR's.

---

### Task A2: Alt+Space hotkey drives `run_translate_session` via `capture_and_translate` (full generation-token + capture path) + central `to: ""` resolution

> **Ordering:** A2 runs AFTER A3 (geometry) so `set_popup_mode`/`show_at_sized`/`loading_with_source` exist.

**Files:**
- Modify: `src-tauri/src/lib.rs`:
  - `run_translate_session` (lib.rs:492) — resolve `to: ""` centrally at the top.
  - `on_hotkey` (lib.rs:1982) — replace the capture+translate block with `capture_and_translate`, passing `gen`.
  - **(new)** `async fn capture_and_translate(app, state, app_state, x, y, gen)` — the shared pipeline (P1-1).
- Test: `src-tauri/tests/hotkey_session.rs` **(new)**.

**Interfaces:**
- Consumes: `run_translate_session(db, client, keystore, app, text, from, to) -> Result<TranslateSessionResult, String>` (lib.rs:492). `run_translate_session` resolves `to: ""` to `settings::load(app).target_language` at the TOP, so EVERY caller gets the sentinel resolved centrally.
- Produces:
  - `async fn capture_and_translate(app: &tauri::AppHandle, state: &Arc<Session>, app_state: &Arc<AppState>, supplied_text: Option<String>, x: f64, y: f64, gen: u64) -> ()` — the shared pipeline. `x`/`y` are the PHYSICAL cursor coords (from `cursor::position()`); the helper resolves the cursor's monitor via `monitor_from_point` and converts to logical via THAT monitor's scale_factor (rev-7-1). `gen` is the generation token; the helper checks `state.gen.is_latest(gen)` at every await boundary (P1-1), including after the `spawn_blocking` that acquires the db Arc (rev-9-1: the gate guard is taken + dropped INSIDE the blocking closure so the async session never holds the gate). Used by hotkey, tray `translate-selection`, and Retry.
  - `on_hotkey` allocates `gen = state.gen.next()` synchronously (as today), then calls `capture_and_translate(&app2, &state, &app_state, x, y, gen)` where `x,y` are captured under the selection lock — the capture block STAYS in `on_hotkey` because it must run under the lock BEFORE the popup steals focus; the helper receives the already-captured text + cursor.

> **P1-1 design note (verified):** the generation token MUST be allocated synchronously in `on_hotkey` (lib.rs:1998 today). The capture-under-lock block (lib.rs:2010-2030) ALSO stays in `on_hotkey` because it must read the cursor + selection atomically before the popup steals focus. `capture_and_translate` takes the ALREADY-captured `(x, y, captured)` is NOT possible because the tray/Retry paths need the helper to do their own capture. Resolution: `capture_and_translate` accepts a `supplied_text: Option<String>` (Retry: skip capture; tray/hotkey: capture inside the helper under the lock). For the hotkey path, `on_hotkey` passes `supplied_text = None` and lets the helper capture — BUT the synchronous `gen.next()` stays in `on_hotkey`. The capture-under-lock moves INTO the helper so hotkey/tray share it.

- [x] **Step 1: Write the failing test (central `to` resolver + decision routing + source-structure grep)**

Create `src-tauri/tests/hotkey_session.rs`:

```rust
//! Task A2: prove (1) the central `to: ""` resolver lives in run_translate_session,
//! (2) the decision router (single/multi/error) is shared, and (3) on_hotkey no
//! longer references translate_with_fallback (source-structure assertion).
use linguaray_lib::{decide_clipboard_popup, ClipboardPopupDecision, TranslateSessionResult};
use linguaray_lib::service::{Translation, TranslationOutcome};

#[test]
fn empty_target_is_resolved_centrally_to_settings_value() {
    assert_eq!(
        linguaray_lib::resolve_target_language("", "zh"),
        "zh",
        "to:\"\" must resolve to settings.target_language inside run_translate_session"
    );
}

#[test]
fn explicit_target_is_passed_through_unchanged() {
    assert_eq!(linguaray_lib::resolve_target_language("ja", "zh"), "ja");
}

#[test]
fn hotkey_routes_multi_success_to_multi_event() {
    let result = TranslateSessionResult {
        outcomes: vec![
            TranslationOutcome {
                uuid: "u1".into(),
                result: Ok(Translation { text: "你好".into(), engine: "provider/u1".into() }),
            },
            TranslationOutcome {
                uuid: "u2".into(),
                result: Ok(Translation { text: "您好".into(), engine: "provider/u2".into() }),
            },
        ],
        actual_engine: None,
    };
    let decision = decide_clipboard_popup(&result);
    assert!(matches!(decision, ClipboardPopupDecision::Multi));
}

#[test]
fn hotkey_routes_single_success_to_result_event() {
    let result = TranslateSessionResult {
        outcomes: vec![TranslationOutcome {
            uuid: "u1".into(),
            result: Ok(Translation { text: "你好".into(), engine: "openai".into() }),
        }],
        actual_engine: Some("openai".into()),
    };
    let decision = decide_clipboard_popup(&result);
    match decision {
        ClipboardPopupDecision::SingleSuccess { engine, .. } => {
            assert_eq!(engine, "openai");
        }
        other => panic!("expected SingleSuccess, got {other:?}"),
    }
}

#[test]
fn hotkey_routes_all_failed_to_error_event() {
    use linguaray_lib::Error;
    let result = TranslateSessionResult {
        outcomes: vec![
            TranslationOutcome { uuid: "u1".into(), result: Err(Error::LocalNoFallback) },
            TranslationOutcome { uuid: "u2".into(), result: Err(Error::LocalNoFallback) },
        ],
        actual_engine: None,
    };
    let decision = decide_clipboard_popup(&result);
    assert!(matches!(decision, ClipboardPopupDecision::Error(_)));
}

#[test]
fn on_hotkey_does_not_call_translate_with_fallback() {
    let src = include_str!("../src/lib.rs");
    let start = src.find("fn on_hotkey").expect("on_hotkey fn not found");
    let body = &src[start..];
    let end = body[1..]
        .find("\nfn ")
        .or_else(|| body[1..].find("\nasync fn "))
        .or_else(|| body[1..].find("\npub fn "))
        .or_else(|| body[1..].find("\npub async fn "))
        .map(|i| i + 1)
        .unwrap_or(body.len());
    let on_hotkey_body = &body[..end];
    assert!(
        !on_hotkey_body.contains("translate_with_fallback("),
        "on_hotkey must not call translate_with_fallback; it should route through capture_and_translate -> run_translate_session.\n--- on_hotkey body ---\n{}",
        on_hotkey_body,
    );
    assert!(
        on_hotkey_body.contains("capture_and_translate("),
        "on_hotkey must call capture_and_translate.\n--- on_hotkey body ---\n{}",
        on_hotkey_body,
    );
}
```

- [x] **Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --features xproc-test-helper --test hotkey_session`
Expected: FAIL — `cannot find function resolve_target_language` / `decide_clipboard_popup` (private today); the grep assertion fails because `on_hotkey` still contains `translate_with_fallback(`.

- [x] **Step 3: Expose the decision fn + the central target resolver**

In `src-tauri/src/lib.rs`, make `decide_clipboard_popup` (lib.rs:449) and `ClipboardPopupDecision` (lib.rs:440) `pub`, and re-export at the crate root. Add the central resolver near the decision fn:

```rust
/// Central sentinel resolver: the frontend passes `to: ""` to mean "use the
/// stored target language". Exposed so the hotkey contract is locked by an
/// integration test.
pub fn resolve_target_language(to: &str, settings_target: &str) -> String {
    if to.is_empty() {
        settings_target.to_string()
    } else {
        to.to_string()
    }
}
```

Change `enum ClipboardPopupDecision` → `pub enum ClipboardPopupDecision` and `fn decide_clipboard_popup` → `pub fn decide_clipboard_popup`. Add `pub use ...` re-exports at the crate root if the integration test path requires them (the test uses `linguaray_lib::decide_clipboard_popup` / `linguaray_lib::ClipboardPopupDecision` / `linguaray_lib::TranslateSessionResult`, so ensure all three are reachable from the crate root).

- [x] **Step 4: Resolve `to: ""` centrally inside run_translate_session**

In `src-tauri/src/lib.rs`, `run_translate_session` (lib.rs:492). At the TOP of the fn body, before reading `fallback_engine`, add:

```rust
async fn run_translate_session(
    db: &Arc<Database>,
    client: &reqwest::Client,
    keystore: &keystore::Keystore,
    app: &tauri::AppHandle,
    text: &str,
    from: &str,
    to: &str,
) -> Result<TranslateSessionResult, String> {
    // P1-C: resolve the "" sentinel CENTRALLY so on_hotkey, translate_session,
    // translate_selection_ipc, and the tray all agree.
    let settings_target = settings::load(app).target_language;
    let to = resolve_target_language(to, &settings_target);
    // (existing fallback_engine read + delegation to run_translate_session_with_fallback)
    let fallback_box = settings::load(app).fallback_engine.as_deref().and_then(engines::find);
    let fallback: Option<Arc<dyn engines::TraditionalEngine>> =
        fallback_box.map(Arc::<dyn engines::TraditionalEngine>::from);
    run_translate_session_with_fallback(db, client, keystore, text, from, &to, fallback).await
}
```

- [x] **Step 5: Extract `capture_and_translate` from on_hotkey (P1-1: full + generation-token-checked)**

Add a new async helper near `on_hotkey`. This is the COMPLETE body (no placeholders), reusing the exact HWND resolution + capture_selection + client/keystore/db acquisition from on_hotkey (lib.rs:1999-2124), factored so the hotkey, tray `translate-selection`, and Retry share it. `gen` is checked at every await boundary.

```rust
/// Shared selection-capture + translate-session pipeline. Used by on_hotkey,
/// translate_selection_ipc (tray + Retry). Emits the popup state per outcome.
///
/// - `supplied_text = Some(t)` (Retry): skip capture, use the saved SOURCE text.
/// - `supplied_text = None` (hotkey/tray): run the selection_lock +
///   capture_selection block on_hotkey used (lib.rs:1999-2030).
/// - `x`, `y`: the PHYSICAL cursor coords (from cursor::position() at the call
///   site). The helper resolves the cursor's monitor via `monitor_from_point` and
///   converts to logical via THAT monitor's scale_factor (rev-7-1).
/// - `gen`: the generation token. Checked at every await boundary so a stale
///   run never overwrites a fresher popup (P1-1). rev-9-1: re-checked after the
///   `spawn_blocking` that acquires the db Arc (the gate guard is taken + dropped
///   INSIDE the blocking closure, mirroring translate_clipboard lib.rs:366-387).
///
/// `to` is passed as `""` so run_translate_session's central resolver handles it.
#[allow(clippy::too_many_arguments)]
async fn capture_and_translate(
    app: &tauri::AppHandle,
    state: &Arc<Session>,
    app_state: &Arc<AppState>,
    supplied_text: Option<String>,
    x: f64,
    y: f64,
    gen: u64,
) {
    // 1. Acquire text (capture or supplied).
    let (text, anchor) = match supplied_text {
        Some(t) if !t.is_empty() => {
            let anchor = match build_popup_anchor(app, x, y) {
                Some(a) => a,
                None => return,
            };
            (t, anchor)
        }
        _ => {
            // The SAME selection_lock + capture_selection(800, owner) block
            // on_hotkey uses (lib.rs:1999-2030).
            let captured: Result<selection_engine::Capture, String> = {
                let _g = state.gen.selection_lock();
                #[cfg(target_os = "windows")]
                let owner = match app
                    .get_webview_window("main")
                    .ok_or_else(|| "main window unavailable".to_string())
                    .and_then(|w| w.hwnd().map(|h| h.0).map_err(|e| e.to_string()))
                {
                    Ok(h) => h,
                    Err(e) => {
                        log::warn!("clipboard restore skipped: no owner HWND ({e})");
                        return;
                    }
                };
                #[cfg(not(target_os = "windows"))]
                let owner = ();
                match selection::capture_selection(800, owner) {
                    Ok(selection_engine::Capture::Selected(t)) => Ok(t),
                    Ok(selection_engine::Capture::NoSelection) => {
                        let anchor = match build_popup_anchor(app, x, y) {
                            Some(a) => a,
                            None => return,
                        };
                        // P1-2: NoSelection also goes through show_at_sized.
                        let (px, py, pw, ph) =
                            popup::compute_popup_geometry_logical(popup::PopupMode::Error, &anchor);
                        let _ = popup::show_at_sized(app, px, py, pw, ph);
                        let _ = popup::error(
                            app,
                            if !a11y::enabled() {
                                "No selection captured. Grant Accessibility in System Settings → Privacy → Accessibility."
                            } else {
                                "No text selected."
                            },
                        );
                        return;
                    }
                    Err(e) => {
                        let anchor = match build_popup_anchor(app, x, y) {
                            Some(a) => a,
                            None => return,
                        };
                        let (px, py, pw, ph) =
                            popup::compute_popup_geometry_logical(popup::PopupMode::Error, &anchor);
                        let _ = popup::show_at_sized(app, px, py, pw, ph);
                        let _ = popup::error(app, &e);
                        return;
                    }
                }
            };
            if !state.gen.is_latest(gen) {
                return;
            }
            let text = match captured {
                Ok(t) => t,
                Err(_) => return, // handled inside the lock block above
            };
            let anchor = match build_popup_anchor(app, x, y) {
                Some(a) => a,
                None => return,
            };
            (text, anchor)
        }
    };

    // 2. Show loading popup sized + clamped, carrying the source (P1-3).
    if !state.gen.is_latest(gen) {
        return;
    }
    let _ = popup::loading_with_source(app, &anchor, Some(&text));

    // 3. client/keystore guards acquired from Session FIRST (mirrors
    //    translate_clipboard lib.rs:347-364 — these are NOT under the data_gate).
    //    rev-6-1: every error path here carries &text via error_with_source so
    //    Retry stays available (the source is now known).
    let client = match state.client.as_ref() {
        Some(c) => c.clone(),
        None => {
            if state.gen.is_latest(gen) {
                let _ = popup::set_popup_mode(app, popup::PopupMode::Error, &anchor);
                let _ = popup::error_with_source(
                    app,
                    "HTTP client unavailable: startup build failed (recovery required)",
                    &text,
                );
            }
            return;
        }
    };
    let keystore = match state.keystore.as_ref() {
        Some(k) => k,
        None => {
            if state.gen.is_latest(gen) {
                let _ = popup::set_popup_mode(app, popup::PopupMode::Error, &anchor);
                let _ = popup::error_with_source(
                    app,
                    "keystore unavailable: startup init failed (recovery required)",
                    &text,
                );
            }
            return;
        }
    };

    // rev-9-1 (load-bearing, mirrors translate_clipboard lib.rs:366-387 EXACTLY):
    // acquire the db Arc via a `spawn_blocking` that takes `data_gate.read()` +
    // `require_ready_gated` INSIDE the blocking closure and returns the
    // `Arc<Database>`. The gate guard is DROPPED when `spawn_blocking` returns —
    // it is NEVER held across `run_translate_session(...).await` (a locking
    // antipattern). `parking_lot::RwLock::read()` returns `RwLockReadGuard`
    // DIRECTLY (NOT a Result), so there is NO `Ok/Err` match on the guard; only
    // the OUTER `spawn_blocking().await` result is matched on
    // `Result<Result<Arc<Database>, String>, JoinError>`.
    //
    // Note: `app_state` here is already `&Arc<AppState>` (capture_and_translate's
    // param), so `app_state.clone()` yields an OWNED `Arc<AppState>` that can move
    // into the blocking closure. (translate_clipboard uses `app_state.inner()`
    // because ITS `app_state` is a `tauri::State<Arc<AppState>>`; capture_and_translate
    // receives the Arc directly, so there is no `.inner()` step.)
    let app_arc = app_state.clone();
    let db = match tauri::async_runtime::spawn_blocking(move || -> Result<Arc<Database>, String> {
        let _gate = app_arc.data_gate.read();
        require_ready_gated(&app_arc, &_gate)
    })
    .await
    {
        Ok(Ok(db)) => db,
        Ok(Err(msg)) => {
            if state.gen.is_latest(gen) {
                let _ = popup::set_popup_mode(app, popup::PopupMode::Error, &anchor);
                let _ = popup::error_with_source(app, &msg, &text);
            }
            return;
        }
        Err(e) => {
            if state.gen.is_latest(gen) {
                let _ = popup::set_popup_mode(app, popup::PopupMode::Error, &anchor);
                let _ = popup::error_with_source(app, &format!("join error: {e}"), &text);
            }
            return;
        }
    };

    // rev-9-1: re-check the generation token AFTER spawn_blocking returns (the
    // await boundary) and BEFORE the session. A stale run never reaches the emit.
    if !state.gen.is_latest(gen) {
        return;
    }

    // 4. run_translate_session — to:"" is resolved centrally inside it (Step 4).
    //    rev-9-1: `db` is the `Arc<Database>` from spawn_blocking; pass `&db`
    //    (`&Arc<Database>`, verified fn signature at lib.rs:492-505).
    let session_result = match run_translate_session(
        &db, &client, keystore, app, &text, "auto", "",
    )
    .await
    {
        Ok(r) => r,
        Err(msg) => {
            if state.gen.is_latest(gen) {
                let _ = popup::set_popup_mode(app, popup::PopupMode::Error, &anchor);
                let _ = popup::error_with_source(app, &msg, &text);
            }
            return;
        }
    };
    if !state.gen.is_latest(gen) {
        return;
    }

    // 5. Route per decision + size per state. P1-3: every route carries source_text.
    match decide_clipboard_popup(&session_result) {
        ClipboardPopupDecision::SingleSuccess { text: t, engine } => {
            let _ = popup::set_popup_mode(app, popup::PopupMode::Single, &anchor);
            let _ = popup::result_with_source(app, &t, &engine, &text);
        }
        ClipboardPopupDecision::Multi => {
            let _ = popup::set_popup_mode(app, popup::PopupMode::Multi, &anchor);
            let _ = popup::multi_result_with_source(app, &session_result.outcomes, &text);
        }
        ClipboardPopupDecision::Error(msg) => {
            let _ = popup::set_popup_mode(app, popup::PopupMode::Error, &anchor);
            let _ = popup::error_with_source(app, &msg, &text);
        }
    }
}

/// Build a PopupAnchor from the physical cursor coords. The scale factor used
/// to convert the work area AND the cursor is the TARGET MONITOR's
/// `scale_factor()` — NOT the popup window's. On a multi-monitor setup the
/// popup window may live on a different-density display than the cursor; using
/// the window's factor converted the target monitor's coordinates by the wrong
/// number and the popup landed off-screen (rev-7-1).
///
/// rev-7-1 (verified against Tauri 2.11.5 source):
/// - `AppHandle::monitor_from_point(x: f64, y: f64) -> Result<Option<Monitor>>` (f64, NOT i32).
/// - `Monitor::work_area(&self) -> &PhysicalRect<i32, u32>` returns the REAL usable work area.
/// - `Monitor::scale_factor(&self) -> f64` returns `f64` DIRECTLY (NOT `Result`).
///   This is the factor used to convert the target monitor's work area + the cursor.
/// - `WebviewWindow::scale_factor(&self) -> Result<f64>` — ONLY the fallback when
///   `monitor_from_point` returns `None` (e.g. a headless test). `unwrap_or(1.0)`.
/// - The scale factor is validated (`sf > 0.0 && sf.is_finite()`) before use so a
///   degenerate value cannot poison the division; a bad value falls back to 1.0.
fn build_popup_anchor(app: &tauri::AppHandle, x_phys: f64, y_phys: f64) -> Option<popup::PopupAnchor> {
    use tauri::Manager;
    let win = app.get_webview_window("popup")?;

    // rev-7-1: resolve the TARGET MONITOR first. Its scale_factor converts BOTH
    // its own work_area and the cursor. Fall back to the primary monitor, then
    // (headless) to the popup window's factor.
    let monitor = app
        .monitor_from_point(x_phys, y_phys)
        .ok()
        .flatten()
        .or_else(|| app.primary_monitor().ok().flatten());

    // The scale factor comes from the TARGET MONITOR when available; only when
    // no monitor resolved do we fall back to the popup window's factor.
    let mut sf = match &monitor {
        Some(m) => m.scale_factor(),
        None => win.scale_factor().unwrap_or(1.0),
    };
    // rev-7-1: guard against a degenerate scale (NaN / 0 / negative) that would
    // divide by zero or flip coordinates. Fall back to 1.0.
    if !(sf > 0.0 && sf.is_finite()) {
        sf = 1.0;
    }
    let cursor_logical = (x_phys / sf, y_phys / sf);

    let work_area_logical = if let Some(m) = &monitor {
        // Monitor::work_area() returns the REAL usable rect (accounts for the OS
        // menu bar / dock / task bar). Convert physical -> logical via the SAME
        // target-monitor scale factor used for the cursor.
        let wa = m.work_area();
        let pos = &wa.position;
        let sz = &wa.size;
        let left = pos.x as f64 / sf;
        let top = pos.y as f64 / sf;
        let right = left + sz.width as f64 / sf;
        let bottom = top + sz.height as f64 / sf;
        popup::LogicalWorkArea { left, top, right, bottom }
    } else {
        // No monitor at all (headless test): clamp to a 1x1 logical area at the
        // cursor so the popup is forced to the margin. Documented fallback.
        let (cx, cy) = cursor_logical;
        popup::LogicalWorkArea { left: cx, top: cy, right: cx + 1.0, bottom: cy + 1.0 }
    };

    Some(popup::PopupAnchor {
        cursor_logical,
        work_area: work_area_logical,
        scale_factor: sf,
    })
}
```

> **rev-7-1 work-area + scale note (load-bearing):** `Monitor::work_area(&self) -> &PhysicalRect<i32, u32>` (`tauri-2.11.5/src/window/mod.rs:96`) returns the real OS-reported usable work area (already accounts for the macOS menu bar, the Windows task bar, docks). `Monitor::scale_factor(&self) -> f64` (`tauri-2.11.5/src/window/mod.rs`) returns the factor DIRECTLY (not `Result`). `build_popup_anchor` resolves the target monitor via `app.monitor_from_point(x, y)` and uses THAT monitor's `scale_factor()` to convert both its `work_area()` and the physical cursor — so a Retina popup window showing a popup near a cursor on a 1× monitor converts with the 1× factor (and vice-versa). The popup window's `win.scale_factor().unwrap_or(1.0)` is ONLY the `None`-monitor fallback. The factor is guarded (`sf > 0.0 && sf.is_finite()`) before any division. There is NO `system_bar_deduction()` approximation.

The helper references `popup::result_with_source`, `popup::multi_result_with_source`, and `popup::error_with_source`. A3 added `loading_with_source`; B4 Step 3 adds the three result/error emitters. **To keep A2 compilable on its own, add minimal versions of these three emitters now (A2) and let B4 refine them:**

```rust
/// A2: emit result WITH source (B4 refines the serialization). Carries source_text
/// so Retry always has the original text (P1-3).
pub fn result_with_source(
    app: &tauri::AppHandle,
    text: &str,
    engine: &str,
    source_text: &str,
) -> Result<(), String> {
    let win = window(app)?;
    win.emit(
        "popup-state",
        Payload { status: "result", text, engine, source_text: Some(source_text) },
    )
    .map_err(|e| e.to_string())
}

/// A2: emit multi-result WITH source.
/// rev-6-2: PopupMultiPayload.source_text is `Option<String>` (OWNED), so the
/// `&str` arg is converted via `.to_owned()`. (A borrowed `Option<&str>` cannot
/// populate an `Option<String>` field; rev-5 shipped a duplicate body that used
/// `Some(source_text)` here — that version is DELETED.)
pub fn multi_result_with_source(
    app: &tauri::AppHandle,
    outcomes: &[crate::service::TranslationOutcome],
    source_text: &str,
) -> Result<(), String> {
    let win = window(app)?;
    let payload = PopupMultiPayload {
        outcomes: outcomes.iter().map(TranslationOutcomeSerialized::from).collect(),
        source_text: Some(source_text.to_owned()),
    };
    win.emit(POPUP_MULTI_EVENT, payload).map_err(|e| e.to_string())
}

/// A2: emit error WITH source so the popup can still offer Retry (P1-3).
pub fn error_with_source(
    app: &tauri::AppHandle,
    msg: &str,
    source_text: &str,
) -> Result<(), String> {
    let win = window(app)?;
    win.emit(
        "popup-state",
        Payload { status: "error", text: msg, engine: "", source_text: Some(source_text) },
    )
    .map_err(|e| e.to_string())
}
```

And extend `PopupMultiPayload` (popup.rs) with the source field. **rev-5-1 + rev-6-2:** `source_text` is `Option<String>` (OWNED) — a runtime `&str` (the captured/clipboard text) CANNOT be borrowed as `&'static str`, so the rev-4 `Option<&'static str>` form does not compile, AND a `&str` arg cannot populate an `Option<String>` field without `.to_owned()`. The single `multi_result_with_source` body above passes `Some(source_text.to_owned())`; there is no duplicate body (rev-6-2 deleted the rev-5 leftover that used the non-compiling `Some(source_text)`).

```rust
#[derive(Clone, serde::Serialize)]
struct PopupMultiPayload {
    outcomes: Vec<TranslationOutcomeSerialized>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_text: Option<String>,
}
```

The existing `multi_result` fn must be updated to pass `source_text: None`. The frontend `PopupMultiPayload.source_text?: string` (types.ts) is unchanged — `Option<String>` serializes to `string | undefined`.

- [x] **Step 6: Rewrite on_hotkey to call capture_and_translate**

In `src-tauri/src/lib.rs`, replace the spawn block body (lib.rs:2001-2124, i.e. everything inside `tauri::async_runtime::spawn(async move { ... })`) with:

```rust
    let state = app.state::<Arc<Session>>().inner().clone();
    let gen = state.gen.next();

    let app2 = app.clone();
    tauri::async_runtime::spawn(async move {
        let state = app2.state::<Arc<Session>>().inner().clone();
        let app_state = app2.state::<Arc<AppState>>().inner().clone();

        // Capture cursor position under the selection lock BEFORE the popup
        // steals focus. The capture_selection itself happens inside
        // capture_and_translate so hotkey/tray share it; but the cursor read
        // must precede any popup show.
        let (x, y) = {
            let _g = state.gen.selection_lock();
            let pos = cursor::position();
            (pos.0 as f64, pos.1 as f64)
        };

        capture_and_translate(&app2, &state, &app_state, None, x, y, gen).await;
    });
}
```

- [x] **Step 7: Build + run the full Rust suite**

Run:
- `cargo build --manifest-path src-tauri/Cargo.toml --features xproc-test-helper`
- `cargo test --manifest-path src-tauri/Cargo.toml --features xproc-test-helper`

Expected: clean build; the existing `translate_session.rs`, `fallback.rs`, `translate_parallel.rs`, and `popup_geometry` tests pass; the 5 new `hotkey_session` tests pass (including the grep assertion).

- [x] **Step 8: Commit**

```bash
git diff --check
git add src-tauri/src/lib.rs src-tauri/src/popup.rs src-tauri/tests/hotkey_session.rs
git commit -m "fix(hotkey): route Alt+Space through capture_and_translate -> run_translate_session; central to:\"\" resolution; gen-checked pipeline; grep-assert no translate_with_fallback (P1-1)"
```

---

### Task A4: Tauri permissions + clipboard plugin + tray-action listener + main window hidden + Switch Provider submenu + navigate event (Surface 04 Normal executable; Active/Error pulse deferred to A5; Update badge deferred per user-approved scope decision)

**Surface 04 scope declaration (rev-11/rev-12):** see the "Surface 审核快照（rev-12 用户已批准）" table at the top of this plan for the per-state implementation matrix. This task implements the Normal state and the menu actions; the Active-pulse and Error red-dot icon states are implemented in the FOLLOWING Task A5 (rev-11/rev-12) via the `TrayStateController` reducer; the Update-badge state is deferred to R5/R6 per user-approved scope decision (the R5/R6 updater backend does not exist).

- **Implemented now (this task):**
  - Menu items: Translate Selection / Translate Clipboard / Switch Provider (submenu of enabled providers) / Settings / Quit.
  - Icons: normal (Tauri default window icon). The frozen pages/04 red-dot-on-icon Error state and the Active-pulse state are implemented in **Task A5 (rev-14/rev-15/rev-16/rev-17/rev-18/rev-19)**; A4 ships only the Normal icon + the switch-provider failure tooltip (A4's intent was `"Switch failed: <msg>"`; **rev-21-2 (A5 Step 10 wrapper) sets the prefixed tooltip `format!("Switch failed: {msg}")`** — the visible textual signal complementing the red dot; verified in `handle_tray_menu_event` Step 9, which preserves the old primary when the write tx rolls back). **rev-15 P1-3 / rev-16 / rev-18-1 / rev-19-5 / rev-21-2:** A5 then layers the switch flow onto the extracted SYNC `handle_switch_provider` helper (rev-16 P2-2; rev-18-1: `pub fn` SYNC — `set_active_primary_core` is SYNC, no `.await`; rev-19-5: the wrapper sets the failure tooltip AFTER `refresh_tray_if_available` so the refresh's tooltip is not clobbered; rev-21-2: the tooltip carries the `"Switch failed: "` prefix): `let rev = tray.lock().begin_switch();` → `set_active_primary_core(...)` (SYNC) → `tray.lock().finish_switch(rev, success)` (rev-16-3: revision-tagged, NO gen arg, stale `rev != switch_revision` ignored) — switch does NOT touch the translation `GenerationToken`; the tray.switch arm runs the SYNC helper via `tauri::async_runtime::spawn_blocking`.
  - Status item shows the current primary provider name (read from db).
  - **rev-8-8 Refresh:** the tray menu is rebuilt after EVERY provider mutation — `provider_create`/`provider_update`/`provider_delete`/`provider_toggle`/`provider_reorder`/`provider_set_active`/`provider_duplicate`/`provider_confirm_and_set_active` — via `refresh_tray_if_available(&app_handle)` on each command's success path (Step 9b). Each of the eight commands gains an `app_handle: tauri::AppHandle` parameter (they previously had none) and renames the local `app` (AppState clone) to `app_state`.
- **Disabled + labeled "Coming later":** OCR / History (the menu items exist and are disabled with the real copy; they emit `tray-action` but the handler is a no-op).
- **Deferred per user-approved scope decision (rev-11):** Update badge (R5/R6 updater does not exist). This is NOT wired this stage. The Active-pulse + Error red-dot states are implemented in the immediately following Task A5 (rev-11/rev-12 — rev-12 makes the pulse a real icon frame-swap and the red-dot a true overlay on the base icon), NOT deferred.

**Switch Provider (P1-5):** a `SubmenuBuilder` lists the enabled providers (read via the db at tray-build time). Clicking one calls the extracted `set_active_primary_core(app_state, uuid)` (sync) inside `spawn_blocking`, then refreshes the status item label.

**navigate event (P1-5):** `App.tsx` listens for `navigate` (emitted by `open_settings_window`) and sets `SettingsShell`'s `activePage` signal (controlled prop).

**Files:**
- Modify: `src/App.tsx` — add `listen("tray-action", ...)` + `listen("navigate", ...)`; `translate-selection` calls `translateSelection`; `switch-provider` opens Settings.
- Modify: `src/features/settings/SettingsShell.tsx` — `activePage` is a controlled signal (settable via prop + onNavigate).
- Modify: `src-tauri/src/lib.rs` `build_tray` — full Switch Provider submenu (P1-5); disable OCR/History with "Coming later"; status item reads primary provider name; `set_active_primary_core` extracted.
- Modify: `src-tauri/tauri.conf.json` — main window `visible: false`.
- Modify: `src-tauri/build.rs` — add `translate_session`, `translate_selection_ipc`, `provider_get_active_selection`, `open_settings_window`.
- Modify: `src-tauri/Cargo.toml` — add `tauri-plugin-clipboard-manager = "2"` (P1-6, BEFORE capabilities reference it).
- Modify: `src-tauri/capabilities/input.json` — add `allow-translate-session` + `allow-provider-list` (P1-6).
- Modify: `src-tauri/capabilities/popup.json` — add popup-window permissions + clipboard.
- Modify: `src-tauri/capabilities/main.json` — add new commands.
- Create: `src-tauri/tests/capabilities.rs` **(new)** (P1-6 capability-set integration test).
- Test: `test/tray-action.test.tsx` **(new)**.

**Interfaces:**
- Produces (backend):
  - `fn set_active_primary_core(app_state: Arc<AppState>, uuid: String) -> Result<SetActiveResult, String>` — the sync core of `provider_set_active` (spawn_blocking + gate + db), reused by the tray (P1-5).
  - `open_settings_window(section: Option<String>) -> Result<(), String>` — shows main + emits `navigate`.
  - `translate_selection_ipc(text: Option<String>) -> Result<(), ()>` — `text = Some(t)` (Retry) uses the saved SOURCE; `text = None` (tray) captures fresh. NEVER reads the clipboard.
  - `provider_get_active_selection() -> ActiveSelection`.
- Produces (frontend): App.tsx mounts `listen("tray-action", ...)` + `listen("navigate", ...)`; `SettingsShell` accepts `activePage` + `onNavigate` props (controlled).

- [x] **Step 1: Add the clipboard plugin to Cargo.toml (P1-6, before capabilities)**

Edit `src-tauri/Cargo.toml`, add to the `[dependencies]` table:

```toml
tauri-plugin-clipboard-manager = "2"
```

Register the plugin in `src-tauri/src/lib.rs` inside `tauri::Builder::default()` (find the existing `.plugin(...)` chain and append):

```rust
        .plugin(tauri_plugin_clipboard_manager::init())
```

- [x] **Step 2: Add the missing backend commands to build.rs (permission manifest source)**

`src-tauri/build.rs` currently lacks `translate_session`, `translate_selection_ipc`, `provider_get_active_selection`, `open_settings_window`. Add all four to the `.commands(&[...])` list:

```rust
                    .commands(&[
                        "translate", "translate_default", "translate_clipboard",
                        "translate_session",
                        "translate_selection_ipc",
                        "list_engines", "set_key", "delete_key", "key_status",
                        "get_settings", "set_setting",
                        "a11y_status", "keystore_health", "archive_keystore", "reset_keystore",
                        "get_data_readiness",
                        "provider_list", "provider_create", "provider_update",
                        "provider_duplicate", "provider_delete", "provider_reorder",
                        "provider_toggle", "provider_set_key", "provider_set_active",
                        "provider_get_active_selection",
                        "provider_confirm_and_set_active",
                        "provider_get_models", "provider_test_connection",
                        "archive_database",
                        "open_settings_window",
                    ]),
```

Run `cargo build --manifest-path src-tauri/Cargo.toml --features xproc-test-helper` and confirm the autogenerated permission TOMLs now include `translate_session.toml`, `translate_selection_ipc.toml`, `provider_get_active_selection.toml`, `open_settings_window.toml` under `src-tauri/permissions/autogenerated/`.

- [x] **Step 3: Authorize the commands in capabilities (P1-6)**

`src-tauri/capabilities/input.json` — add `allow-translate-session` AND `allow-provider-list`:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "input",
  "description": "Capability for the input window — translate_session (multi-engine) + translate_default + provider_list",
  "windows": ["input"],
  "permissions": [
    "core:default",
    "allow-translate-default",
    "allow-translate-session",
    "allow-provider-list"
  ]
}
```

`src-tauri/capabilities/main.json` — add the new commands:

```json
  "permissions": [
    "core:default",
    "allow-translate",
    "allow-translate-default",
    "allow-translate-clipboard",
    "allow-translate-session",
    "allow-translate-selection-ipc",
    "allow-list-engines",
    "allow-set-key",
    "allow-delete-key",
    "allow-key-status",
    "allow-get-settings",
    "allow-set-setting",
    "allow-a11y-status",
    "allow-keystore-health",
    "allow-archive-keystore",
    "allow-reset-keystore",
    "allow-get-data-readiness",
    "allow-provider-list",
    "allow-provider-create",
    "allow-provider-update",
    "allow-provider-duplicate",
    "allow-provider-delete",
    "allow-provider-reorder",
    "allow-provider-toggle",
    "allow-provider-set-key",
    "allow-provider-set-active",
    "allow-provider-confirm-and-set-active",
    "allow-provider-get-active-selection",
    "allow-provider-get-models",
    "allow-provider-test-connection",
    "allow-archive-database",
    "allow-open-settings-window"
  ]
```

`src-tauri/capabilities/popup.json` — add popup-window permissions + clipboard:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "popup",
  "description": "Capability for the popup window — hide, friendly labels, retry, settings/recovery CTAs, clipboard",
  "windows": ["popup"],
  "permissions": [
    "core:default",
    "core:window:allow-hide",
    "allow-provider-list",
    "allow-provider-get-active-selection",
    "allow-translate-selection-ipc",
    "allow-open-settings-window",
    "clipboard-manager:allow-write-text"
  ]
}
```

- [x] **Step 4: Add the capability-set integration test (P1-6)**

Create `src-tauri/tests/capabilities.rs`:

```rust
//! Task A4 (P1-6): assert the capability set contains every required permission.
//! This is an integration test, not a grep — it parses the JSON and validates the
//! structure so a missing/misnamed permission fails loudly.
use std::collections::HashSet;
use std::fs;

fn permission_set(path: &str) -> HashSet<String> {
    let raw = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("read {path}: {e}"));
    let v: serde_json::Value = serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("parse {path}: {e}"));
    v["permissions"]
        .as_array()
        .unwrap_or_else(|| panic!("{path} has no permissions array"))
        .iter()
        .map(|p| p.as_str().unwrap_or("").to_string())
        .collect()
}

#[test]
fn input_window_authorizes_session_and_provider_list() {
    let perms = permission_set("capabilities/input.json");
    for required in ["allow-translate-default", "allow-translate-session", "allow-provider-list"] {
        assert!(
            perms.contains(required),
            "input.json missing {required}; has: {:?}",
            perms
        );
    }
}

#[test]
fn popup_window_authorizes_selection_clipboard_and_settings() {
    let perms = permission_set("capabilities/popup.json");
    for required in [
        "allow-provider-list",
        "allow-provider-get-active-selection",
        "allow-translate-selection-ipc",
        "allow-open-settings-window",
        "clipboard-manager:allow-write-text",
    ] {
        assert!(
            perms.contains(required),
            "popup.json missing {required}; has: {:?}",
            perms
        );
    }
}

#[test]
fn main_window_authorizes_every_new_command() {
    let perms = permission_set("capabilities/main.json");
    for required in [
        "allow-translate-session",
        "allow-translate-selection-ipc",
        "allow-provider-get-active-selection",
        "allow-open-settings-window",
    ] {
        assert!(
            perms.contains(required),
            "main.json missing {required}; has: {:?}",
            perms
        );
    }
}
```

`serde_json` is already a dependency of the crate (verified at `src-tauri/Cargo.toml:44` — `serde_json = "1"` under `[dependencies]`), so it is available to the integration test with no `Cargo.toml` edit.

- [x] **Step 5: Write the failing frontend test**

Create `test/tray-action.test.tsx` (uses the verified `vi.hoisted + invokeMock` pattern):

```tsx
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, cleanup } from "@solidjs/testing-library";

const { invokeMock, listenMock, unlistenMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(async (_cmd: string, _args?: unknown): Promise<unknown> => undefined),
  listenMock: vi.fn(async (_event: string, _cb: (e: { payload: unknown }) => void) => () => {}),
  unlistenMock: vi.fn(() => {}),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: listenMock,
}));
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));
vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: vi.fn() }));

vi.mock("../src/features/settings/SettingsShell", () => ({
  default: (props: { activePage?: string; children: unknown }) => (
    <div data-testid="shell" data-page={props.activePage}>{props.children}</div>
  ),
}));
vi.mock("../src/features/settings/ProviderCenter", () => ({
  default: () => <div data-testid="provider-center" />,
}));
vi.mock("../src/features/settings/KeystoreRecovery", () => ({
  default: () => <div data-testid="keystore-recovery" />,
}));

import App from "../src/App";

beforeEach(() => {
  invokeMock.mockReset();
  invokeMock.mockResolvedValue(undefined);
  listenMock.mockReset();
  listenMock.mockResolvedValue(unlistenMock);
});

afterEach(() => cleanup());

async function getHandler(event: string): Promise<(e: { payload: string }) => void> {
  const call = listenMock.mock.calls.find((c) => c[0] === event);
  if (!call) throw new Error(`no listen("${event}") registered; calls: ${listenMock.mock.calls.map((c) => c[0]).join(",")}`);
  return call[1] as (e: { payload: string }) => void;
}

describe("App tray-action + navigate listeners", () => {
  it("registers tray-action and navigate listeners on mount", async () => {
    render(() => <App />);
    const events = listenMock.mock.calls.map((c) => c[0]);
    expect(events).toContain("tray-action");
    expect(events).toContain("navigate");
  });

  it("translate-clipboard action invokes translate_clipboard", async () => {
    render(() => <App />);
    const handler = await getHandler("tray-action");
    handler({ payload: "translate-clipboard" });
    await Promise.resolve();
    expect(invokeMock.mock.calls.some((c) => c[0] === "translate_clipboard")).toBe(true);
  });

  it("translate-selection action calls translate_selection_ipc, NOT translate_clipboard", async () => {
    render(() => <App />);
    const handler = await getHandler("tray-action");
    handler({ payload: "translate-selection" });
    await Promise.resolve();
    expect(invokeMock.mock.calls.some((c) => c[0] === "translate_selection_ipc")).toBe(true);
    expect(invokeMock.mock.calls.some((c) => c[0] === "translate_clipboard")).toBe(false);
  });

  it("switch-provider action opens settings on the provider page", async () => {
    const { findByTestId } = render(() => <App />);
    const handler = await getHandler("tray-action");
    handler({ payload: "switch-provider" });
    await Promise.resolve();
    const shell = await findByTestId("shell");
    expect(shell.getAttribute("data-page")).toBe("provider-center");
  });

  it("navigate event sets the active page on the shell", async () => {
    const { findByTestId } = render(() => <App />);
    const navHandler = await getHandler("navigate");
    navHandler({ payload: "keystore-recovery" });
    await Promise.resolve();
    const shell = await findByTestId("shell");
    expect(shell.getAttribute("data-page")).toBe("keystore-recovery");
  });
});
```

- [x] **Step 6: Run test to verify it fails**

Run: `pnpm vitest run test/tray-action.test.tsx`
Expected: FAIL — the listeners are not registered yet.

- [x] **Step 7: Add the selection-ipc module + the listeners to App.tsx**

Create `src/features/translation/selection-ipc.ts`:

```ts
import { invoke } from "@tauri-apps/api/core";

/**
 * Translate the live OS selection (fresh capture) OR a caller-supplied SOURCE
 * text (Retry). Calls the backend `translate_selection_ipc` command which NEVER
 * reads the clipboard. Distinct from `translateClipboard` which reads the
 * clipboard. Used by the tray `translate-selection` action and the popup Retry.
 *
 * P1-3: `sourceText` is the ORIGINAL selected text, not a translation result.
 */
export const translateSelection = (sourceText?: string): Promise<void> =>
  invoke<void>("translate_selection_ipc", sourceText !== undefined ? { text: sourceText } : {});

/** Translate the clipboard contents. Distinct from selection translation. */
export const translateClipboard = (): Promise<void> =>
  invoke<void>("translate_clipboard");
```

Replace `src/App.tsx` with:

```tsx
/**
 * R3a App mount + R2/R3a audit Task A4: hosts the tray-action + navigate
 * listeners. The tray (Surface 04) emits `tray-action`; `open_settings_window`
 * emits `navigate`. The shell's activePage is a CONTROLLED signal (P1-5) so the
 * tray / popup CTAs can drive navigation.
 *
 * Surface 04 scope (rev-10): normal icon, provider name status,
 * translate-selection/clipboard/switch-provider/settings/quit are live. OCR +
 * History are disabled with "Coming later". Update badge, active-translation
 * pulse, and Balance are not implemented (see Surface status table).
 */
import { createSignal, onCleanup, onMount, type Component } from "solid-js";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import SettingsShell, { type SettingsSection } from "./features/settings/SettingsShell";
import ProviderCenter from "./features/settings/ProviderCenter";
import KeystoreRecovery from "./features/settings/KeystoreRecovery";
import { SETTINGS_COPY } from "./features/settings/copy";
import { translateSelection, translateClipboard } from "./features/translation/selection-ipc";
import { detectLocale } from "./i18n";

const App: Component = () => {
  const locale = detectLocale();
  const t = SETTINGS_COPY[locale];
  // rev-7-2: activePage uses the EXISTING SettingsSection union (no new type).
  // It is passed as the `activePage` prop so the parent controls the shell.
  const [activePage, setActivePage] = createSignal<SettingsSection>("provider-center");
  const unlisteners: UnlistenFn[] = [];

  onMount(async () => {
    unlisteners.push(
      await listen<string>("tray-action", (e) => {
        const action = e.payload;
        if (action === "translate-clipboard") {
          void translateClipboard();
        } else if (action === "translate-selection") {
          void translateSelection();
        } else if (action === "ocr-capture") {
          // Disabled in the menu (Coming later); no-op here.
        } else if (action === "switch-provider" || action === "settings") {
          setActivePage("provider-center");
        }
      }),
    );
    unlisteners.push(
      await listen<string>("navigate", (e) => {
        const page = e.payload as SettingsSection;
        if (
          page === "provider-center" ||
          page === "keystore-recovery" ||
          page === "shortcuts" ||
          page === "privacy"
        ) {
          setActivePage(page);
        }
      }),
    );
  });

  onCleanup(() => {
    for (const u of unlisteners) u();
  });

  return (
    <SettingsShell activePage={activePage()} onNavigate={setActivePage}>
      {activePage() === "provider-center" ? (
        <ProviderCenter />
      ) : activePage() === "keystore-recovery" ? (
        <KeystoreRecovery />
      ) : (
        <section class="app__placeholder" aria-label={t.nav.placeholderHint}>
          <p>{t.nav.placeholderHint}</p>
        </section>
      )}
    </SettingsShell>
  );
};

export default App;
```

**rev-8-1 (load-bearing): do NOT rewrite `SettingsShell`.** The existing `src/features/settings/SettingsShell.tsx` already renders `WindowChrome` + `SidebarItem`s + `Tooltip` (for disabled + rail items) + the `matchMedia("(min-width: 700px)")` responsive signal (≥700px full labels / 600-699px icon rail) + lazy `getCurrentWindow().close()`/`minimize()` handlers + a disabled-placeholder tooltip (`t.nav.placeholderHint` = "Coming in R3b" / "将在 R3b 中提供"). Apply ONLY these precise modifications to the EXISTING file — do NOT copy the whole file, do NOT leave `// ... existing ...` placeholders:

In `src/features/settings/SettingsShell.tsx`, make these precise edits:

1. **`SettingsShellProps` type — add the optional controlled field.** Insert `activePage?: SettingsSection;` alongside the existing `initialSection?`/`onNavigate?`/`children`:

```diff
 export type SettingsShellProps = {
   /** Initial active section (default: "provider-center"). */
   initialSection?: SettingsSection;
+  /** Controlled active section. When supplied, the parent owns the active
+   *  state; the shell reads `props.activePage` instead of its own signal. */
+  activePage?: SettingsSection;
   /** Called when the user clicks an enabled nav item. */
   onNavigate?: (section: SettingsSection) => void;
   /** Content for the currently-active section. */
   children: JSX.Element;
 };
```

2. **`active` state — TRUE controlled/uncontrolled dual mode (rev-9-2).** The existing line is `const [active, setActive] = createSignal<SettingsSection>(props.initialSection ?? "provider-center");`. rev-8-1 changed the initializer to `createSignal(props.activePage ?? props.initialSection ?? ...)` but a `createSignal` initializer runs ONCE — when the parent later passes a NEW `activePage` (a `navigate` event arrives), `active()` would NOT update and the sidebar highlight + `data-page` would stay stale. **rev-9-2 makes `active` a DERIVATION** of `props.activePage` (the controlled source of truth) falling back to an internal signal (uncontrolled mode). Rename the existing signal to `internalActive`/`setInternalActive` and introduce the `active` accessor:

```diff
-  const [active, setActive] = createSignal<SettingsSection>(
-    props.initialSection ?? "provider-center",
-  );
+  // rev-9-2: controlled-uncontrolled dual mode. `active` is a DERIVATION of
+  // props.activePage (when the parent supplies it, the parent owns the state)
+  // falling back to the internal signal (uncontrolled mode). A plain
+  // createSignal(props.activePage ?? ...) initializer would read props.activePage
+  // ONCE at first render and then go stale when the parent passes a new value.
+  const [internalActive, setInternalActive] = createSignal<SettingsSection>(
+    props.initialSection ?? "provider-center",
+  );
+  const active = (): SettingsSection => props.activePage ?? internalActive();
```

3. **`handleClick` — write the internal signal ONLY in uncontrolled mode (rev-9-2).** The existing body is `setActive(id); props.onNavigate?.(id);`. Replace `setActive(id)` with the conditional write so a controlled shell does not fight the parent (the parent updates `activePage` via `onNavigate`); an uncontrolled shell still self-updates so the click highlight reflects the user's pick. Keep the `onNavigate` call UNCONDITIONAL so the parent is always notified:

```diff
   const handleClick = (id: SettingsSection) => {
-    setActive(id);
+    // rev-9-2: only mutate the internal signal in UNCONTROLLED mode
+    // (props.activePage === undefined). In controlled mode the parent is the
+    // source of truth and updates `activePage` via the onNavigate callback.
+    if (props.activePage === undefined) {
+      setInternalActive(id);
+    }
     props.onNavigate?.(id);
   };
```

4. **`renderItem` — disabled item `ariaLabel`.** The existing line `const content = item.disabled ? t.nav.placeholderHint : item.label;` drives the Tooltip text. Pass an `ariaLabel` to `SidebarItem` so the focusable disabled button announces BOTH its label AND the placeholder hint (rev-8-1: the real copy value, not a "Coming later" invention). `active()` is now the derivation from edit 2 — the `active={active() === item.id}` highlight now follows `props.activePage` in controlled mode (rev-9-2):

```diff
   const renderItem = (item: NavDef) => {
+    const ariaLabel = item.disabled
+      ? `${item.label} — ${t.nav.placeholderHint}`
+      : item.label;
     const node = (
       <SidebarItem
         label={item.label}
+        ariaLabel={ariaLabel}
         icon={item.icon}
         active={active() === item.id}
         disabled={item.disabled}
         onClick={() => handleClick(item.id)}
       />
     );
```

(`t.nav.placeholderHint` is the REAL copy value verified at `src/features/settings/copy.ts:206` = `"Coming in R3b"` (en) and `:325` = `"将在 R3b 中提供"` (zh). The `ariaLabel` therefore reads e.g. `"Shortcuts — Coming in R3b"`. No invented copy.)

5. **Root element — add `data-testid` + `data-page`.** The existing root `<div class="settings-shell" data-layout={wide() ? "full" : "rail"}>` gains the two attributes. `data-page={active()}` now reflects the rev-9-2 derivation so it tracks `props.activePage` reactively:

```diff
-    <div class="settings-shell" data-layout={wide() ? "full" : "rail"}>
+    <div
+      class="settings-shell"
+      data-layout={wide() ? "full" : "rail"}
+      data-testid="shell"
+      data-page={active()}
+    >
```

Everything else in the file — the `matchMedia` signal + `onCleanup` subscription, the `navItems` array, `handleClose`/`handleMinimize` (lazy `getCurrentWindow()`), and the `<WindowChrome>` block — is UNCHANGED.

**Disabled nav items use `aria-disabled` + `tabindex={0}`** via the `SidebarItem` update in C5 Step 4 (the `ariaLabel` from edit 4 above flows through unchanged). The disabled items' `Tooltip` content (`t.nav.placeholderHint`) stays the real copy value.

The shell's root now carries `data-testid="shell"` + `data-page={active()}` (rev-9-2: `active()` is the derivation, so `data-page` updates reactively when the parent changes `props.activePage`) so Playwright + real-DOM Vitest can read the active page WITHOUT mocking SettingsShell (rev-7-6 removes the mock dependency in the keyboard spec). The tray-action Vitest test still mocks SettingsShell (Step 5) because it is a unit test of the App listeners, not the shell DOM. The new controlled-component test in C5 Step 1 (`setPage("keystore-recovery")` → `data-page="keystore-recovery"`) uses the REAL (un-mocked) shell and verifies the reactivity.

- [x] **Step 8: Run test to verify it passes**

Run: `pnpm vitest run test/tray-action.test.tsx`
Expected: PASS (5 tests).

- [x] **Step 9: Extract set_active_primary_core + build the executable tray (P1-5)**

In `src-tauri/src/lib.rs`, extract the sync core of `provider_set_active` (lib.rs:1288) so the tray can call it without the `tauri::State` handle. Add near `provider_set_active`. **rev-5-4 (load-bearing):** uses the REAL write helpers `set_active_slots` (lib.rs:1711) — there is NO `db_providers::write_active_selection` — and matches the REAL `SetActiveOutcome { Written, NeedsConsent { actual_scope } }` variants (lib.rs:1672) with their `actual_scope` field. Because `parallel = []` + `fallback = None`, the consent gate (`if !parallel.is_empty()`) is never entered and the function always writes via `set_active_slots` (the no-parallel branch). No nested `spawn_blocking → block_on → spawn_blocking`: this fn is itself the body the caller runs inside a `spawn_blocking`.

```rust
/// P1-5 + rev-5-4: the sync core of provider_set_active, callable from the tray
/// (which cannot resolve tauri::State). Sets `uuid` as the sole primary, no
/// parallel, no fallback. This is the BODY the tray handler runs inside a
/// `spawn_blocking` — do NOT wrap it in another `block_on(spawn_blocking(...))`.
/// Uses the real write helper `set_active_slots` (lib.rs:1711); there is no
/// `db_providers::write_active_selection`. Because `parallel` is empty, the
/// consent gate is never entered and the NeedsConsent branch is unreachable
/// (kept in the match for exhaustiveness; if it ever fires it maps through).
fn set_active_primary_core(
    app_state: Arc<AppState>,
    uuid: String,
) -> Result<SetActiveResult, String> {
    let app = app_state.clone();
    let outcome = db_set_active_primary(&app, &uuid)?;
    Ok(match outcome {
        SetActiveOutcome::Written => SetActiveResult::Written,
        SetActiveOutcome::NeedsConsent { actual_scope } => {
            SetActiveResult::NeedsConsent { actual_scope }
        }
    })
}

/// rev-5-4: the gate + transaction that `set_active_primary_core` and the tray
/// share. Acquires the write gate, runs `validate_active_selection` + the
/// `set_active_slots` write inside ONE transaction. Returns the internal
/// `SetActiveOutcome` so the caller can map it to the serialized result.
fn db_set_active_primary(
    app: &Arc<AppState>,
    uuid: &str,
) -> Result<SetActiveOutcome, String> {
    let _gate = app.data_gate.write();
    let db = require_ready_gated_write(app, &_gate)?;
    let outcome = db
        .with_conn(|conn| -> Result<SetActiveOutcome, DbErr> {
            let tx = conn.transaction()?;
            let active = db_providers::list(&tx)?;
            db_providers::validate_active_selection(uuid, &[], None, &active)?;
            // parallel is empty → set_active_slots (clears prior consent).
            set_active_slots(&tx, uuid, &[], None)?;
            tx.commit()?;
            Ok(SetActiveOutcome::Written)
        })
        .map_err(|e| e.to_string())?;
    Ok(outcome)
}
```

**rev-5-4 mapping note:** `provider_set_active` (lib.rs:1288) keeps its own closure body AS-IS (it handles the non-empty-parallel consent path); `db_set_active_primary` is the parallel-empty fast path the tray needs. Both call `set_active_slots` / `set_active_slots_keep_consent` (lib.rs:1711 / 1892) — the real write helpers. If `provider_set_active`'s closure is later refactored to call `db_set_active_primary` for the `parallel.is_empty()` arm, do that by replacing its inline write with a call to `db_set_active_primary(&app, &primary)` and keep the consent arm inline.

Rewrite `build_tray` (lib.rs:2157) with the FULL executable menu (P1-5: no pseudocode). Use `tauri::menu::SubmenuBuilder` for Switch Provider:

```rust
/// rev-5-4: build the tray for the FIRST time (registers `"main-tray"`).
/// Subsequent updates go through `refresh_tray` → `build_tray_menu` +
/// `tray.set_menu(...)` so we never register a duplicate tray id.
fn build_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};
    let menu = build_tray_menu(app)?;
    TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().cloned().expect("default window icon"))
        .menu(&menu)
        .tooltip(&read_primary_status(app))
        .show_menu_on_left_click(false)
        .on_menu_event(handle_tray_menu_event)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::DoubleClick { button: MouseButton::Left, .. } = event {
                let app = tray.app_handle();
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
        })
        .build(app)?;
    Ok(())
}

/// rev-5-4: build ONLY the menu (reusable by build_tray + refresh_tray). Returns
/// the full menu with the fresh provider list + status item text.
fn build_tray_menu(app: &tauri::AppHandle) -> tauri::Result<tauri::menu::Menu<tauri::Wry>> {
    use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};

    // Quick actions group.
    let sel = MenuItem::with_id(app, "tray.translate-selection", "Translate Selection", true, None::<&str>)?;
    let clip = MenuItem::with_id(app, "tray.translate-clipboard", "Translate Clipboard", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;

    // Switch Provider submenu: built from the db at menu-build time;
    // refresh_tray() rebuilds it after provider_list/toggle/set_active.
    let switch_sub = build_switch_provider_submenu(app)?;
    let provider_status = MenuItem::with_id(app, "tray.provider-status", &read_primary_status(app), false, None::<&str>)?;
    let sep2 = PredefinedMenuItem::separator(app)?;

    // Disabled "Coming later" items (P1-D).
    let ocr = MenuItem::with_id(app, "tray.ocr-capture", "OCR Translate (Coming later)", false, None::<&str>)?;
    let history = MenuItem::with_id(app, "tray.history", "History (Coming later)", false, None::<&str>)?;
    let sep3 = PredefinedMenuItem::separator(app)?;

    // Navigation + system group.
    let settings = MenuItem::with_id(app, "tray.settings", "Settings", true, None::<&str>)?;
    let sep4 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "tray.quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[
        &sel, &clip, &sep1,
        &switch_sub, &provider_status, &sep2,
        &ocr, &history, &sep3,
        &settings, &sep4,
        &quit,
    ])?;
    Ok(menu)
}

/// Build the Switch Provider submenu from the enabled providers in the db. Each
/// item id encodes the uuid: `tray.switch-<uuid>`. Returns a Submenu.
fn build_switch_provider_submenu(app: &tauri::AppHandle) -> tauri::Result<tauri::menu::Submenu<tauri::Wry>> {
    use tauri::menu::{SubmenuBuilder, MenuItem};
    let mut sub = SubmenuBuilder::new(app, "Switch Provider");
    // Read enabled providers from the db (best-effort; empty submenu on error).
    let enabled: Vec<(String, String)> = read_enabled_providers(app).unwrap_or_default();
    for (uuid, name) in &enabled {
        let item = MenuItem::with_id(app, &format!("tray.switch-{uuid}"), name, true, None::<&str>)?;
        sub = sub.item(&item);
    }
    sub.build()
}

/// Read (uuid, name) for enabled providers. Best-effort: returns empty on db error.
/// rev-6-3: with_conn's closure returns Result<T, DbError> (the DbError enum at
/// db/mod.rs:39, aliased DbErr at lib.rs:1606), NOT rusqlite::Error. The
/// .map_err below maps DbError -> String.
fn read_enabled_providers(app: &tauri::AppHandle) -> Result<Vec<(String, String)>, String> {
    use tauri::Manager;
    let app_state = app.state::<Arc<AppState>>().inner().clone();
    let result = tauri::async_runtime::block_on(tauri::async_runtime::spawn_blocking(move || {
        let _gate = app_state.data_gate.read();
        let db = require_ready_gated(&app_state, &_gate)?;
        db.with_conn(|conn| {
            let list = db_providers::list(conn)?;
            Ok(list.into_iter().filter(|p| p.enabled).map(|p| (p.uuid, p.name)).collect::<Vec<_>>())
        })
        .map_err(|e: DbErr| e.to_string())
    }));
    match result {
        Ok(Ok(v)) => Ok(v),
        Ok(Err(e)) => {
            let _ = e;
            Ok(Vec::new())
        }
        Err(_) => Ok(Vec::new()),
    }
}

/// Read the primary provider name for the status item. Falls back to "No provider".
fn read_primary_status(app: &tauri::AppHandle) -> String {
    use tauri::Manager;
    let app_state = match app.try_state::<Arc<AppState>>() {
        Some(s) => s.inner().clone(),
        None => return "No provider".into(),
    };
    let result = tauri::async_runtime::block_on(tauri::async_runtime::spawn_blocking(move || {
        let _gate = app_state.data_gate.read();
        let db = match require_ready_gated(&app_state, &_gate) {
            Ok(d) => d,
            Err(_) => return "No provider".to_string(),
        };
        let selection = db.with_conn(|conn| db_providers::read_active_selection(conn));
        match selection {
            Ok(sel) => match sel.primary {
                Some(uuid) => {
                    let name = db.with_conn(|conn| db_providers::get(conn, &uuid)).ok().map(|p| p.name);
                    name.unwrap_or_else(|| "Unknown provider".into())
                }
                None => "No provider".into(),
            },
            Err(_) => "No provider".into(),
        }
    }));
    result.unwrap_or_else(|_| "No provider".into())
}

/// Refresh the tray menu + status after a provider mutation. Called from the
/// eight provider mutation command handlers (P1-5) via `refresh_tray_if_available`.
///
/// rev-9-3 (load-bearing): returns `tauri::Result<()>` so the best-effort wrapper
/// `refresh_tray_if_available` can match `if let Err(e) = refresh_tray(app)`.
/// (rev-8 declared this `pub fn refresh_tray(app) -> ()` but Step 9b's wrapper
/// wrote `if let Err(e) = refresh_tray(app)` — `()` has no `Err`, so that line
/// did not compile. rev-9-3 fixes the signature at the source.)
///
/// rev-5-4: refresh the EXISTING `"main-tray"` in place — rebuild the menu +
/// re-set the status tooltip via `app.tray_by_id("main-tray")`. Rebuilding from
/// scratch via `build_tray` would register a DUPLICATE tray icon (the old
/// `"main-tray"` is not destroyed when a second `TrayIconBuilder::with_id` runs
/// — Tauri panics on duplicate id). Instead, fetch the existing tray and update
/// its menu + tooltip. If the tray does not exist yet (first build), fall back
/// to `build_tray` (which itself returns `tauri::Result<()>`, so the `?`/return
/// composes). Errors from `set_menu`/`set_tooltip` are PROPAGATED (not swallowed
/// with `let _ =`) so the wrapper can log them.
pub fn refresh_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    use tauri::Manager;
    if let Some(tray) = app.tray_by_id("main-tray") {
        // Rebuild the menu (fresh provider list + status) and attach it.
        // rev-6-3: TrayIcon::set_menu takes Option<M>, so wrap in Some.
        let menu = build_tray_menu(app)?;
        tray.set_menu(Some(menu))?;
        tray.set_tooltip(Some(&read_primary_status(app)))?;
        Ok(())
    } else {
        // First build (no tray exists yet) — build_tray returns tauri::Result<()>.
        build_tray(app)
    }
}

/// rev-9-3: best-effort tray refresh after a provider mutation. Wraps
/// `refresh_tray` (which returns `tauri::Result<()>`) so a tray rebuild failure
/// (e.g. tray not yet built during startup) NEVER turns a successful provider
/// write into an error. Compiles because `refresh_tray` returns `Result`, not `()`.
pub fn refresh_tray_if_available(app: &tauri::AppHandle) {
    if let Err(e) = refresh_tray(app) {
        log::warn!("tray refresh failed: {e}");
    }
}
```

> **rev-5-4 + rev-6-3:** `build_tray` (above) is split so the menu construction is reusable. Extract the menu-only builder as `build_tray_menu(app: &tauri::AppHandle) -> tauri::Result<tauri::menu::Menu<tauri::Wry>>` containing the `Menu::with_items(...)` call; `build_tray` then calls `build_tray_menu` and wraps it in the `TrayIconBuilder::with_id("main-tray").build(app)`. `refresh_tray` calls `build_tray_menu` and `tray.set_menu(Some(menu))` (rev-6-3: `TrayIcon::set_menu` takes `Option<M>`, verified at `tauri-2.11.5/src/tray/mod.rs:512`) + `tray.set_tooltip(...)`. `TrayIcon::set_menu` and `TrayIcon::set_tooltip` are the Tauri 2.x APIs for updating an existing tray in place.

Update `handle_tray_menu_event` (lib.rs:2214) to handle `tray.switch-<uuid>`. **rev-5-4:** on failure, the old primary is PRESERVED (the write transaction rolled back, so nothing changed) and the tray shows the error in its tooltip; on success, the tray is refreshed. No nested `spawn_blocking → block_on → spawn_blocking` — the handler spawns ONE `spawn_blocking` that runs `set_active_primary_core` (which is the sync body, not an async wrapper). **rev-18-1 (A5 Step 10 supersedes this for the tray-state wiring):** the switch handler is split into a SYNC core `pub fn handle_switch_provider_core(app_state, uuid)` (DB + tray controller — no AppHandle; the testable entry) + a SYNC wrapper `pub fn handle_switch_provider(app, app_state, uuid)` (calls core + `refresh_tray_if_available(&app)`). Both call `set_active_primary_core(...)` directly (SYNC — no `.await`; `set_active_primary_core`'s body is the gate+tx sync payload). The tray.switch arm runs the SYNC wrapper inside `tauri::async_runtime::spawn_blocking(move || handle_switch_provider(&app2, &app_state, &uuid))` (rev-18-1: offload the SYNC SQLite I/O — NOT `spawn(async move { ... .await })`). The DB-level `set_active_primary_core` body itself is unchanged (still sync internally); only the tray arm's call boundary is the A5-extracted wrapper wrapped in `spawn_blocking`.

```rust
fn handle_tray_menu_event(app: &tauri::AppHandle, event: MenuEvent) {
    let id = event.id().as_ref();
    if let Some(uuid) = id.strip_prefix("tray.switch-") {
        // P1-5 + rev-5-4 + rev-18-1/rev-20-4: set this provider as the sole
        // primary, then refresh the tray. On failure, the write tx rolled back
        // (old primary preserved); the A5 wrapper (handle_switch_provider)
        // surfaces the error in the tray tooltip.
        //
        // rev-20-4: the switch arm calls the SYNC wrapper via spawn_blocking
        // (NOT spawn(async move { ... .await })). The wrapper
        // (handle_switch_provider, defined in A5 Step 10) does:
        //   handle_switch_provider_core(app_state, uuid)   // SYNC: DB + tray controller
        //   refresh_tray_if_available(&app)                // best-effort menu refresh
        //   on failure: tray.set_tooltip(Some(&format!("Switch failed: {msg}")))  // rev-19-5 (AFTER refresh) + rev-21-2 (prefix)
        // The core (handle_switch_provider_core) calls set_active_primary_core
        // directly (SYNC — no .await; its body is the gate+tx sync payload),
        // then tray.lock().finish_switch(rev, success) (rev-16-3 revision-tagged).
        use tauri::Manager;
        let app_state = app.state::<Arc<AppState>>().inner().clone();
        let app_clone = app.clone();
        let uuid_owned = uuid.to_string();
        // rev-18-1 / rev-20-4: offload the SYNC wrapper via spawn_blocking.
        // The wrapper is the sole entry — NO nested spawn(async move {...}),
        // NO .await in the arm.
        tauri::async_runtime::spawn_blocking(move || {
            let _ = handle_switch_provider(&app_clone, &app_state, &uuid_owned);
        });
        return;
    }
    match id {
        "tray.translate-selection" => {
            let _ = app.emit("tray-action", "translate-selection");
        }
        "tray.translate-clipboard" => {
            let _ = app.emit("tray-action", "translate-clipboard");
        }
        "tray.ocr-capture" => {
            let _ = app.emit("tray-action", "ocr-capture");
        }
        "tray.settings" => {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
                // rev-6-3: navigate value is a real SettingsPage union member
                // ("provider-center" | "keystore-recovery" | "general"), NOT the
                // generic "settings" string the SettingsPage type rejects.
                let _ = w.emit("navigate", "provider-center");
            }
        }
        "tray.quit" => {
            app.exit(0);
        }
        _ => {}
    }
}
```

> **rev-5-4 verified:** `SetActiveOutcome` (lib.rs:1672) is `enum { Written, NeedsConsent { actual_scope: String } }`; `SetActiveResult` (lib.rs:1620) mirrors it. The real write helper is `set_active_slots` (lib.rs:1711) — there is NO `write_active_selection` in `db_providers`. The tray calls `set_active_primary_core` inside ONE `spawn_blocking`; `set_active_primary_core` itself runs the gate + tx synchronously (no inner `block_on`).

- [x] **Step 9b: Refresh the tray after every provider mutation (rev-8-8: EIGHT commands)**

The tray's Switch Provider submenu + status item are built from the db at menu-build time. If a provider is created/updated/deleted/toggled/reordered/set-active/duplicated/confirm-and-set-active and the tray is NOT refreshed, the submenu + status show stale data. **rev-8-8 (load-bearing, corrects rev-7-8's count):** each of the EIGHT provider mutation commands (`provider_create`/`provider_update`/`provider_delete`/`provider_toggle`/`provider_reorder`/`provider_set_active`/`provider_duplicate`/`provider_confirm_and_set_active`) currently takes `state: tauri::State<'_, Arc<AppState>>` and has NO `AppHandle` parameter — its closure-internal clone is `let app = state.inner().clone();` (an `Arc<AppState>`, NOT an `AppHandle`). So the rev-6-3 instruction "call `refresh_tray(&app)`" could not compile (`refresh_tray` takes `&tauri::AppHandle`, not `&Arc<AppState>`). rev-7-8 fixed six of them; rev-8-8 adds the remaining two (`provider_duplicate` + `provider_confirm_and_set_active`). Concretely:

1. **`refresh_tray_if_available` is already defined in Step 9 (rev-9-3).** It is the best-effort wrapper next to `refresh_tray`:

```rust
/// rev-9-3: best-effort tray refresh after a provider mutation. Wraps
/// `refresh_tray` (which returns `tauri::Result<()>`) so a tray rebuild failure
/// (e.g. tray not yet built during startup) NEVER turns a successful provider
/// write into an error. Compiles because `refresh_tray` returns `Result`, not `()`.
pub fn refresh_tray_if_available(app: &tauri::AppHandle) {
    if let Err(e) = refresh_tray(app) {
        log::warn!("tray refresh failed: {e}");
    }
}
```

`refresh_tray` (Step 9, rev-9-3) returns `tauri::Result<()>` — `Ok(())` on the `Some(tray)` happy path (after `tray.set_menu` + `tray.set_tooltip`, both `?`-propagated), and `build_tray(app)` (also `tauri::Result<()>`) on the `None`-tray first-build branch. Do NOT redefine `refresh_tray_if_available` here — the single definition in the Step 9 code block is the source. The wrapper's `if let Err(e) = refresh_tray(app)` compiles ONLY because `refresh_tray` returns `Result` (rev-9-3 corrected the rev-8 `()` return that made this line non-compiling).

2. **Each of the eight commands gains an `app_handle: tauri::AppHandle` parameter** and renames its local `app` (the `Arc<AppState>` clone) to `app_state` to avoid the name collision. The refresh call runs on the SUCCESS path, AFTER the `spawn_blocking` write resolves Ok and BEFORE the command returns Ok. The exact per-command changes (rev-8-8: eight commands — rev-7-8 listed only six; `provider_duplicate` + `provider_confirm_and_set_active` are added below):

**`provider_create`** (lib.rs:1092) — add `app_handle` + rename `app`→`app_state`:

```rust
#[tauri::command]
async fn provider_create(
    state: tauri::State<'_, Arc<AppState>>,
    app_handle: tauri::AppHandle,
    template_id: String,
    name: String,
    endpoint: String,
    model: Option<String>,
) -> Result<ProviderProfile, String> {
    let app_state = state.inner().clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let _gate = app_state.data_gate.read();
        let db = require_ready_gated(&app_state, &_gate)?;
        db.with_conn(|conn| {
            db_providers::create(conn, &template_id, &name, &endpoint, model.as_deref())
        })
        .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;
    // rev-7-8: refresh the tray AFTER the write commits. Best-effort.
    refresh_tray_if_available(&app_handle);
    Ok(result)
}
```

**`provider_update`** (lib.rs:1116) — same shape:

```rust
#[tauri::command]
async fn provider_update(
    state: tauri::State<'_, Arc<AppState>>,
    app_handle: tauri::AppHandle,
    uuid: String,
    patch: ProviderPatch,
) -> Result<ProviderProfile, String> {
    let app_state = state.inner().clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let _gate = app_state.data_gate.read();
        let db = require_ready_gated(&app_state, &_gate)?;
        db.with_conn(|conn| db_providers::update(conn, &uuid, &patch)).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;
    refresh_tray_if_available(&app_handle);
    Ok(result)
}
```

**`provider_delete`** (lib.rs:1156) — note `keystore_dir` is read off `app_state`:

```rust
#[tauri::command]
async fn provider_delete(
    state: tauri::State<'_, Arc<AppState>>,
    app_handle: tauri::AppHandle,
    uuid: String,
) -> Result<(), String> {
    let app_state = state.inner().clone();
    let keystore_dir = app_state.keystore_dir.clone();
    tauri::async_runtime::spawn_blocking(move || -> Result<(), String> {
        let _gate = app_state.data_gate.write();
        let db = require_ready_gated_write(&app_state, &_gate)?;
        let secret_ref = db
            .with_conn(|conn| db_providers::begin_delete(conn, &uuid))
            .map_err(|e| e.to_string())?;
        let ks = keystore::Keystore::new(keystore_dir).map_err(|e| e.to_string())?;
        ks.delete_key(&secret_ref).map_err(|e| e.to_string())?;
        db.with_conn(|conn| db_providers::finalize_delete(conn, &uuid))
            .map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())??;
    refresh_tray_if_available(&app_handle);
    Ok(())
}
```

**`provider_reorder`** (lib.rs:1195):

```rust
#[tauri::command]
async fn provider_reorder(
    state: tauri::State<'_, Arc<AppState>>,
    app_handle: tauri::AppHandle,
    uuids: Vec<String>,
) -> Result<(), String> {
    let app_state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _gate = app_state.data_gate.write();
        let db = require_ready_gated_write(&app_state, &_gate)?;
        db.with_conn(|conn| db_providers::reorder(conn, &uuids)).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;
    refresh_tray_if_available(&app_handle);
    Ok(())
}
```

**`provider_toggle`** (lib.rs:1213):

```rust
#[tauri::command]
async fn provider_toggle(
    state: tauri::State<'_, Arc<AppState>>,
    app_handle: tauri::AppHandle,
    uuid: String,
    enabled: bool,
) -> Result<(), String> {
    let app_state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _gate = app_state.data_gate.write();
        let db = require_ready_gated_write(&app_state, &_gate)?;
        db.with_conn(|conn| db_providers::toggle(conn, &uuid, enabled)).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;
    refresh_tray_if_available(&app_handle);
    Ok(())
}
```

**`provider_set_active`** (lib.rs:1288) — the `spawn_blocking` body is UNCHANGED (it owns the consent-gate logic); only the signature gains `app_handle` and the success arm refreshes:

```rust
#[tauri::command]
async fn provider_set_active(
    state: tauri::State<'_, Arc<AppState>>,
    app_handle: tauri::AppHandle,
    primary: String,
    parallel: Vec<String>,
    fallback: Option<String>,
) -> Result<SetActiveResult, String> {
    let app_state = state.inner().clone();
    let outcome = tauri::async_runtime::spawn_blocking(move || -> Result<SetActiveResult, String> {
        // rev-7-8: the body is the EXISTING provider_set_active logic, VERBATIM
        // (only `app` is renamed to `app_state`). validate_active_selection +
        // the consent gate + set_active_slots / set_active_slots_keep_consent
        // all run inside ONE transaction.
        let _gate = app_state.data_gate.write();
        let db = require_ready_gated_write(&app_state, &_gate)?;
        let outcome = db
            .with_conn(|conn| -> Result<SetActiveOutcome, DbErr> {
                let tx = conn.transaction()?;
                let active = db_providers::list(&tx)?;
                db_providers::validate_active_selection(
                    &primary,
                    &parallel,
                    fallback.as_deref(),
                    &active,
                )?;
                if !parallel.is_empty() {
                    let actual = db_providers::compute_scope(&primary, &parallel, &active)
                        .map_err(consent_to_db)?;
                    let stored = db_providers::read_consent_scope(&tx)?;
                    if stored.as_deref() != Some(actual.as_str()) {
                        return Ok(SetActiveOutcome::NeedsConsent { actual_scope: actual });
                    }
                }
                if parallel.is_empty() {
                    set_active_slots(&tx, &primary, &parallel, fallback.as_deref())?;
                } else {
                    set_active_slots_keep_consent(
                        &tx,
                        &primary,
                        &parallel,
                        fallback.as_deref(),
                    )?;
                }
                tx.commit()?;
                Ok(SetActiveOutcome::Written)
            })
            .map_err(|e| e.to_string())?;
        Ok(match outcome {
            SetActiveOutcome::Written => SetActiveResult::Written,
            SetActiveOutcome::NeedsConsent { actual_scope } => {
                SetActiveResult::NeedsConsent { actual_scope }
            }
        })
    })
    .await
    .map_err(|e| e.to_string())??;
    // rev-7-8: refresh so the status item + submenu reflect the new primary.
    // (The tray handler in handle_tray_menu_event ALSO refreshes on its success
    // arm; this covers the IPC-driven path.)
    refresh_tray_if_available(&app_handle);
    Ok(outcome)
}
```

(The `provider_set_active` closure body — validate_active_selection + the `if !parallel.is_empty()` consent gate + `set_active_slots` / `set_active_slots_keep_consent` — is reproduced VERBATIM from the current source inside the `with_conn` closure above; only the local `app` → `app_state` rename + the trailing `refresh_tray_if_available(&app_handle)` are new.)

**`provider_duplicate`** (lib.rs:1135) — rev-8-8 (load-bearing): rev-7-8 listed only SIX commands but the source has EIGHT provider mutation commands. `provider_duplicate(state, uuid)` ALSO needs the `app_handle` parameter + `refresh_tray_if_available` because a duplicated provider appears in the Switch Provider submenu. Verified signature + body at lib.rs:1135-1148:

```rust
#[tauri::command]
async fn provider_duplicate(
    state: tauri::State<'_, Arc<AppState>>,
    app_handle: tauri::AppHandle,
    uuid: String,
) -> Result<ProviderProfile, String> {
    let app_state = state.inner().clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        // Acquire the gate FIRST (see provider_list).
        let _gate = app_state.data_gate.read();
        let db = require_ready_gated(&app_state, &_gate)?;
        db.with_conn(|conn| db_providers::duplicate(conn, &uuid)).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;
    // rev-8-8: refresh the tray AFTER the write commits. Best-effort.
    refresh_tray_if_available(&app_handle);
    Ok(result)
}
```

**`provider_confirm_and_set_active`** (lib.rs:1377) — rev-8-8 (load-bearing): the consent-confirming counterpart to `provider_set_active`. Verified signature + body at lib.rs:1377-1432. It returns `Result<i64, ProviderCommandError>` (NOT `String`) — the typed error carries the `StaleScope` variant so the frontend can re-prompt. Only the signature gains `app_handle` + the success arm refreshes; the `spawn_blocking` body (validate_active_selection + scope check + `write_consented_selection` + the `ConfirmActiveOutcome` mapping) is VERBATIM:

```rust
#[tauri::command]
async fn provider_confirm_and_set_active(
    state: tauri::State<'_, Arc<AppState>>,
    app_handle: tauri::AppHandle,
    primary: String,
    parallel: Vec<String>,
    fallback: Option<String>,
    expected_scope: String,
) -> Result<i64, ProviderCommandError> {
    let app_state = state.inner().clone();
    let version = tauri::async_runtime::spawn_blocking(move || -> Result<i64, ProviderCommandError> {
        // Acquire the gate FIRST (see provider_list).
        let _gate = app_state.data_gate.write();
        let db = require_ready_gated_write(&app_state, &_gate).map_err(ProviderCommandError::from)?;
        let outcome = db.with_conn(|conn| -> Result<ConfirmActiveOutcome, DbErr> {
            let tx = conn.transaction()?;
            let active = db_providers::list(&tx)?;
            db_providers::validate_active_selection(
                &primary,
                &parallel,
                fallback.as_deref(),
                &active,
            )?;
            let actual_scope = db_providers::compute_scope(&primary, &parallel, &active)
                .map_err(consent_to_db)?;
            if expected_scope != actual_scope {
                // Stale frontend: the scope it asserts doesn't match what the
                // backend recomputes (it raced a provider change). Carried out
                // as a typed variant — no sentinel string to parse.
                return Ok(ConfirmActiveOutcome::StaleScope { actual_scope });
            }
            let new_version = write_consented_selection(
                &tx,
                &primary,
                &parallel,
                fallback.as_deref(),
                &actual_scope,
            )?;
            tx.commit()?;
            Ok(ConfirmActiveOutcome::Written { version: new_version })
        });
        // Map the typed outcome: StaleScope → ProviderCommandError::StaleScope
        // (structured wire error), Written → the consent version. Everything
        // else (real DB errors) stays an error.
        outcome
            .map(|o| match o {
                ConfirmActiveOutcome::Written { version } => Ok(version),
                ConfirmActiveOutcome::StaleScope { actual_scope } => {
                    Err(ProviderCommandError::StaleScope { actual_scope })
                }
            })
            .map_err(ProviderCommandError::from)?
    })
    .await
    .map_err(|e| ProviderCommandError::Db {
        message: format!("{e:?}"),
    })??;
    // rev-8-8: refresh so the status item + submenu reflect the new primary.
    refresh_tray_if_available(&app_handle);
    Ok(version)
}
```

> **rev-8-8 note (EIGHT commands, not six):** rev-7-8 enumerated only six provider mutation commands, but the verified source (`src-tauri/src/lib.rs`) has EIGHT: `provider_create`/`provider_update`/`provider_delete`/`provider_toggle`/`provider_reorder`/`provider_set_active`/`provider_duplicate`/`provider_confirm_and_set_active`. All eight now gain the `app_handle: tauri::AppHandle` parameter + call `refresh_tray_if_available(&app_handle)` on their success path. The tray's own `handle_tray_menu_event` Switch-Provider arm (Step 9) calls `refresh_tray_if_available(&app_for_refresh)` on success (rev-9-3: returns `()`, logs on failure) so the spawn stays unit-typed. The eight IPC commands above are the frontend-driven mutations (Create/Update/Delete/Toggle/Reorder/SetActive/Duplicate/ConfirmAndSetActive from the Settings UI) — those are the ones that need the new `app_handle` parameter + the `refresh_tray_if_available` call so the tray stays in sync when the user edits providers from the settings window.

- [x] **Step 10: Make the main window hidden by default**

In `src-tauri/tauri.conf.json`, the first window entry. Update it:

```json
      {
        "label": "main",
        "title": "LinguaRay",
        "width": 800,
        "height": 600,
        "minWidth": 600,
        "minHeight": 400,
        "visible": false
      },
```

- [x] **Step 11: Build + run Rust tests + frontend tests**

Run:
- `cargo build --manifest-path src-tauri/Cargo.toml --features xproc-test-helper`
- `cargo test --manifest-path src-tauri/Cargo.toml --features xproc-test-helper`
- `pnpm test`

Expected: clean build; all tests pass. Verify the four new permission TOMLs exist under `src-tauri/permissions/autogenerated/`.

- [x] **Step 12: Commit**

```bash
git diff --check
git add src/App.tsx src/features/translation/selection-ipc.ts src/features/settings/SettingsShell.tsx src-tauri/build.rs src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/capabilities/input.json src-tauri/capabilities/main.json src-tauri/capabilities/popup.json src-tauri/src/lib.rs src-tauri/tauri.conf.json src-tauri/permissions/autogenerated/translate_session.toml src-tauri/permissions/autogenerated/translate_selection_ipc.toml src-tauri/permissions/autogenerated/provider_get_active_selection.toml src-tauri/permissions/autogenerated/open_settings_window.toml src-tauri/tests/capabilities.rs test/tray-action.test.tsx
git commit -m "feat(permissions+tray+clipboard): authorize new commands + clipboard plugin + tray-action/navigate listeners + executable Switch Provider submenu + hide main (P1-5, P1-6)"
```

---

### Task A5: Tray visual-state controller — Error red-dot overlay + Active-pulse + Normal restore (rev-19: fresh_db fixture for functional switch test + PulseWorker struct drops `notify` field + `worker_start_count` no-churn assertion + switch-failure tooltip on wrapper + dynamic `tray.switch-<uuid>` submenu note; rev-18: sync handle_switch_provider + spawn_blocking offload + real-DB functional switch test + deterministic PulseWorker tests; rev-17: PulseEvent enum + latest_translation_gen + delete dead switch mutators; rev-16: NO overloading + gen guards + switch revision + notify-channel tests + default-build verification; rev-15: PulseWorker channel-quit + finish_translation merge + single timer model; rev-14 sync `parking_lot::Mutex` + current_state-gated worker swap + `std::thread` timer; rev-13 RAII guard + generation-aware error + injectable TrayRenderer; rev-12 reducer + real icon pulse + base-icon overlay; rev-11 user-approved A-path)

> **rev-12 scope (supersedes rev-11's "tooltip-only pulse" + "solid-red square" + "direct-override set_tray_visual_state"):** this task implements the user-approved A-paths for Surface 04 Error red-dot (pages/04 "red-dot on icon") and Active-pulse (a VISIBLE pulse during in-flight translate). The controller is PURE RUST in `src-tauri/src/tray_state.rs` — it does NOT route through the Web frontend, does NOT emit any Tauri event, and does NOT depend on the popup/input/main window. The translate/clipboard/switch flows call it directly.
>
> **rev-12 corrections over rev-11 (4 P1 + 2 P2):**
> - **P1-1 (real icon pulse, not tooltip-only):** rev-11 left the icon at the app default during `ActiveTranslation` (only the tooltip changed). rev-12 drives a REAL icon frame-switch pulse — a background timer swaps `set_icon(normal)` ↔ `set_icon(dimmed)` every 800ms (the dimmed variant is build-time-generated `tray-active-32.png`).
> - **P1-2 (red-dot OVERLAY on the base icon, not a solid-red square):** rev-11 filled the whole 32×32 buffer red. rev-12 composites a ~10px `#DC2626` dot at the top-right ON TOP OF the existing app default icon (`src-tauri/icons/32x32.png`).
> - **P1-3 (real reducer, not direct override):** rev-11's `set_tray_visual_state` was a direct overwrite with no concurrency awareness. rev-12 introduces `TrayStateController { active_translations: u32, has_error: bool, pulse_timer }` with `begin_translation`/`end_translation`/`set_error` → `recompute()` so two concurrent translations don't prematurely reset to `Normal`, and `Error` is not clobbered by a subsequent success.
> - **P1-4 (`pub mod tray_state`):** rev-11 used `mod tray_state;` (private) but the test imported via the module path — would not compile. rev-12 uses `pub mod tray_state;`.
> - **P2 (latency + localization, also fixed in this rev):** C3c's `as_millis() as u32` → `u32::try_from(...).unwrap_or(u32::MAX)` (saturation) + a test asserting `latency_ms` reflects the real `Instant` probe; tray tooltip text reads `locale` from settings (en/zh) via `tray_tooltip_text(state, locale)`.
>
> The `UpdateAvailable` arm is RETAINED in the enum (so the priority ordering is unit-testable) but is NEVER activated by `recompute()` this stage — it is deferred to R5/R6 per user-approved scope decision.
>
> **rev-13 corrections over rev-12 (5 P1 + housekeeping — verified against source):**
>
> **Verified source facts driving rev-13:**
> - `Session` (lib.rs:70-74) has `client: Option<reqwest::Client>`, `keystore: Option<keystore::Keystore>`, `gen: concurrency::GenerationToken` — NO tray field.
> - `AppState` (lib.rs:99-106) has `db`, `data_gate`, `readiness`, `db_path`, `keystore_dir`, `settings_path` — NO tray field.
> - AppState is constructed at 5 sites: lib.rs:2513 (`app.manage`), lib.rs:2597 (test), lib.rs:2620 (test), tests/recovery.rs:42, tests/recovery.rs:248. ALL 5 must add the `tray` field.
> - `Cargo.toml:102` — `tokio = { version = "1", features = ["macros", "rt-multi-thread"] }` — NO `time`/`sync`. rev-14 does NOT add `time`/`sync` to the RUNTIME tokio (the timer is a `std::thread`, the mutex is `parking_lot`); rev-14 adds `test-util` to the DEV tokio only.
> - `Settings` (settings.rs:9-15) has `default_provider`, `target_language`, `fallback_engine` — NO `locale`. rev-13/rev-14 do NOT depend on Settings for locale; they read the SYSTEM locale.
> - `capture_and_translate` (A2/A4, new helper) has a signature that already includes `app_state: &Arc<AppState>` and over 10 early returns (capture fail, stale gen, anchor, client/keystore/db acquisition failure, …). rev-12 put `begin_translation` at the very top → every early return leaks an Active count. rev-13 uses an RAII guard.
> - `translate_clipboard` (lib.rs:329-333) owns `app: tauri::AppHandle` by value and takes `app_state: tauri::State<'_, Arc<AppState>>`.
>
> - **P1-1 (controller ownership + call-site naming):** rev-12 was ambiguous about whether the `tray` field lives on `Session` or `AppState`, and the call sites mixed `state.tray` (a `Session`-style name) with `app_state.tray`. rev-13 FIXES this once: the `tray` field is on **`AppState`** (`pub tray: Arc<parking_lot::Mutex<TrayStateController>>` — rev-14: synchronous `parking_lot::Mutex`, NOT `tokio::sync::Mutex`), and ALL call sites use **`app_state.tray`** (the `AppState` parameter name). `capture_and_translate` already has `app_state: &Arc<AppState>`; the Switch Provider branch clones `app_state` BEFORE `spawn_blocking` so the controller can be touched inside the blocking/task context.
> - **P1-2 (single-entry/single-exit — RAII guard for begin/end pairing):** rev-12 put `begin_translation` at the top of `capture_and_translate` and relied on manually adding `end_translation` to every terminal branch — but the helper has over 10 early returns (capture fail, stale gen, client/keystore/db acquisition failure, …), so missing one leaks the Active counter permanently. rev-13 introduces a `TranslationGuard` RAII guard: `begin_translation` runs AFTER the preflight (text captured + anchor built), and the guard's `Drop` runs `end_translation + recompute` exactly once on scope exit (covering EVERY return path, panic, and `?`). **rev-14: `Drop` runs these SYNCHRONOUSLY** (the controller mutex is `parking_lot::Mutex`, whose `lock()` is a blocking sync call — no `spawn`, no detached future), so the RAII guarantee is REAL (the counter is decremented before `drop` returns). `translate_clipboard` gets the same guard pattern (begin after clipboard read succeeds).
> - **P1-3 (generation-aware error clearing):** rev-12's `has_error: bool` is sticky and only cleared by an explicit `set_error(false)` (switch-provider success) — a failed translation's red dot never disappears even if the user's Retry succeeds (because the success path only calls `end_translation`, not `set_error(false)`). rev-13 replaces the bool with `error_gen: Option<u64>` (which generation produced the error) + `visual_epoch: u64` (bumped on every state transition): `begin_translation` clears a prior error if it belongs to an OLDER generation; `end_translation` on the LAST active translation clears the error ONLY if the current generation succeeded; switch-provider success clears unconditionally. A Retry success (new generation) clears the prior generation's red dot.
> - **P1-4 (timer epoch serialization):** rev-12's pulse timer keeps ticking after the controller leaves `Active` until the handle is `abort()`ed — but `abort()` is non-blocking and the already-scheduled tick can still write the icon AFTER the controller has moved to `Error`, clobbering the red dot. rev-13 adds `visual_epoch: u64`: every state transition bumps the epoch FIRST (invalidating all live timers) then mutates state; each tick re-checks its captured epoch against the controller's current epoch and exits if they differ (or the controller is no longer `Active`). **rev-14 tightens this into a RenderGate:** the epoch check AND the `render()` icon write happen atomically inside the SAME `parking_lot::Mutex` guard — a tick holds the lock, checks `my_epoch == visual_epoch`, and only then calls `render()`; there is no window between the check and the write. **rev-14 P1-2 also gates the epoch bump:** `recompute` only bumps `visual_epoch` (and churns the timer) when the resolved `new_state != current_state` — so a counter bump Active → Active does NOT kill and restart the pulse.
> - **P1-5 (test-injected renderer + deterministic timer):** rev-12's tests called `set_pulsing(true/false)` directly, which does NOT exercise the real timer. rev-13 abstracts the tray leaf behind `trait TrayRenderer { fn set_icon_normal/set_icon_dimmed/set_icon_error_dot/set_tooltip }`; the production Impl wraps `TrayIcon` (looked up via `app.tray_by_id("main-tray")`); tests inject a mock renderer that records all calls. **rev-14: tests are SYNC** — the controller methods are sync, so tests call `c.begin_translation(1);` (no `.await`); the timer is a `std::thread` and its ticks are observed via the `RecordingRenderer` after a small `thread::sleep` (the test timer uses a tiny interval, e.g. 5ms, so the test does not sleep 800ms in real time). Tests assert (a) Active → alternating frames on the mock, (b) a second begin does NOT start a second timer (epoch unchanged), (c) the last end stops the timer, (d) Error produces no active frame, (e) a tick whose epoch is stale is rejected (RenderGate).
> - **Housekeeping (also rev-13/rev-14):**
>   - **parking_lot (rev-14):** `parking_lot = "0.12"` is ALREADY a production dep (`Cargo.toml:53`) — the `tray: Arc<parking_lot::Mutex<TrayStateController>>` field needs NO new runtime dep. rev-14 does NOT add `time`/`sync` to the RUNTIME `tokio` (the timer is `std::thread`, the mutex is `parking_lot`).
>   - **sys-locale (rev-14 P2):** `sys-locale = "0.3"` is added to `[dependencies]`; `detect_system_locale()` uses `sys_locale::get_locale()` (cross-platform — works on Windows/macOS/Linux; `std::env::var("LANG")` is Unix-only and would return `None` on Windows).
>   - **AppState construction:** all 5 sites listed with the exact `tray:` initializer line each.
>   - **locale source:** `detect_system_locale() -> Locale` (rev-14: `sys_locale::get_locale()`; rev-13 said `std::env::var("LANG")` — superseded) — does NOT depend on `Settings` (which has no locale field).
>   - **build.rs:** drop the unused `imageops` import (only `ImageBuffer` + `Rgba` are load-bearing for the manual `put_pixel` loop).
>   - **Debug derive (rev-14 P2):** `TrayStateController` does NOT derive `Debug` (it holds `Arc<dyn TrayRenderer>`, and `dyn Trait` does not auto-implement `Debug`); test accessors are plain `pub fn`.
>   - **RecordingRenderer cfg (rev-14 P2 — SUPERSEDED by rev-15 P1-2):** rev-14 wrote `#[cfg(test)]`. **This is WRONG for integration tests** — `src-tauri/tests/tray_state.rs` is a SEPARATE crate that consumes `linguaray_lib` as a normal dependency; when the lib is compiled for an integration test, `cfg(test)` is NOT enabled (it is only enabled when compiling the lib's OWN unit-test targets). So `use linguaray_lib::tray_state::RecordingRenderer;` fails with `unresolved import`. **rev-15 P1-2 changes this to `#[cfg(any(test, feature = "xproc-test-helper"))]`** (struct + all `impl` blocks + `RenderedIcon` + the `lib.rs` re-export). The `[features] xproc-test-helper = []` line ALREADY EXISTS in `Cargo.toml` (verified); this plan's verification commands all carry `--features xproc-test-helper`, so the integration test sees the type; `cargo build` (no feature) does NOT compile the mock.
>   - **latency test:** test the pure `Duration → u32` saturation function directly + a static assertion that the probe calls it (no real-timing flakiness) — this lives in C3c, not A5.
>   - **red-dot pixel test (rev-14 P2):** load the generated `tray-error-32.png`, assert the base icon's pixels are unchanged OUTSIDE the dot circle AND that the dot circle contains `#DC2626` pixels. **rev-14: `panic!("build.rs output not found: {error_png}")` if the file is missing** (does NOT silently `return` — a silent skip would let a build.rs regression pass unnoticed).
>
> **rev-15 corrections over rev-14 (4 P1 + housekeeping — verified against source):**
>
> **Verified source facts driving rev-15:**
> - `concurrency.rs` (full source verified): `GenerationToken::next(&self) -> u64 { self.current.fetch_add(1, Ordering::SeqCst) + 1 }` — calling `next()` ADVANCES the current generation, so any in-flight `gen` allocated earlier becomes stale (`is_latest(older)` returns false). rev-14's switch-provider path called `session.gen.next()` to obtain a gen for the tray error tag — this STALES the in-flight translation (the tray error tag and the translation generation are unrelated concerns; coupling them via `next()` is a bug).
> - `src-tauri/Cargo.lock` IS git-tracked (`git ls-files` confirms) — adding the `sys-locale = "0.3"` runtime dependency updates it, so it MUST be in the A5 commit.
> - Integration test crate visibility: `src-tauri/tests/*.rs` are separate crates that depend on `linguaray_lib`; compiling the lib for these crates does NOT set `cfg(test)`. So `#[cfg(test)] pub struct RecordingRenderer` is INVISIBLE to `src-tauri/tests/tray_state.rs`. rev-14's `#[cfg(test)]` claim ("integration tests compile under `#[cfg(test)]`") is incorrect.
> - rev-14's pulse timer is `std::thread::spawn(move || loop { sleep(interval); render })` — an INFINITE loop with no exit path. `TrayStateController::stop_timer()` calls `handle.join()`, which waits for the thread to exit — but the thread never exits → `join()` blocks forever → the whole app hangs on the first transition out of `Active`.
>
> - **P1-1 (PulseWorker channel-quit — replaces rev-14's infinite-loop + join deadlock; rev-16 P2-1 adds a per-tick `notify` channel):** rev-14's pulse thread had no exit condition and `stop_timer()`'s `join()` deadlocked. rev-15 introduces `PulseWorker`: a `pub struct PulseWorker { stop_tx: std::sync::mpsc::Sender<()>, handle: Option<std::thread::JoinHandle<()>> }` (**rev-16 P2-1: + `notify: Option<std::sync::mpsc::Sender<()>>`** — **rev-19-3/rev-20-1: this `notify` field is REMOVED from the struct; the notify Sender is moved into the worker thread closure instead, so the FINAL struct is `{ stop_tx, handle }` only**). `PulseWorker::start(renderer: Arc<dyn TrayRenderer>, interval: Duration, notify: Option<Sender<()>>) -> Self` creates an `mpsc::channel()`, spawns a thread whose body is `let mut dimmed = false; loop { match stop_rx.recv_timeout(interval) { Ok(()) => return, Err(RecvTimeoutError::Disconnected) => return, Err(RecvTimeoutError::Timeout) => { dimmed = !dimmed; if dimmed { renderer.set_icon_dimmed() } else { renderer.set_icon_normal() } if let Some(tx) = notify.as_ref() { let _ = tx.send(()); } } } }`. `PulseWorker::stop(&mut self) { let _ = self.stop_tx.send(()); if let Some(h) = self.handle.take() { let _ = h.join(); } }` — the `send` wakes the worker from `recv_timeout`, it returns, and `join` completes (NO deadlock). `impl Drop for PulseWorker { fn drop(&mut self) { self.stop(); } }`. The controller holds `pulse_worker: Option<PulseWorker>` (replaces rev-14's `pulse_timer: Option<JoinHandle<()>>`). Entering `Active`: `pulse_worker = Some(PulseWorker::start(self.renderer.clone(), self.tick_interval, self.notify_tx.clone()))`. Leaving `Active`: `self.pulse_worker.take()` — `take()` drops the old `PulseWorker`, whose `Drop` calls `stop()` (send + join). `TrayStateController::drop` also lets any residual `pulse_worker` drop. The worker holds an INDEPENDENT `Arc<dyn TrayRenderer>` — it does NOT lock the controller, so no `visual_epoch` check is needed inside the tick.
> - **P1-2 (RecordingRenderer cfg visibility — `#[cfg(any(test, feature = "xproc-test-helper"))]`):** rev-14's `#[cfg(test)]` is invisible to integration tests. rev-15 gates `RecordingRenderer` (+ its `impl` blocks + `RenderedIcon` + the `lib.rs` re-export) behind `#[cfg(any(test, feature = "xproc-test-helper"))]`. The `xproc-test-helper` feature ALREADY EXISTS in `Cargo.toml` (verified) and is enabled by every verification command in this plan, so the integration test compiles; `cargo build` (no feature) does NOT compile the mock into the production binary.
> - **P1-3 (Switch Provider does NOT bump the translation generation — no-gen switch methods):** rev-14's switch-provider handler called `session.gen.next()` to tag the tray error — but `next()` advances the generation (verified concurrency.rs), staling any in-flight translation. rev-15 DECOUPLES switch from the translation generation: `TrayStateController` gains a sticky `has_error: bool` flag for the switch flow (independent of the translation `error_gen`). **rev-16-1 (NO function overloading):** rev-15's two overloads `record_error(&mut self, gen: u64)` (translation) and `record_error(&mut self)` (switch) have the SAME name — Rust does NOT support function overloading, so this does not compile (`E0592`). rev-16-1 renames them to DISTINCT method names: `record_translation_error(gen)` (translation flow) + `record_switch_error()` (switch flow) + `clear_switch_error()` (switch success) + `clear_translation_error(gen)` (translation success clear, gen-guarded). **rev-16-3 (switch revision, replaces rev-15's sticky `has_error: bool`):** rev-15's sticky bool has NO revision — two concurrent switch completions that reorder (switch A fails → `has_error=true`; switch B succeeds → `has_error=false`; then A's late result arrives → `has_error=true` again) would show the wrong final state (the user's latest intent was B-success, but A's late failure wins). rev-16-3 replaces `has_error: bool` with `switch_revision: u64` (monotonic, incremented by `begin_switch()`) + `switch_error_rev: Option<u64>` (which revision produced the error). `begin_switch() -> u64` bumps the revision and returns it; `finish_switch(rev, success)` IGNORES the result if `rev != switch_revision` (a stale/late switch result cannot clobber the latest); `record_switch_error()` sets `switch_error_rev = Some(switch_revision)`; `clear_switch_error()` sets it to `None`. `recompute_pure` ORs: `Error if error_gen.is_some() || switch_error_rev.is_some()`. The switch-provider handler in Step 10 calls `begin_switch()` (capturing `rev`) BEFORE `spawn_blocking`, then `finish_switch(rev, success)` AFTER; it NO LONGER calls `session.gen.next()` and NO LONGER acquires a translation `gen`. Regression test `switch_does_not_bump_translation_generation` (renamed/extended in rev-16 P2-2 to `switch_handler_does_not_call_gen_next`): allocate `g1 = token.next()`, call the extracted `handle_switch_provider` helper (which does NOT touch the token), assert `token.is_latest(g1) == true` (the generation was NOT advanced) AND a structural test reads `lib.rs` via `include_str!` and asserts the switch arm contains no `.gen.next()`.
> - **P1-4 (single timer model — PulseWorker only; delete the epoch/RenderGate/tick_render narration):** rev-14's prose contained TWO contradictory timer models — (a) "timer holds an independent `Arc<dyn TrayRenderer>`; stop is a sync channel signal" and (b) "timer locks the controller, checks `my_epoch == visual_epoch`, calls `tick_render()` (RenderGate: epoch check + render in the SAME lock)". The rev-14 `spawn_pulse_timer` CODE actually matched (a) (it captured `renderer` + `locale`, never locked the controller), but the rev-14 PROSE described (b). rev-15 KEEPS ONLY (a): `PulseWorker` holds an independent renderer; the stop barrier is `stop_tx.send(())` + `join()` (NOT an epoch check). All "epoch check in timer", "`tick_render()`", "RenderGate", "thread self-exits", "`AtomicRender`", and "`visual_epoch`" narration is DELETED. The `visual_epoch` FIELD is removed (no longer needed — the channel-quit barrier is the single sync point). The `current_state: TrayVisualState` field is KEPT (rev-14 P1-2's "only swap the worker when `new_state != current_state`" logic is still needed to avoid restarting the worker on an Active→Active counter bump). The `stale_epoch_tick_does_not_clobber_error` test is RETAINED but renamed to `leaving_active_stops_the_worker_no_stale_frames` and its assertion changes: after `record_error(1)` switches to `Error` + drops the worker (channel-quit), the `RecordingRenderer` receives NO further dimmed frames (verified by sleeping past one tick interval and re-checking the call list) — there is no epoch, the worker is simply dead.
> - **Housekeeping (rev-15):**
>   - **finish_translation merge:** the `end_translation(gen)` + (if `succeeded`) `clear_error_for_gen(gen)` + `recompute()` sequence in the rev-14 `TranslationGuard::drop` is collapsed into ONE method `finish_translation(&mut self, gen: u64, success: bool)`: `active_translations = saturating_sub(1); if success { error_gen = None; } recompute();`. `TranslationGuard::drop` calls `c.finish_translation(self.gen, self.succeeded)` ONCE. The translation failure branch calls `controller.lock().record_error(gen)` BEFORE the guard drops; the guard's `finish_translation(gen, false)` then only decrements the counter + recomputes (does NOT clear `error_gen` — the `success` flag is false). The translation success branch calls `guard.mark_success()` (sets `succeeded = true`); the guard's `finish_translation(gen, true)` clears the error + decrements + recomputes. `mark_success(&mut self)` on the guard just sets the flag (no controller call). The standalone `end_translation`/`clear_error_for_gen` methods are REMOVED (their logic lives inside `finish_translation`).
>   - **Cargo.lock in commit:** `src-tauri/Cargo.lock` is git-tracked (verified); adding `sys-locale = "0.3"` updates it. Step 12's `git add` list gains `src-tauri/Cargo.lock`.
>   - **Test count = 33 (rev-17 + 1 `stale_gen_error_ignored_after_newer_begin` over rev-16's 32 — rev-16's 32 enumeration is retained below for the breakdown; rev-17/rev-18/rev-19/rev-20 all keep 33. The authoritative count is the grep of `^#[test]$` + `^#[tokio::test]$` in the Step 2 code block — 33, ALL `#[test]` after rev-18-1):** the breakdown is 6 priority + 6 reducer concurrency (5 counter/error-driven + `switch_flow_error_is_independent_of_translation_error_gen` — rev-16-3 renamed; assertions use `switch_error_rev()`) + 2 RAII guard + 2 generation-aware error + **2 gen-guard (rev-16-2 NEW: `older_success_does_not_clear_newer_error`, `older_error_does_not_replace_newer_error`)** + 4 renderer (rev-16 P2-1: notify channel, NO `thread::sleep`) + PulseWorker-lifecycle (`active_emits_alternating_frames_on_the_renderer`, `second_begin_does_not_churn_the_worker`, `last_finish_stops_the_worker`, `error_produces_no_active_pulse_frame`) + 2 PulseWorker channel-quit (`stop_signal_joins_the_worker`, `drop_stops_the_worker`) + 1 worker-stop barrier (`leaving_active_stops_the_worker_no_stale_frames` — rev-16 P2-1: notify Disconnected) + 2 localization + 1 pixel-diff + **2 switch does NOT bump generation (rev-16 P2-2: `switch_handler_does_not_call_gen_next` functional + `switch_arm_source_has_no_gen_next_call` structural)** + **2 switch-revision ordering (rev-16-3 NEW: `two_concurrent_switches_second_wins`, `stale_switch_result_ignored`)** + **1 latest_translation_gen guard (rev-17-3 NEW: `stale_gen_error_ignored_after_newer_begin`)** = **33 tests**. (The count is verified by grepping `^#[test]$` + `^#[tokio::test]$` in the Step 2 code block.)
>   - **Deterministic timer tests (rev-16 P2-1):** ALL PulseWorker tests use NO `thread::sleep` — the channel-quit tests (`stop_signal_joins_the_worker`, `drop_stops_the_worker`) call `PulseWorker::stop()` / drop and assert `join` returned; the pulse-worker-lifecycle tests (alternating frames, etc.) use the worker's `notify` channel and `recv_timeout` to deterministically wait for N frames (NO fixed-duration sleep); the worker-stop barrier test asserts the `notify` channel goes `Disconnected` when the worker drops. This eliminates CI-flake from slow machines (the rev-15 `thread::sleep(20ms)` approach could miss ticks on a loaded CI runner).

**Files:**
- Modify: `src-tauri/Cargo.toml` — add `image = { version = "0.25", default-features = false, features = ["png"] }` under `[build-dependencies]` AND under `[dev-dependencies]`; **rev-14:** add `sys-locale = "0.3"` to `[dependencies]` (cross-platform locale detection — `sys_locale::get_locale()`; `std::env::var("LANG")` is Unix-only and returns `None` on Windows). `parking_lot = "0.12"` is ALREADY a production dep (`Cargo.toml:53` — verified), so NO new mutex dep is needed. **rev-14:** the RUNTIME `tokio` line at `Cargo.toml:102` (`tokio = { version = "1", features = ["macros", "rt-multi-thread"] }`) is LEFT UNCHANGED — rev-14 uses `parking_lot::Mutex` (not `tokio::sync::Mutex`) and a `std::thread` timer (not `tokio::time`), so the runtime `tokio` needs NO `time`/`sync` features. **rev-14:** the DEV `tokio` line at `Cargo.toml:102` (under `[dev-dependencies]`) gains `test-util` (`tokio = { version = "1", features = ["macros", "rt-multi-thread", "test-util"] }`) so `#[tokio::test(start_paused = true)]` remains usable if a test wants it.
- Create: `src-tauri/src/tray_state.rs` **(new)** — `TrayVisualState` enum + `tray_state_priority` + `Locale` + `tray_tooltip_text` + `detect_system_locale` (rev-14: `sys_locale::get_locale()`, not `LANG`) + `trait TrayRenderer` (rev-14: `set_icon_normal`/`set_icon_dimmed`/`set_icon_error_dot`/`set_tooltip` — discrete methods, NOT a `set_icon(Option<Image>)` taking an enum) + `TrayIconRenderer` (prod) + `RecordingRenderer` (test mock, **rev-15 P1-2: `#[cfg(any(test, feature = "xproc-test-helper"))]`-gated** — NOT `#[cfg(test)]` which is invisible to integration tests) + `PulseEvent` enum (rev-17-2: `{ Tick, Stopped }`) + `PulseWorker` (rev-15 P1-1 + rev-16 P2-1 + rev-17-2 + **rev-19-3: struct drops `notify` field** → `pub struct PulseWorker { stop_tx: mpsc::Sender<()>, handle: Option<JoinHandle<()>> }` — the `notify` Sender is MOVED into the worker thread closure, NOT stored on the struct, so there is no `dead_code` warning on the prod `notify = None` path; with channel-quit + per-tick `PulseEvent::Tick` + on-exit `PulseEvent::Stopped` + Drop-impl-stop) + `TrayStateController` reducer (**rev-17-3 adds `latest_translation_gen: u64`**; **rev-19-4 adds `worker_start_count: u32`** (monotonic counter incremented each time `recompute` starts a new `PulseWorker` — the no-churn test asserts it does NOT increase on an Active→Active counter bump); **rev-16 fields: `active_translations: u32` + `error_gen: Option<u64>` (translation flow) + `switch_revision: u64` + `switch_error_rev: Option<u64>` (switch flow, rev-16-3 — REPLACES rev-15's `has_error: bool`) + `current_state: TrayVisualState` + `pulse_worker: Option<PulseWorker>` + `tick_interval: Duration` + `renderer: Arc<dyn TrayRenderer>` + `notify_tx: Option<mpsc::Sender<PulseEvent>>` (rev-17-2: `PulseEvent`, was `()`; rev-16 P2-1) + `locale: Locale` — does NOT derive `Debug`; rev-15 REMOVES `visual_epoch`, `pulse_frame`, `pulse_timer`; rev-16-3 REMOVES `has_error`**) + `TranslationGuard` (rev-14/rev-15/rev-16: SYNCHRONOUS `Drop` via `parking_lot::Mutex`; rev-16-2 gen guard on the success clear; `Drop` calls `finish_translation(gen, succeeded)`) + the controller's `render(&mut self)` method (single sync entry point that writes icon+tooltip based on `current_state`). **rev-16-1 / rev-17-4 methods (NO overloading + NO dead switch mutators):** `begin_translation(gen)` (rev-17-3: bumps `latest_translation_gen`)/`finish_translation(gen, success)`/`record_translation_error(gen)` (translation, gen-guarded rev-16-2 + `latest_translation_gen` guard rev-17-3) + `begin_switch()`/`finish_switch(rev, success)` (switch, rev-16-3 — **rev-17-4: `record_switch_error()`/`clear_switch_error()` DELETED**, finish_switch is the sole switch mutator).
- Modify: `src-tauri/src/lib.rs` — `pub mod tray_state;` (rev-12 P1-4: PUBLIC module so the test's `use linguaray_lib::tray_state::...` path resolves); add the `tray` field to **`AppState`** (rev-13 P1-1: NOT `Session`) at all **5** construction sites (lib.rs:2513, 2597, 2620; tests/recovery.rs:42, 248) — **rev-14: `pub tray: Arc<parking_lot::Mutex<TrayStateController>>`** (NOT `tokio::sync::Mutex`); wire `TrayStateController` into `capture_and_translate` (Step 8, via `TranslationGuard`), `translate_clipboard` (lib.rs:329, via `TranslationGuard`), and `handle_tray_menu_event`'s switch-provider success/failure branches (**rev-16 / rev-18-1: Step 10 extracts TWO helpers visible to tests — `pub fn handle_switch_provider_core(app_state: &Arc<AppState>, uuid: &str) -> Result<(), String>` (SYNC core, NO AppHandle — the testable entry) + `pub fn handle_switch_provider(app: &tauri::AppHandle, app_state: &Arc<AppState>, uuid: &str) -> Result<(), String>` (SYNC wrapper, calls core + `refresh_tray_if_available`); rev-18-1: SYNC — `set_active_primary_core` is SYNC, so no `.await`; rev-17-1's `async` was based on the wrong premise that `set_active_primary_core` was async; both acquire ONLY `app_state` (the wrapper via `app.state::<Arc<AppState>>()`) — they do NOT acquire `Session`, do NOT call `session.gen.next()`, and use `begin_switch()` → `finish_switch(rev, success)` with NO gen arg**) — translation callers invoke `app_state.tray.lock().record_translation_error(gen)` (gen-tagged, rev-16-1 renamed from `record_error(gen)`) while switch callers invoke `controller.lock().begin_switch()` + `controller.lock().finish_switch(rev, success)` (rev-16-3, revision-tagged; **rev-17-4: `record_switch_error`/`clear_switch_error` DELETED**); the tray.switch arm calls `tauri::async_runtime::spawn_blocking(move || { let _ = handle_switch_provider(&app2, &app_state, &uuid); })` (rev-18-1: offload the SYNC wrapper via `spawn_blocking` — NOT `spawn(async move { ... .await })`); all SYNC `parking_lot::Mutex::lock` for the controller calls (no `.await` anywhere in the helper — `set_active_primary_core(...)` is a SYNC fn call, not `.await`ed). The `RecordingRenderer` re-export is **rev-15 P1-2: `#[cfg(any(test, feature = "xproc-test-helper"))]`**-gated; **rev-17-2: `PulseEvent` added to the always-on re-export**.
- Modify: `src-tauri/build.rs` — append a build-time block that writes TWO PNGs into `OUT_DIR`: `tray-error-32.png` (red-dot OVERLAY composited on the base icon `src-tauri/icons/32x32.png`) + `tray-active-32.png` (dimmed variant for the pulse frame-swap). **rev-13:** drop the unused `imageops` from the `use` list (only `ImageBuffer` + `Rgba` are load-bearing; rev-12's note already flagged this — rev-13 makes it the spec). (rev-14 does NOT change build.rs.)
- Create: `src-tauri/tests/tray_state.rs` **(new)** — pure-Rust priority-ordering tests + rev-12 reducer concurrency tests (keep, assertions use the rev-13 generation-aware accessors) + **rev-13/rev-14/rev-15/rev-16/rev-17:** generation-aware error tests + PulseWorker channel-quit tests (rev-18-5: `match` the `recv_timeout` result against `PulseEvent::Tick`/`Stopped` — deterministic) + `RecordingRenderer` + **rev-14/rev-15/rev-16/rev-17/rev-18: SYNC tests** (the controller methods are sync; the PulseWorker is observed via the `notify` channel carrying `PulseEvent::Tick`/`PulseEvent::Stopped` (rev-17-2) — NO `thread::sleep`) + **rev-15 P1-3 / rev-16 P2-2 / rev-17 P2-1 / rev-18-3 / rev-19-2: `switch_handler_does_not_call_gen_next`** regression test (**rev-18-3: `#[test]` (NOT `#[tokio::test]`), calls the SYNC core `handle_switch_provider_core(&app_state, &uuid)` — NO AppHandle — directly against a REAL temp DB + an inserted provider, asserts `db_providers::read_active_selection` shows `primary == Some(uuid)` + tray `switch_error_rev() == None` + `current_state() == Normal` on success; the `GenerationToken` is NOT advanced; NO mock controller, NO `tauri::test::mock_app`; rev-19-2: the fixture uses the `fresh_db` pattern — `Database::open` + `schema::create_all_tables` + `schema::seed_singletons` inside a transaction FIRST (mirrors tests/provider_crud.rs:21-34), THEN `db_providers::create` — without this the test panics "no such table: providers"**) + **rev-16 P2-2 / rev-18 P2-4: structural `switch_arm_source_has_no_gen_next_call`** (`include_str!` grep — rev-18 P2-4: ALSO asserts the switch arm has no `.await` / no `spawn(async move` / no `pub async fn handle_switch_provider`) + red-dot pixel-diff test (load the generated PNG, assert base pixels unchanged outside the dot + `#DC2626` inside; **rev-14 P2: `panic!` if the PNG is missing**, does NOT silently skip). Test count = **33** (rev-17: 32 from rev-16 + 1 `stale_gen_error_ignored_after_newer_begin` (rev-17-3); rev-18 keeps 33; rev-19 keeps 33 — the functional switch test fixture + the no-churn assertion were rewritten in-place, no test added/removed; verified by grepping `^#\[test\]$`/`^#\[tokio::test\]$` in the Step 2 code block — ALL `#[test]` after rev-18-1, 0 `#[tokio::test]`).

**Verified API facts (rev-11/rev-12; rev-14 amendments):**
- `tauri::tray::TrayIcon::set_icon(&self, icon: Option<Image<'_>>) -> crate::Result<()>` (`tauri-2.11.5/src/tray/mod.rs`). **rev-12 load-bearing for P1-1:** calling `set_icon` with a NEW `Image` IMMEDIATELY replaces the displayed icon — this is the primitive the pulse timer toggles between `normal` and `dimmed` every 800ms (Tauri tray on macOS does NOT support opacity/animation, so the pulse MUST be real byte-level icon swaps).
- `tauri::tray::TrayIcon::set_tooltip<S: AsRef<str>>(&self, tooltip: Option<S>) -> crate::Result<()>` (`tauri-2.11.5/src/tray/mod.rs`).
- `tauri::image::Image::from_bytes(bytes: &[u8]) -> crate::Result<Image>` (`tauri-2.11.5/src/image/mod.rs:76`). The crate already enables `image-png` in the `tauri` Cargo feature list (`tauri = { version = "2", features = ["macos-private-api", "tray-icon", "image-png"] }`), so `from_bytes` on PNG bytes decodes without an extra runtime feature.
- `tauri::Manager::tray_by_id(&self, id: &str) -> Option<TrayIcon>` — already used in A4's `refresh_tray` (`main-tray`).
- `tauri::AppHandle::default_window_icon(&self) -> Option<&Image>` — already used in `build_tray` (lib.rs:2183) — the `Normal` icon source.
- `std::time::Instant::now()` / `Instant::elapsed()` — `std`, no dep.
- **rev-15/rev-16/rev-17 (P1-1 PulseWorker channel-quit — `std::thread` + `mpsc`, NOT tokio; SUPERSEDES the rev-14 infinite-loop + join deadlock):** `std::sync::mpsc::channel() -> (Sender<()>, Receiver<()>)` + `Receiver::recv_timeout(duration) -> Result<(), RecvTimeoutError>` (returns `Ok(())` on signal, `Err(Timeout)` to fire a tick, `Err(Disconnected)` if the sender dropped — both `Ok` and `Disconnected` cause the worker to emit `PulseEvent::Stopped` then `return`). `std::thread::spawn(f) -> std::thread::JoinHandle<T>` + `JoinHandle::join(&self) -> thread::Result<T>`. The controller holds `pulse_worker: Option<PulseWorker>` (NOT a bare `JoinHandle`). **rev-16 P2-1 / rev-17-2: `PulseWorker::start(renderer, interval, notify: Option<Sender<PulseEvent>>)`** spawns the worker whose body loops on `recv_timeout(interval)` and, on each `Timeout` tick, toggles a frame AND `notify.send(PulseEvent::Tick)` (if `notify` is `Some`); on `Ok(())`/`Disconnected` it sends `PulseEvent::Stopped` (if `notify` is `Some`) then returns. `PulseWorker::stop(&mut self)` does `stop_tx.send(())` (wakes the worker) then `handle.take().join()` (the worker returns from `recv_timeout` on the signal so `join` completes — NO deadlock). `impl Drop for PulseWorker { fn drop(&mut self) { self.stop(); } }`. Leaving `Active` = `pulse_worker.take()` (drops the old worker → `stop()` → send + join; the `notify` Sender also drops → the test's receiver gets `Err(Disconnected)` if it did not receive the explicit `PulseEvent::Stopped`, proving the worker is dead). NO `tokio::time`, NO `tauri::async_runtime::spawn`, NO `visual_epoch` (rev-15 P1-4 removes it — the channel-quit barrier replaces the epoch). NO `loop { sleep; render }` without an exit path (rev-14's bug).
- **rev-12 (P1-2 base icon):** `image::open(path) -> ImageResult<DynamicImage>` + `DynamicImage::to_rgba8() -> RgbaImage` + `RgbaImage::get_pixel_mut(x, y) -> &mut Rgba<u8>` (`image 0.25`, the build-dependency added in Step 1). The repo's `src-tauri/icons/32x32.png` (verified: 974 bytes, exists) is the base. No extra drawing crate — the dot is drawn with a manual circle test.
- **rev-14 (P1-3 reducer mutex — `parking_lot::Mutex`, NOT `tokio::sync::Mutex`):** `parking_lot = "0.12"` is ALREADY a production dep (verified `Cargo.toml:53`). `parking_lot::Mutex::lock(&self) -> MutexGuard<R>` is a BLOCKING sync call (parks the calling OS thread until the lock is free) — this is what makes `TranslationGuard::drop` able to run `finish_translation` SYNCHRONOUSLY (no `spawn`, no detached future, no `.await`). The field is `pub tray: Arc<parking_lot::Mutex<TrayStateController>>`. `AppState` already holds `parking_lot::RwLock` fields (`db`, `data_gate`, `readiness`), so this follows the existing pattern.
- **rev-14 (tokio features — NONE added to runtime):** `src-tauri/Cargo.toml:102` is `tokio = { version = "1", features = ["macros", "rt-multi-thread"] }`. rev-14 does NOT add `time`/`sync` to the RUNTIME tokio: the mutex is `parking_lot` (not `tokio::sync`), the timer is `std::thread` (not `tokio::time`). The DEV tokio (also at line 102, under `[dev-dependencies]`) gains `test-util` so `#[tokio::test(start_paused = true)]` remains available; rev-14's sync tests do not strictly need it, but it is added so the feature resolves if a future test uses it.
- **rev-14/rev-15 (P1-2 RAII guard — SYNCHRONOUS Drop):** Rust's `Drop` trait is the single-exit guarantee. `struct TranslationGuard<'a> { controller: &'a Arc<parking_lot::Mutex<TrayStateController>>, gen: u64, succeeded: bool }` with **rev-15: `impl Drop for TranslationGuard<'_> { fn drop(&mut self) { let mut c = self.controller.lock(); c.finish_translation(self.gen, self.succeeded); } }`** — the guard's `Drop` runs SYNCHRONOUSLY on EVERY return path (early return, `?`, panic unwinding) so `finish_translation` is called exactly once per `begin_translation`, BEFORE `drop` returns. The controller mutex is `parking_lot::Mutex` whose `lock()` is a sync blocking call — no async runtime needed. The guard is constructed AFTER the preflight (text captured + anchor built) so a capture/stale-gen failure does NOT begin a translation that then has to be ended.
- **rev-13/rev-15/rev-16/rev-17 (P1-3 generation-aware error):** the controller carries `error_gen: Option<u64>` (translation flow) + `latest_translation_gen: u64` (rev-17-3 — the newest begin_translation gen) + **rev-16-3: `switch_revision: u64` + `switch_error_rev: Option<u64>` (switch flow — replaces rev-15's sticky `has_error: bool` to avoid concurrent-switch reordering)**. `u64` arithmetic + `Option<u64>` comparison are `std` — no new API. `GenerationToken` (concurrency.rs, already on `Session` as `gen`) exposes a monotonic generation counter that the controller mirrors as `error_gen` + `latest_translation_gen`; `begin_translation(gen)` clears a prior error iff `error_gen < Some(gen)` AND bumps `latest_translation_gen = max(self, gen)` (rev-17-3). **rev-16 P1-1: NO function overloading** — the translation-flow error setter is `record_translation_error(gen)` (gen-guarded: only updates if `gen >= latest_translation_gen` (rev-17-3) AND `gen >= error_gen`, so a stale late error does not clobber a newer one). **rev-17-4: the switch flow has NO low-level `record_switch_error()`/`clear_switch_error()`** — the sole switch mutator is `finish_switch(rev, success)` (which records/clears `switch_error_rev` for the matching revision; a stale `rev != switch_revision` is ignored). `begin_switch() -> u64` bumps `switch_revision` + returns it. `recompute` resolves `Error` iff `error_gen.is_some() || switch_error_rev.is_some()`.
- **rev-15 (P1-4 single timer model — DELETES the rev-14 RenderGate / `tick_render()` / `visual_epoch` narration):** rev-14's prose described a "RenderGate" where the timer thread locks the controller, checks `my_epoch == visual_epoch`, and calls `tick_render()` — but the rev-14 `spawn_pulse_timer` CODE captured only `renderer` + `locale` and never locked the controller (prose and code disagreed). **rev-15 keeps the CODE model only:** the worker (`PulseWorker`) holds an independent `Arc<dyn TrayRenderer>` and writes directly through it on each tick; it does NOT lock the controller and does NOT check any epoch. The stop barrier is `PulseWorker::stop()`'s `send` + `join` (the worker returns from `recv_timeout` on the signal). There is NO `visual_epoch` field, NO `tick_render()` method, NO in-timer epoch check. The `render(&mut self)` method is called ONLY by `recompute` (inside the controller's `&mut self` lock) to write the icon+tooltip for a state transition; the worker's per-tick writes go through the renderer directly (serialized against `recompute`'s `render` by the `take()`+`Drop`+`join` of the worker BEFORE `recompute` renders the new state — the worker is dead before the new-state render runs).
- **rev-14/rev-15/rev-16 (worker-swap gating — replaces rev-14 epoch bump gating):** `recompute` resolves `new_state` and compares to `self.current_state`; ONLY when `new_state != current_state` does it (1) drop the old worker (`pulse_worker.take()` → `Drop` → `stop()` → send + join), (2) start a new worker if `new_state == Active` (**rev-16 P2-1: `pulse_worker = Some(PulseWorker::start(self.renderer.clone(), self.tick_interval, self.notify_tx.clone()))`** — the `notify_tx` clone lets the test observe frames), (3) set `current_state = new_state`, (4) call `render()`. A counter bump that keeps the state at `Active` (e.g. a second `begin_translation` while one is in flight) does NOT swap the worker and does NOT churn it. (rev-14 called this "epoch bump gating"; rev-15 renames it to "worker-swap gating" since there is no epoch.)
- **rev-13 (P1-5 injectable renderer):** `tauri::tray::TrayIcon` (the type returned by `app.tray_by_id`) implements the `TrayRenderer` trait via a thin `TrayIconRenderer` wrapper. **rev-14 trait shape (discrete methods, NOT `set_icon(Option<Image>)`):** `trait TrayRenderer: Send + Sync { fn set_icon_normal(&self); fn set_icon_dimmed(&self); fn set_icon_error_dot(&self); fn set_tooltip(&self, text: &str); }` — the renderer DECIDES which embedded PNG / default icon each variant maps to (prod: `TrayIconRenderer` maps them to the real `Image`; test: `RecordingRenderer` records the variant). The controller holds `renderer: Arc<dyn TrayRenderer>` so the SAME controller works in prod (real tray) and tests (`RecordingRenderer`).
- **rev-14 (locale source, verified):** `Settings` (settings.rs:9-15) has ONLY `default_provider`, `target_language`, `fallback_engine` — NO `locale`. rev-14 reads the SYSTEM locale via the `sys-locale` crate: `sys_locale::get_locale() -> Option<String>` (cross-platform — on Unix it reads `LANG`/`LC_*`, on macOS it calls `CFLocaleCopyCurrent`, on Windows it calls `GetUserDefaultLocaleName`); a `starts_with("zh")` check on the returned string yields `Zh`, otherwise `En` (including `None`). `sys-locale = "0.3"` is added to `[dependencies]`. rev-13's `std::env::var("LANG")` is SUPERSEDED (it is Unix-only and returns `None` on Windows).

**Interfaces (produces):**
- `pub enum TrayVisualState { Normal, ActiveTranslation, Error, UpdateAvailable }` — `#[derive(Clone, Copy, PartialEq, Eq, Debug)]`. The priority is encoded by `tray_state_priority` (NOT by a derived `Ord`, to keep the order explicit and the enum field-free).
- `pub fn tray_state_priority(state: TrayVisualState) -> u8` — returns `0` (Normal) < `1` (ActiveTranslation) < `2` (UpdateAvailable) < `3` (Error), matching `Error > Update > Active > Normal`.
- **rev-12 (P2 localization):** `pub enum Locale { En, Zh }` + `pub fn tray_tooltip_text(state: TrayVisualState, locale: Locale) -> &'static str` — en: `"LinguaRay"` / `"Translating…"` / `"LinguaRay — Error"`; zh: `"LinguaRay"` / `"翻译中…"` / `"LinguaRay — 错误"` (`Normal` is `"LinguaRay"` in both).
- **rev-14 (locale source):** `pub fn detect_system_locale() -> Locale` — uses `sys_locale::get_locale()`; `starts_with("zh")` → `Zh`, otherwise `En` (incl. `None`). Does NOT touch `Settings` (which has no locale field). Does NOT read `std::env::var("LANG")` directly (the `sys-locale` crate does that portably internally).
- **rev-14 (P1-5 injectable renderer — discrete methods):**
  ```rust
  /// The tray rendering surface — abstracted so the controller is testable
  /// without a real Tauri tray. Prod: `TrayIconRenderer` wraps a `TrayIcon`.
  /// Test: `RecordingRenderer` records every call (**rev-15 P1-2:
  /// `#[cfg(any(test, feature = "xproc-test-helper"))]`-gated** — NOT
  /// `#[cfg(test)]`, which is invisible to the integration-test crate).
  /// rev-14: discrete methods (NOT a `set_icon(Option<Image>)` taking an enum) so
  /// the renderer DECIDES which embedded PNG/default icon each variant maps to.
  pub trait TrayRenderer: Send + Sync {
      fn set_icon_normal(&self);
      fn set_icon_dimmed(&self);
      fn set_icon_error_dot(&self);
      fn set_tooltip(&self, text: &str);
  }
  pub struct TrayIconRenderer { /* wraps tauri::AppHandle, looks up main-tray */ }
  #[cfg(any(test, feature = "xproc-test-helper"))]
  pub struct RecordingRenderer { /* Mutex<Vec<(RenderedIcon, Option<String>)>> */ }
  ```
  The controller's `render(&mut self)` dispatches on `self.current_state` to call these methods — so callers do NOT pass `app` and the same controller works in prod and tests. The `PulseWorker` (rev-15) ALSO calls these methods directly on each tick (it holds its own `Arc<dyn TrayRenderer>` clone); its writes are serialized against `render` by the worker being stopped (`take()` → `Drop` → `stop()` → send + join) BEFORE `recompute` renders a new state.
- **rev-15/rev-16 (P1-1 + P1-3 + P1-4 controller — PulseWorker, finish_translation, no-overloading switch methods, switch revision, no visual_epoch):**
  ```rust
  /// rev-16 model: SYNC methods, current_state-gated worker swap, PulseWorker
  /// channel-quit, finish_translation merge, NO function overloading (distinct
  /// method names per flow), gen-guarded error clear/set (rev-16-2), switch
  /// revision (rev-16-3 replaces has_error bool). Does NOT derive Debug (holds
  /// Arc<dyn TrayRenderer>).
  pub struct TrayStateController {
      active_translations: u32,
      error_gen: Option<u64>,              // translation flow (gen-tagged)
      switch_revision: u64,                // switch flow (rev-16-3: monotonic, incremented by begin_switch)
      switch_error_rev: Option<u64>,       // switch flow (rev-16-3: which revision errored)
      current_state: TrayVisualState,
      pulse_worker: Option<PulseWorker>,
      tick_interval: std::time::Duration,
      renderer: Arc<dyn TrayRenderer>,
      locale: Locale,
  }
  ```
  Methods (all SYNC — `&mut self`, NO `async`, NO `.await`; **rev-16-1: NO overloading — every method has a distinct name**; **rev-17-4: `record_switch_error`/`clear_switch_error`/`clear_translation_error` DELETED**):
  - `begin_translation(&mut self, gen: u64)` — **rev-17-3: updates `latest_translation_gen = max(self, gen)`;** if `error_gen.map_or(false, |e| e < gen)` clear it (a new generation supersedes a prior translation error); `active_translations += 1`; `recompute()`. (Does NOT touch `switch_revision`/`switch_error_rev`.)
  - `finish_translation(&mut self, gen: u64, success: bool)` — **rev-15 merge + rev-16-2 gen guard:** `active_translations = saturating_sub(1)`; if `success` AND `error_gen.is_some_and(|eg| eg <= gen)` { `error_gen = None` } (an OLDER gen's success does NOT clear a NEWER gen's error — rev-16-2); `recompute()`. Called by `TranslationGuard::drop`.
  - `record_translation_error(&mut self, gen: u64)` — **translation flow (rev-16-1 renamed from `record_error(gen)`); rev-17-3 latest_translation_gen guard + rev-16-2 gen guard:** only update if `gen >= self.latest_translation_gen` (rev-17-3 — a stale OLDER gen that began before a newer gen is ignored) AND `error_gen.is_none_or(|eg| gen >= eg)` (rev-16-2); `error_gen = Some(gen)`; `recompute()`. (Used by `capture_and_translate`/`translate_clipboard` error branches.)
  - `begin_switch(&mut self) -> u64` — **rev-16-3:** `switch_revision += 1`; return `switch_revision`. The caller captures the returned `rev` and passes it to `finish_switch(rev, ...)` after the switch resolves.
  - `finish_switch(&mut self, rev: u64, success: bool)` — **rev-16-3 (sole switch mutator after rev-17-4):** if `rev != self.switch_revision` return (stale/late switch result ignored — only the LATEST revision can update state); else set `switch_error_rev = if success { None } else { Some(rev) }`; `recompute()`. (Replaces the deleted `record_switch_error()`/`clear_switch_error()` — same effect, plus the stale-revision guard.)
  - The controller does NOT expose `end_translation`/`clear_translation_error`/`record_switch_error`/`clear_switch_error`/`mark_success`/`tick_render` (rev-15 collapses/merges; rev-16-1 adds `record_translation_error`/`begin_switch`/`finish_switch` as the sole error/switch mutators; **rev-17-4 deletes the dead `record_switch_error`/`clear_switch_error`**; **rev-17 P2-3 deletes the never-called `clear_translation_error`**).
  - `recompute(&mut self)` — resolve `new_state`; ONLY if `new_state != self.current_state`: drop old worker (`pulse_worker.take()` → `Drop` → `stop()` → send + join); start a new `PulseWorker` if `new_state == Active`; set `self.current_state = new_state`; `self.render()`. (If `new_state == current_state`, do nothing — no worker swap.) `new_state` = `if error_gen.is_some() || switch_error_rev.is_some() { Error } else if active_translations > 0 { Active } else { Normal }` (`UpdateAvailable` is NEVER produced — deferred to R5/R6).
  - `render(&mut self)` — dispatch on `self.current_state` → `Normal`: `renderer.set_icon_normal()`; `Active`: `renderer.set_icon_dimmed()` (the first frame is dimmed for instant feedback; subsequent frames are driven by the `PulseWorker`); `Error`: `renderer.set_icon_error_dot()`; then `renderer.set_tooltip(&self.tooltip_text())`. Called ONLY by `recompute` (inside the controller's `&mut self` lock). The `PulseWorker`'s per-tick writes go through the renderer directly (serialized against `render` by the worker-stop barrier — rev-15 P1-4).
  - `tooltip_text(&self) -> String` — `tray_tooltip_text(self.current_state, self.locale).to_owned()`.
  - Accessors (test-visible, plain `pub fn`): `active_translations(&self) -> u32`, `error_gen(&self) -> Option<u64>`, `switch_revision(&self) -> u64`, `switch_error_rev(&self) -> Option<u64>`, `current_state(&self) -> TrayVisualState`, `is_pulsing(&self) -> bool` (`pulse_worker.is_some()`). (**rev-16-3: `has_error()` accessor REPLACED by `switch_revision()`/`switch_error_rev()`** — the bool no longer exists.)
- **rev-15/rev-16/rev-17/rev-19 (P1-1 PulseWorker; rev-16 P2-1 notify channel; rev-17-2 PulseEvent; rev-19-3 struct drops `notify` field):**
  ```rust
  /// rev-15 P1-1 + rev-16 P2-1 + rev-17-2 + rev-19-3: a background pulse worker
  /// that exits via an mpsc channel signal (NOT an infinite loop + join deadlock).
  /// Holds an independent Arc<dyn TrayRenderer> and toggles dimmed/normal on each
  /// recv_timeout Timeout. rev-17-2: after each tick, sends `PulseEvent::Tick` into
  /// an OPTIONAL `notify` channel so tests can deterministically wait for N frames
  /// via recv_timeout (NO thread::sleep); sends `PulseEvent::Stopped` before exit.
  /// `stop()` sends the signal and joins; `Drop` calls `stop()`.
  /// rev-19-3 (P1-3): the struct does NOT hold a `notify` field — the `notify`
  /// Sender is MOVED into the worker thread closure (the worker owns + drops it).
  /// The struct holds ONLY `stop_tx` + `handle` (both read by `stop()`/`Drop`, no
  /// dead_code warning — rev-18's `notify` field was never read by the struct).
  pub struct PulseWorker {
      stop_tx: std::sync::mpsc::Sender<()>,
      handle: Option<std::thread::JoinHandle<()>>,
  }
  impl PulseWorker {
      /// rev-17-2: `notify` is `Some(Sender<PulseEvent>)` in tests (the test waits
      /// on Tick/Stopped); `None` in prod. rev-19-3: `notify` is moved into the
      /// thread closure — NOT stored on the struct.
      pub fn start(
          renderer: Arc<dyn TrayRenderer>,
          interval: std::time::Duration,
          notify: Option<std::sync::mpsc::Sender<PulseEvent>>,
      ) -> Self {
          let (stop_tx, stop_rx) = std::sync::mpsc::channel();
          // rev-19-3: move (not clone) — the struct no longer holds a notify field.
          let notify_for_thread = notify;
          let handle = std::thread::spawn(move || {
              let mut dimmed = false;
              loop {
                  match stop_rx.recv_timeout(interval) {
                      Ok(()) | Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                          if let Some(tx) = notify_for_thread.as_ref() {
                              let _ = tx.send(PulseEvent::Stopped);
                          }
                          return;
                      }
                      Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                          dimmed = !dimmed;
                          if dimmed { renderer.set_icon_dimmed(); }
                          else { renderer.set_icon_normal(); }
                          // rev-17-2: notify the test that a frame fired (deterministic wait).
                          if let Some(tx) = notify_for_thread.as_ref() {
                              let _ = tx.send(PulseEvent::Tick);
                          }
                      }
                  }
              }
          });
          Self { stop_tx, handle: Some(handle) }
      }
      pub fn stop(&mut self) {
          let _ = self.stop_tx.send(());
          if let Some(h) = self.handle.take() {
              let _ = h.join();
          }
      }
  }
  impl Drop for PulseWorker {
      fn drop(&mut self) { self.stop(); }
  }
  ```
- **rev-14/rev-15 (P1-2 RAII guard — SYNCHRONOUS Drop; rev-15 finish_translation merge):**
  ```rust
  /// Guarantees `finish_translation` runs exactly once per `begin_translation`,
  /// on EVERY return path (early return, `?`, panic). Construct AFTER preflight.
  /// rev-14/rev-15: SYNCHRONOUS — the controller mutex is parking_lot::Mutex, so
  /// Drop runs finish_translation(gen, succeeded) on the CALLING THREAD before
  /// Drop returns (no spawn, no detached future).
  pub struct TranslationGuard<'a> {
      controller: &'a std::sync::Arc<parking_lot::Mutex<TrayStateController>>,
      gen: u64,
      succeeded: bool,
  }
  impl<'a> TranslationGuard<'a> {
      /// Begin a translation (gen-tagged). Synchronous.
      pub fn new(controller: &'a Arc<parking_lot::Mutex<TrayStateController>>, gen: u64) -> Self {
          controller.lock().begin_translation(gen);
          Self { controller, gen, succeeded: false }
      }
      /// Mark the guarded translation as succeeded — the guard's Drop then calls
      /// finish_translation(gen, true), which clears error_gen + decrements +
      /// recomputes. Called on the success branch, BEFORE the guard drops.
      pub fn mark_success(&mut self) {
          self.succeeded = true;
      }
  }
  impl Drop for TranslationGuard<'_> {
      fn drop(&mut self) {
          // Synchronous: park on the mutex, then finish (decrement + maybe clear
          // + recompute) in ONE atomic method call.
          let mut c = self.controller.lock();
          c.finish_translation(self.gen, self.succeeded);
      }
  }
  ```
  Callers (translation flow): `let mut _tray_guard = tray_state::TranslationGuard::new(&app_state.tray, gen);` after the preflight (capture + anchor); on a success branch call `_tray_guard.mark_success();` BEFORE returning (the guard's Drop calls `finish_translation(gen, true)` → clears the prior-gen error IF `error_gen <= gen` (rev-16-2 guard) + decrements + recomputes); on an error branch call `app_state.tray.lock().record_translation_error(gen);` (**rev-16-1 renamed from `record_error(gen)`**, gen-guarded set per rev-16-2 + rev-17-3 `latest_translation_gen` guard; sync) — the guard's Drop calls `finish_translation(gen, false)` (decrement + recompute, does NOT clear `error_gen`). **Switch flow (rev-16-3 / rev-17-1: does NOT use the guard, is async)** — the switch handler calls `let rev = controller.lock().begin_switch();` BEFORE `spawn_blocking`/the async DB call, then after the switch resolves calls `controller.lock().finish_switch(rev, success)` (which records/clears `switch_error_rev` for that revision; a stale `rev != switch_revision` is ignored). **rev-17-4: `record_switch_error()`/`clear_switch_error()` are DELETED** — `finish_switch` is the sole switch mutator. `begin_translation` is called INSIDE `TranslationGuard::new` (so a guard either exists with a begun translation, or does not exist).

- [x] **Step 1: Add the `image` build-dependency + `image` dev-dependency + `sys-locale` runtime dep + `test-util` dev tokio feature (rev-14) in Cargo.toml** (rev-15 note: the `[features] xproc-test-helper = []` line ALREADY EXISTS — no feature addition needed; `src-tauri/Cargo.lock` is git-tracked and will be updated by adding `sys-locale`, so it is committed in Step 12)

Edit `src-tauri/Cargo.toml`. Find the existing `[build-dependencies]` section (it already contains `tauri-build = { version = "2", features = [] }`) and add the `image` line (rev-12: this dep now generates BOTH the red-dot overlay PNG and the dimmed pulse PNG):

```toml
[build-dependencies]
tauri-build = { version = "2", features = [] }
# rev-11/rev-12 / Task A5: generate the tray PNGs at build time. Build-only,
# NOT a runtime dep — the bytes are embedded via include_bytes!(env!("OUT_DIR").
# rev-12: produces TWO icons — tray-error-32.png (red-dot overlay on the base
# icon) + tray-active-32.png (dimmed variant for the pulse frame-swap).
image = { version = "0.25", default-features = false, features = ["png"] }
```

Then **rev-14:** add `sys-locale = "0.3"` to the `[dependencies]` section (cross-platform locale detection — `sys_locale::get_locale()`; `std::env::var("LANG")` is Unix-only and returns `None` on Windows). `parking_lot = "0.12"` is ALREADY present at `Cargo.toml:53` (verified) — do NOT re-add it. The RUNTIME `tokio` line in `[dependencies]` is LEFT UNCHANGED at `tokio = { version = "1", features = ["macros", "rt-multi-thread"] }` (rev-14 uses `parking_lot::Mutex`, not `tokio::sync::Mutex`; a `std::thread` timer, not `tokio::time` — so NO `time`/`sync` runtime features are added). Add `sys-locale` near the other runtime deps:

```toml
# rev-14 / Task A5: cross-platform system locale detection (macOS CFLocale,
# Windows GetUserDefaultLocaleName, Unix LANG/LC_*). rev-13's std::env::var("LANG")
# was Unix-only; sys-locale works on all three platforms.
sys-locale = "0.3"
```

**rev-14:** add `image` as a `[dev-dependencies]` entry AND add `test-util` to the DEV `tokio` line. The current `[dev-dependencies]` section (`Cargo.toml:100-104`) is:

```toml
[dev-dependencies]
wiremock = "0.6"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
tempfile = "3"
crossbeam-channel = "0.5"
```

Replace it with (add the `image` line + the `test-util` feature to the dev tokio):

```toml
[dev-dependencies]
wiremock = "0.6"
# rev-14 / Task A5: add `test-util` so #[tokio::test(start_paused = true)] resolves
# if a test wants it (rev-14's tray_state tests are sync and do not strictly need it,
# but the feature must resolve to avoid a compile error from the attribute).
tokio = { version = "1", features = ["macros", "rt-multi-thread", "test-util"] }
tempfile = "3"
crossbeam-channel = "0.5"
# rev-13/rev-14 / Task A5: the red-dot pixel-diff test (Step 2) loads the generated
# tray-error-32.png and the base icon to assert the overlay is localized to the
# dot circle. Build-dep `image` is NOT visible to integration tests, so this is
# a separate dev-dep entry (same version).
image = { version = "0.25", default-features = false, features = ["png"] }
```

> **rev-14 vs rev-13 Cargo.toml summary:** rev-13 ADDED `time`+`sync` to the RUNTIME tokio (for `tokio::sync::Mutex` + `tokio::time`). rev-14 REVERTS that (the runtime tokio stays `["macros", "rt-multi-thread"]`) because the mutex is now `parking_lot` and the timer is `std::thread`. rev-14 ADDS `sys-locale = "0.3"` (runtime) + `test-util` (dev tokio) + `image` (dev). `parking_lot` was already present — no change there.

- [x] **Step 2: Write the failing test (priority ordering + rev-12 reducer concurrency + rev-15/rev-16/rev-17/rev-19 sync guard/generation/PulseWorker(channel-quit + PulseEvent)/renderer/worker-stop-barrier/switch-revision/gen-guards/latest_translation_gen/worker_start_count-no-churn/functional-switch(fresh_db fixture)/structural(dynamic-tray.switch-submenu)/pixel — 33 tests; rev-19: controller_with_notify passes `Some(notify_tx)` + fresh_db fixture + worker_start_count no-churn assertion)**

Create `src-tauri/tests/tray_state.rs`. The test has THIRTEEN sections (rev-18 enumeration = **33 tests** — unchanged from rev-17; rev-19 keeps 33 — the functional switch test FIXTURE is rewritten to the `fresh_db` pattern + the no-churn assertion to `worker_start_count`, NO test added/removed; ALL `#[test]` SYNC, 0 `#[tokio::test]`; verified by grepping `^#[test]$` + `^#[tokio::test]` in the code block below — 33 `#[test]`, 0 `#[tokio::test]`; rev-18-3 rewrites the functional switch test in-place from `#[tokio::test]`/async to `#[test]`/SYNC against a real DB, no count delta):
1. **rev-11 priority ordering** (6 tests, unchanged pure functions).
2. **rev-12/rev-16 reducer concurrency** (**6 tests** — rev-16 P2-5: header corrected 5→6; unchanged shape, but assertions use `finish_translation`/`switch_error_rev` accessors + SYNC calls — no `.await`).
3. **rev-13/rev-14/rev-15/rev-16 (P1-2) RAII guard** (2 tests) — `TranslationGuard::new` calls `begin_translation` (sync); `Drop` calls `finish_translation(gen, succeeded)` (sync, synchronous Drop via `parking_lot::Mutex`); `mark_success` sets the flag so Drop clears the prior-gen error (rev-16-2: only if `error_gen <= gen`).
4. **rev-13/rev-16 (P1-3) generation-aware error** (2 tests) — a failed gen's red dot is cleared by a successful Retry of a NEW gen, but NOT by `finish_translation(false)` alone. **rev-16-2 adds 2 gen-guard tests** (`older_success_does_not_clear_newer_error`, `older_error_does_not_replace_newer_error`) in a new section 4b. **rev-17-3 adds 1 latest_translation_gen test** (`stale_gen_error_ignored_after_newer_begin`) in section 4b.
5. **rev-14/rev-15/rev-16 (P1-2 + P1-5) TrayRenderer + PulseWorker** (4 tests) — `RecordingRenderer` + a `PulseWorker` (channel-quit); **rev-16 P2-1: tests wait for frames via the `notify` channel (`recv_timeout`), NOT `thread::sleep`**; assert alternating frames, no worker churn (a second `begin` while Active does NOT swap the worker), last `finish` stops the worker, Error produces no active frame.
6. **rev-15 (P1-1) PulseWorker channel-quit** (2 tests, NEW) — `PulseWorker::stop()` sends + joins (returns, no deadlock); `PulseWorker::drop` stops the worker. These call `PulseWorker` directly (NO sleep — they consume the `JoinHandle`). **rev-17 P2-4: they assert `PulseEvent::Stopped` is received on the notify channel before `join()` returns (deterministic).**
7. **rev-15 (P1-4) worker-stop barrier** (1 test, renamed from rev-14's stale-epoch test) — `leaving_active_stops_the_worker_no_stale_frames`: after `record_translation_error(1)` (**rev-16-1 renamed from `record_error(1)`**) switches to `Error` + drops the worker (channel-quit), the `RecordingRenderer` receives NO further dimmed frames (verified via the `notify` channel emitting `PulseEvent::Stopped` — rev-17-2 / rev-16 P2-1: NO `thread::sleep`).
8. **rev-12 (P2) + rev-14 localization** (2 tests, unchanged).
9. **rev-14/rev-15/rev-16 red-dot pixel-diff** (1 test, unchanged) — load the generated `tray-error-32.png`, assert base pixels are unchanged OUTSIDE the dot and `#DC2626` INSIDE the dot; **`panic!` if the PNG is missing**.
10. **rev-15 (P1-3) + rev-16 (P1-3 + P2-2) + rev-17 P2-1 + rev-18-1/3 switch does NOT bump translation generation** — **rev-18-3: the functional test now calls the REAL SYNC core `handle_switch_provider_core(app_state, uuid)` (rev-18-1: NO AppHandle, NO `.await`) against a REAL temp DB + inserted provider, not a mock controller simulation and not an async `.await`; rev-16: section split into 1 functional + 1 structural.** (a) `switch_handler_does_not_call_gen_next` (rev-16 P2-2 renamed/extended from `switch_does_not_bump_translation_generation`; rev-18-3: functional via the real SYNC core): build a temp DB + insert a provider via `db_providers::create`, allocate `g1 = token.next()`, call the SYNC `handle_switch_provider_core(&app_state, &uuid)` (rev-18-1: NO `.await`, NO AppHandle), assert `token.is_latest(g1) == true` AND `db_providers::read_active_selection().primary == Some(uuid)` AND tray `switch_error_rev() == None` + `current_state() == Normal`; ALSO a FAILURE sub-path switches to an unknown uuid and asserts the tray goes to Error + DB unchanged. AND a structural `#[test]` reads `lib.rs` via `include_str!("../src/lib.rs")` and asserts the `tray.switch-` arm contains no `.gen.next()` AND (rev-18 P2-4) no `.await`/`spawn(async move`/`pub async fn handle_switch_provider` (regression guard). (b) **rev-16-3: 2 NEW switch-revision tests** (`two_concurrent_switches_second_wins`, `stale_switch_result_ignored`) in a new section 11.

The pure priority tests + the reducer tests construct the controller directly and assert the counter/error/`switch_error_rev` transitions WITHOUT driving the real Tauri tray (the controller's `render` writes through the injected `Arc<dyn TrayRenderer>`, so the test injects a `RecordingRenderer` — NO Tauri runtime link required). **rev-14/rev-15/rev-16: ALL tests are SYNC** — the controller methods are sync (`c.begin_translation(1);`, no `.await`), so tests are plain `#[test]`. **rev-16 P2-1: the PulseWorker-lifecycle tests construct a controller whose worker uses a TINY interval (2ms) AND a `notify` channel — the test `recv_timeout` on the notify channel to deterministically wait for N frames (NO `thread::sleep`)**. The PulseWorker channel-quit tests call `PulseWorker::stop()` / drop directly with NO sleep (deterministic — the `send` + `join` is the synchronization). This avoids both real-time 800ms waits AND `tokio::time` AND `thread::sleep` flakiness on slow CI.

```rust
//! Task A5 (rev-11 → rev-18): tray visual-state controller.
//!
//! Sections (33 tests, rev-17 enumeration):
//! 1. rev-11 priority ordering (6 pure-function tests).
//! 2. rev-12/rev-16 TrayStateController reducer concurrency (6 tests, counter
//!    saturating-sub + generation-aware error + switch_revision/switch_error_rev
//!    switch flag — rev-16-3 replaces has_error bool; rev-16 P2-5: header 5→6;
//!    rev-17-4: switch mutator is finish_switch, not record_switch_error)
//!    — rev-14/rev-15/rev-16/rev-17: SYNC calls, finish_translation merge, no .await.
//! 3. rev-13/rev-14/rev-15/rev-16 (P1-2) TranslationGuard RAII (2 tests) —
//!    begin/finish pairing on every path (synchronous Drop via parking_lot::Mutex).
//! 4. rev-13/rev-16 (P1-3 + rev-16-2) generation-aware error (2 tests) —
//!    Retry-of-new-gen clears prior red dot.
//! 4b. rev-16 (P1-2) + rev-17-3 gen guards (3 tests) — older gen's success does
//!    NOT clear a newer gen's error; older gen's late error does NOT clobber a
//!    newer gen's error; rev-17-3: a stale gen's late error is IGNORED after a
//!    newer gen began (latest_translation_gen guard).
//! 5. rev-14/rev-15/rev-16/rev-17 (P1-2 + P1-5) TrayRenderer + PulseWorker (4 tests) —
//!    RecordingRenderer + tiny interval + rev-16 P2-1 / rev-17-2 notify channel
//!    carrying PulseEvent (NO sleep); assert alternating frames, no worker churn
//!    on second begin, last finish stops the worker, Error → no active frame.
//! 6. rev-15 (P1-1) PulseWorker channel-quit (2 tests, NEW) — stop() sends + joins
//!    (returns, no deadlock); Drop stops the worker. NO sleep. rev-17 P2-4: assert
//!    PulseEvent::Stopped is received before join (deterministic).
//! 7. rev-15 (P1-4) worker-stop barrier (1 test, renamed) — leaving Active stops
//!    the worker; no stale dimmed frames reach the renderer afterwards
//!    (rev-17-2: notify channel emits PulseEvent::Stopped — NO thread::sleep).
//! 8. rev-12 (P2) + rev-14 localization (2 tests).
//! 9. rev-14/rev-15/rev-16 red-dot pixel-diff (1 test) — panics if PNG missing.
//! 10. rev-15 (P1-3) + rev-16 (P1-3 + P2-2) + rev-17 P2-1 switch does NOT bump
//!    translation generation — functional test via the REAL async
//!    `handle_switch_provider` (rev-17 P2-1) + structural `include_str!` grep
//!    asserting the switch arm has no `.gen.next()`.
//! 11. rev-16 (P1-3) switch revision ordering (2 tests, NEW) —
//!    two_concurrent_switches_second_wins + stale_switch_result_ignored.
//!
//! rev-14/rev-15/rev-16/rev-17: ALL tests are SYNC (the controller is parking_lot-based). No
//! Tauri runtime is linked: the controller takes an injected `Arc<dyn TrayRenderer>`,
//! and the tests inject `RecordingRenderer` (rev-15 P1-2: visible under
//! `--features xproc-test-helper`, NOT `#[cfg(test)]`). rev-16 P2-3: the test
//! imports do NOT name `RenderedIcon` or `TrayRenderer` directly (they are reached
//! via `RecordingRenderer::calls()` element methods `.is_dimmed()` etc.) — clippy
//! `unused_imports`-clean. rev-17-2: the notify channel carries `PulseEvent`.
use linguaray_lib::tray_state::{
    detect_system_locale, recompute_pure, tray_state_priority, tray_tooltip_text, Locale,
    PulseEvent, PulseWorker, RecordingRenderer, TrayStateController,
    TrayVisualState, TranslationGuard,
};
use std::sync::Arc;

// ─── 1. rev-11: priority ordering (pure functions) ──────────────────────────

#[test]
fn normal_is_lowest_priority() {
    assert_eq!(tray_state_priority(TrayVisualState::Normal), 0);
}

#[test]
fn active_beats_normal() {
    assert!(
        tray_state_priority(TrayVisualState::ActiveTranslation)
            > tray_state_priority(TrayVisualState::Normal)
    );
}

#[test]
fn update_beats_active() {
    assert!(
        tray_state_priority(TrayVisualState::UpdateAvailable)
            > tray_state_priority(TrayVisualState::ActiveTranslation)
    );
}

#[test]
fn error_is_highest_priority() {
    assert!(
        tray_state_priority(TrayVisualState::Error)
            > tray_state_priority(TrayVisualState::UpdateAvailable)
    );
    assert!(
        tray_state_priority(TrayVisualState::Error)
            > tray_state_priority(TrayVisualState::ActiveTranslation)
    );
    assert!(
        tray_state_priority(TrayVisualState::Error)
            > tray_state_priority(TrayVisualState::Normal)
    );
}

#[test]
fn full_ordering_is_error_update_active_normal() {
    let mut ordered = [
        TrayVisualState::Normal,
        TrayVisualState::Error,
        TrayVisualState::ActiveTranslation,
        TrayVisualState::UpdateAvailable,
    ];
    ordered.sort_by_key(|s| tray_state_priority(*s));
    assert_eq!(
        ordered,
        [
            TrayVisualState::Normal,
            TrayVisualState::ActiveTranslation,
            TrayVisualState::UpdateAvailable,
            TrayVisualState::Error,
        ]
    );
}

#[test]
fn update_arm_exists_but_is_documented_deferred() {
    // The UpdateAvailable variant is RETAINED so the priority ordering is
    // testable, but `recompute` NEVER produces it this stage (deferred to
    // R5/R6 per user-approved scope decision).
    let _ = TrayVisualState::UpdateAvailable;
}

// ─── 2. rev-12/rev-13/rev-14/rev-15/rev-16: TrayStateController reducer concurrency ─
// rev-14/rev-15/rev-16: SYNC methods (no .await). Controller backed by a RecordingRenderer.

fn test_controller() -> TrayStateController {
    // rev-14/rev-15: tiny tick interval so the PulseWorker tests don't sleep 800ms.
    TrayStateController::with_renderer_and_interval(
        Arc::new(RecordingRenderer::default()),
        Locale::En,
        std::time::Duration::from_millis(2),
    )
}

#[test]
fn recompute_pure_normal_when_idle() {
    let c = test_controller();
    assert_eq!(recompute_pure(&c), TrayVisualState::Normal);
}

#[test]
fn begin_then_finish_two_translations_keeps_active_until_last_finishes() {
    // Two concurrent translations: begin/begin → Active; finish (first done) →
    // STILL Active (counter saturating-sub, one remains); finish (last done) →
    // Normal. rev-15: finish_translation(success) merges end + clear + recompute.
    let mut c = test_controller();
    c.begin_translation(1);
    c.begin_translation(2);
    assert_eq!(recompute_pure(&c), TrayVisualState::ActiveTranslation);
    c.finish_translation(1, false); // → 1 remains (success=false keeps any error_gen)
    assert_eq!(
        recompute_pure(&c),
        TrayVisualState::ActiveTranslation,
        "still Active while one translation remains"
    );
    c.finish_translation(2, false); // → 0
    assert_eq!(recompute_pure(&c), TrayVisualState::Normal);
}

#[test]
fn finish_translation_saturates_at_zero() {
    // Defending against underflow: more finish_translation calls than begin.
    let mut c = test_controller();
    c.finish_translation(1, false); // 0 saturating-sub
    c.finish_translation(2, false); // 0 saturating-sub
    assert_eq!(c.active_translations(), 0);
    assert_eq!(recompute_pure(&c), TrayVisualState::Normal);
}

#[test]
fn translation_error_overrides_active_and_survives_finish_false() {
    // rev-13 P1-3 / rev-15: Error has priority and survives finish_translation(false).
    // A concurrent translation finishing with success=false must NOT clear an
    // error recorded by its own gen.
    let mut c = test_controller();
    c.begin_translation(1);
    c.record_translation_error(1); // rev-16-1 renamed from record_error(1): gen 1 errored
    assert_eq!(recompute_pure(&c), TrayVisualState::Error);
    c.finish_translation(1, false); // finishes without success → error persists
    assert_eq!(
        recompute_pure(&c),
        TrayVisualState::Error,
        "Error must NOT be cleared by finish_translation(false) alone"
    );
}

#[test]
fn recompute_never_produces_update_available() {
    // UpdateAvailable is deferred to R5/R6 — recompute_pure must NEVER return it
    // regardless of the counter/error/switch_error_rev state.
    let mut c = test_controller();
    c.begin_translation(1);
    assert_ne!(recompute_pure(&c), TrayVisualState::UpdateAvailable);
    c.record_translation_error(1); // rev-16-1 renamed
    assert_ne!(recompute_pure(&c), TrayVisualState::UpdateAvailable);
    let rev = c.begin_switch(); // rev-16-3: switch flow (no gen, uses switch_revision)
    c.finish_switch(rev, false); // rev-17-4: finish_switch (NOT record_switch_error) sets switch_error_rev
    assert_ne!(recompute_pure(&c), TrayVisualState::UpdateAvailable);
}

#[test]
fn switch_flow_error_is_independent_of_translation_error_gen() {
    // rev-15 P1-3 + rev-16-3 + rev-17-4: the switch flow's `switch_error_rev` is a
    // SEPARATE flag from the translation flow's `error_gen`. finish_switch(rev,
    // false) sets switch_error_rev; record_translation_error(gen) sets error_gen.
    // Both OR into Error. rev-17-4: record_switch_error()/clear_switch_error() are
    // DELETED — finish_switch is the sole switch mutator.
    let mut c = test_controller();
    let rev = c.begin_switch();
    c.finish_switch(rev, false); // switch failure → switch_error_rev = Some(rev)
    assert_eq!(c.switch_error_rev(), Some(rev));
    assert_eq!(c.error_gen(), None); // translation error_gen untouched
    assert_eq!(recompute_pure(&c), TrayVisualState::Error);
    c.finish_switch(rev, true); // switch success → switch_error_rev = None
    assert_eq!(c.switch_error_rev(), None);
    assert_eq!(recompute_pure(&c), TrayVisualState::Normal);
}

// ─── 3. rev-13/rev-14/rev-15 (P1-2): TranslationGuard RAII (synchronous Drop) ─

#[test]
fn guard_drop_finishes_translation_on_every_return_path() {
    // rev-14/rev-15: the guard's Drop runs finish_translation SYNCHRONOUSLY
    // (parking_lot mutex), even on early scope-end. No detached spawn.
    let controller = Arc::new(parking_lot::Mutex::new(test_controller()));
    {
        let _guard = TranslationGuard::new(&controller, 1);
        assert_eq!(controller.lock().active_translations(), 1);
        // Early return here (simulated by scope end) — Drop must fire synchronously.
    }
    assert_eq!(
        controller.lock().active_translations(),
        0,
        "guard Drop decremented the counter synchronously"
    );
}

#[test]
fn guard_marks_success_and_clears_prior_gen_error() {
    // A successful translation (mark_success → finish_translation(gen, true))
    // clears an error recorded by an OLDER generation (rev-16-2: only if error_gen <= gen).
    let controller = Arc::new(parking_lot::Mutex::new(test_controller()));
    controller.lock().record_translation_error(1); // rev-16-1 renamed: old gen errored
    {
        let mut guard = TranslationGuard::new(&controller, 2); // new gen
        guard.mark_success(); // this gen succeeded → Drop calls finish_translation(2, true)
    }
    assert_eq!(
        controller.lock().error_gen(),
        None,
        "successful Retry of a new gen clears the prior gen's error (1 <= 2)"
    );
}

// ─── 4. rev-13/rev-16 (P1-3 + rev-16-2 gen guards): generation-aware error ────

#[test]
fn retry_of_new_gen_clears_prior_red_dot() {
    // The user-visible contract: a failed translation shows the red dot; a
    // subsequent Retry that SUCCEEDS (new gen) clears it.
    let mut c = test_controller();
    c.begin_translation(1);
    c.record_translation_error(1); // rev-16-1 renamed: gen 1 failed → red dot
    assert_eq!(c.error_gen(), Some(1));

    // Retry: new gen begins — begin_translation(2) clears the OLD gen's error
    // because 1 < 2.
    c.begin_translation(2);
    assert_eq!(
        c.error_gen(),
        None,
        "begin_translation of a newer gen clears the older gen's error"
    );
    c.finish_translation(2, true); // gen 2 succeeds
    assert_eq!(recompute_pure(&c), TrayVisualState::Normal);
}

#[test]
fn same_gen_retry_does_not_clear_error() {
    // If the SAME generation retries (shouldn't happen via the guard, but the
    // controller must be robust), the error must persist — only a STRICTLY newer
    // gen clears it.
    let mut c = test_controller();
    c.begin_translation(1);
    c.record_translation_error(1); // rev-16-1 renamed
    c.begin_translation(1); // same gen (not newer)
    assert_eq!(c.error_gen(), Some(1), "same-gen begin must NOT clear error");
}

// ─── 4b. rev-16 (P1-2) + rev-17-3 (P1-3): gen guards + latest_translation_gen ───
// rev-16-2: an OLDER gen's late success must NOT clear a NEWER gen's error,
// and an OLDER gen's late error must NOT clobber a NEWER gen's error.
// rev-17-3: a gen that began BEFORE a newer gen but reports its error LATE must
// be IGNORED (the user now sees the newer gen; the stale gen's error is noise).

#[test]
fn older_success_does_not_clear_newer_error() {
    // gen 1 translation errors (error_gen = Some(1)); gen 2 translation begins
    // and errors (error_gen = Some(2)); then gen 1's late success arrives —
    // finish_translation(1, true) must NOT clear the gen 2 error (1 <= 2 is true,
    // but the guard is `error_gen <= gen` which for error_gen=2, gen=1 → 2 <= 1
    // is FALSE → no clear). Wait — re-derive: rev-16-2 clears iff
    // error_gen.is_some_and(|eg| eg <= gen). For error_gen=Some(2), gen=1: 2 <= 1
    // is false → no clear. Correct: the newer error persists.
    let mut c = test_controller();
    c.begin_translation(1);
    c.record_translation_error(1); // gen 1 errored
    c.begin_translation(2);
    c.record_translation_error(2); // gen 2 errored (newer) — rev-16-2: gen >= eg so 2 >= 1 → replaces
    assert_eq!(c.error_gen(), Some(2));
    c.finish_translation(1, true); // gen 1 late success — must NOT clear gen 2's error (2 <= 1 is false)
    assert_eq!(
        c.error_gen(),
        Some(2),
        "older gen's success must NOT clear a newer gen's error (rev-16-2 gen guard)"
    );
    assert_eq!(recompute_pure(&c), TrayVisualState::Error);
}

#[test]
fn older_error_does_not_replace_newer_error() {
    // gen 2 errors first (error_gen = Some(2)); gen 1's late error arrives —
    // record_translation_error(1) must NOT replace the gen 2 error
    // (rev-16-2: gen >= eg → 1 >= 2 is false → no replace).
    let mut c = test_controller();
    c.begin_translation(2);
    c.record_translation_error(2); // gen 2 errored
    assert_eq!(c.error_gen(), Some(2));
    c.record_translation_error(1); // gen 1 late error — must NOT replace gen 2's error
    assert_eq!(
        c.error_gen(),
        Some(2),
        "older gen's late error must NOT clobber a newer gen's error (rev-16-2 gen guard)"
    );
}

#[test]
fn stale_gen_error_ignored_after_newer_begin() {
    // rev-17-3 latest_translation_gen guard: gen 1 begins (no error yet); gen 2
    // begins (latest_translation_gen = 2); THEN gen 1's LATE error arrives —
    // record_translation_error(1) must be IGNORED (gen 1 < latest_translation_gen = 2),
    // so error_gen stays None. The user is now waiting on gen 2; gen 1's stale
    // error must NOT surface a red dot. (rev-16-2's `gen >= error_gen` guard alone
    // is insufficient here: error_gen is None, so is_none_or returns true and the
    // stale gen 1 error would be recorded without the latest_translation_gen gate.)
    let mut c = test_controller();
    c.begin_translation(1); // latest_translation_gen = 1
    c.begin_translation(2); // latest_translation_gen = 2
    assert_eq!(c.error_gen(), None);
    c.record_translation_error(1); // gen 1 LATE error — must be IGNORED (1 < 2)
    assert_eq!(
        c.error_gen(),
        None,
        "a stale gen's late error must be ignored after a newer gen began (rev-17-3 latest_translation_gen guard)"
    );
    assert_eq!(recompute_pure(&c), TrayVisualState::ActiveTranslation);
    // Sanity: gen 2's OWN error IS recorded (gen 2 == latest_translation_gen).
    c.record_translation_error(2);
    assert_eq!(c.error_gen(), Some(2));
}

// ─── 5. rev-14/rev-15/rev-16/rev-17 (P1-2 + P1-5): TrayRenderer + PulseWorker ────
// The RecordingRenderer records every set_icon_*/set_tooltip call. The PulseWorker
// is a std::thread with a TINY interval (2ms). rev-16 P2-1 / rev-17-2: the worker
// emits a per-tick PulseEvent::Tick (and PulseEvent::Stopped on exit) into a
// `notify` mpsc channel; the test `recv_timeout` on it to deterministically wait
// for N frames (NO thread::sleep — CI-flake-free).

/// rev-16 P2-1 / rev-17-2: build a controller whose PulseWorker notifies a fresh
/// channel on each tick + on exit. Returns (controller, renderer_clone,
/// notify_receiver) — the test reads `renderer_clone.calls()` directly (the clone
/// shares the same inner `Mutex<Vec<...>>` as the renderer the controller holds).
/// No downcast needed. rev-17-2: the channel carries `PulseEvent` (Tick/Stopped).
fn controller_with_notify() -> (
    TrayStateController,
    Arc<RecordingRenderer>,
    std::sync::mpsc::Receiver<PulseEvent>,
) {
    let (notify_tx, notify_rx) = std::sync::mpsc::channel();
    let renderer = Arc::new(RecordingRenderer::default());
    // rev-19-1: pass `Some(notify_tx)` — the constructor's 4th param is
    // `Option<mpsc::Sender<PulseEvent>>`, a bare `Sender` would be a type
    // mismatch (`Sender` vs `Option<Sender>`). `worker_start_count` starts at 0
    // (rev-19-4 — the no-churn test asserts it stays 0 across the second begin).
    let c = TrayStateController::with_renderer_interval_and_notify(
        renderer.clone(),
        Locale::En,
        std::time::Duration::from_millis(2),
        Some(notify_tx),
    );
    (c, renderer, notify_rx)
}

#[test]
fn active_emits_alternating_frames_on_the_renderer() {
    let (mut c, renderer, notify_rx) = controller_with_notify();
    c.begin_translation(1);
    // rev-16 P2-1: wait for at least 2 ticks deterministically (NO thread::sleep).
    // rev-18-5: each recv_timeout is matched explicitly against PulseEvent::Tick
    // (not `let _ =`) — a non-Tick event (Stopped/Disconnected) fails the test.
    match notify_rx.recv_timeout(std::time::Duration::from_millis(50)) {
        Ok(PulseEvent::Tick) => { /* frame 1 (dimmed) */ }
        other => panic!("expected PulseEvent::Tick (frame 1), got {other:?}"),
    }
    match notify_rx.recv_timeout(std::time::Duration::from_millis(50)) {
        Ok(PulseEvent::Tick) => { /* frame 2 (normal) */ }
        other => panic!("expected PulseEvent::Tick (frame 2), got {other:?}"),
    }
    let calls = renderer.calls();
    assert!(
        calls.iter().any(|(icon, _)| icon.is_dimmed()),
        "expected at least one dimmed pulse frame"
    );
    assert!(
        calls.iter().any(|(icon, _)| icon.is_normal()),
        "expected at least one normal pulse frame"
    );
    c.finish_translation(1, true);
}

#[test]
fn second_begin_does_not_churn_the_worker() {
    // rev-15 P1-4 (was rev-14 P1-2 "epoch bump gating"): a second begin while
    // Active keeps the state at Active, so the PulseWorker is NOT swapped (no
    // churn). rev-19-4 (determinism, replaces rev-18-5's frame-count comparison):
    // assert via `worker_start_count` — a monotonic counter incremented ONLY when
    // `recompute` enters the `new_state == ActiveTranslation` branch and starts a
    // new `PulseWorker`. The first begin bumps it to 1; the second begin hits the
    // `new_state == current_state` early-return in `recompute` and does NOT reach
    // the start branch, so the count stays 1 (the SAME worker is reused). This is
    // fully deterministic — no timing-sensitive frame-count comparison. The
    // notify-channel Tick observation is retained as a secondary confirmation
    // (the same worker keeps emitting Tick events across the second begin), using
    // an explicit `match` on recv_timeout (not `let _ =`).
    let (mut c, _renderer, notify_rx) = controller_with_notify();
    assert_eq!(c.worker_start_count(), 0, "no worker started before any begin");
    c.begin_translation(1);
    let count_after_first = c.worker_start_count();
    assert_eq!(count_after_first, 1, "first begin started exactly one worker");
    assert!(c.is_pulsing(), "worker running after first begin");
    assert_eq!(c.current_state(), TrayVisualState::ActiveTranslation);
    // Observe at least one frame from the worker (confirm it is ticking).
    match notify_rx.recv_timeout(std::time::Duration::from_millis(50)) {
        Ok(PulseEvent::Tick) => { /* worker ticking */ }
        other => panic!("expected PulseEvent::Tick after first begin, got {other:?}"),
    }
    c.begin_translation(2); // counter → 2, state stays Active (recompute early-returns)
    // (a) Deterministic no-churn assertion: worker_start_count did NOT increase —
    // the second begin reused the existing worker (recompute never started a new one).
    assert_eq!(
        c.worker_start_count(),
        count_after_first,
        "second begin did NOT churn the worker (recompute early-returned on Active→Active)"
    );
    assert!(c.is_pulsing(), "the worker is still running (not swapped) after the second begin");
    assert_eq!(
        c.current_state(),
        TrayVisualState::ActiveTranslation,
        "state stays Active across the second begin (no churn)"
    );
    // (b) Secondary: the SAME worker keeps ticking (a Tick event flows).
    match notify_rx.recv_timeout(std::time::Duration::from_millis(50)) {
        Ok(PulseEvent::Tick) => { /* same worker ticking */ }
        other => panic!("expected PulseEvent::Tick from the same worker after second begin, got {other:?}"),
    }
    c.finish_translation(1, true);
    c.finish_translation(2, true);
}

#[test]
fn last_finish_stops_the_worker() {
    let (mut c, renderer, notify_rx) = controller_with_notify();
    c.begin_translation(1);
    c.finish_translation(1, true); // → Normal, worker dropped (→ stop → send + join)
    assert!(!c.is_pulsing());
    let calls_before = renderer.calls().len();
    // rev-17-2: the worker emits PulseEvent::Stopped immediately before its thread
    // returns (inside stop() → send signal → worker wakes → send Stopped → return →
    // join completes). So recv_timeout returns Ok(PulseEvent::Stopped), deterministically
    // proving the worker is dead (NOT relying on the Disconnected side-effect of the
    // Sender dropping).
    match notify_rx.recv_timeout(std::time::Duration::from_millis(50)) {
        Ok(PulseEvent::Stopped) => { /* expected — worker signaled stop */ }
        other => panic!("expected PulseEvent::Stopped after worker drop, got {other:?}"),
    }
    assert_eq!(
        renderer.calls().len(),
        calls_before,
        "no further frames after the last finish (worker stopped via channel-quit)"
    );
}

#[test]
fn error_produces_no_active_pulse_frame() {
    let (mut c, renderer, _notify_rx) = controller_with_notify();
    c.record_translation_error(1); // rev-16-1 renamed: Error WITHOUT a begin
    // No PulseWorker is started for Error (recompute goes Active→Error or Normal→Error,
    // neither starts a worker). The notify channel stays connected-but-empty.
    assert!(
        !renderer.calls().iter().any(|(icon, _)| icon.is_dimmed()),
        "Error must not start the Active pulse"
    );
    assert!(
        renderer.calls().iter().any(|(icon, _)| icon.is_error_dot()),
        "Error must emit the red-dot overlay"
    );
}

// ─── 6. rev-15 (P1-1): PulseWorker channel-quit (NEW — no sleep) ────────────
// These tests call PulseWorker directly (NOT through the controller) to verify
// the channel-quit mechanism deterministically — stop() sends + joins, and the
// JoinHandle is consumed. NO thread::sleep: the send + join IS the
// synchronization (the worker returns from recv_timeout on the signal).

#[test]
fn stop_signal_joins_the_worker() {
    // rev-15 P1-1: PulseWorker::stop() sends the signal and joins. The worker's
    // recv_timeout returns Ok(()) on the signal, the thread returns, and join()
    // completes — NO infinite-loop + join deadlock (the rev-14 bug).
    // rev-18-5 (determinism): (1) use a TINY interval + recv_timeout to FIRST
    // observe a PulseEvent::Tick — proving the worker is actually running — THEN
    // call stop() and observe PulseEvent::Stopped. rev-17 P2-4 only asserted
    // Stopped after an immediate stop (800ms interval, no Tick observed), so a
    // worker that never started would also "pass"; rev-18-5 closes that gap by
    // confirming the worker ran (Tick) BEFORE confirming it stopped (Stopped).
    let renderer = Arc::new(RecordingRenderer::default());
    let (notify_tx, notify_rx) = std::sync::mpsc::channel();
    let mut worker = PulseWorker::start(
        renderer.clone(),
        std::time::Duration::from_millis(2), // tiny interval so the first Tick arrives fast
        Some(notify_tx),
    );
    // (1) Confirm the worker is RUNNING: it emits a Tick after one interval.
    match notify_rx.recv_timeout(std::time::Duration::from_millis(50)) {
        Ok(PulseEvent::Tick) => { /* worker is alive */ }
        other => panic!("expected PulseEvent::Tick (worker running) before stop, got {other:?}"),
    }
    // (2) Stop the worker: send the signal, then join. The worker wakes from
    // recv_timeout on the signal, emits PulseEvent::Stopped, the thread exits,
    // join() completes (NO deadlock).
    worker.stop();
    // (3) Deterministically assert the worker emitted PulseEvent::Stopped (it is
    // sent before the thread returns, and join() has now completed, so the event
    // MUST be available on the channel).
    match notify_rx.recv_timeout(std::time::Duration::from_millis(50)) {
        Ok(PulseEvent::Stopped) => { /* expected — worker signaled stop before exit */ }
        other => panic!("expected PulseEvent::Stopped after stop(), got {other:?}"),
    }
    // After stop, the handle is taken (None) — calling stop again is a no-op.
    worker.stop();
    // The worker stopped (no further Tick events; recv now returns Disconnected).
    match notify_rx.recv_timeout(std::time::Duration::from_millis(50)) {
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => { /* channel closed */ }
        other => panic!("expected Disconnected after worker drop, got {other:?}"),
    }
}

#[test]
fn drop_stops_the_worker() {
    // rev-15 P1-1: dropping a PulseWorker stops it (Drop calls stop()). The drop
    // must return (not deadlock) — verified by the test completing.
    // rev-18-5 (determinism): use a TINY interval + observe a Tick (worker running)
    // BEFORE the drop, then observe Stopped AFTER the drop. rev-17 P2-4 accepted
    // BOTH Ok(Stopped) AND Err(Disconnected) after the drop — the Disconnected arm
    // is a Sender-drop side-effect, not the worker's explicit signal, so a worker
    // that forgot to emit Stopped would still "pass". rev-18-5 asserts Stopped
    // explicitly (the worker emits it inside Drop → stop → send before join).
    let renderer = Arc::new(RecordingRenderer::default());
    let (notify_tx, notify_rx) = std::sync::mpsc::channel();
    {
        let _worker = PulseWorker::start(
            renderer.clone(),
            std::time::Duration::from_millis(2), // tiny interval so the first Tick arrives fast
            Some(notify_tx),
        );
        // (1) Confirm the worker is RUNNING before we drop it.
        match notify_rx.recv_timeout(std::time::Duration::from_millis(50)) {
            Ok(PulseEvent::Tick) => { /* worker is alive */ }
            other => panic!("expected PulseEvent::Tick (worker running) before drop, got {other:?}"),
        }
        // _worker drops here → Drop → stop() → send(Stopped) + join → returns.
    }
    // (2) Assert the worker emitted PulseEvent::Stopped (NOT Disconnected — the
    // explicit signal is deterministic; the Sender drop side-effect is not).
    match notify_rx.recv_timeout(std::time::Duration::from_millis(50)) {
        Ok(PulseEvent::Stopped) => { /* expected — worker signaled stop before exit */ }
        other => panic!("expected PulseEvent::Stopped after drop (NOT Disconnected), got {other:?}"),
    }
    // (3) The channel is now closed (Sender dropped with the worker).
    match notify_rx.recv_timeout(std::time::Duration::from_millis(50)) {
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => { /* channel closed */ }
        other => panic!("expected Disconnected after the Stopped event, got {other:?}"),
    }
    // If drop deadlocked (rev-14 bug), the test would never reach this line.
}

// ─── 7. rev-15/rev-16 (P1-4): worker-stop barrier (renamed from rev-14 stale-epoch) ─

#[test]
fn leaving_active_stops_the_worker_no_stale_frames() {
    // Active → Error: recompute drops the PulseWorker (take → Drop → stop →
    // send + join), so by the time render() writes the Error frame, the worker
    // is DEAD. No further dimmed frames can reach the renderer after the Error
    // transition — there is no epoch, the channel-quit barrier is the guarantee.
    // rev-17-2: verified via the notify channel emitting PulseEvent::Stopped (the
    // worker sends it before its thread returns) — NO thread::sleep.
    let (mut c, renderer, notify_rx) = controller_with_notify();
    c.begin_translation(1);
    // rev-18-5: confirm the worker ran (Tick) before the Error transition.
    match notify_rx.recv_timeout(std::time::Duration::from_millis(50)) {
        Ok(PulseEvent::Tick) => { /* first tick(s) — worker running */ }
        other => panic!("expected PulseEvent::Tick before the Error transition, got {other:?}"),
    }
    let dimmed_before = renderer.calls().iter().filter(|(i, _)| i.is_dimmed()).count();
    c.record_translation_error(1); // rev-16-1 renamed: → Error, worker dropped (send + join under the lock)
    assert!(!c.is_pulsing(), "the worker was dropped on the Active → Error transition");
    // rev-17-2: the worker emitted PulseEvent::Stopped before its thread exited.
    match notify_rx.recv_timeout(std::time::Duration::from_millis(50)) {
        Ok(PulseEvent::Stopped) => { /* worker is dead */ }
        other => panic!("expected PulseEvent::Stopped after Active→Error worker drop, got {other:?}"),
    }
    let dimmed_after = renderer.calls().iter().filter(|(i, _)| i.is_dimmed()).count();
    assert_eq!(
        dimmed_after, dimmed_before,
        "no new dimmed frames after Error — the worker was stopped (channel-quit barrier, not an epoch check)"
    );
}

// ─── 8. rev-12 (P2) + rev-14: localization ──────────────────────────────────

#[test]
fn tooltip_text_is_localized() {
    assert_eq!(tray_tooltip_text(TrayVisualState::Normal, Locale::En), "LinguaRay");
    assert_eq!(tray_tooltip_text(TrayVisualState::Normal, Locale::Zh), "LinguaRay");
    assert_eq!(
        tray_tooltip_text(TrayVisualState::ActiveTranslation, Locale::En),
        "Translating…"
    );
    assert_eq!(
        tray_tooltip_text(TrayVisualState::ActiveTranslation, Locale::Zh),
        "翻译中…"
    );
    assert_eq!(
        tray_tooltip_text(TrayVisualState::Error, Locale::En),
        "LinguaRay — Error"
    );
    assert_eq!(
        tray_tooltip_text(TrayVisualState::Error, Locale::Zh),
        "LinguaRay — 错误"
    );
}

#[test]
fn detect_system_locale_never_panics() {
    // rev-14: uses sys_locale::get_locale() (cross-platform). The contract is
    // "never panics, always En or Zh".
    let _ = detect_system_locale(); // must not panic
}

// ─── 9. rev-14/rev-15: red-dot pixel-diff (overlay, not solid square) ────────
// Loads the generated PNG via the image crate (dev-dependency). rev-14: PANICS
// if the PNG is missing (does NOT silently skip — a silent skip would let a
// build.rs regression pass unnoticed).

#[test]
fn red_dot_overlay_preserves_base_icon_outside_the_dot() {
    let error_png = concat!(env!("OUT_DIR"), "/tray-error-32.png");
    let base_png = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("icons/32x32.png");
    let error_img = image::open(error_png)
        .unwrap_or_else(|e| panic!("build.rs output not found: {error_png} ({e})"));
    let base_img = image::open(&base_png)
        .unwrap_or_else(|e| panic!("base icon not found: {} ({e})", base_png.display()));
    let error_rgba = error_img.to_rgba8();
    let base_rgba = base_img.to_rgba8();
    let dot_center = (26i32, 6i32);
    let dot_radius = 5i32;
    let mut base_unchanged = 0;
    let mut dot_pixels = 0;
    let mut red_dot_pixels = 0;
    for y in 0..32i32 {
        for x in 0..32i32 {
            let dx = x - dot_center.0;
            let dy = y - dot_center.1;
            let in_dot = dx * dx + dy * dy <= dot_radius * dot_radius;
            let b = base_rgba.get_pixel(x as u32, y as u32);
            let e = error_rgba.get_pixel(x as u32, y as u32);
            if in_dot {
                dot_pixels += 1;
                if e.0 == [220, 38, 38, 255] {
                    red_dot_pixels += 1;
                }
            } else if b == e {
                base_unchanged += 1;
            }
        }
    }
    assert!(
        red_dot_pixels > 0,
        "the dot region must contain #DC2626 pixels (found {red_dot_pixels})"
    );
    assert!(
        base_unchanged >= 32 * 32 - dot_pixels - 4,
        "base icon pixels OUTSIDE the dot must be unchanged"
    );
}

// ─── 10. rev-15 (P1-3) + rev-16 (P1-3 + P2-2) + rev-17 P2-1 + rev-18-1/3: switch ──
// rev-15 P1-3: the switch-provider path must NOT call session.gen.next() — doing
// so would stale any in-flight translation (GenerationToken::next advances the
// current gen). rev-16 P2-2 / rev-17 P2-1 / rev-18-3: this is verified TWO ways —
// (a) FUNCTIONALLY, via the extracted `pub fn handle_switch_provider` (rev-18-1:
// SYNC — the test calls the REAL helper against a REAL temp DB + an inserted
// provider, NOT a mock controller simulation and NOT an async `.await`), and
// (b) structurally, via an `include_str!` grep asserting the switch arm has no
// `.gen.next()` AND (rev-18 P2-4) no `.await` / no `spawn(async move` /
// no `pub async fn handle_switch_provider`. rev-16-3 / rev-17-4: the controller's
// switch-flow methods are `begin_switch()`/`finish_switch(rev, success)` (NO gen
// arg; record_switch_error/clear_switch_error are DELETED — finish_switch is the
// sole switch mutator).
//
// rev-18-3 test fixture: the test calls `handle_switch_provider_core(app_state,
// uuid)` — the SYNC core (rev-18-1) that does DB + tray controller work and does
// NOT touch `AppHandle` (no `refresh_tray_if_available`, no icon/menu). This is
// the load-bearing reason for the core+wrapper split: the integration-test crate
// has NO Tauri test runtime (`tauri::test::mock_app` is unavailable without a
// tauri test feature the current `Cargo.toml` does not enable), so the test calls
// the core directly and needs NO mock AppHandle. The `AppState` is built with a
// REAL temp DB (`tempfile::tempdir()` + `Database::open(&db_path)`) + a REAL
// provider inserted via `db_providers::create(...)` + a `RecordingRenderer`-backed
// `tray` field (mirrors `tests/recovery.rs::Harness::new_ready`). The DB write is
// exercised via the REAL sync `set_active_primary_core` (rev-18-1: NOT `.await`ed
// — the core is SYNC). The SUCCESS path is asserted on the DB row
// (`read_active_selection().primary == Some(uuid)`) AND the tray state
// (`switch_error_rev() == None`, `current_state() == Normal`); the FAILURE path
// (an unknown uuid) asserts the tray goes to Error (`switch_error_rev() ==
// Some(rev)`, `current_state() == Error`) while the DB is unchanged. The
// generation invariance is asserted on BOTH paths. NO mock controller, NO
// AppHandle — the real `TrayStateController` (parking_lot) + the real
// `set_active_primary_core` (gate + tx) are exercised end-to-end through the core.

#[test]
fn switch_handler_does_not_call_gen_next() {
    // rev-18-3 FUNCTIONAL (real DB, NO AppHandle): call the SYNC core
    // `handle_switch_provider_core(app_state, uuid)` (rev-18-1) and assert (1) the
    // translation GenerationToken is NOT advanced, (2) the DB primary_uuid is
    // updated, (3) the tray controller reflects success (Normal). The core is
    // `pub fn handle_switch_provider_core(app_state, uuid)` in lib.rs (Step 10);
    // it is SYNC, does NOT take an AppHandle, and does NOT touch the token
    // (rev-15 P1-3 / rev-16 P1-3).
    //
    // rev-19-2 (fresh_db fixture): `Database::open(path)` (db/mod.rs:93) only opens
    // the file + sets pragmas — it does NOT create tables. Calling
    // `db_providers::create` directly after `open` panics with "no such table:
    // providers". The fixture MUST run `schema::create_all_tables` +
    // `schema::seed_singletons` inside a transaction FIRST (the exact pattern from
    // tests/provider_crud.rs:21-34 `fresh_db()`), THEN `db_providers::create`.
    use linguaray_lib::concurrency::GenerationToken;
    use linguaray_lib::db::Database;
    use linguaray_lib::db::readiness::DataReadiness;
    use linguaray_lib::db::providers as db_providers;
    use linguaray_lib::db::schema;
    use linguaray_lib::tray_state::{Locale, RecordingRenderer, TrayStateController};
    use std::sync::Arc;

    // rev-19-2: fresh_db pattern — open + create_all_tables + seed_singletons, THEN
    // create the provider. Mirrors tests/provider_crud.rs:21-34 verbatim.
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("linguaray.db");
    let db = Database::open(&db_path).expect("Database::open");
    db.with_conn(|conn| {
        let tx = conn.transaction()?;
        schema::create_all_tables(&tx)?;
        schema::seed_singletons(&tx)?;
        tx.commit()?;
        Ok(())
    })
    .expect("create_all_tables + seed_singletons");
    let uuid = {
        // Insert one enabled provider; capture its uuid for the switch.
        // db_providers::create takes `&mut Connection` (db/providers.rs:357), so it
        // is called inside `db.with_conn(|conn| ...)`.
        let profile = db
            .with_conn(|conn| db_providers::create(conn, "custom", "Test Provider", "http://localhost:11434", None))
            .expect("db_providers::create");
        profile.uuid
    };

    // Build AppState with the DB installed + a RecordingRenderer-backed tray
    // (NO real tray registration — the core never refreshes the tray).
    let renderer = Arc::new(RecordingRenderer::default());
    let app_state = Arc::new(linguaray_lib::AppState {
        db: parking_lot::RwLock::new(Some(Arc::new(db))),
        data_gate: parking_lot::RwLock::new(()),
        readiness: parking_lot::RwLock::new(DataReadiness::Ready),
        db_path: db_path.clone(),
        keystore_dir: dir.path().join("keystore"),
        settings_path: Some(dir.path().join("settings.json")),
        tray: Arc::new(parking_lot::Mutex::new(
            TrayStateController::with_renderer(renderer.clone(), Locale::En),
        )),
    });

    // A translation is in flight with gen = g1 (verified against the real token).
    let token = GenerationToken::new();
    let g1 = token.next();
    assert!(token.is_latest(g1));

    // rev-18-1/3: call the SYNC CORE (no AppHandle). NO `.await`.
    let result = linguaray_lib::handle_switch_provider_core(&app_state, &uuid);

    // (1) The in-flight translation's generation is STILL latest — the switch
    // core did NOT advance the token (rev-15 P1-3 / rev-16 P1-3 / rev-18-1).
    assert!(
        token.is_latest(g1),
        "switch-provider must NOT bump the translation GenerationToken (rev-15 P1-3 / rev-16 P1-3 / rev-18-1 SYNC core)"
    );

    // (2) The DB primary_uuid was updated to the switched provider's uuid.
    assert!(result.is_ok(), "switch to an existing provider succeeds: {:?}", result);
    let db_read = app_state.db.read().clone().expect("db slot Some");
    let selection = db_read
        .with_conn(|conn| db_providers::read_active_selection(conn))
        .expect("read_active_selection");
    assert_eq!(
        selection.primary,
        Some(uuid.clone()),
        "the switch core wrote primary_uuid = the switched provider's uuid"
    );

    // (3) The tray controller reflects the success: switch_error_rev cleared,
    // current_state back to Normal, switch_revision bumped (begin → finish).
    {
        let c = app_state.tray.lock();
        assert_eq!(c.switch_error_rev(), None, "a successful switch clears switch_error_rev");
        assert_eq!(c.current_state(), linguaray_lib::tray_state::TrayVisualState::Normal);
        assert!(c.switch_revision() >= 1, "begin_switch bumped switch_revision");
    }
    let _ = g1;

    // ── FAILURE path: switching to an UNKNOWN uuid leaves the DB unchanged AND
    // surfaces the error in the tray (rev-18-3: real DB write fails validation).
    let token2 = GenerationToken::new();
    let g2 = token2.next();
    let fail_result = linguaray_lib::handle_switch_provider_core(&app_state, "nonexistent-uuid");
    assert!(fail_result.is_err(), "switch to an unknown uuid fails");
    assert!(token2.is_latest(g2), "the failed switch also does NOT bump the token");
    let selection_after_fail = db_read
        .with_conn(|conn| db_providers::read_active_selection(conn))
        .expect("read_active_selection after fail");
    assert_eq!(
        selection_after_fail.primary,
        Some(uuid),
        "the failed switch did NOT change the DB primary (transaction rolled back)"
    );
    {
        let c = app_state.tray.lock();
        assert_eq!(
            c.switch_error_rev(),
            Some(c.switch_revision()),
            "a failed switch sets switch_error_rev = the (latest) revision"
        );
        assert_eq!(
            c.current_state(),
            linguaray_lib::tray_state::TrayVisualState::Error,
            "a failed switch drives the tray to Error (red dot)"
        );
    }
}

#[test]
fn switch_arm_source_has_no_gen_next_call() {
    // rev-16 P2-2 + rev-18 P2-4 + rev-20-2 + rev-21-1 + rev-22: STRUCTURAL
    // regression guard. Read lib.rs at compile time and assert the
    // switch-provider flow contains no `.gen.next()` / `session.gen` call AND
    // (rev-18-1) no async pattern (`.await` / `spawn(async move` /
    // `pub async fn handle_switch_provider`). This catches a future regression
    // that re-introduces the rev-14 bug (acquiring session.gen.next() in the
    // switch handler stales in-flight translations) or the rev-17-1 async mistake
    // (`.await`ing a sync fn, or spawning an async task for a sync helper).
    //
    // rev-20-2: assert against the source from the switch arm onward (no
    // `take(4096)` truncation — a window cap could let a grep assertion
    // false-pass/false-fail in a large file whose handler is split across >4KB).
    // rev-21-1: the FAILURE messages print only the first 500 chars of the
    // relevant body (NOT the whole tail of lib.rs); the cap only affects
    // diagnostic output.
    //
    // rev-22-1: the preview is UTF-8-SAFE — `chars().take(500).collect::<String>()`
    // takes the first 500 CHARACTERS (not bytes), so a multi-byte char (e.g. a
    // Chinese comment or a non-ASCII provider name) straddling byte offset 500
    // does NOT panic with "byte index 500 is not a char boundary" (the
    // rev-21-1 `&window[..window.len().min(500)]` byte slice WOULD panic).
    //
    // rev-22-2: the grep window is a PRECISE function body, not `&src[switch_start..]`
    // to EOF. `extract_function_body` walks the source from a signature's opening
    // `{` tracking brace depth until back to 0, returning the exact slice from
    // the signature start to (and including) the matching `}`. This avoids a
    // too-wide window (to EOF) that could false-fail on an unrelated subsequent
    // function that happens to contain `.await` / `spawn(async move` / `.gen.next()`.
    //
    // rev-22-3: assertions are split across THREE function bodies
    // (`handle_tray_menu_event` covers the tray.switch- arm; the SYNC core; the
    // SYNC wrapper) so a regression is pinpointed to the exact function.
    //
    // rev-22-4: `extract_function_body` is a LOCAL helper fn INSIDE this test — it
    // is NOT a `#[test]`, so the test count stays 33 (rev-20/rev-21 enumeration).

    /// rev-22-2: extract a function body by its exact signature prefix. Walks the
    /// source from the signature's opening `{` tracking brace depth until back to
    /// 0; returns the slice from the signature start to (and including) the
    /// matching `}`. Panics if the signature or its `{` are not found, or if the
    /// braces are unbalanced. UTF-8-safe: iterates via `char_indices` and matches
    /// on the ASCII `{` / `}` (never splits a multi-byte char).
    fn extract_function_body<'a>(src: &'a str, signature: &str) -> &'a str {
        let start = src.find(signature)
            .unwrap_or_else(|| panic!("rev-22-2: expected `{signature}` in lib.rs"));
        let brace_offset = src[start..].find('{')
            .unwrap_or_else(|| panic!("rev-22-2: expected `{{` after `{signature}`"));
        let brace_start = start + brace_offset;
        let mut depth = 0i32;
        let mut end = brace_start + 1; // default: include at least the opening brace
        for (i, ch) in src[brace_start..].char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = brace_start + i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }
        assert!(
            depth == 0,
            "rev-22-2: unbalanced braces in the body of `{signature}`"
        );
        &src[start..end]
    }

    let src = include_str!("../src/lib.rs");

    // rev-22-2: three PRECISE function bodies (brace-matched), not a to-EOF slice.
    // `handle_tray_menu_event`'s body covers the `tray.switch-<uuid>` arm +
    // `spawn_blocking` dispatch. `handle_switch_provider` is the SYNC wrapper.
    // `handle_switch_provider_core` is the SYNC core (test entry, no AppHandle).
    let handler_body = extract_function_body(src, "fn handle_tray_menu_event(");
    let core_body = extract_function_body(src, "pub fn handle_switch_provider_core(");
    let wrapper_body = extract_function_body(src, "pub fn handle_switch_provider(");

    // rev-22-1: UTF-8-safe preview (500 CHARS, not bytes) for diagnostics.
    let handler_preview: String = handler_body.chars().take(500).collect();
    let core_preview: String = core_body.chars().take(500).collect();
    let wrapper_preview: String = wrapper_body.chars().take(500).collect();

    // ── core_body: SYNC, decoupled from the translation GenerationToken ──────
    // rev-18-1: set_active_primary_core is SYNC → the core is SYNC (no `.await`).
    // rev-15 P1-3 / rev-16-1: the core must NOT acquire the translation gen.
    assert!(
        !core_body.contains("session.gen")
            && !core_body.contains(".gen.next()")
            && !core_body.contains(".gen .next()"),
        "rev-22-3: handle_switch_provider_core must NOT acquire the translation GenerationToken / call `.gen.next()` (switch is decoupled from translation gen — rev-15 P1-3 / rev-16-1) (first 500 chars of core body: {core_preview})"
    );
    assert!(
        !core_body.contains(".await"),
        "rev-22-3: handle_switch_provider_core must be SYNC (set_active_primary_core is SYNC) — no `.await` in its body (rev-18-1) (first 500 chars of core body: {core_preview})"
    );

    // ── wrapper_body: SYNC `pub fn`, no async dispatch ───────────────────────
    assert!(
        !wrapper_body.contains("pub async fn"),
        "rev-22-3: handle_switch_provider must be `pub fn` (SYNC), not `pub async fn` (rev-18-1) (first 500 chars of wrapper body: {wrapper_preview})"
    );
    assert!(
        !wrapper_body.contains(".await") && !wrapper_body.contains("spawn(async move"),
        "rev-22-3: handle_switch_provider wrapper must NOT `.await` or spawn an async task (rev-18-1 SYNC model) (first 500 chars of wrapper body: {wrapper_preview})"
    );

    // ── handler_body: the tray.switch- arm dispatches via spawn_blocking(SYNC),
    //    not spawn(async move { .await }); and must not touch the translation gen.
    assert!(
        !handler_body.contains(".gen.next()")
            && !handler_body.contains(".gen .next()")
            && !handler_body.contains("session.gen"),
        "rev-22-3: the tray.switch- arm in handle_tray_menu_event must NOT call `.gen.next()` / acquire the translation GenerationToken (rev-16 P1-3 / rev-18-1) (first 500 chars of handler body: {handler_preview})"
    );
    assert!(
        !handler_body.contains("spawn(async move"),
        "rev-22-3: the tray.switch- arm must NOT spawn(async move {{ ... .await }}) — it uses spawn_blocking for a SYNC fn (rev-18-1) (first 500 chars of handler body: {handler_preview})"
    );

    // rev-19 P2-1: the tray.switch-<uuid> submenu is DYNAMIC (one MenuItem per
    // enabled provider, built by `build_switch_provider_submenu` from the db). Guard
    // against a regression that replaces it with a single fixed `MenuItem::with_id(
    // app, "tray.switch-provider", ...)` (which would not carry a uuid). The
    // submenu builder + the `tray.switch-{uuid}` format string MUST both exist.
    assert!(
        src.contains("build_switch_provider_submenu"),
        "rev-19 P2-1: the dynamic Switch Provider submenu builder `build_switch_provider_submenu` must exist in lib.rs"
    );
    assert!(
        src.contains("\"tray.switch-{uuid}\"") || src.contains("tray.switch-{uuid}"),
        "rev-19 P2-1: the submenu must format item ids as `tray.switch-{uuid}` (one per provider)"
    );
}

// ─── 11. rev-16 (P1-3): switch revision ordering (NEW) ───────────────────────
// rev-16-3: concurrent switch completions must be ordered by revision — the
// LATEST revision's result wins; a stale (older) revision's late result is ignored.

#[test]
fn two_concurrent_switches_second_wins() {
    // switch A (rev=1) succeeds; switch B (rev=2) fails. B is the latest, so the
    // final state must be Error (B's failure wins, even though A succeeded later).
    let mut c = test_controller();
    let rev_a = c.begin_switch(); // rev=1
    let rev_b = c.begin_switch(); // rev=2 (latest)
    c.finish_switch(rev_a, true); // A succeeds — but rev_a != switch_revision(2) → ignored
    c.finish_switch(rev_b, false); // B fails — rev_b == switch_revision(2) → switch_error_rev = Some(2)
    assert_eq!(
        c.switch_error_rev(),
        Some(rev_b),
        "the LATEST revision (B, failed) wins — its error is recorded"
    );
    assert_eq!(recompute_pure(&c), TrayVisualState::Error);
    // A's late success did NOT clear the error (rev_a was stale).
}

#[test]
fn stale_switch_result_ignored() {
    // rev=1 switch fails; rev=2 switch succeeds (latest). Then rev=1's late
    // failure result arrives — it must be IGNORED (rev=1 != switch_revision=2),
    // so the final state stays Normal (rev=2 succeeded).
    let mut c = test_controller();
    let rev1 = c.begin_switch(); // rev=1
    let rev2 = c.begin_switch(); // rev=2 (latest)
    c.finish_switch(rev2, true); // latest succeeds → switch_error_rev = None
    assert_eq!(c.switch_error_rev(), None);
    assert_eq!(recompute_pure(&c), TrayVisualState::Normal);
    c.finish_switch(rev1, false); // rev1's LATE failure — stale, ignored
    assert_eq!(
        c.switch_error_rev(),
        None,
        "stale revision's late result is ignored — the latest revision's success stands"
    );
    assert_eq!(recompute_pure(&c), TrayVisualState::Normal);
}
```

> **rev-15/rev-16/rev-17/rev-18/rev-19 test-design notes:** (a) **rev-16 P2-1 / rev-17-2:** the PulseWorker-lifecycle tests construct a controller via `controller_with_notify()` (which calls `with_renderer_interval_and_notify(renderer.clone(), locale, interval, Some(notify_tx))`); the worker emits `notify_tx.send(PulseEvent::Tick)` per tick (rev-17-2: was `send(())`), and the test `recv_timeout` on the receiver to deterministically wait for N frames — **NO `thread::sleep`** (CI-flake-free). The worker-stop tests (`last_finish_stops_the_worker`, `leaving_active_stops_the_worker_no_stale_frames`) assert `Ok(PulseEvent::Stopped)` on the notify channel (rev-17-2: the worker emits Stopped before its thread exits — was the `Disconnected` side-effect). The `controller_with_notify()` helper returns `(controller, Arc<RecordingRenderer>, Receiver<PulseEvent>)` (rev-17-2: `PulseEvent`, was `()`) — the test reads `renderer.calls()` on the SAME `Arc` it passed to the constructor (no downcast, no `renderer_snapshot()` accessor needed). (b) All reducer/guard/generation tests are plain `#[test]` with SYNC calls (`c.begin_translation(1);`, `_guard.mark_success();`, `c.finish_translation(1, true);`) — no `#[tokio::test]`, no `.await`. (c) The `leaving_active_stops_the_worker_no_stale_frames` test (renamed from rev-14's `stale_epoch_tick_does_not_clobber_error`) verifies the rev-15 worker-stop barrier: after `record_translation_error(1)` (rev-16-1 renamed) switches to `Error` + drops the worker (channel-quit: `send` + `join`), the notify channel returns `Ok(PulseEvent::Stopped)` (rev-17-2 — no `thread::sleep`). There is NO epoch check (rev-15 P1-4 removed `visual_epoch`) — the channel-quit is the barrier. (d) The pixel-diff test `panic!`s if the generated PNG is missing (rev-14 P2). (e) **rev-15 P1-2:** `RecordingRenderer` is `#[cfg(any(test, feature = "xproc-test-helper"))]`-gated in the module (NOT `#[cfg(test)]`); the integration test crate is compiled under `--features xproc-test-helper` (every verification command in this plan carries it), so the type resolves; `cargo build` (no feature) does NOT compile the mock. (f) **rev-17 P2-4 / rev-18-5:** the two `PulseWorker` channel-quit tests (`stop_signal_joins_the_worker`, `drop_stops_the_worker`) use NO `thread::sleep` — they pass a `notify` channel, call `PulseWorker::stop()` / drop, and `match` the `recv_timeout` result against `Ok(PulseEvent::Tick)` (confirming the worker ran) / `Ok(PulseEvent::Stopped)` (confirming the worker exited) — rev-18-5: explicit `match` (not `let _ =`); `drop_stops_the_worker` asserts `Stopped` (NOT `Disconnected` — the explicit signal is deterministic, the Disconnected side-effect is not). (g) **rev-16 P2-2 / rev-18-3:** `switch_handler_does_not_call_gen_next` (rev-18-3: `#[test]` — NOT `#[tokio::test]`; calls the REAL SYNC core `handle_switch_provider_core(&app_state, &uuid)` — NO AppHandle — against a REAL temp DB + an inserted provider — rev-17 P2-1 `.await`ed an async helper, rev-16 simulated the controller only) verifies rev-15 P1-3 / rev-16 P1-3 / rev-18-1 functionally (the core never touches a `GenerationToken`) AND asserts the DB `primary_uuid` was updated + the tray controller reflects success/failure, AND `switch_arm_source_has_no_gen_next_call` is a STRUCTURAL `include_str!` grep that asserts the switch handler source contains no `.gen.next()` / `session.gen` / `.await` / `spawn(async move` / `pub async fn handle_switch_provider` (rev-18 P2-4: the SYNC model is enforced structurally). (h) **rev-16-2 / rev-17-3 gen guards:** `older_success_does_not_clear_newer_error` + `older_error_does_not_replace_newer_error` (rev-16-2) + `stale_gen_error_ignored_after_newer_begin` (rev-17-3 NEW) verify the `finish_translation`/`record_translation_error` gen guards + the `latest_translation_gen` guard. (i) **rev-16-3 switch revision:** `two_concurrent_switches_second_wins` + `stale_switch_result_ignored` verify stale switch results are ignored and the latest revision wins. (j) **rev-16 P2-3:** the test imports do NOT name `RenderedIcon` or `TrayRenderer` directly — they are reached via `RecordingRenderer::calls()` element methods (`.is_dimmed()`, `.is_normal()`, `.is_error_dot()`), so there is no `unused_imports` clippy warning. (k) **rev-16 P2-5:** the file header says "6 tests" for the reducer concurrency section (rev-15 said "5" — undercount). (l) **rev-17-4:** the switch-flow tests use `finish_switch(rev, false)`/`finish_switch(rev, true)` — NOT the deleted `record_switch_error()`/`clear_switch_error()`. (m) **rev-18-1/3/5:** the functional switch test is SYNC (`#[test]`, no `.await`) and calls the SYNC core `handle_switch_provider_core(&app_state, &uuid)` (NO AppHandle — `tauri::test::mock_app` is unavailable without a tauri test feature the current `Cargo.toml` does not enable) against a REAL temp DB + inserted provider; all `recv_timeout` use explicit `match` (not `let _ =`); `drop_stops_the_worker` asserts `Stopped` (not `Disconnected`). (n) **rev-19-1/2/3/4:** `controller_with_notify()` passes `Some(notify_tx)` (not a bare `Sender`) — the 4th param of `with_renderer_interval_and_notify` is `Option<mpsc::Sender<PulseEvent>>`; the functional switch test fixture uses the `fresh_db` pattern (`Database::open` + `schema::create_all_tables` + `schema::seed_singletons` inside a transaction FIRST, THEN `db_providers::create`) — without this the test panics "no such table: providers" (`Database::open` does not create tables, db/mod.rs:93); the `PulseWorker` struct has NO `notify` field (the Sender is moved into the thread closure — rev-19-3, no `dead_code` warning); the `second_begin_does_not_churn_the_worker` test asserts `worker_start_count` stays at 1 across the second `begin_translation` (rev-19-4 — deterministic, replaces rev-18-5's timing-sensitive frame-count comparison). **Test count = 33** (rev-19: unchanged from rev-18's 33 — rev-19 rewrites the fixture + the no-churn assertion in-place, no test added/removed; rev-18: unchanged from rev-17's 33 — the functional switch test is REWRITTEN in-place from `#[tokio::test]`/async to `#[test]`/SYNC with a real DB, no count delta; verified by grepping `^#[test]$` + `^#[tokio::test]` in the Step 2 code block — after rev-18 ALL are `#[test]`, 0 `#[tokio::test]`). (o) **rev-22-1/2/3/4:** the `switch_arm_source_has_no_gen_next_call` STRUCTURAL grep test is REWRITTEN in place (no `#[test]` count delta): (1) the preview is `chars().take(500).collect::<String>()` (UTF-8-safe, 500 CHARS not bytes — no "byte index is not a char boundary" panic); (2) the grep window is narrowed to brace-matched `extract_function_body(src, signature)` of the THREE switch functions (`handle_tray_menu_event` / `handle_switch_provider_core` / `handle_switch_provider`) instead of `&src[switch_start..]` to EOF (avoids false-failing on an unrelated subsequent function); (3) each of `core_body` / `wrapper_body` / `handler_body` is asserted INDEPENDENTLY for `.await` / `pub async fn` / `spawn(async move` / `.gen.next()` / `session.gen` (a regression is pinpointed to the exact function); (4) `extract_function_body` is a LOCAL helper `fn` INSIDE the test (NOT a `#[test]`), so the test count stays 33.

- [x] **Step 3: Run test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --features xproc-test-helper --test tray_state`
Expected: FAIL — `cannot find module \`tray_state\` in crate \`linguaray_lib\`` (or `unresolved import` for `TrayStateController`/`recompute_pure`/`Locale`/`tray_tooltip_text`/`detect_system_locale`/`RecordingRenderer`/`TranslationGuard`/`with_renderer_and_interval`). The rev-11 priority tests would have failed on the private-module path too; rev-12 (P1-4) fixes that with `pub mod tray_state;` in Step 6. The rev-13/rev-14 tests additionally fail on the missing `RecordingRenderer`/`TranslationGuard`/`detect_system_locale`/`with_renderer_and_interval` symbols until Step 5 lands them.
- [x] **Step 4: Generate the red-dot OVERLAY PNG + dimmed pulse PNG in build.rs (rev-12 P1-1 + P1-2)**

Append this block to `src-tauri/build.rs` (AFTER the existing `tauri_build::try_build` call so a build-failure short-circuits before the icons are regenerated; the block is gated to always run because both PNGs are needed for the `include_bytes!` in Step 5).

**rev-12 P1-2 (overlay, not solid square):** the red-dot icon is a COMPOSITE — load the existing app default icon `src-tauri/icons/32x32.png` (verified: 974 bytes, ships in the repo) and draw a ~10px-diameter `#DC2626` = `[220, 38, 38, 255]` dot at the top-right. The dot is drawn with a manual circle test `dx*dx + dy*dy <= r*r` (no extra drawing crate). The frozen danger color is `#DC2626` — NOT rev-11's `#E5484D`/`(229,72,77)` (user-specified hard constraint).

**rev-12 P1-1 (dimmed pulse variant):** `tray-active-32.png` is the same base icon with every pixel's RGB scaled to ~60% brightness (alpha unchanged) — this is the "dimmed" frame the pulse timer swaps in every 800ms.

```rust
// ─── rev-12 / Task A5: generate the tray PNGs (build-time) ───────────────────
// TWO icons are written to OUT_DIR so the runtime embeds them via
// include_bytes!(concat!(env!("OUT_DIR"), "/tray-error-32.png")) and
// ".../tray-active-32.png"). Both are PROGRAMMATIC COMPOSITES over the repo's
// existing app default icon src-tauri/icons/32x32.png (NOT new design assets).
fn build_tray_icons(out_dir: &std::path::Path) {
    use image::{ImageBuffer, Rgba};
    const SIZE: u32 = 32;

    // Load the repo's existing app default icon as the base for BOTH variants.
    // CARGO_MANIFEST_DIR = src-tauri/ (build scripts run with this set).
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR");
    let base_path = std::path::Path::new(&manifest_dir)
        .join("icons")
        .join("32x32.png");
    let base = image::open(&base_path)
        .expect("open src-tauri/icons/32x32.png (base icon for tray composites)")
        .to_rgba8();

    // ── tray-error-32.png: red-dot OVERLAY on the base (rev-12 P1-2) ─────────
    // Draw a ~10px-diameter dot at the top-right. Center ~(26, 6), radius 5.
    // Color #DC2626 = [220, 38, 38, 255] — frozen danger color (user-specified).
    let mut error_img = base.clone();
    let dot_center: (i32, i32) = (26, 6);
    let dot_radius: i32 = 5;
    let dot_color: [u8; 4] = [220, 38, 38, 255]; // #DC2626
    for y in 0..SIZE as i32 {
        for x in 0..SIZE as i32 {
            let dx = x - dot_center.0;
            let dy = y - dot_center.1;
            if dx * dx + dy * dy <= dot_radius * dot_radius {
                // In-bounds AND inside the dot circle → overwrite with dot color.
                error_img.put_pixel(x as u32, y as u32, Rgba(dot_color));
            }
        }
    }
    let error_path = out_dir.join("tray-error-32.png");
    image::DynamicImage::ImageRgba8(error_img)
        .save(&error_path)
        .expect("write tray-error-32.png to OUT_DIR");

    // ── tray-active-32.png: dimmed variant for the pulse (rev-12 P1-1) ───────
    // Each pixel's RGB scaled to ~60% brightness; alpha unchanged. This is the
    // "dimmed" frame the pulse timer swaps in every 800ms (the visible pulse).
    let mut active_img: ImageBuffer<Rgba<u8>, Vec<u8>> = base.clone();
    for px in active_img.pixels_mut() {
        let channels = px.0;
        // Scale RGB to 60%; keep alpha. Integer math: (c * 60) / 100 = c*3/5.
        px.0 = [
            (channels[0] as u16 * 60 / 100) as u8,
            (channels[1] as u16 * 60 / 100) as u8,
            (channels[2] as u16 * 60 / 100) as u8,
            channels[3], // alpha unchanged
        ];
    }
    let active_path = out_dir.join("tray-active-32.png");
    image::DynamicImage::ImageRgba8(active_img)
        .save(&active_path)
        .expect("write tray-active-32.png to OUT_DIR");

    // Re-run if the base icon or this script changes.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=icons/32x32.png");
}

// Invoke it inside `main()` of build.rs, AFTER `tauri_build::try_build(...)`:
//   let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
//   build_tray_icons(&out_dir);
```

> **rev-13 (imageops import):** the `use` list is `image::{ImageBuffer, Rgba}` ONLY — rev-12's note flagged that `imageops` was unused (the dot is drawn with the manual `put_pixel` loop; `image` 0.25's drawing helpers are behind a non-default `drawing` feature). rev-13 drops `imageops` from the import as the spec, so clippy does not flag an unused import.

The `build.rs` `main()` function's final form (AFTER the existing `tauri_build::try_build(...)?.try_build()` body) calls this helper. Concretely, append these two lines to the END of `build.rs`'s `main()`:

```rust
    // rev-12 / Task A5: write the tray red-dot-overlay + dimmed-pulse PNGs to OUT_DIR.
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    build_tray_icons(&out_dir);
```

Run `cargo build --manifest-path src-tauri/Cargo.toml --features xproc-test-helper` and confirm BOTH `target/.../build/<pkg>-<hash>/out/tray-error-32.png` AND `.../out/tray-active-32.png` exist (the `OUT_DIR` path). This step does NOT need the test to pass yet — it only needs the PNGs to be generated for Step 5's `include_bytes!`.

**Verification (P1-2) — rev-14:** the red-dot-overlay-vs-solid-square guarantee is asserted PROGRAMMATICALLY in Step 2's `red_dot_overlay_preserves_base_icon_outside_the_dot` test (loads the generated `tray-error-32.png` via the `image` dev-dependency, compares pixel-by-pixel against the base `icons/32x32.png`: asserts every pixel OUTSIDE the dot circle is unchanged AND that the dot circle contains `#DC2626` pixels). No manual "open the PNG and eyeball" step — rev-12's note is superseded by this assertion. **rev-14 P2: the test `panic!`s if the PNG is not present** (`unwrap_or_else(|e| panic!("build.rs output not found: {error_png} ({e})"))`) — it does NOT silently `return` (a silent skip would let a build.rs regression pass unnoticed). When this Step 4 build script has produced the file (Step 4's `cargo build` runs first), the test asserts the overlay.

- [x] **Step 5: Create the tray_state module (rev-19: PulseWorker struct drops `notify` field + `worker_start_count` field on controller; rev-18: sync handle_switch_provider doc note; rev-17: PulseEvent enum + latest_translation_gen + delete record_switch_error/clear_switch_error/clear_translation_error; rev-16: NO overloading + gen guards + switch revision + notify channel; rev-15: `PulseWorker` channel-quit + `finish_translation` merge + no `visual_epoch`/`tick_render`/`stop_timer`; rev-14 sync `parking_lot::Mutex` controller + `current_state`-gated worker swap + `std::thread` timer; rev-13 RAII guard + generation-aware error + injectable TrayRenderer; rev-12 reducer + real icon pulse + base-icon overlay)**

Create `src-tauri/src/tray_state.rs`. **rev-15/rev-16/rev-17 restructure over rev-14:** the module now exposes (a) the `TrayVisualState` enum + `tray_state_priority` (unchanged); (b) a `Locale` enum + `tray_tooltip_text` + `detect_system_locale` (rev-14: `sys_locale::get_locale()`, NOT `std::env::var("LANG")`); (c) `trait TrayRenderer` (rev-14: discrete `set_icon_normal`/`set_icon_dimmed`/`set_icon_error_dot`/`set_tooltip` methods — NOT `set_icon(Option<Image>)`) + `TrayIconRenderer` (prod) + `RecordingRenderer` (test mock, **rev-15 P1-2: `#[cfg(any(test, feature = "xproc-test-helper"))]`-gated** — NOT `#[cfg(test)]`); (d) `TrayStateController` (**rev-17-3 adds `latest_translation_gen: u64`**; **rev-19-4 adds `worker_start_count: u32`** (monotonic counter incremented each time `recompute` enters the `new_state == ActiveTranslation` branch + starts a `PulseWorker`; the no-churn test asserts it does NOT increase on an Active→Active counter bump; initial value 0; `pub fn worker_start_count(&self) -> u32` accessor); **rev-16 fields: `active_translations: u32` + `error_gen: Option<u64>` (translation flow) + `switch_revision: u64` + `switch_error_rev: Option<u64>` (switch flow, rev-16-3 — REPLACES rev-15's `has_error: bool`) + `current_state: TrayVisualState` + `pulse_worker: Option<PulseWorker>` + `tick_interval: Duration` + `renderer: Arc<dyn TrayRenderer>` + `notify_tx: Option<mpsc::Sender<PulseEvent>>` (rev-17-2: `PulseEvent` enum, was `Sender<()>`) + `locale: Locale` — does NOT derive `Debug`; rev-15 REMOVES `visual_epoch`, `pulse_frame`, `pulse_timer`; rev-16-3 REMOVES `has_error`**); (e) `PulseEvent` enum (rev-17-2: `{ Tick, Stopped }`) + `PulseWorker` (**rev-19-3: `pub struct PulseWorker { stop_tx, handle }`** — the struct does NOT hold a `notify` field; the `notify` Sender passed to `start` is MOVED into the worker thread closure which owns + drops it; rev-15 P1-1 + rev-16 P2-1 + rev-17-2: with `mpsc::channel()` + `recv_timeout` loop + per-tick `notify.send(PulseEvent::Tick)` + on-exit `notify.send(PulseEvent::Stopped)` + `stop()` send+join + `Drop`); (f) `TranslationGuard` RAII guard (rev-14/rev-15/rev-16: SYNCHRONOUS `Drop` via `parking_lot::Mutex`; rev-16-2 gen guard; `Drop` calls `finish_translation(gen, succeeded)`); (g) the controller's `render(&mut self)` method (single sync entry point for state-transition writes; the `PulseWorker` writes directly through its own renderer clone on each tick — NO `tick_render()`, NO `stop_timer()`, NO `visual_epoch`). **rev-17-4 methods (NO overloading + NO dead code):** `begin_translation(gen)`/`finish_translation(gen, success)` (gen-guarded, rev-16-2)/`record_translation_error(gen)` (rev-17-3: gen-guarded set gated by `latest_translation_gen`) (translation) + `begin_switch()`/`finish_switch(rev, success)` (switch, rev-16-3 — **rev-17-4: record_switch_error/clear_switch_error DELETED**, finish_switch is the sole switch mutator).

```rust
//! Task A5 (rev-11 → rev-16): the pure-Rust tray visual-state controller.
//!
//! This module drives the Surface 04 (pages/04-tray-menu.md) Normal / Active /
//! Error icon + tooltip states WITHOUT routing through the Web frontend. The
//! translate / clipboard flows call the [`TranslationGuard`] (rev-13 P1-2 /
//! rev-15 finish_translation merge / rev-16-2 gen guard) / [`TrayStateController`]
//! reducer (rev-12 P1-3), which owns the active-translation counter, the
//! generation-tagged error (translation flow), the switch-flow `switch_revision`
//! + `switch_error_rev` (rev-16-3 — replaces rev-15's sticky `has_error` bool so
//! concurrent switch completions are ordered by revision), the current visual
//! state, and the `PulseWorker`, and resolves the highest-priority state via
//! `recompute`. The switch-provider flow calls `begin_switch()` →
//! `finish_switch(rev, success)` (rev-16-1 distinct method names — NO overloading;
//! rev-16-3 revision-tagged) directly — it does NOT touch the translation
//! `GenerationToken` (rev-15 P1-3). The Update-available state is deferred to
//! R5/R6 per user-approved scope decision — the [`TrayVisualState::UpdateAvailable`]
//! variant is retained so the priority ordering is unit-testable, but `recompute`
//! NEVER produces it.
//!
//! rev-12 corrections over rev-11:
//! - P1-1: ActiveTranslation drives a REAL icon frame-switch pulse.
//! - P1-2: Error overlays a red-dot on the BASE icon (composited in build.rs).
//! - P1-3: TrayStateController reducer replaces the direct-override.
//! - P2:   tooltip text is localized via tray_tooltip_text(state, locale).
//!
//! rev-13 corrections over rev-12:
//! - P1-1: the `tray` field lives on `AppState`; all call sites use `app_state.tray`.
//! - P1-2: `TranslationGuard` RAII guarantees begin/end pairing on every return.
//! - P1-3: `error_gen: Option<u64>` is generation-aware (a newer gen's Retry
//!   success clears an older gen's red dot).
//! - P1-4: `visual_epoch` serializes the timer (a stale-epoch tick self-rejects).
//! - P1-5: `trait TrayRenderer` is injectable; `RecordingRenderer` is the test mock.
//!
//! rev-14 corrections over rev-13:
//! - P1-1: SYNCHRONOUS `parking_lot::Mutex` (NOT `tokio::sync::Mutex`). All
//!   controller methods are SYNC (no `async`, no `.await`). `TranslationGuard::drop`
//!   runs `finish_translation` SYNCHRONOUSLY on the calling thread (no detached
//!   spawn) — the RAII guarantee is REAL.
//! - P1-2: `recompute` only swaps the timer/worker when `new_state != current_state`
//!   (Active → Active counter bump does NOT restart the pulse).
//! - P2: `detect_system_locale()` uses `sys_locale::get_locale()` (cross-platform,
//!   NOT `std::env::var("LANG")` which is Unix-only). `TrayStateController` does
//!   NOT derive `Debug`.
//!
//! rev-15 corrections over rev-14 (the load-bearing ones):
//! - P1-1: `PulseWorker` channel-quit — replaces rev-14's infinite `loop { sleep;
//!   render }` + `stop_timer()` `join()` deadlock. The worker's body loops on
//!   `stop_rx.recv_timeout(interval)`; `Ok(())`/`Err(Disconnected)` → return,
//!   `Err(Timeout)` → toggle a frame. `PulseWorker::stop()` = `stop_tx.send(())`
//!   + `handle.take().join()` (the worker returns from `recv_timeout` on the
//!   signal so `join` completes — NO deadlock). `impl Drop for PulseWorker` calls
//!   `stop()`. Leaving `Active` = `pulse_worker.take()` (Drop → stop).
//! - P1-2: `RecordingRenderer` is `#[cfg(any(test, feature = "xproc-test-helper"))]`-gated
//!   (NOT `#[cfg(test)]`, which is invisible to the integration-test crate). The
//!   `lib.rs` re-export is gated identically.
//! - P1-3: Switch Provider does NOT call `session.gen.next()` — `GenerationToken::next()`
//!   ADVANCES the generation (verified concurrency.rs), staling any in-flight
//!   translation. The switch flow is DECOUPLED from the translation generation.
//! - P1-4: SINGLE timer model — `PulseWorker` (channel-quit) only. rev-14's
//!   `visual_epoch` field, `tick_render()` method, `stop_timer()` method, and
//!   "RenderGate" narration are DELETED. The worker holds an independent
//!   `Arc<dyn TrayRenderer>`; the stop barrier is the channel-quit (`send` + `join`).
//! - Housekeeping: `finish_translation(gen, success)` merges `end_translation` +
//!   (if `success`) clear-error + `recompute` into ONE method; `TranslationGuard::drop`
//!   calls it once.
//!
//! rev-16 corrections over rev-15 (the load-bearing ones):
//! - P1-1 (NO function overloading): rev-15 defined TWO methods named `record_error`
//!   (`record_error(&mut self, gen: u64)` for translation + `record_error(&mut self)`
//!   for switch). Rust does NOT support function overloading — this fails to
//!   compile (`E0592: duplicate definitions`). rev-16-1 renames them to DISTINCT
//!   names: `record_translation_error(gen)` (translation) + `begin_switch()` /
//!   `finish_switch(rev, success)` (switch, revision-tagged — replaces the no-gen
//!   `record_error()`/`clear_error()` overloads).
//! - P1-2 (gen guards): rev-15's `finish_translation(gen, true)` unconditionally
//!   cleared `error_gen` — a stale OLDER gen's late success would clear a NEWER
//!   gen's error. rev-16-2 adds `if self.error_gen.is_some_and(|eg| eg <= gen)` to
//!   `finish_translation` AND `if self.error_gen.is_none_or(|eg| gen >= eg)` to
//!   `record_translation_error` (a stale OLDER gen's late error cannot clobber a
//!   NEWER gen's error).
//! - P1-3 (switch revision, replaces rev-15's sticky `has_error: bool`): rev-15's
//!   sticky bool had no revision, so two concurrent switch completions that
//!   re-order would show the wrong final state. rev-16-3 replaces `has_error:
//!   bool` with `switch_revision: u64` (monotonic, incremented by `begin_switch()`)
//!   + `switch_error_rev: Option<u64>`. `begin_switch() -> u64` returns the new
//!   revision; `finish_switch(rev, success)` IGNORES the result if
//!   `rev != switch_revision` (only the latest revision can update state).
//!   `recompute_pure` ORs: `Error iff error_gen.is_some() || switch_error_rev.is_some()`.
//! - P2-1 (notify channel, replaces `thread::sleep` in tests): `PulseWorker::start`
//!   takes an `Option<Sender<()>> notify`; the worker emits `notify.send(())` per
//!   tick; tests `recv_timeout` on it to deterministically wait for N frames.
//!   The `PulseWorker::start` signature is `(renderer, interval, notify)`.
//! - P2-3: the test imports do NOT name `RenderedIcon`/`TrayRenderer` directly
//!   (unused-import clean).
//!
//! rev-17 corrections over rev-16 (4 P1 + 4 P2 — fixing the user audit notes):
//! - P1-1 (async handle_switch_provider — SUPERSEDED by rev-18-1): rev-16's
//!   `handle_switch_provider` was a synchronous `pub fn` but its body `await`s a
//!   `spawn_blocking` — a sync fn cannot `.await`. rev-17-1 made it `pub async fn
//!   handle_switch_provider(app, app_state, uuid)` and the tray.switch arm spawned
//!   it via `tauri::async_runtime::spawn`. **rev-18-1 SUPERSEDED this**: the async
//!   model was based on the wrong premise that `set_active_primary_core` was async
//!   (it is SYNC). The ACTIVE model is now SYNC core `handle_switch_provider_core`
//!   + SYNC wrapper `handle_switch_provider`, offloaded via `spawn_blocking` (NOT
//!   `spawn(async move)`). rev-17-1's async form is retained here ONLY as history.
//! - P1-2 (PulseEvent enum): rev-16's `notify: Option<Sender<()>>` only sent an
//!   empty signal — tests could not distinguish a Tick from a worker Stopped.
//!   rev-17-2 introduces `pub enum PulseEvent { Tick, Stopped }`; the worker sends
//!   `PulseEvent::Tick` after each frame and `PulseEvent::Stopped` before exiting.
//! - P1-3 (latest_translation_gen guard): rev-16's `record_translation_error(gen)`
//!   guard was only `gen >= error_gen` — a stale OLDER gen's late error (after a
//!   newer gen already began) could still set `error_gen`. rev-17-3 adds a
//!   `latest_translation_gen: u64` field; `record_translation_error` only records
//!   when `gen >= latest_translation_gen` (a stale gen's late error is ignored).
//! - P1-4 (delete record_switch_error/clear_switch_error): rev-16 kept both the
//!   low-level `record_switch_error()`/`clear_switch_error()` AND the
//!   revision-protected `finish_switch(rev, success)` — the former are dead code
//!   (finish_switch fully replaces them). rev-17-4 deletes them.
//! - P2-1 (functional switch test): the switch-no-bump test now calls the real
//!   `handle_switch_provider` (rev-17-1 async), not a manual controller
//!   simulation.
//! - P2-2 (Step 11 test count 31→33): rev-16's Step 11 text said "31 tests" but
//!   the actual rev-16 enumeration was 32; rev-17 adds 1 `latest_translation_gen`
//!   test (rev-17-3), bringing the authoritative count to 33.
//! - P2-3 (delete clear_translation_error): rev-16-1 listed
//!   `clear_translation_error(gen)` but it is never called — `finish_translation`
//!   already merges the clear. rev-17 deletes it from the method list.
//! - P2-4 (PulseWorker send+join deterministic test): the channel-quit tests now
//!   assert `PulseEvent::Stopped` is received on the notify channel before join
//!   (deterministic, not "test completes = pass").
//!
//! rev-18 corrections over rev-17 (6 P1 + P2 — fixing the second-round audit notes;
//! the architecture direction of rev-17 is retained — PulseEvent, switch_revision,
//! gen guards, latest_translation_gen, delete dead switch mutators all stay):
//! - P1-1 (sync handle_switch_provider): rev-17-1 made `handle_switch_provider`
//!   `pub async fn` and `.await`ed `set_active_primary_core(...)` — but
//!   `set_active_primary_core` is itself a SYNC fn (A4 Step 9: its body is the
//!   `spawn_blocking` payload — `db_set_active_primary` gate + tx, NOT async).
//!   A sync fn cannot be `.await`ed; rev-17-1's premise was wrong. rev-18-1 reverts
//!   to `pub fn handle_switch_provider(app: &tauri::AppHandle, app_state: &Arc<AppState>, uuid: &str) -> Result<(), String>`
//!   (SYNC, `&` borrows — rev-16's form). The body calls `set_active_primary_core`
//!   directly (no `.await`, no nested `spawn_blocking`). The caller offloads THIS
//!   sync fn via `tauri::async_runtime::spawn_blocking` (NOT `spawn(async move { ... .await })`).
//! - P1-2 (controller_with_notify initializes notify_tx): confirmed — all three
//!   delegating constructors (`new`, `with_renderer`, `with_renderer_and_interval`)
//!   pass `None` for `notify_tx` to `with_renderer_interval_and_notify`, which
//!   initializes the field. No missing-field compile error.
//! - P1-3 (real-DB functional switch test): the `switch_handler_does_not_call_gen_next`
//!   test now uses a real temp DB + an inserted provider (mirroring `tests/recovery.rs::Harness`)
//!   and asserts (a) `db_providers::read_active_selection` shows `primary == Some(uuid)`,
//!   (b) the tray controller's `switch_error_rev() == None` + `current_state() == Normal`
//!   on success, (c) `token.is_latest(g1)` unchanged, (d) no mock controller.
//! - P1-4 (notify_for_thread is used): confirmed — `notify_for_thread` is `move`d
//!   into the worker closure and `.as_ref()`'d each tick (Tick send) + on exit
//!   (Stopped send); even when `None` in prod, the variable is read in the closure
//!   body, so clippy does not flag dead_code.
//! - P1-5 (deterministic PulseWorker tests): the 3 channel-quit / lifecycle tests
//!   now `match` the `recv_timeout` result against `PulseEvent::Tick`/`Stopped`
//!   explicitly (not `let _ =`), and `drop_stops_the_worker` asserts `Stopped`
//!   (not `Disconnected`).
//! - P1-6 (record_switch_error/clear_switch_error = 0 in active code): confirmed —
//!   the only references are in the historical changelog (rev-16/rev-17) where they
//!   are described as "deleted"; the module, Step 10, the tests, and the interface
//!   table use `finish_switch` exclusively.
//! - P2: test count stays 33 (verified by grepping `^#[test]$` + `^#[tokio::test]`
//!   in the Step 2 code block); the structural test also forbids `.await` /
//!   `spawn(async move` in the switch arm; all `recv_timeout` use `match` (50ms).

use std::sync::Arc;

/// Tray visual state priority: `Error > Update > Active > Normal`.
///
/// The variant order is NOT the priority order — the priority is encoded by
/// [`tray_state_priority`], which makes the order explicit and keeps this enum
/// field-free.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TrayVisualState {
    Normal,
    /// Pulse — shown during an in-flight translate. rev-12 (P1-1): a REAL icon
    /// frame-switch pulse — rev-15 (P1-1): the controller's `pulse_worker`
    /// (`Option<PulseWorker>`) starts a background `std::thread` whose body loops
    /// on `mpsc::Receiver::recv_timeout(interval)`, toggling `set_icon_normal` ↔
    /// `set_icon_dimmed` on each `Timeout` (the dimmed variant is the
    /// build-time-generated `tray-active-32.png`). The worker exits via the
    /// channel signal (`stop_tx.send(())` + `join()` — NO infinite-loop + join
    /// deadlock). The localized tooltip ("Translating…"/"翻译中…") is an
    /// auxiliary signal.
    ActiveTranslation,
    /// Red-dot overlay on the tray icon. rev-12 (P1-2): a build-time-composited
    /// PNG — the app default icon (`src-tauri/icons/32x32.png`) with a ~10px
    /// `#DC2626` dot drawn at the top-right. Embedded via
    /// `include_bytes!(concat!(env!("OUT_DIR"), "/tray-error-32.png"))`.
    Error,
    /// Update-available badge — RETAINED so the priority ordering is testable,
    /// but `recompute` NEVER produces it this stage. Deferred to R5/R6 per
    /// user-approved scope decision (the updater backend does not exist).
    UpdateAvailable,
}

/// Priority rank: higher beats lower. `Normal`=0 < `ActiveTranslation`=1 <
/// `UpdateAvailable`=2 < `Error`=3, matching `Error > Update > Active > Normal`.
pub fn tray_state_priority(state: TrayVisualState) -> u8 {
    match state {
        TrayVisualState::Normal => 0,
        TrayVisualState::ActiveTranslation => 1,
        TrayVisualState::UpdateAvailable => 2,
        TrayVisualState::Error => 3,
    }
}

// ─── rev-14: localization (system locale via sys-locale, NOT Settings) ───────

/// UI locale for tray tooltip text. rev-14: read via [`detect_system_locale`]
/// using the `sys-locale` crate (cross-platform) — NOT from `Settings` (which
/// has no `locale` field) and NOT from `std::env::var("LANG")` (Unix-only).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Locale {
    En,
    Zh,
}

/// rev-14: read the system locale via `sys_locale::get_locale()` (macOS
/// CFLocaleCopyCurrent, Windows GetUserDefaultLocaleName, Unix LANG/LC_*).
/// Returns [`Locale::Zh`] if the value starts with `"zh"`, otherwise
/// [`Locale::En`] (including when the detector returns `None`). Does NOT touch
/// `Settings`. Never panics.
pub fn detect_system_locale() -> Locale {
    match sys_locale::get_locale() {
        Some(v) if v.starts_with("zh") => Locale::Zh,
        _ => Locale::En,
    }
}

/// Localized tooltip text for a tray visual state. `Normal` is `"LinguaRay"` in
/// both locales; `ActiveTranslation`/`Error` are translated.
pub fn tray_tooltip_text(state: TrayVisualState, locale: Locale) -> &'static str {
    match (state, locale) {
        (TrayVisualState::Normal, _) => "LinguaRay",
        (TrayVisualState::ActiveTranslation, Locale::En) => "Translating…",
        (TrayVisualState::ActiveTranslation, Locale::Zh) => "翻译中…",
        (TrayVisualState::Error, Locale::En) => "LinguaRay — Error",
        (TrayVisualState::Error, Locale::Zh) => "LinguaRay — 错误",
        // recompute never produces UpdateAvailable this stage; return a stable
        // placeholder so the match is exhaustive without driving a real tooltip.
        (TrayVisualState::UpdateAvailable, _) => "LinguaRay",
    }
}

// ─── rev-13/rev-14 (P1-5): injectable renderer (rev-14: discrete methods) ────

/// The tray rendering surface, abstracted so the controller is testable WITHOUT
/// a real Tauri tray (rev-13 P1-5). Prod: [`TrayIconRenderer`] wraps a
/// `TrayIcon` looked up via `app.tray_by_id("main-tray")`. Test:
/// [`RecordingRenderer`] records every call for assertion (**rev-15 P1-2:
/// `#[cfg(any(test, feature = "xproc-test-helper"))]`-gated** — NOT `#[cfg(test)]`,
/// which is invisible to the integration-test crate).
///
/// rev-14: DISCRETE methods (NOT `set_icon(Option<Image>)` taking an enum) — the
/// renderer DECIDES which embedded PNG / default icon each variant maps to, so
/// the controller never builds an `Image` and the test mock never decodes a PNG.
/// `dyn`-compatible: all methods take `&self` and have no generics.
pub trait TrayRenderer: Send + Sync {
    /// The app default window icon (`app.default_window_icon()`).
    fn set_icon_normal(&self);
    /// The dimmed pulse frame (`tray-active-32.png`).
    fn set_icon_dimmed(&self);
    /// The red-dot error overlay (`tray-error-32.png`).
    fn set_icon_error_dot(&self);
    /// Apply a tooltip.
    fn set_tooltip(&self, text: &str);
}

/// Production renderer: wraps a `tauri::AppHandle`. Looks up the `main-tray` on
/// each call (the tray may be created lazily).
pub struct TrayIconRenderer {
    app: tauri::AppHandle,
}

impl TrayIconRenderer {
    pub fn new(app: tauri::AppHandle) -> Self {
        Self { app }
    }

    /// Helper: look up the main tray and pass it to `f`. Logs + no-ops if absent.
    fn with_tray<F: FnOnce(&tauri::tray::TrayIcon)>(&self, f: F) {
        let Some(tray) = self.app.tray_by_id("main-tray") else {
            log::debug!("TrayIconRenderer: main-tray not present");
            return;
        };
        f(&tray);
    }

    /// Helper: set the icon to the embedded PNG at `bytes`, decoded.
    fn set_icon_bytes(&self, bytes: &'static [u8]) {
        self.with_tray(|tray| match tauri::image::Image::from_bytes(bytes) {
            Ok(img) => {
                if let Err(e) = tray.set_icon(Some(img)) {
                    log::debug!("TrayIconRenderer: set_icon failed: {e}");
                }
            }
            Err(e) => log::debug!("TrayIconRenderer: decode failed: {e}"),
        });
    }
}

impl TrayRenderer for TrayIconRenderer {
    fn set_icon_normal(&self) {
        self.with_tray(|tray| {
            if let Some(icon) = self.app.default_window_icon().cloned() {
                if let Err(e) = tray.set_icon(Some(icon)) {
                    log::debug!("TrayIconRenderer: set_icon(normal) failed: {e}");
                }
            }
        });
    }

    fn set_icon_dimmed(&self) {
        let bytes = include_bytes!(concat!(env!("OUT_DIR"), "/tray-active-32.png"));
        self.set_icon_bytes(bytes);
    }

    fn set_icon_error_dot(&self) {
        let bytes = include_bytes!(concat!(env!("OUT_DIR"), "/tray-error-32.png"));
        self.set_icon_bytes(bytes);
    }

    fn set_tooltip(&self, text: &str) {
        self.with_tray(|tray| {
            if let Err(e) = tray.set_tooltip(Some(text)) {
                log::debug!("TrayIconRenderer: set_tooltip failed: {e}");
            }
        });
    }
}

/// A tagged icon variant the test mock records (rev-14). Prod never builds
/// these — the discrete `TrayRenderer` methods keep the controller free of
/// `Image` construction. The test mock records which method was called.
/// rev-15 P1-2: gated behind `any(test, feature = "xproc-test-helper")` so the
/// integration-test crate (compiled under `--features xproc-test-helper`) sees
/// it and `cargo build` (no feature) does not compile it.
#[cfg(any(test, feature = "xproc-test-helper"))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderedIcon {
    Normal,
    Dimmed,
    ErrorDot,
}

#[cfg(any(test, feature = "xproc-test-helper"))]
impl RenderedIcon {
    pub fn is_dimmed(&self) -> bool {
        matches!(self, RenderedIcon::Dimmed)
    }
    pub fn is_normal(&self) -> bool {
        matches!(self, RenderedIcon::Normal)
    }
    pub fn is_error_dot(&self) -> bool {
        matches!(self, RenderedIcon::ErrorDot)
    }
}

/// Test mock renderer (rev-13 P1-5; **rev-15 P1-2:
/// `#[cfg(any(test, feature = "xproc-test-helper"))]`-gated** — NOT `#[cfg(test)]`,
/// which is invisible to the integration-test crate `src-tauri/tests/tray_state.rs`):
/// records every `set_icon_*`/`set_tooltip` call so the PulseWorker-lifecycle
/// tests can assert the exact frame sequence. Visible to integration tests because
/// the module is `pub` and the test harness compiles with `--features xproc-test-helper`.
#[cfg(any(test, feature = "xproc-test-helper"))]
#[derive(Default)]
pub struct RecordingRenderer {
    calls: std::sync::Mutex<Vec<(RenderedIcon, Option<String>)>>,
}

#[cfg(any(test, feature = "xproc-test-helper"))]
impl RecordingRenderer {
    /// Snapshot of the recorded (icon, tooltip) pairs, in call order.
    pub fn calls(&self) -> Vec<(RenderedIcon, Option<String>)> {
        self.calls.lock().expect("RecordingRenderer poisoned").clone()
    }
}

#[cfg(any(test, feature = "xproc-test-helper"))]
impl TrayRenderer for RecordingRenderer {
    fn set_icon_normal(&self) {
        let mut g = self.calls.lock().expect("RecordingRenderer poisoned");
        g.push((RenderedIcon::Normal, None));
    }
    fn set_icon_dimmed(&self) {
        let mut g = self.calls.lock().expect("RecordingRenderer poisoned");
        g.push((RenderedIcon::Dimmed, None));
    }
    fn set_icon_error_dot(&self) {
        let mut g = self.calls.lock().expect("RecordingRenderer poisoned");
        g.push((RenderedIcon::ErrorDot, None));
    }
    fn set_tooltip(&self, text: &str) {
        // Fold the tooltip into the most recent icon record if its tooltip slot
        // is empty; otherwise append as a no-op icon record.
        let mut g = self.calls.lock().expect("RecordingRenderer poisoned");
        if let Some(last) = g.last_mut() {
            if last.1.is_none() {
                last.1 = Some(text.to_owned());
                return;
            }
        }
        g.push((RenderedIcon::Normal, Some(text.to_owned())));
    }
}

// ─── rev-14 (P1-2 + P1-3 + P1-4): TrayStateController reducer ────────────────

/// The tray visual-state reducer (rev-12 P1-3, rev-13 generation-aware, rev-14
/// SYNC + current_state-gated worker swap, rev-15 PulseWorker + no-gen switch +
/// finish_translation merge, rev-16 NO overloading + gen guards + switch revision,
/// rev-17 latest_translation_gen guard + PulseEvent notify + async switch handler).
/// Owns the active-translation counter, the generation-tagged error (translation
/// flow), the switch-flow `switch_revision` + `switch_error_rev` (rev-16-3 —
/// replaces rev-15's sticky `has_error` bool so concurrent switch completions are
/// ordered by revision), the CURRENT resolved visual state, the `PulseWorker`,
/// the injected renderer, and the locale. Stored in
/// `Arc<parking_lot::Mutex<TrayStateController>>` on `AppState` (rev-14:
/// synchronous `parking_lot::Mutex`, NOT `tokio::sync::Mutex`).
///
/// rev-14 P2: does NOT derive `Debug` (holds `Arc<dyn TrayRenderer>`).
pub struct TrayStateController {
    /// Count of in-flight translations (multiple can overlap). saturating-sub
    /// on finish so it never underflows.
    active_translations: u32,
    /// rev-13 (P1-3): `Some(gen)` if a TRANSLATION-FLOW error is currently shown
    /// — the gen that PRODUCED the error. A `begin_translation` of a STRICTLY
    /// NEWER gen clears this; `finish_translation(_, false)` does NOT.
    /// `finish_translation(_, true)` clears it ONLY if `error_gen <= gen`
    /// (rev-16-2 gen guard — a stale OLDER gen's success does NOT clear a NEWER
    /// gen's error). `record_translation_error(gen)` sets it ONLY if
    /// `gen >= error_gen` (rev-16-2) AND `gen >= latest_translation_gen`
    /// (rev-17-3 — a stale OLDER gen's late error does NOT clobber a NEWER gen's
    /// error, even when error_gen is None).
    error_gen: Option<u64>,
    /// rev-17-3 (P1-3): the gen of the MOST RECENT `begin_translation` call.
    /// `record_translation_error(gen)` only records the error if
    /// `gen >= latest_translation_gen` (a stale OLDER gen that began before a
    /// newer gen but reports its error late is ignored — the newer gen is what
    /// the user sees). Updated by `begin_translation` to `max(self, gen)`.
    latest_translation_gen: u64,
    /// rev-16-3 (P1-3): SWITCH-FLOW revision counter — monotonically incremented
    /// by `begin_switch()` and returned to the caller, who passes it back to
    /// `finish_switch(rev, success)`. Replaces rev-15's sticky `has_error: bool`
    /// (which had no revision, so re-ordered concurrent switch completions could
    /// show the wrong final state). The switch flow does NOT touch the
    /// translation `GenerationToken`.
    switch_revision: u64,
    /// rev-16-3 (P1-3): `Some(rev)` if the switch flow at revision `rev` produced
    /// an error. Set ONLY by `finish_switch(rev, false)` (only when
    /// `rev == switch_revision`); cleared by `finish_switch(rev, true)`.
    /// (**rev-17-4: `record_switch_error()`/`clear_switch_error()` DELETED** —
    /// `finish_switch` is the sole switch mutator.)
    switch_error_rev: Option<u64>,
    /// rev-14 (P1-2) / rev-15: the CURRENT resolved visual state. `recompute`
    /// only swaps the `PulseWorker` when `new_state != current_state` (Active →
    /// Active counter bump does NOT churn the worker).
    current_state: TrayVisualState,
    /// rev-15 (P1-1): the background `PulseWorker`. `Some` while in
    /// ActiveTranslation, `None` otherwise. `recompute` starts it on entry and
    /// drops it on exit (`take()` → `Drop` → `stop()` → send + join — the
    /// channel-quit barrier that prevents stale writes; NO infinite loop, NO
    /// deadlock, NO `visual_epoch`).
    pulse_worker: Option<PulseWorker>,
    /// rev-14/rev-15: the tick interval (800ms in prod; a tiny value in tests so
    /// the suite does not sleep 800ms).
    tick_interval: std::time::Duration,
    /// rev-13 (P1-5): the injected renderer. Prod wraps `TrayIcon`; test wraps
    /// `RecordingRenderer`. `Arc<dyn>` so the controller is clone-cheap to move.
    /// The `PulseWorker` holds its OWN clone and writes through it directly on
    /// each tick (no controller lock on the tick path).
    renderer: Arc<dyn TrayRenderer>,
    /// rev-17-2: the per-tick notification channel passed to each `PulseWorker`
    /// this controller starts (carries `PulseEvent` — Tick per frame, Stopped on
    /// worker exit). `None` in prod; `Some` in tests (so tests `recv_timeout` on
    /// `PulseEvent::Tick`/`PulseEvent::Stopped` instead of `thread::sleep`).
    notify_tx: Option<std::sync::mpsc::Sender<PulseEvent>>,
    /// rev-14: captured once at construction via `detect_system_locale()`.
    locale: Locale,
    /// rev-19-4 (P1-4): monotonic counter incremented each time `recompute` enters
    /// the `new_state == ActiveTranslation` branch and starts a new `PulseWorker`.
    /// Used by the `second_begin_does_not_churn_the_worker` test to assert that an
    /// Active→Active counter bump does NOT restart the worker (the count must stay
    /// the same across the second `begin_translation`). Initial value 0.
    worker_start_count: u32,
}

impl TrayStateController {
    /// Production constructor: wraps the real tray. `locale` is read from the
    /// system here (NOT at each call site — the controller is long-lived). The
    /// tick interval is 800ms (rev-12 P1-1). rev-16 P2-1: `notify_tx` is `None`
    /// in prod (the worker does not notify anyone; tests pass `Some`).
    pub fn new(app: tauri::AppHandle) -> Self {
        Self::with_renderer_interval_and_notify(
            Arc::new(TrayIconRenderer::new(app)),
            detect_system_locale(),
            std::time::Duration::from_millis(800),
            None,
        )
    }

    /// rev-13 (P1-5): constructor with an injected renderer (test entry point).
    /// Uses the 800ms prod tick interval. Prod calls this with `TrayIconRenderer`;
    /// tests call it with `RecordingRenderer`.
    pub fn with_renderer(renderer: Arc<dyn TrayRenderer>, locale: Locale) -> Self {
        Self::with_renderer_interval_and_notify(
            renderer, locale, std::time::Duration::from_millis(800), None,
        )
    }

    /// rev-14/rev-15: constructor with an injected renderer AND a custom tick
    /// interval (test entry point — the PulseWorker-lifecycle tests pass a tiny
    /// interval so they don't sleep 800ms in real time). rev-16 P2-1: delegates
    /// to `with_renderer_interval_and_notify` with `notify_tx = None`.
    pub fn with_renderer_and_interval(
        renderer: Arc<dyn TrayRenderer>,
        locale: Locale,
        tick_interval: std::time::Duration,
    ) -> Self {
        Self::with_renderer_interval_and_notify(renderer, locale, tick_interval, None)
    }

    /// rev-16 P2-1 / rev-17-2: constructor with an injected renderer, a custom tick
    /// interval, AND a per-tick notify channel (test entry point — the PulseWorker-
    /// lifecycle tests pass `Some(notify_tx)` so they can `recv_timeout` on
    /// `PulseEvent::Tick`/`PulseEvent::Stopped` instead of `thread::sleep`). Prod
    /// passes `None`. **rev-18-2: this is the SOLE constructor that initializes the
    /// `notify_tx` field — `new`, `with_renderer`, and `with_renderer_and_interval`
    /// all delegate here passing `None`, so every `Self { ... }` literal initializes
    /// the field (no missing-field compile error).**
    pub fn with_renderer_interval_and_notify(
        renderer: Arc<dyn TrayRenderer>,
        locale: Locale,
        tick_interval: std::time::Duration,
        notify_tx: Option<std::sync::mpsc::Sender<PulseEvent>>,
    ) -> Self {
        let mut c = Self {
            active_translations: 0,
            error_gen: None,
            latest_translation_gen: 0, // rev-17-3: tracks the newest begin_translation gen
            switch_revision: 0,
            switch_error_rev: None,
            current_state: TrayVisualState::Normal,
            pulse_worker: None,
            tick_interval,
            renderer,
            notify_tx,
            locale,
            worker_start_count: 0, // rev-19-4: no PulseWorker started yet
        };
        // Render the initial Normal state so the icon matches from the start.
        c.render();
        c
    }

    // ── test accessors (plain pub — visible to the integration test crate) ──

    pub fn active_translations(&self) -> u32 {
        self.active_translations
    }

    /// rev-13 (P1-3): the generation that produced the current TRANSLATION-FLOW
    /// error, or `None`.
    pub fn error_gen(&self) -> Option<u64> {
        self.error_gen
    }

    /// rev-17-3 (P1-3): the gen of the MOST RECENT `begin_translation` call. A
    /// late error from an OLDER gen (`gen < latest_translation_gen`) is ignored by
    /// `record_translation_error`. Exposed for tests that assert the gate.
    pub fn latest_translation_gen(&self) -> u64 {
        self.latest_translation_gen
    }

    /// rev-16-3 (P1-3): the current switch revision (incremented by `begin_switch`).
    pub fn switch_revision(&self) -> u64 {
        self.switch_revision
    }

    /// rev-16-3 (P1-3): `Some(rev)` if the switch flow at revision `rev` produced
    /// an error, else `None`. (Replaces rev-15's `has_error: bool` accessor.)
    pub fn switch_error_rev(&self) -> Option<u64> {
        self.switch_error_rev
    }

    /// rev-14 (P1-2) / rev-15: the current resolved visual state.
    pub fn current_state(&self) -> TrayVisualState {
        self.current_state
    }

    /// rev-15: true iff a `PulseWorker` is currently running (i.e. the controller
    /// is in `ActiveTranslation`).
    pub fn is_pulsing(&self) -> bool {
        self.pulse_worker.is_some()
    }

    /// rev-19-4 (P1-4): the monotonic count of `PulseWorker`s this controller has
    /// started. Increments each time `recompute` enters the
    /// `new_state == ActiveTranslation` branch. The no-churn test asserts this does
    /// NOT increase across a second `begin_translation` while already Active (the
    /// worker is reused, not restarted).
    pub fn worker_start_count(&self) -> u32 {
        self.worker_start_count
    }

    // ── real mutators (drive the tray via recompute — ALL SYNC) ─────────────

    /// A translation started (rev-13: gen-tagged). If `error_gen` belongs to an
    /// OLDER generation, clear it (a new translation supersedes a prior error
    /// once it begins). Then increment + recompute. Does NOT touch the switch
    /// flow (`switch_revision`/`switch_error_rev`). rev-14/rev-15/rev-16/rev-17:
    /// SYNC. rev-17-3: also updates `latest_translation_gen` to `max(self, gen)`
    /// so a LATE error from an OLDER gen (that began before this newer gen) is
    /// ignored by `record_translation_error`.
    pub fn begin_translation(&mut self, gen: u64) {
        // rev-17-3: remember the newest begin_translation gen — a late error
        // from an OLDER gen (that began before this one) must not surface.
        if gen > self.latest_translation_gen {
            self.latest_translation_gen = gen;
        }
        // rev-13 (P1-3): a strictly-newer gen clears a prior translation red dot.
        if self.error_gen.map_or(false, |e| e < gen) {
            self.error_gen = None;
        }
        self.active_translations = self.active_translations.saturating_add(1);
        self.recompute();
    }

    /// rev-15 (housekeeping merge) + rev-16-2 (gen guard): finish a translation
    /// in ONE atomic call. Decrements the counter; if `success`, clears
    /// `error_gen` ONLY when `error_gen <= gen` (an OLDER gen's late success must
    /// NOT clear a NEWER gen's error — rev-16-2). Always recomputes. Called by
    /// `TranslationGuard::drop` (which passes `self.succeeded`). SYNC.
    pub fn finish_translation(&mut self, gen: u64, success: bool) {
        self.active_translations = self.active_translations.saturating_sub(1);
        if success {
            // rev-16-2 gen guard: only clear an error that belongs to this gen or
            // an OLDER gen. A newer gen's error must survive a stale older-gen
            // success (e.g. gen=1 late-arriving success must not clear gen=2's error).
            if self.error_gen.is_some_and(|eg| eg <= gen) {
                self.error_gen = None;
            }
        }
        let _ = gen;
        self.recompute();
    }

    /// rev-13 (P1-3) + rev-16-1 (renamed from `record_error`) + rev-16-2 (gen guard)
    /// + rev-17-3 (latest_translation_gen guard): record that generation `gen`
    /// produced a TRANSLATION-FLOW error. Sets `error_gen = Some(gen)` ONLY if
    /// BOTH `gen >= latest_translation_gen` (rev-17-3 — a stale OLDER gen that
    /// began before a newer gen but reports its error late is IGNORED; the newer
    /// gen is what the user sees) AND `gen >= error_gen` (rev-16-2 — a stale
    /// OLDER gen's late error does NOT clobber a NEWER gen's error that was
    /// already recorded). Recomputes (Error has priority). Used by the
    /// `capture_and_translate`/`translate_clipboard` error branches. SYNC.
    pub fn record_translation_error(&mut self, gen: u64) {
        // rev-17-3 + rev-16-2: only record a same-or-newer-than-latest-begin gen's
        // error, and only let it replace an existing error_gen if it is not older.
        if gen >= self.latest_translation_gen && self.error_gen.is_none_or(|eg| gen >= eg) {
            self.error_gen = Some(gen);
        }
        self.recompute();
    }

    /// rev-16-3 (P1-3): begin a new switch revision. Bumps `switch_revision` and
    /// returns the new value. The caller captures the returned `rev` and passes
    /// it to `finish_switch(rev, ...)` after the switch resolves. The switch flow
    /// does NOT touch the translation `GenerationToken` (calling `gen.next()`
    /// would stale in-flight translations — verified concurrency.rs). SYNC.
    /// (**rev-17-4: the low-level `record_switch_error()`/`clear_switch_error()`
    /// are DELETED** — `finish_switch(rev, success)` is the sole switch mutator
    /// and carries the stale-revision guard that the low-level methods lacked.)
    pub fn begin_switch(&mut self) -> u64 {
        self.switch_revision = self.switch_revision.saturating_add(1);
        self.switch_revision
    }

    /// rev-16-3 (P1-3): finish a switch revision. If `rev != self.switch_revision`,
    /// this is a STALE/late switch result — IGNORE it (only the LATEST revision
    /// can update state; this prevents re-ordered concurrent switch completions
    /// from clobbering the latest user intent). Otherwise, set `switch_error_rev`
    /// based on `success` (`Some(rev)` on failure, `None` on success — the latter
    /// is the clear). Recomputes. SYNC.
    pub fn finish_switch(&mut self, rev: u64, success: bool) {
        if rev != self.switch_revision {
            return; // stale switch result — ignore
        }
        self.switch_error_rev = if success { None } else { Some(rev) };
        self.recompute();
    }

    /// Resolve the highest-priority state from the counter + error_gen +
    /// switch_error_rev, and ONLY if it differs from `current_state`: drop the
    /// old `PulseWorker` (if leaving Active), start a new one (if entering
    /// Active), update `current_state`, and `render()`. `UpdateAvailable` is
    /// NEVER produced (deferred to R5/R6). rev-14 P1-2 / rev-15/rev-16: a
    /// counter bump that keeps the state at Active does NOT swap the worker.
    fn recompute(&mut self) {
        let new_state = recompute_pure(self);
        if new_state == self.current_state {
            // No transition — no worker swap. (Active → Active on a second
            // begin_translation lands here.)
            return;
        }
        // Manage the PulseWorker: drop the old one if leaving Active, start a new
        // one if entering Active.
        if self.current_state == TrayVisualState::ActiveTranslation {
            // Leaving Active: `take()` drops the PulseWorker → Drop → stop() →
            // stop_tx.send(()) + handle.join(). The worker's recv_timeout returns
            // on the signal, the thread exits, join completes — by the time we
            // render the new state below, the worker is DEAD (no stale tick can
            // overwrite the new icon). This is the channel-quit barrier (rev-15
            // P1-1 / P1-4 — NOT an epoch check).
            self.pulse_worker.take();
        }
        if new_state == TrayVisualState::ActiveTranslation {
            // Entering Active: start a new PulseWorker. rev-16 P2-1: pass the
            // controller's `notify_tx` clone so tests can recv_timeout on frames.
            self.pulse_worker = Some(PulseWorker::start(
                self.renderer.clone(),
                self.tick_interval,
                self.notify_tx.clone(),
            ));
            // rev-19-4 (P1-4): bump the monotonic worker-start counter so the
            // no-churn test can assert it does NOT increase on an Active→Active
            // counter bump (which lands in the `new_state == current_state`
            // early-return above and never reaches this branch).
            self.worker_start_count = self.worker_start_count.saturating_add(1);
        }

        self.current_state = new_state;
        self.render();
    }

    /// rev-15: the SINGLE sync entry point that writes icon + tooltip based on
    /// `current_state`. Called ONLY by `recompute` (inside the controller's
    /// `&mut self` lock). The `PulseWorker`'s per-tick writes go through its OWN
    /// renderer clone directly (no controller lock); they are serialized against
    /// this `render` by the `pulse_worker.take()` (→ Drop → stop → send + join)
    /// that `recompute` performs BEFORE rendering a new state — the worker is
    /// dead before the new-state render runs. So all icon writes are ordered.
    fn render(&mut self) {
        match self.current_state {
            TrayVisualState::Normal => {
                self.renderer.set_icon_normal();
                self.renderer.set_tooltip(tray_tooltip_text(self.current_state, self.locale));
            }
            TrayVisualState::ActiveTranslation => {
                // The PulseWorker drives the visible icon swaps via its direct
                // renderer calls on each tick; this initial render (on entering
                // Active) sets the first dimmed frame + tooltip for instant
                // feedback.
                self.renderer.set_icon_dimmed();
                self.renderer.set_tooltip(tray_tooltip_text(self.current_state, self.locale));
            }
            TrayVisualState::Error => {
                // rev-12 P1-2: red-dot OVERLAY on the base icon (composited in
                // build.rs), NOT a solid-red square. Embedded at compile time.
                self.renderer.set_icon_error_dot();
                self.renderer.set_tooltip(tray_tooltip_text(self.current_state, self.locale));
            }
            TrayVisualState::UpdateAvailable => {
                // recompute NEVER produces this (deferred to R5/R6). The arm
                // exists so the match is exhaustive; logging surfaces any
                // accidental call.
                log::warn!(
                    "render(UpdateAvailable) invoked — this state is deferred to R5/R6 per \
                     user-approved scope decision and should not be reached this stage"
                );
            }
        }
    }
}

// ─── rev-15/rev-16/rev-17 (P1-1): PulseEvent + PulseWorker (channel-quit + notify) ──

/// rev-17-2: the events a [`PulseWorker`] emits on its optional `notify` channel.
/// rev-16's `notify: Option<Sender<()>>` only carried an empty `()` signal — a
/// test `recv_timeout` could not tell whether a frame had fired (`Tick`) or the
/// worker had exited (`Stopped`). The `PulseEvent` enum disambiguates: `Tick` is
/// sent after each frame toggle (the test counts frames); `Stopped` is sent
/// immediately before the worker thread returns (the test treats it as "the
/// worker is dead"). In prod `notify` is `None` and no events are emitted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PulseEvent {
    /// One pulse frame completed (the worker toggled dimmed↔normal + rendered).
    Tick,
    /// The worker is about to exit (stop signal received or sender disconnected).
    /// Sent exactly once, immediately before the worker thread returns.
    Stopped,
}

/// rev-15 P1-1 + rev-16 P2-1 + rev-17-2: a background pulse worker that exits via
/// an `mpsc` channel signal — NOT an infinite `loop { sleep; render }` whose
/// `join()` would deadlock (the rev-14 bug). The worker holds an independent
/// `Arc<dyn TrayRenderer>` and toggles dimmed/normal on each `recv_timeout`
/// `Timeout`. `stop()` sends the signal and joins; `Drop` calls `stop()`.
/// rev-16 P2-1: after each tick the worker emits a notify event so tests can
/// deterministically wait for N frames via `recv_timeout` (NO `thread::sleep`).
/// rev-17-2: the notify channel carries `PulseEvent` — `Tick` per frame,
/// `Stopped` on exit (so a test distinguishes "a frame fired" from "the worker
/// died" without relying on the `Disconnected` side-effect of the Sender dropping).
///
/// The worker thread's body loops on `stop_rx.recv_timeout(interval)`:
/// - `Ok(())` (signal received) → emit `PulseEvent::Stopped` (if notify) → `return`.
/// - `Err(RecvTimeoutError::Disconnected)` (sender dropped) → emit
///   `PulseEvent::Stopped` (if notify) → `return` (exit).
/// - `Err(RecvTimeoutError::Timeout)` → toggle `dimmed` +
///   `renderer.set_icon_dimmed/normal()` + emit `PulseEvent::Tick` (rev-17-2, if
///   `notify` is `Some`).
///
/// `stop()` does `stop_tx.send(())` (wakes the worker from `recv_timeout`) then
/// `handle.take().join()` (the worker returns from `recv_timeout` on the signal,
/// so `join` completes — NO deadlock). `impl Drop for PulseWorker` calls `stop()`.
/// When the worker is dropped, the `notify` Sender (owned by the thread closure,
/// NOT a struct field — rev-19-3) is dropped too — if a test did not receive the
/// explicit `PulseEvent::Stopped`, the subsequent `recv_timeout` returns
/// `Err(Disconnected)`, proving the worker is dead.
///
/// rev-19-3 (P1-3): the struct does NOT hold a `notify` field. The `notify`
/// Sender passed to `start` is MOVED into the worker thread closure (the worker
/// owns its own Sender and drops it on exit). The struct holds ONLY `stop_tx` +
/// `handle` — both are read by `stop()`/`Drop`, so there is no `dead_code`
/// warning (rev-18's `notify` field was never read by the struct, only by the
/// thread, which clippy flags in the prod `notify = None` path).
pub struct PulseWorker {
    stop_tx: std::sync::mpsc::Sender<()>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl PulseWorker {
    /// Start a new pulse worker. The worker immediately begins toggling the
    /// renderer every `interval` (first tick after one `interval`). rev-16 P2-1 /
    /// rev-17-2: `notify` is `Some` in tests (the test `recv_timeout`s on
    /// `PulseEvent::Tick` per frame and `PulseEvent::Stopped` on exit) and `None`
    /// in prod (no one to notify).
    pub fn start(
        renderer: Arc<dyn TrayRenderer>,
        interval: std::time::Duration,
        notify: Option<std::sync::mpsc::Sender<PulseEvent>>,
    ) -> Self {
        let (stop_tx, stop_rx) = std::sync::mpsc::channel();
        // rev-19-3 (replaces rev-18-4): `notify_for_thread` is MOVED (not cloned) into
        // the worker closure — the struct no longer holds a `notify` field (rev-19-3
        // removed it to avoid the `dead_code` warning rev-18-4 was working around).
        // The worker owns its Sender and drops it on exit; a test receiver then sees
        // `Err(Disconnected)` if it did not first receive the explicit
        // `PulseEvent::Stopped`. When prod passes `notify = None`,
        // `notify_for_thread.as_ref()` returns `None` and the `if let Some(tx)` body
        // does not execute — but the variable is STILL read (`.as_ref()`), so clippy
        // does not flag the closure capture as dead_code.
        let notify_for_thread = notify;
        let handle = std::thread::spawn(move || {
            let mut dimmed = false;
            loop {
                match stop_rx.recv_timeout(interval) {
                    Ok(()) | Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                        // rev-17-2: signal the worker is about to exit, then return.
                        if let Some(tx) = notify_for_thread.as_ref() {
                            let _ = tx.send(PulseEvent::Stopped);
                        }
                        return;
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        dimmed = !dimmed;
                        if dimmed {
                            renderer.set_icon_dimmed();
                        } else {
                            renderer.set_icon_normal();
                        }
                        // rev-17-2: notify the test that a frame fired (do NOT
                        // panic on send failure — the test receiver may have
                        // dropped, but that is not a worker bug).
                        if let Some(tx) = notify_for_thread.as_ref() {
                            let _ = tx.send(PulseEvent::Tick);
                        }
                    }
                }
            }
        });
        // rev-19-3: `notify` is NOT stored on the struct — it was moved into the
        // worker closure above (`notify_for_thread`). The struct holds only
        // `stop_tx` + `handle`, both read by `stop()`/`Drop` (no dead_code).
        Self { stop_tx, handle: Some(handle) }
    }

    /// Stop the worker: send the quit signal, then join the handle. The worker
    /// returns from `recv_timeout` on the signal, so `join` completes — NO
    /// deadlock (the rev-14 `loop { sleep; render }` + `join()` deadlock is
    /// fixed). Idempotent: a second call is a no-op (the handle was taken).
    pub fn stop(&mut self) {
        let _ = self.stop_tx.send(());
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

impl Drop for PulseWorker {
    fn drop(&mut self) {
        self.stop();
        // rev-19-3: the worker thread owns its own `notify` Sender (moved into the
        // closure at `start`, NOT a struct field). `stop()` sends the quit signal +
        // joins; the worker's `recv_timeout` returns on the signal, the thread emits
        // `PulseEvent::Stopped` (if notify was `Some`) then returns — at which point
        // its `notify_for_thread` Sender drops, disconnecting the channel. A test that
        // did not receive the explicit `PulseEvent::Stopped` then observes
        // `Err(Disconnected)`, proving the worker is dead (used by
        // `last_finish_stops_the_worker` +
        // `leaving_active_stops_the_worker_no_stale_frames`).
    }
}

/// The pure resolution function (no renderer side-effects) — extracted so the
/// reducer logic is unit-testable. `Error > Active > Normal` (`UpdateAvailable`
/// is never produced — deferred to R5/R6). rev-15: reads BOTH `error_gen.is_some()`
/// (translation flow) AND the switch-flow error flag. rev-16-3: the switch-flow
/// flag is `switch_error_rev: Option<u64>` (replaces rev-15's `has_error: bool`) —
/// either triggers `Error`. rev-13/rev-14: SYNC (the controller is sync).
pub fn recompute_pure(c: &TrayStateController) -> TrayVisualState {
    if c.error_gen.is_some() || c.switch_error_rev.is_some() {
        TrayVisualState::Error
    } else if c.active_translations > 0 {
        TrayVisualState::ActiveTranslation
    } else {
        TrayVisualState::Normal
    }
}

// ─── rev-13/rev-14/rev-15 (P1-2): TranslationGuard RAII (synchronous Drop) ────

/// RAII guard guaranteeing `finish_translation` runs exactly once per
/// `begin_translation`, on EVERY return path (early return, `?`, panic).
///
/// Construct AFTER the preflight (text captured + anchor built) so a capture or
/// stale-gen failure does NOT begin a translation that then has to be finished.
/// The constructor calls `begin_translation(gen)`; `Drop` calls
/// `finish_translation(gen, succeeded)` (rev-15 merge — ONE atomic method:
/// decrement + clear-on-success + recompute).
///
/// rev-14/rev-15: SYNCHRONOUS — the controller mutex is `parking_lot::Mutex`,
/// whose `lock()` is a blocking sync call, so `Drop` runs `finish_translation`
/// on the CALLING THREAD before `Drop` returns (no `spawn`, no detached future).
/// This restores the true RAII guarantee (rev-13's detached
/// `tauri::async_runtime::spawn` in `Drop` returned before the counter was
/// decremented).
///
/// On a SUCCESS branch, call [`TranslationGuard::mark_success`] BEFORE the
/// guard goes out of scope — `Drop` then calls `finish_translation(gen, true)`,
/// which clears `error_gen` ONLY if `error_gen <= gen` (rev-16-2 gen guard — an
/// OLDER gen's late success must NOT clear a NEWER gen's error). On an ERROR
/// branch, call `controller.lock().record_translation_error(gen)` (**rev-16-1
/// renamed from `record_error(gen)`**; gen-guarded set per rev-16-2; sync)
/// BEFORE the guard drops — `Drop` then calls `finish_translation(gen, false)`,
/// which does NOT clear `error_gen` (the error persists until a same-or-newer-gen
/// success clears it).
pub struct TranslationGuard<'a> {
    controller: &'a Arc<parking_lot::Mutex<TrayStateController>>,
    gen: u64,
    succeeded: bool,
}

impl<'a> TranslationGuard<'a> {
    /// Begin a translation (gen-tagged). rev-14/rev-15: SYNCHRONOUS.
    pub fn new(controller: &'a Arc<parking_lot::Mutex<TrayStateController>>, gen: u64) -> Self {
        controller.lock().begin_translation(gen);
        Self {
            controller,
            gen,
            succeeded: false,
        }
    }

    /// Mark the guarded translation as succeeded — the guard's `Drop` then calls
    /// `finish_translation(gen, true)`, which clears `error_gen` IF `error_gen <= gen`
    /// (rev-16-2 gen guard — the prior-gen red dot disappears on a successful
    /// Retry of this gen OR an older gen, but a NEWER gen's error survives).
    /// Called on the success branch, BEFORE the guard drops. Idempotent (just
    /// sets the flag). rev-14/rev-15/rev-16: SYNCHRONOUS.
    pub fn mark_success(&mut self) {
        self.succeeded = true;
    }
}

impl Drop for TranslationGuard<'_> {
    fn drop(&mut self) {
        // rev-15 merge: ONE atomic finish_translation call — decrement + (if
        // succeeded) clear error_gen + recompute. No spawn, no detached future:
        // the counter is decremented before drop returns.
        let mut c = self.controller.lock();
        c.finish_translation(self.gen, self.succeeded);
    }
}
```

> **rev-15/rev-16 PulseWorker / channel-quit implementation note:** the per-tick invariant is enforced by the controller's `pulse_worker.take()` (→ `PulseWorker::Drop` → `stop()` → `stop_tx.send(())` + `handle.join()`) being performed in `recompute` BEFORE the new state is rendered. The worker's `recv_timeout` returns `Ok(())` on the signal, the thread exits, and `join()` completes — so by the time `recompute` calls `self.render()` for the new state, the worker thread is DEAD. There is no window for a stale tick to overwrite the new icon. The `RecordingRenderer` test `leaving_active_stops_the_worker_no_stale_frames` (renamed from rev-14's `stale_epoch_tick_does_not_clobber_error`) asserts this: after `record_translation_error(1)` (**rev-16-1 renamed from `record_error(1)`**) switches to `Error` + drops the worker (send + join completes), the `notify` channel returns `Err(Disconnected)` (rev-16 P2-1 — NO `thread::sleep`), proving no further ticks fire. **This is rev-15 P1-1's fix for the rev-14 deadlock:** rev-14's pulse thread was an infinite `loop { sleep; render }` with no exit path, and `stop_timer()`'s `join()` waited for it to exit — which never happened → `join()` blocked forever → the whole app hung on the first transition out of `Active`. The `PulseWorker`'s `recv_timeout` + signal-exit makes `join()` return. **rev-15 P1-4 removes the `visual_epoch` field entirely** (it was the marker for the rev-14 in-timer epoch check, which rev-14's prose described but its `spawn_pulse_timer` code never performed — prose and code disagreed; rev-15 keeps only the code model: the worker holds an independent renderer clone and the channel-quit is the sole barrier). The worker does NOT re-lock the controller (the controller cannot own the `Arc<Mutex<TrayStateController>>` it is stored in — a back-reference would be a borrow cycle). **rev-16 P2-1:** the worker carries an `Option<Sender<()>> notify`; per tick it `notify.send(())` (no-op if `None`). Tests `recv_timeout` on the receiver for deterministic frame counting (NO `thread::sleep`); when the worker drops, the Sender drops and the test observes `Err(Disconnected)`.

- [x] **Step 6: Declare the module in lib.rs (rev-12 P1-4: PUBLIC module) + add `tray` to `AppState` at ALL 5 construction sites + locale at construction (rev-14: parking_lot::Mutex + sys-locale)**

In `src-tauri/src/lib.rs`, alongside the other `mod` declarations near the top of the file, add (rev-12 P1-4: `pub mod` — NOT `mod tray_state;` as in rev-11, because the integration test imports via the module path `linguaray_lib::tray_state::...`):

```rust
pub mod tray_state;
```

Then add a crate-root convenience re-export (so callers can also `use linguaray_lib::{TrayStateController, TrayVisualState};`). The module-path import in the test works because the module is now `pub`; the re-export is additive and does not conflict. **rev-14:** add the new symbols (`detect_system_locale`, `TranslationGuard`, `RecordingRenderer`, `TrayRenderer`, `RenderedIcon`, `TrayIconRenderer`). **rev-14: drop `apply_visual_state`** (rev-13's leaf is replaced by the controller's `render(&mut self)` method, which is private — callers drive the tray via the guard/reducer, NOT via a standalone fn). **rev-15:** add `PulseWorker` to the re-export; **rev-15 P1-2: gate the test-only re-exports (`RecordingRenderer`, `RenderedIcon`) behind `#[cfg(any(test, feature = "xproc-test-helper"))]`** — they are `#[cfg(any(test, feature = "xproc-test-helper"))]` in the module (Step 5), so the re-export must carry the SAME cfg or it fails to resolve under `cargo build` (no feature). The production re-exports (`TrayStateController`, `TranslationGuard`, `PulseWorker`, `TrayIconRenderer`, `TrayRenderer`, `TrayVisualState`, `Locale`, the pure fns) are always-on:

```rust
// rev-15 P1-2: production re-exports — always available.
#[cfg(any(test, feature = "xproc-test-helper"))]
pub use tray_state::{RecordingRenderer, RenderedIcon};
pub use tray_state::{
    detect_system_locale, PulseEvent, PulseWorker, recompute_pure, tray_state_priority,
    tray_tooltip_text, Locale, TrayIconRenderer, TrayRenderer,
    TrayStateController, TrayVisualState, TranslationGuard,
};
```

(If `lib.rs` already has a `pub use` block for other modules, append to it; otherwise add the lines above after the `pub mod tray_state;` declaration. The two blocks can be adjacent — Rust allows multiple `pub use` items in sequence. The test crate, compiled under `--features xproc-test-helper`, resolves both; `cargo build` (no feature) resolves only the always-on block.)

**Then add the `tray` field to `AppState` (rev-13 P1-1: on `AppState`, NOT `Session`).** The current `AppState` (lib.rs:99-106) has `db`, `data_gate`, `readiness`, `db_path`, `keystore_dir`, `settings_path` — NO tray field. Add:

```rust
pub struct AppState {
    pub db: parking_lot::RwLock<Option<Arc<Database>>>,
    pub data_gate: parking_lot::RwLock<()>,
    pub readiness: parking_lot::RwLock<DataReadiness>,
    pub db_path: PathBuf,
    pub keystore_dir: PathBuf,
    pub settings_path: Option<PathBuf>,
    /// rev-14 / Task A5: the tray visual-state reducer (rev-12 P1-3, rev-13
    /// generation-aware, rev-14 SYNC). Behind a SYNCHRONOUS `parking_lot::Mutex`
    /// (NOT `tokio::sync::Mutex`) so concurrent translate/clipboard/switch calls
    /// serialize state transitions AND so `TranslationGuard::drop` runs
    /// `end_translation` SYNCHRONOUSLY on the calling thread (true RAII — no
    /// detached spawn). Lives on `AppState` (rev-13 P1-1) — NOT `Session`
    /// (`Session` has no tray field). `parking_lot = "0.12"` is already a
    /// production dep (Cargo.toml:53).
    pub tray: std::sync::Arc<parking_lot::Mutex<tray_state::TrayStateController>>,
}
```

**rev-13: there are FIVE `AppState { ... }` construction sites — ALL must add the `tray` field.** The rev-12 spec was vague ("the initialization site"); rev-13 enumerates every one (verified against source):

1. **`lib.rs:2513`** (production startup, inside `app.manage(Arc::new(AppState { ... }))`): the controller needs a `tauri::AppHandle`, which is available at this point. Add at the end of the struct literal (**rev-14: `parking_lot::Mutex`, NOT `tokio::sync::Mutex`**):
   ```rust
       tray: std::sync::Arc::new(parking_lot::Mutex::new(
           tray_state::TrayStateController::new(app.handle().clone()),
       )),
   ```
   (`app.handle()` is the `&tauri::AppHandle` in scope at the manage site — adjust to the local binding name if it differs.)

2. **`lib.rs:2597`** (`#[test] fn read_data_readiness_from_state_returns_typed_object`): a test that does NOT have an `AppHandle`. Use `TrayStateController::with_renderer(Arc::new(RecordingRenderer::default()), Locale::En)` so the test does not need a runtime (**rev-14: `parking_lot::Mutex`**):
   ```rust
           tray: std::sync::Arc::new(parking_lot::Mutex::new(
               tray_state::TrayStateController::with_renderer(
                   std::sync::Arc::new(tray_state::RecordingRenderer::default()),
                   tray_state::Locale::En,
               ),
           )),
   ```

3. **`lib.rs:2620`** (`#[test] fn read_data_readiness_preserves_reason_payload`): same as #2 — use `with_renderer(RecordingRenderer, Locale::En)`.

4. **`tests/recovery.rs:42`** (`RecoveryFixture::new`): same as #2 — `with_renderer(RecordingRenderer, Locale::En)`. (The recovery tests never exercise the tray; the mock renderer is inert.)

5. **`tests/recovery.rs:248`** (`archive_database_refuses_when_settings_path_unresolved`): same as #2.

> **Why `with_renderer` at the test sites:** the prod `TrayStateController::new(app)` needs a `tauri::AppHandle`, which the tests do not have. `with_renderer` takes an `Arc<dyn TrayRenderer>` + a `Locale`, so the test sites inject an inert `RecordingRenderer` and `Locale::En`. The `RecordingRenderer` is `pub` in the module (rev-13 P1-5) so the test files can name it. Add `use linguaray_lib::tray_state::{RecordingRenderer, Locale, TrayStateController};` (or the crate-root re-export) at the top of each test file that constructs `AppState`.

**rev-14 locale: NO `read_locale(state)` helper.** rev-12 added a `read_locale` that read `state.settings_locale()` — but `AppState` has NO `settings_locale()` accessor AND `Settings` (settings.rs:9-15) has NO `locale` field (only `default_provider`/`target_language`/`fallback_engine`). rev-13/rev-14 DELETES that helper entirely: the locale is captured ONCE, inside `TrayStateController::new(app)` / `with_renderer(renderer, locale)`, via `detect_system_locale()`. **rev-14: `detect_system_locale()` uses `sys_locale::get_locale()`** (cross-platform — the `sys-locale = "0.3"` crate added in Step 1; rev-13's `std::env::var("LANG")` was Unix-only and returns `None` on Windows). Call sites do NOT pass a locale — they pass only `gen` to the guard/`begin`/`end`/`record_translation_error` methods (rev-16-1 renamed), and the controller uses its captured `self.locale`. The signature change (controller methods take `gen: u64` (translation) or `rev: u64` (switch, rev-16-3), not `app` + `locale`) is reflected in Steps 8-10.

- [x] **Step 7: Run the test to verify it passes**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --features xproc-test-helper --test tray_state`
Expected: PASS (33 tests, rev-18 count — ALL `#[test]` (SYNC); rev-18-1/3: the functional switch test is now `#[test]` against a real DB, NOT `#[tokio::test]`):
- 6 priority-ordering (rev-11): `normal_is_lowest_priority`, `active_beats_normal`, `update_beats_active`, `error_is_highest_priority`, `full_ordering_is_error_update_active_normal`, `update_arm_exists_but_is_documented_deferred`.
- 6 reducer concurrency (rev-12 shape, rev-13 accessors, rev-14/rev-15/rev-16/rev-17 SYNC + finish_translation + switch_error_rev — **rev-16 P2-5: header corrected 5→6**; rev-17-4: switch mutator is `finish_switch`, not `record_switch_error`): `recompute_pure_normal_when_idle`, `begin_then_finish_two_translations_keeps_active_until_last_finishes`, `finish_translation_saturates_at_zero`, `translation_error_overrides_active_and_survives_finish_false`, `recompute_never_produces_update_available`, `switch_flow_error_is_independent_of_translation_error_gen` (**rev-16-3 renamed from `switch_flow_has_error_is_independent_of_translation_error_gen`; assertions use `switch_error_rev()` accessor; rev-17-4: uses `finish_switch`**).
- 2 RAII guard (rev-14/rev-15 P1-2, synchronous Drop + finish_translation merge + rev-16-2 gen guard): `guard_drop_finishes_translation_on_every_return_path`, `guard_marks_success_and_clears_prior_gen_error`.
- 2 generation-aware error (rev-13 P1-3 / rev-16-1 renamed `record_translation_error`): `retry_of_new_gen_clears_prior_red_dot`, `same_gen_retry_does_not_clear_error`.
- 3 gen-guard tests (rev-16-2 + rev-17-3): `older_success_does_not_clear_newer_error`, `older_error_does_not_replace_newer_error` (rev-16-2) + `stale_gen_error_ignored_after_newer_begin` (rev-17-3 NEW — `latest_translation_gen` guard).
- 4 renderer + PulseWorker (rev-14/rev-15/rev-16/rev-17 P1-2 + P1-5; **rev-16 P2-1 / rev-17-2: notify channel carrying PulseEvent, NO thread::sleep**): `active_emits_alternating_frames_on_the_renderer`, `second_begin_does_not_churn_the_worker`, `last_finish_stops_the_worker` (rev-17-2: asserts `PulseEvent::Stopped`), `error_produces_no_active_pulse_frame`.
- 2 PulseWorker channel-quit (rev-15 P1-1, NEW — no sleep; rev-17 P2-4: assert `PulseEvent::Stopped` then join — deterministic): `stop_signal_joins_the_worker`, `drop_stops_the_worker`.
- 1 worker-stop barrier (rev-15 P1-4, renamed from rev-14 stale-epoch; rev-17-2: notify emits `PulseEvent::Stopped`): `leaving_active_stops_the_worker_no_stale_frames`.
- 2 localization (rev-12 P2 + rev-14): `tooltip_text_is_localized`, `detect_system_locale_never_panics`.
- 1 pixel-diff (rev-14): `red_dot_overlay_preserves_base_icon_outside_the_dot` (`panic!`s if the PNG is not yet built — rev-14 P2).
- 2 switch does NOT bump generation (rev-15 P1-3 + rev-16 P2-2 + rev-18-3 — 1 functional `switch_handler_does_not_call_gen_next` (**rev-18-3: `#[test]`, calls the REAL SYNC core `handle_switch_provider_core(&app_state, &uuid)` — NO AppHandle — against a real temp DB + inserted provider — NOT `#[tokio::test]`/async**) + 1 structural `switch_arm_source_has_no_gen_next_call` (rev-18 P2-4: ALSO asserts no `.await`/`spawn(async move`/`pub async fn handle_switch_provider`)): replaces the single rev-15 `switch_does_not_bump_translation_generation`.
- 2 switch revision ordering (rev-16-3, NEW): `two_concurrent_switches_second_wins`, `stale_switch_result_ignored`.

- [x] **Step 8: Wire TrayStateController into capture_and_translate via TranslationGuard (rev-14 sync P1-1 + P1-2; rev-15 finish_translation merge)**

In A4's `capture_and_translate` helper (the new fn extracted from `on_hotkey`), add the tray transitions. This is a `diff`-style instruction against the A4 helper body — do NOT rewrite the helper, only add the calls.

**rev-14 (P1-1):** the field is `app_state.tray` (the helper's parameter is `app_state: &Arc<AppState>` per A2/A4 Step 5 — NOT `state`, which is the `&Arc<Session>` parameter). All tray calls go through `app_state.tray.lock()` — **rev-14: SYNCHRONOUS `parking_lot::Mutex::lock`, NO `.await`**.

**rev-13/rev-14 (P1-2 — RAII guard) + rev-15 (finish_translation merge):** rev-12 put `begin_translation` at the very top and relied on manually adding `end_translation` to every terminal branch — but the helper has over 10 early returns (capture fail, stale gen, client/keystore/db acquisition failure, …). rev-13 replaces this with a `TranslationGuard`; rev-14 makes its `Drop` SYNCHRONOUS (parking_lot mutex); **rev-15 collapses the Drop's `end_translation + clear_error_for_gen + recompute` into ONE `finish_translation(gen, succeeded)` call.**
- `begin_translation(gen)` runs AFTER the preflight (text captured + anchor built), so a capture/stale-gen failure does NOT begin a translation that then has to be finished.
- The guard's `Drop` runs `finish_translation(gen, succeeded)` **synchronously** on EVERY return path (early return, `?`, panic) — a single atomic method: decrement + (if `succeeded`) clear `error_gen` + recompute.
- On a SUCCESS branch, call `guard.mark_success();` (sets `succeeded = true`; Drop calls `finish_translation(gen, true)` → clears `error_gen` ONLY if `error_gen <= gen` per rev-16-2 gen guard) BEFORE the return.
- On an ERROR branch, call `app_state.tray.lock().record_translation_error(gen);` (**rev-16-1 renamed from `record_error(gen)`**; gen-guarded set per rev-16-2; sets `error_gen = Some(gen)` only if `gen >= error_gen`) — the guard's Drop calls `finish_translation(gen, false)` (decrement + recompute, does NOT clear `error_gen`).

**Concretely** (assume the helper's locals are `app: &tauri::AppHandle`, `state: &Arc<Session>`, `app_state: &Arc<AppState>`, `gen: u64`):

- AFTER the preflight (capture succeeded + anchor built + the first `state.gen.is_latest(gen)` check that gates the DB-acquisition `spawn_blocking`), create the guard. This is the SINGLE `begin`:
  ```rust
      // rev-14 P1-2 / rev-15: begin ONLY after preflight. The guard's SYNCHRONOUS
      // Drop (parking_lot::Mutex) calls finish_translation(gen, succeeded) on
      // every path before Drop returns (ONE atomic method; rev-16-2 gen guard
      // gates the success clear).
      let mut _tray_guard = tray_state::TranslationGuard::new(&app_state.tray, gen);
  ```
  All subsequent early returns (stale-gen post-spawn, client/keystore/db acquisition failure, `run_translate_session` error) are now SAFE — the guard's Drop calls `finish_translation(gen, false)` automatically. NO manual `end_translation`/`finish_translation` calls anywhere on the return paths.

- On EVERY terminal SUCCESS branch (`popup::result` / `popup::multi_result` emit, and the stale-gen no-op return that is NOT an error), call BEFORE the return/emit:
  ```rust
      _tray_guard.mark_success(); // rev-14 P1-3 / rev-15: Drop calls finish_translation(gen, true) → clears error_gen iff error_gen <= gen (rev-16-2)
  ```
  (For the stale-gen no-op, `mark_success` is correct — the no-op is not an error, just superseded.)

- On EVERY terminal ERROR branch (the paths that emit `popup::error`: `Ok(Err(msg))` from `spawn_blocking`, `Err(JoinError)`, `client`/`keystore` acquisition failure, `Err(msg)` from `run_translate_session`), call BEFORE the `popup::error` emit:
  ```rust
      app_state.tray.lock().record_translation_error(gen); // rev-16-1 renamed: tag the error with this gen (sync; gen-guarded set rev-16-2)
  ```
  The guard's Drop then calls `finish_translation(gen, false)` — the counter is decremented, and `error_gen` persists (`finish_translation(false)` does NOT clear it). The next translation's `begin_translation(new_gen)` clears it iff `new_gen > gen`.

**rev-15/rev-16 precedence summary:** stale-gen no-op → `mark_success` → Drop `finish_translation(gen, true)` → `Normal` (error_gen cleared iff `<= gen`, rev-16-2); `popup::error` → `record_translation_error(gen)` (rev-16-1) → Drop `finish_translation(gen, false)` → `Error` (persists until a same-or-newer-gen success); success emit → `mark_success` → Drop `finish_translation(gen, true)` → `Normal`. **The switch-provider success path (Step 10) calls `finish_switch(rev, true)` (rev-16-3 — NOT a translation-gen method) — switch does not participate in the translation generation.**

- [x] **Step 9: Wire TrayStateController into translate_clipboard (lib.rs:329) via TranslationGuard**

`translate_clipboard` (lib.rs:329-333) owns `app: tauri::AppHandle` by value and takes `app_state: tauri::State<'_, Arc<AppState>>`. Apply the SAME guard pattern as Step 8 (rev-14 P1-1: `app_state.tray`, NOT `state.tray`; rev-14 SYNC — no `.await`):

- AFTER the clipboard read succeeds (the `let text = { let _g = state.gen.selection_lock(); clipboard::get_text()? };` block at lib.rs:336-338 — so a clipboard-read failure does NOT begin a translation), create the guard:
  ```rust
      let mut _tray_guard = tray_state::TranslationGuard::new(&app_state.tray, gen);
  ```
  (gen was allocated at lib.rs:335 via `state.gen.next()`.)

- The `Ok(r) => match decide_clipboard_popup(&r) { SingleSuccess { .. } | Multi => ... }` SUCCESS arms (lines 401-405): call `_tray_guard.mark_success();` BEFORE the `popup::result`/`popup::multi_result` emit. The guard's Drop calls `finish_translation(gen, true)`.
- The `ClipboardPopupDecision::Error(msg)` arm (lines 407-409) AND the outer `Err(msg)` arm (lines 411-413): call `app_state.tray.lock().record_translation_error(gen);` (**rev-16-1 renamed from `record_error(gen)`**) BEFORE the `popup::error` emit. The guard's Drop calls `finish_translation(gen, false)`.
- The `session_client(&state)` / `session_keystore(&state)` `Err(msg)` early-returns (lines 349-364) and the `spawn_blocking` `Ok(Err(msg))` / `Err(e)` arms (lines 375-386): same — `record_translation_error(gen)` (rev-16-1) before `popup::error`. The guard's Drop calls `finish_translation(gen, false)` — counter handled.

> **rev-14/rev-15/rev-16 vs rev-12/rev-13:** rev-12 added manual `begin` at line 335 and manual `end`/`set_error` to every branch — risky given the early returns. rev-13's guard covers ALL paths via Drop; rev-14 makes that Drop SYNCHRONOUS (parking_lot mutex) so the counter is decremented before Drop returns; rev-15 collapses the Drop's work into ONE `finish_translation(gen, succeeded)` call; rev-16-1 renames the error setter to `record_translation_error(gen)` (NO overloading) and rev-16-2 adds the gen guard to both `finish_translation` and `record_translation_error`. The only manual calls are `mark_success` (success) and `record_translation_error(gen)` (error, rev-16-1 renamed) — both SYNC. The `translate_clipboard` body does NOT pass `app`/`locale` to the controller — the controller captured both at construction (Step 6), and the methods take only `gen`.

- [x] **Step 10: Wire TrayStateController into the switch-provider handler (A4 Step 9) — extract `handle_switch_provider_core` (SYNC, no AppHandle — test entry) + `handle_switch_provider` wrapper (SYNC, AppHandle refresh — tray arm entry) (rev-18-1) + begin_switch/finish_switch revision (rev-16-3) — acquire ONLY app_state via app.state; switch does NOT touch the translation GenerationToken**

In `handle_tray_menu_event` (lib.rs:2214, the `tray.switch-<uuid>` arm), the A4 Step 9 spec already preserves the old primary on failure and refreshes on success. **rev-16 P2-2 / rev-18-1: EXTRACT the switch handler into TWO functions** so the integration test (Step 2, section 10) can call the testable core directly WITHOUT a `tauri::AppHandle` (the integration-test crate has no Tauri test runtime / `tauri::test::mock_app` — `mock_app` requires a tauri test feature the current `Cargo.toml` does not enable):
- **`pub fn handle_switch_provider_core(app_state: &Arc<AppState>, uuid: &str) -> Result<(), String>`** — pure SYNC core: DB mutation (`set_active_primary_core`) + tray controller (`begin_switch`/`finish_switch`). Does NOT touch `AppHandle` (no `refresh_tray_if_available`, no icon/tooltip/menu). The functional test (Step 2 section 10) calls THIS fn — no mock AppHandle needed.
- **`pub fn handle_switch_provider(app: &tauri::AppHandle, app_state: &Arc<AppState>, uuid: &str) -> Result<(), String>`** — wrapper: calls `handle_switch_provider_core(app_state, uuid)` + uses `app` for `refresh_tray_if_available(&app)` + logs the failure. The `tray.switch-<uuid>` arm runs THIS wrapper via `tauri::async_runtime::spawn_blocking` (the menu-event callback is sync; the SQLite I/O inside `set_active_primary_core` must not run on the UI thread; `spawn_blocking` is the offload for a SYNC blocking fn).

**rev-16 (P1-3 — acquire ONLY AppState from `app`; do NOT acquire Session; do NOT call `gen.next()`; use switch revision) + rev-18-1 (sync core+wrapper):** the VERIFIED signature of `handle_tray_menu_event` is `fn handle_tray_menu_event(app: &tauri::AppHandle, event: MenuEvent)` (lib.rs:2214) — it has ONLY `app`, NO `state`/`session` parameter. rev-14 acquired `Session` + `gen = session.gen.next()` + `AppState` here — **but `GenerationToken::next()` ADVANCES the current generation** (verified concurrency.rs: `fetch_add(1, SeqCst) + 1`), which STALES any in-flight translation (its `is_latest(its_gen)` becomes false → its result is dropped, its `TranslationGuard` finishes with `succeeded=false`). rev-15 DECOUPLED switch from the translation generation (sticky `has_error: bool`). **rev-16-3 replaces the sticky bool with a switch revision**: the core calls `tray.lock().begin_switch()` → captures `rev` → runs the DB mutation → calls `tray.lock().finish_switch(rev, success)`. `finish_switch` IGNORES stale `rev != switch_revision` (a re-ordered late switch cannot clobber the latest). **rev-18-1: the core is SYNC and calls `set_active_primary_core(app_state.clone(), uuid.to_string())` directly (no `.await` — `set_active_primary_core` is a sync fn (A4 Step 9 verified signature: `fn set_active_primary_core(app_state: Arc<AppState>, uuid: String) -> Result<SetActiveResult, String>`, owned params — its body is the `db_set_active_primary` gate + tx, NOT async)); `begin_switch` runs BEFORE the sync DB call, `finish_switch` AFTER.** Both fns acquire ONLY the `AppState` (for the `tray` field) + the existing DB inputs (the uuid). The two bodies:

```rust
/// rev-18-1: pure SYNC core — DB mutation (set_active_primary_core) + tray controller
/// (begin_switch / finish_switch). Does NOT touch AppHandle (no refresh_tray / icon /
/// tooltip / menu). The integration test (Step 2, section 10) calls THIS fn directly —
/// it needs NO mock AppHandle / `tauri::test::mock_app` / `build_test_app_handle`. SYNC
/// because `set_active_primary_core` is itself SYNC (its body is the `db_set_active_primary`
/// gate + tx — A4 Step 9 verified signature, owned `Arc<AppState>` + owned `String`).
/// rev-16-3: switch revision — `begin_switch()` BEFORE the DB call, `finish_switch(rev,
/// success)` AFTER; a stale `rev != switch_revision` is ignored by `finish_switch`.
pub fn handle_switch_provider_core(app_state: &Arc<AppState>, uuid: &str) -> Result<(), String> {
    // rev-16-3: begin a switch revision BEFORE the DB mutation. The caller (this
    // fn) captures `rev` and passes it to finish_switch after the switch resolves.
    // A stale rev (a re-ordered late switch) is ignored by finish_switch.
    let rev = {
        let mut c = app_state.tray.lock();
        c.begin_switch()
    };
    // rev-18-1: set_active_primary_core is SYNC (its body is the db_set_active_primary
    // gate + tx). Owned params (Arc<AppState> + String) because its internal spawn_blocking
    // payload needs owned values (A4 Step 9). No .await.
    let result = set_active_primary_core(app_state.clone(), uuid.to_string());
    // rev-16-3: finish_switch(rev, success) — only the latest revision can update
    // state. On Err, finish_switch(rev, false) sets switch_error_rev (red dot);
    // on Ok, finish_switch(rev, true) clears it. A stale rev is ignored.
    {
        let mut c = app_state.tray.lock();
        c.finish_switch(rev, result.is_ok());
        // finish_switch(rev, false) already recorded the error — do NOT clear it.
    }
    result.map(|_| ()).map_err(|e| e.to_string())
}

/// rev-18-1: wrapper — calls `handle_switch_provider_core` + uses the AppHandle for
/// the tray visual refresh (`refresh_tray_if_available`). The `tray.switch-<uuid>` arm
/// in `handle_tray_menu_event` runs THIS wrapper via `tauri::async_runtime::spawn_blocking`
/// (the menu-event callback is sync; the SQLite I/O inside `set_active_primary_core`
/// must not run on the UI thread; `spawn_blocking` is the offload for a SYNC blocking fn).
///
/// rev-19-5 (P1-5): on FAILURE, sets the tray tooltip AFTER
/// `refresh_tray_if_available` (so the refresh's own `set_tooltip` does NOT clobber
/// the failure tooltip). DB rollback preserves the prior primary
/// (`set_active_primary_core`'s tx guarantees), and the controller's `finish_switch(rev,
/// false)` already drives the red-dot Error state — the tooltip is the textual signal
/// complementing the visual red dot. `app.tray_by_id("main-tray")` is the same lookup
/// `refresh_tray` uses (A4 Step 9); `TrayIcon::set_tooltip<Option<S: AsRef<str>>>` is the
/// verified Tauri 2 API (rev-11 verified API facts).
///
/// rev-21-2: the failure tooltip is the PREFIXED `format!("Switch failed: {msg}")`
/// (user-facing contract `"Switch failed: <msg>"`, NOT the raw `msg`). The `log::warn!`
/// line still uses the raw `msg` (logs do not need the user-facing prefix).
pub fn handle_switch_provider(
    app: &tauri::AppHandle,
    app_state: &Arc<AppState>,
    uuid: &str,
) -> Result<(), String> {
    let result = handle_switch_provider_core(app_state, uuid);
    // Use the AppHandle to refresh the tray (menu/icon/tooltip). On success this
    // shows the new primary in the status item; on failure it rebuilds the menu
    // (the prior primary is unchanged — the write tx rolled back).
    let _ = refresh_tray_if_available(app);
    if let Err(ref msg) = result {
        // rev-19-5: AFTER refresh, set the failure tooltip (so refresh's set_tooltip
        // does not overwrite it). Visible textual feedback for the failed switch.
        // rev-21-2: the tooltip carries the "Switch failed: " prefix (user-facing
        // contract: `"Switch failed: <msg>"`); the log line uses the raw `msg`.
        let tooltip = format!("Switch failed: {msg}");
        if let Some(tray) = app.tray_by_id("main-tray") {
            let _ = tray.set_tooltip(Some(&tooltip));
        }
        log::warn!("switch provider failed: {msg}");
    }
    result
}
```

The `tray.switch-<uuid>` arm in `handle_tray_menu_event` (rev-18-1: offload the SYNC wrapper via `spawn_blocking`). **rev-19 P2-1 (dynamic submenu clarification):** the `tray.switch-<uuid>` menu items are NOT a single fixed `MenuItem::with_id(app, "tray.switch-provider", ...)` — they are a DYNAMIC submenu generated per-provider by A4 Step 9's `build_switch_provider_submenu` (L2344-2354): `read_enabled_providers(app)` reads `(uuid, name)` for enabled providers from the db, and for EACH one creates `MenuItem::with_id(app, &format!("tray.switch-{uuid}"), &name, true, None::<&str>)?` inside a `SubmenuBuilder::new(app, "Switch Provider")`. So the arm below matches a different id per provider (`tray.switch-<uuid1>`, `tray.switch-<uuid2>`, …) and `strip_prefix("tray.switch-")` extracts the uuid. `refresh_tray_if_available` (called by the wrapper on both success and failure, + by all 8 provider-mutation commands on their success path) rebuilds the ENTIRE menu including the submenu, so newly created / deleted / renamed providers appear/disappear from the submenu immediately.

```rust
let _ = id.strip_prefix("tray.switch-").map(|uuid| {
    let app2 = app.clone();
    let app_state = app.state::<std::sync::Arc<AppState>>().inner().clone();
    let uuid = uuid.to_owned();
    // rev-18-1: spawn_blocking offloads the SYNC SQLite operation (set_active_primary_core
    // is sync; its body is the blocking gate + tx). NOT spawn(async move { ... .await }).
    tauri::async_runtime::spawn_blocking(move || {
        let _ = handle_switch_provider(&app2, &app_state, &uuid);
    });
});
```

**rev-13/rev-14 / rev-18-1 (clone the Arc before the blocking task):** the switch-provider flow runs the DB mutation inside a blocking task (`tauri::async_runtime::spawn_blocking`). The `tauri::State<'_, Arc<AppState>>` is NOT `Send` and cannot be moved into the closure, so `app_state` is cloned to an owned `Arc<AppState>` (via `.inner().clone()`) BEFORE the closure; the owned `Arc` IS `Send` and is moved into the `move` closure. Inside the wrapper, `begin_switch()` runs FIRST (in the core), then the sync DB call, then `finish_switch`.

The visual-state transitions are inside `finish_switch(rev, success)` (**rev-14/rev-15/rev-16/rev-17/rev-18 SYNC, no `.await`; rev-16-3: revision-tagged, NO `gen` argument, NO overloading; rev-17-4: sole switch mutator; rev-18-1: helper is SYNC**):

- On the FAILURE branch (`set_active_primary_core(...)` returned `Err`): `finish_switch(rev, false)` sets `switch_error_rev = Some(rev)` (IF `rev == switch_revision` — a stale rev is ignored). **rev-16-3 / rev-17-4: `finish_switch` is the SOLE switch mutator** (the low-level `record_switch_error()`/`clear_switch_error()` are DELETED — `finish_switch(rev, false)` = record, `finish_switch(rev, true)` = clear, plus the stale-revision guard). A switch is not a translation, so do NOT `begin_translation`/`TranslationGuard`. The A4 switch-failure tooltip `"Switch failed: <msg>"` stays as the textual signal; the red dot is the visual signal.
- On the SUCCESS branch (`finish_switch(rev, true)` → `switch_error_rev = None` IF `rev == switch_revision`, then `refresh_tray_if_available(&app)`). If a translation is in flight (its own `error_gen` is `None`), `recompute` resolves back to `ActiveTranslation` (the reducer ORs `error_gen.is_some() || switch_error_rev.is_some()`); if a translation's `error_gen` is `Some` (a translation also failed), the red dot PERSISTS until that translation's same-or-newer-gen success clears it (the switch `finish_switch(_, true)` only clears `switch_error_rev`, not `error_gen` — the two flags are independent by design, rev-15 P1-3 / rev-16-3).

> **rev-16 vs rev-15 vs rev-14, + rev-17 + rev-18:** rev-14 acquired `session` + `gen = session.gen.next()` + called `record_error(gen)` / `clear_error_for_gen(gen)` (gen-arg overloads). **rev-15 P1-3 removed the `session`/`gen` acquisition** (calling `gen.next()` stales in-flight translations) and used the no-gen `record_error()` / `clear_error()` overloads (sticky `has_error` flag). **rev-16-1 renames them to DISTINCT methods** (NO overloading — `record_translation_error(gen)` for the translation flow). **rev-16-3 replaces the sticky `has_error: bool` with `switch_revision: u64` + `switch_error_rev: Option<u64>`** so concurrent switch completions are ordered (a stale late switch cannot clobber the latest); the switch mutators are `begin_switch()`/`finish_switch(rev, success)`. **rev-17-4 deletes the dead `record_switch_error()`/`clear_switch_error()`** — `finish_switch` is the sole switch mutator. **rev-16 P2-2 / rev-18-1 extracts `handle_switch_provider` as `pub fn` (SYNC)** so the integration test verifies (functionally) that it does NOT touch the translation `GenerationToken`, and a structural `include_str!` grep test guards against regression. **rev-17-1 (superseded by rev-18-1) made it `async`** based on the wrong premise that `set_active_primary_core` was async — rev-18-1 reverts to the SYNC `pub fn` (rev-16's form) because `set_active_primary_core` is itself SYNC (its body is the `spawn_blocking` payload). The regression tests `switch_handler_does_not_call_gen_next` + `switch_arm_source_has_no_gen_next_call` (Step 2, section 10) verify both (rev-18-1: the structural test ALSO asserts no `.await` / no `spawn(async move` in the switch arm). **rev-14 P1-5 (load-bearing, retained):** the handler signature has only `app`, so `app_state` is acquired via `app.state::<Arc<AppState>>().inner().clone()`.

- [x] **Step 11: Run the full backend test suite + clippy + default build (rev-16 P2-4; rev-17 P2-2 fixes the test count to 32 → rev-17-3 brings it to 33; rev-18-1 SYNC handle_switch_provider + rev-18-3 real-DB functional test; rev-20-2 the structural grep test uses the FULL source window — no `take(4096)` cap; rev-21-1 grep assertion failure messages truncated to first 500 chars of the switch-arm window; rev-21-3 test count reconfirmed as 33 (not 32); rev-22-1 grep preview is UTF-8-safe via `chars().take(500).collect::<String>()` (no byte-slice panic on multi-byte chars); rev-22-2 the grep window is narrowed to brace-matched `extract_function_body` of the three switch functions (core/wrapper/handler) instead of `&src[switch_start..]` to EOF; rev-22-3 SYNC core + SYNC wrapper + switch-arm each asserted independently (no `.await`/`spawn(async move`/`.gen.next()`); rev-22-4 test count reconfirmed as 33 — `extract_function_body` is a local helper fn inside the grep test, NOT a new `#[test]`)**

Run:

```bash
cargo build --manifest-path src-tauri/Cargo.toml --features xproc-test-helper
cargo test --manifest-path src-tauri/Cargo.toml --features xproc-test-helper
cargo clippy --manifest-path src-tauri/Cargo.toml --features xproc-test-helper --all-targets -- -D warnings
# rev-16 P2-4: DEFAULT build (NO feature) — verifies the cfg-gated re-export
# (RecordingRenderer/RenderedIcon) does NOT leak into the production binary
# and the production re-export block resolves without the test feature.
cargo build --manifest-path src-tauri/Cargo.toml
```

Expected: all existing tests pass + the **33** `tray_state` tests pass (rev-19: unchanged from rev-18's 33 — rev-19 rewrites the functional switch test FIXTURE to the `fresh_db` pattern + the no-churn assertion to `worker_start_count`, but adds/removes NO test; rev-18: unchanged from rev-17's 33 = 32 from rev-16 + 1 `stale_gen_error_ignored_after_newer_begin` (rev-17-3); rev-18-1/3: ALL tests are `#[test]` SYNC — the functional switch test was rewritten from `#[tokio::test]`/async to `#[test]`/SYNC against a real DB, no count delta; verified by grepping `^#[test]$`/`^#[tokio::test]` in the Step 2 code block — ALL `#[test]`, 0 `#[tokio::test]`); clippy clean (rev-19-3: the `PulseWorker` struct no longer has an unread `notify` field, so no `dead_code` warning in the prod `notify = None` path); the default `cargo build` (no feature) succeeds (the `#[cfg(any(test, feature = "xproc-test-helper"))]`-gated symbols are NOT compiled, and the always-on `pub use tray_state::{...}` block still resolves). rev-14/rev-15/rev-16/rev-17/rev-18/rev-19 compile checkpoints:
- `parking_lot = "0.12"` is ALREADY a production dep (Cargo.toml:53) — the `tray: Arc<parking_lot::Mutex<TrayStateController>>` field needs NO new runtime dep (Step 1).
- `sys-locale = "0.3"` resolves `sys_locale::get_locale()` in `detect_system_locale()` (Step 1). **rev-15:** adding this runtime dep updates `src-tauri/Cargo.lock` (git-tracked — verified `git ls-files`), so Cargo.lock is committed in Step 12.
- The RUNTIME `tokio` line is UNCHANGED (`["macros", "rt-multi-thread"]`) — rev-14/rev-15/rev-16 use `parking_lot::Mutex` (not `tokio::sync`) + `std::thread` worker (not `tokio::time`), so NO `time`/`sync` runtime features. The DEV `tokio` line has `test-util` (Step 1).
- The `image` dev-dependency resolves the pixel-diff test's `image::open` (Step 1).
- The controller's SYNC `render(&mut self)` writes icon+tooltip through the injected `Arc<dyn TrayRenderer>`; `TrayIconRenderer` (prod, discrete `set_icon_normal`/`set_icon_dimmed`/`set_icon_error_dot`/`set_tooltip` wrapping `TrayIcon`) and `RecordingRenderer` (test, **rev-15 P1-2: `#[cfg(any(test, feature = "xproc-test-helper"))]`-gated** — NOT `#[cfg(test)]`, which is invisible to the integration-test crate) both implement `TrayRenderer`; `RenderedIcon` is `Clone + Debug + PartialEq + Eq` (Step 5).
- `TrayStateController::with_renderer_and_interval` / `with_renderer_interval_and_notify` are `pub` so the test crate can construct the controller with `RecordingRenderer` + a tiny interval (+ notify channel); `new(app)` is the prod entry; `with_renderer` is the 800ms prod-interval test entry (Step 5).
- `TranslationGuard::new(&Arc<parking_lot::Mutex<...>>, gen)` compiles and its SYNCHRONOUS `Drop` runs **rev-15/rev-16: `finish_translation(gen, succeeded)`** (the merged single call — decrement + clear-on-success-iff-`error_gen <= gen` (rev-16-2 guard) + recompute) on the calling thread; `mark_success` sets the flag (Step 5).
- **rev-15 P1-1 / rev-16 P2-1 / rev-17-2: `PulseWorker` + `PulseEvent`** compiles — `mpsc::channel()` + `recv_timeout` loop + per-tick `notify.send(PulseEvent::Tick)` (rev-17-2: was `send(())`) + on-exit `notify.send(PulseEvent::Stopped)` (rev-17-2) + `stop()` (send + join) + `Drop`; the controller's `recompute` does `pulse_worker.take()` on leaving Active (Drop → stop → send + join — NO deadlock, the worker exits on the signal). NO `spawn_pulse_timer`, NO `stop_timer()`, NO `visual_epoch`, NO `tick_render()` (all removed in rev-15 P1-4). `PulseWorker::start` signature is `(renderer, interval, notify: Option<Sender<PulseEvent>>)` (rev-17-2: `PulseEvent`, was `Sender<()>`).
- **rev-16 P1-1 / rev-17-4: NO function overloading + NO dead switch mutators** — the controller exposes `record_translation_error(gen)` (translation flow, `error_gen`) + `begin_switch()`/`finish_switch(rev, success)` (switch flow, `switch_revision`/`switch_error_rev`). There are NO two methods named `record_error` (the rev-15 overload that did not compile). **rev-17-4: `record_switch_error()`/`clear_switch_error()` are DELETED** — `finish_switch` is the sole switch mutator. `recompute_pure` ORs `error_gen.is_some() || switch_error_rev.is_some()` for `Error`.
- **rev-16-2 / rev-17-3 gen guards:** `finish_translation(gen, true)` clears `error_gen` ONLY if `error_gen <= gen`; `record_translation_error(gen)` sets `error_gen` ONLY if `gen >= latest_translation_gen` (rev-17-3) AND `gen >= error_gen` (rev-16-2). The `older_success_does_not_clear_newer_error` + `older_error_does_not_replace_newer_error` + `stale_gen_error_ignored_after_newer_begin` (rev-17-3 NEW) tests (Step 2) verify all three.
- **rev-16-3 switch revision:** `begin_switch()` bumps `switch_revision` and returns it; `finish_switch(rev, success)` IGNORES stale `rev != switch_revision`. The `two_concurrent_switches_second_wins` + `stale_switch_result_ignored` tests (Step 2) verify the ordering.
- **rev-16 P2-2 / rev-18-1: switch does NOT call `session.gen.next()`** — the extracted SYNC core `pub fn handle_switch_provider_core(app_state, uuid)` + wrapper `pub fn handle_switch_provider(app, app_state, uuid)` (rev-18-1: SYNC, `&` borrows; the core takes NO AppHandle) acquire ONLY `app_state` (NOT `Session`, NOT `gen`) and use `begin_switch()`/`finish_switch(rev, success)`; the wrapper runs inside `tauri::async_runtime::spawn_blocking` (rev-18-1: offload the SYNC `set_active_primary_core` SQLite I/O — NOT `spawn(async move { ... .await })`). The functional `switch_handler_does_not_call_gen_next` test (calls the core directly — NO AppHandle) + the structural `switch_arm_source_has_no_gen_next_call` (which reads `lib.rs` via `include_str!` and asserts the switch arm has no `.gen.next()` / `session.gen` / `.await` / `spawn(async move`) verify this (Step 2, section 10).
- Both `include_bytes!(concat!(env!("OUT_DIR"), "/tray-error-32.png"))` and `.../tray-active-32.png"` resolve because Step 4 generated both files during build (Step 4).
- The 5 `AppState { ... }` construction sites (lib.rs:2513/2597/2620; recovery.rs:42/248) each have the `tray:` field — the prod site uses `parking_lot::Mutex::new(new(app.handle().clone()))`, the 4 test sites use `parking_lot::Mutex::new(with_renderer(RecordingRenderer, Locale::En))` (Step 6).
- The `app_state.tray.lock()` callsites in Steps 8-10 compile against the `Arc<parking_lot::Mutex<TrayStateController>>` field — SYNC, no `.await` (Steps 8-10).
- **rev-16-3 / rev-18-1: the switch-provider arm (extracted SYNC `handle_switch_provider` wrapper, which delegates to the SYNC `handle_switch_provider_core`) acquires ONLY `app_state` via `app.state::<Arc<AppState>>().inner().clone()`** (NOT `Session`, NOT `gen`) before the `tauri::async_runtime::spawn_blocking` (Step 10). The translation flows (Steps 8-9) still use the gen-tagged `record_translation_error(gen)` (rev-16-1 renamed) via the guard.
- `TrayStateController` does NOT derive `Debug` (rev-14 P2: holds `Arc<dyn TrayRenderer>`) — no `#[derive(Debug)]` on it.
- `detect_system_locale()` is `pub` and used at controller construction via `sys_locale::get_locale()` (NOT at each call site — Step 6).
- **rev-15 P1-2 + rev-16 P2-4 + rev-17-2: the `lib.rs` re-export splits into two blocks** — `#[cfg(any(test, feature = "xproc-test-helper"))] pub use tray_state::{RecordingRenderer, RenderedIcon};` (test-only) and an always-on `pub use tray_state::{...PulseEvent, PulseWorker...}` block (rev-17-2: `PulseEvent` added). Both resolve under `--features xproc-test-helper`; ONLY the always-on block resolves under `cargo build` (no feature) — the **rev-16 P2-4 default-build step verifies this** (a production binary must NOT link `RecordingRenderer`).
- **rev-19-1: `controller_with_notify()` passes `Some(notify_tx)`** (not a bare `Sender`) to `with_renderer_interval_and_notify` — the 4th param is `Option<mpsc::Sender<PulseEvent>>`, so a bare `Sender` would be a type-mismatch compile error. The three delegating constructors (`new`/`with_renderer`/`with_renderer_and_interval`) all pass `None` (rev-18-2 still holds).
- **rev-19-2: the functional switch test fixture runs `schema::create_all_tables` + `schema::seed_singletons` inside a transaction AFTER `Database::open` and BEFORE `db_providers::create`** — `Database::open` does NOT create tables (db/mod.rs:93), so without this the test panics "no such table: providers" (mirrors tests/provider_crud.rs:21-34 `fresh_db`). `db_providers::create` takes `&mut Connection` (db/providers.rs:357), so it is called inside `db.with_conn(|conn| ...)`.
- **rev-19-3: `PulseWorker` struct has ONLY `stop_tx` + `handle`** (NO `notify` field) — the `notify` Sender is moved into the worker thread closure at `start`, so the struct has no unread field and no `dead_code` warning in the prod `notify = None` path. `start`'s `Self { stop_tx, handle: Some(handle) }` literal has no `notify`.
- **rev-19-4: `TrayStateController` has a `worker_start_count: u32` field** (initial 0), incremented by `recompute` inside the `new_state == ActiveTranslation` branch (after `PulseWorker::start`), and a `pub fn worker_start_count(&self) -> u32` accessor. The `second_begin_does_not_churn_the_worker` test asserts `worker_start_count` stays at 1 across the second `begin_translation` (the Active→Active bump hits `recompute`'s early return and never reaches the increment).
- **rev-19-5 + rev-21-2: `handle_switch_provider` wrapper sets the failure tooltip AFTER `refresh_tray_if_available`** — `if let Err(ref msg) = result { let tooltip = format!("Switch failed: {msg}"); if let Some(tray) = app.tray_by_id("main-tray") { let _ = tray.set_tooltip(Some(&tooltip)); } log::warn!(...); }` (rev-21-2: tooltip is the PREFIXED `format!("Switch failed: {msg}")`, NOT the raw `msg`; the `log::warn!` line still uses the raw `msg`). The order (refresh THEN set tooltip) ensures the refresh's own `set_tooltip` does not clobber the failure message. The functional test does NOT exercise the wrapper (it calls the core, which has no AppHandle) — the tooltip behavior is covered by the A4 Step 9 spec + rev-19-5/rev-21-2 doc, not by a separate test.
- **rev-19 P2-1: the `switch_arm_source_has_no_gen_next_call` structural test ALSO asserts `build_switch_provider_submenu` + the `tray.switch-{uuid}` format string exist in lib.rs** — guards against a regression that replaces the dynamic per-provider submenu with a single fixed `MenuItem::with_id(app, "tray.switch-provider", ...)`.

- [x] **Step 12: Commit**

```bash
git diff --check
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/build.rs src-tauri/src/tray_state.rs src-tauri/src/lib.rs src-tauri/tests/tray_state.rs src-tauri/tests/recovery.rs
git commit -m "feat(tray): sync handle_switch_provider + spawn_blocking + real-DB switch test + deterministic PulseWorker tests (rev-18, A5)"
```

> **rev-18 note on the commit scope:** `src-tauri/tests/recovery.rs` is added (rev-12 omitted it) because Step 6 adds the `tray:` field to its TWO `AppState` construction sites (lines 42 + 248). **`src-tauri/Cargo.lock` is added** because it is git-tracked (verified `git ls-files`) and the `sys-locale = "0.3"` runtime dependency updates it (retained from rev-14). The commit message reflects the rev-18 load-bearing changes: **rev-18-1** (SYNC `pub fn handle_switch_provider` — rev-17-1's `async` was based on the wrong premise that `set_active_primary_core` was async; it is SYNC, so the caller offloads it via `tauri::async_runtime::spawn_blocking`), **rev-18-2** (`controller_with_notify` constructors initialize `notify_tx` — confirmed all delegating constructors pass `None`), **rev-18-3** (real-DB functional switch test — temp DB + inserted provider + assertions on `read_active_selection`/`switch_error_rev`/`current_state`, no mock controller), **rev-18-4** (`notify_for_thread` is used in the worker closure — no dead_code), **rev-18-5** (deterministic PulseWorker tests — `match` the `recv_timeout` result against `PulseEvent::Tick`/`Stopped`; `drop_stops_the_worker` asserts `Stopped` not `Disconnected`), **rev-18-6** (`record_switch_error`/`clear_switch_error` = 0 in active code — only historical changelog references). The rev-17 architecture direction is RETAINED: `PulseEvent { Tick, Stopped }` (rev-17-2), `latest_translation_gen` guard (rev-17-3), delete dead switch mutators (rev-17-4). The rev-14/rev-15/rev-16 base is retained: synchronous `parking_lot::Mutex`, `PulseWorker` channel-quit, `RecordingRenderer` `#[cfg(any(test, feature = "xproc-test-helper"))]` visibility, single timer model (no `visual_epoch`), `sys-locale`, switch revision (`begin_switch`/`finish_switch`), gen guards (`error_gen <= gen` / `gen >= error_gen`).

---

### Stage A Verification

Run before starting Stage B:

```bash
cargo build --manifest-path src-tauri/Cargo.toml --features xproc-test-helper
cargo test --manifest-path src-tauri/Cargo.toml --features xproc-test-helper
cargo clippy --manifest-path src-tauri/Cargo.toml --features xproc-test-helper --all-targets -- -D warnings
pnpm test
pnpm typecheck
pnpm build
```

Confirm:
- All Stage A tests pass (`hotkey_session`, `popup_geometry`, `theme`, `tray-action`, `popupGeometry`, `capabilities`, **`tray_state` (rev-11/rev-12/rev-13/rev-14/rev-15/rev-16/rev-17/rev-18 — 33 tests, ALL `#[test]` SYNC)**).
- `src/index.tsx`, `src/popup-entry.tsx`, `src/input-entry.tsx` each call `initTheme()` before `render`.
- `on_hotkey` no longer references `translate_with_fallback` (the grep test in `hotkey_session.rs` passes).
- `capture_and_translate` checks `state.gen.is_latest(gen)` at every await boundary (P1-1).
- `run_translate_session` resolves `to: ""` centrally (the `resolve_target_language` test passes).
- `tauri.conf.json`'s first window has `"visible": false`.
- `src-tauri/build.rs` lists `translate_session`, `translate_selection_ipc`, `provider_get_active_selection`, `open_settings_window`.
- `src-tauri/Cargo.toml` has `tauri-plugin-clipboard-manager = "2"`, and (rev-11/rev-12/rev-13) `image = { version = "0.25", ... }` under `[build-dependencies]`, `image` duplicated under `[dev-dependencies]` (rev-13: the pixel-diff test), and (rev-13) `tokio = { version = "1", features = ["macros", "rt-multi-thread", "time", "sync"] }` in `[dependencies]`.
- `src-tauri/permissions/autogenerated/` contains the four new TOMLs.
- `src-tauri/capabilities/input.json` includes `allow-translate-session` AND `allow-provider-list`.
- The `capabilities.rs` integration test passes.
- The `translate-selection` tray action routes through `translateSelection`, NOT `translate_clipboard`.
- The `navigate` listener controls `SettingsShell.activePage`.
- The tray Switch Provider submenu is built from the db and calls `set_active_primary_core` (P1-5).
- **rev-8-8:** every provider mutation command (`provider_create`/`provider_update`/`provider_delete`/`provider_toggle`/`provider_reorder`/`provider_set_active`/`provider_duplicate`/`provider_confirm_and_set_active` — EIGHT, not six) gains an `app_handle: tauri::AppHandle` parameter, renames its local `app` (AppState clone) to `app_state`, and calls `refresh_tray_if_available(&app_handle)` on its success path; the tray menu's `set_menu(Some(menu))` call type-checks; `read_enabled_providers` maps `DbErr`.
- **rev-14/rev-15/rev-16/rev-17 (A5):** `AppState` carries `tray: Arc<parking_lot::Mutex<TrayStateController>>` at ALL 5 construction sites (lib.rs:2513/2597/2620; recovery.rs:42/248) — synchronous `parking_lot::Mutex` (NOT `tokio::sync::Mutex`), so `TranslationGuard::drop` runs **rev-15/rev-16: `finish_translation(gen, succeeded)`** SYNCHRONOUSLY on the calling thread (true RAII — the merged single call: decrement + clear-on-success-iff-`error_gen <= gen` (rev-16-2 guard) + recompute); `capture_and_translate` + `translate_clipboard` create a `TranslationGuard` AFTER preflight (rev-13 P1-2: RAII — no manual end), call `guard.mark_success()` on success (→ Drop calls `finish_translation(gen, true)`) + `app_state.tray.lock().record_translation_error(gen)` (**rev-16-1 renamed from `record_error(gen)`**; gen-guarded set per rev-16-2 + `latest_translation_gen` guard rev-17-3) on error (→ Drop calls `finish_translation(gen, false)`) (all SYNC, no `.await`); **rev-16-3 / rev-18-1: the extracted SYNC core `pub fn handle_switch_provider_core(app_state, uuid)` (NO AppHandle — testable) + wrapper `pub fn handle_switch_provider(app, app_state, uuid)` (rev-16 P2-2; rev-18-1: SYNC — `set_active_primary_core` is SYNC, no `.await`; rev-17-1's `async` was based on the wrong premise it was async) acquire ONLY `app_state = app.state::<Arc<AppState>>().inner().clone()`** (NOT `Session`, NOT `gen = session.gen.next()` — calling `next()` stales in-flight translations, verified concurrency.rs), then call `tray.lock().begin_switch()` → captures `rev` → `set_active_primary_core(app_state.clone(), uuid.to_string())` (SYNC) → `tray.lock().finish_switch(rev, success)` (rev-16-3: stale `rev != switch_revision` is ignored); the tray.switch arm runs the SYNC wrapper via `tauri::async_runtime::spawn_blocking` (offload the SYNC SQLite I/O — NOT `spawn(async move { ... .await })`); the controller's SYNC methods are **rev-16-1 / rev-17-4: NO overloading + NO dead switch mutators — distinct names** — translation flow `begin_translation(gen)` (rev-17-3: bumps `latest_translation_gen`)/`finish_translation(gen, success)` (gen-guarded, rev-16-2)/`record_translation_error(gen)` (gen-guarded set, rev-16-2 + `latest_translation_gen` guard rev-17-3), switch flow `begin_switch()`/`finish_switch(rev, success)` (NO gen, uses `switch_revision`/`switch_error_rev` — rev-16-3 replaces rev-15's sticky `has_error: bool`; **rev-17-4: `record_switch_error()`/`clear_switch_error()` DELETED — finish_switch is the sole switch mutator**) — locale is captured at construction via `detect_system_locale()` (rev-14 P2: uses `sys_locale::get_locale()`, NOT `std::env::var("LANG")`); `recompute()` only swaps the `PulseWorker` when `new_state != current_state` (rev-14 P1-2 / rev-15/rev-16/rev-17: Active → Active does NOT churn the worker); `recompute_pure` ORs `error_gen.is_some() || switch_error_rev.is_some()` → `Error` (rev-16-3); all state-transition icon writes go through the sync `render(&mut self)` (called by `recompute` inside the `&mut self` lock); **rev-15 P1-1 / rev-16 P2-1 / rev-17-2: the pulse worker is a `PulseWorker`** — `mpsc::channel()` + worker body loops on `recv_timeout(interval)` (`Ok`/`Disconnected`→emit `PulseEvent::Stopped`+return, `Err(Timeout)`→toggle dimmed/normal via the worker's own `Arc<dyn TrayRenderer>` clone + `notify.send(PulseEvent::Tick)` per tick (rev-16 P2-1 / rev-17-2)); `PulseWorker::stop()` = `stop_tx.send(())` + `handle.take().join()` (the worker returns on the signal so `join` completes — NO infinite-loop + join deadlock, the rev-14 bug); `impl Drop for PulseWorker` calls `stop()`; the controller holds `pulse_worker: Option<PulseWorker>`, leaving Active = `pulse_worker.take()` (Drop → stop); **rev-15 P1-4: NO `visual_epoch`, NO `tick_render()`, NO `stop_timer()`** (rev-14 prose described an in-timer epoch check the `spawn_pulse_timer` code never performed — prose and code disagreed; rev-15 keeps only the code model: the worker holds an independent renderer clone, the channel-quit is the sole barrier); **rev-15 P1-2: `RecordingRenderer` + `RenderedIcon` are `#[cfg(any(test, feature = "xproc-test-helper"))]`-gated** (NOT `#[cfg(test)]`, which is invisible to the integration-test crate `src-tauri/tests/tray_state.rs`); the `lib.rs` re-export splits into a cfg-gated test-only block + an always-on block (rev-17-2: the always-on block ALSO re-exports `PulseEvent`); `TrayStateController` does NOT derive `Debug` (rev-14 P2: holds `Arc<dyn TrayRenderer>`); the `tray_state` integration test passes (33 tests, rev-17: priority + reducer + guard + generation + gen-guards + `latest_translation_gen` (rev-17-3) + PulseWorker alternating-frames (rev-17-2 `PulseEvent::Tick` notify, NO `thread::sleep`) + PulseWorker channel-quit (rev-17 P2-4 assert `PulseEvent::Stopped` then join) + worker-stop barrier (rev-17-2 `PulseEvent::Stopped`) + worker no-churn + switch-flow switch_error_rev independence (rev-16-3 renamed, rev-17-4 uses `finish_switch`) + switch-revision ordering (rev-16-3 NEW) + switch-does-not-bump-generation functional (rev-18-3: REAL SYNC core `handle_switch_provider_core(&app_state, &uuid)` — NO AppHandle — against a real temp DB + inserted provider, `#[test]` — NOT `#[tokio::test]`) + structural grep (rev-16 P2-2 / rev-18 P2-4: ALSO asserts no `.await`/`spawn(async move`/`pub async fn handle_switch_provider`) + localization + pixel-diff); `target/.../out/tray-error-32.png` (red-dot OVERLAY on the base icon, asserted by the pixel-diff test which `panic!`s if the file is missing — rev-14 P2) AND `target/.../out/tray-active-32.png` (dimmed pulse variant) were generated by `build.rs` and embedded via `include_bytes!`; the `PulseWorker` starts on entering `ActiveTranslation` (via `PulseWorker::start`) and is dropped on leaving (`take()` → Drop → stop → send + join); `UpdateAvailable` is present but never produced by `recompute`; `build.rs`'s `use` list is `image::{ImageBuffer, Rgba}` (NO unused `imageops`); **rev-15: `src-tauri/Cargo.lock` is in the A5 commit** (git-tracked, updated by `sys-locale = "0.3"`); **rev-16 P2-4: `cargo build` (NO feature) succeeds** — the cfg-gated re-export does NOT leak `RecordingRenderer` into the production binary, and the always-on `pub use tray_state::{...}` block resolves without the test feature.
- Clippy is clean (`-D warnings`).

Stop here. Do not begin Stage B until the reviewer signs off.

---

## Stage B: Surface 01-04 Contracts

Checkpoint goal: the popup renders every state with friendly engine labels and working Copy/Retry (saved SOURCE text, available even in error/loading)/settings/recovery actions; the input window persists drafts and renders multi/partial/all-failed with friendly labels; parallel results arrive in strict input order with bounded, local-sacred-aware fallback (that can actually trigger because primary failures preserve their original Error) and a single fallback result card per session.

### Task B1: InputPanel multi-engine rendering + friendly engine labels

**Files:**
- Modify: `src/InputPanel.tsx` — render `multi-success`, `partial`, all-failed; show friendly engine labels (not secret_ref/uuid).
- Create: `src/features/translation/inputController.ts` — the friendly-label map for InputPanel.
- Test: `test/InputPanel.test.tsx` — extend.

**Interfaces:**
- Consumes: `TranslationState` from `src/features/translation/types.ts` (the `multi-success`/`partial` variants carry `results: ResultEntry[]`).
- Produces: no new exports; InputPanel renders a `ResultCard` grid for multi/partial, with `engineLabel` resolved via the input controller's name map.

- [x] **Step 1: Write the failing tests (rev-6-5: unified routeInvoke, NO mockResolvedValueOnce)**

Append to `test/InputPanel.test.tsx`. The test uses the `vi.hoisted + invokeMock + routeInvoke` pattern consistent with `ProviderCenter.test.tsx`. **rev-6-5 (load-bearing):** do NOT mix `mockResolvedValueOnce` (consumed in declaration order, not call order) with the route table — `provider_list` is consumed at mount, so a `mockResolvedValueOnce` for the session result would be eaten by the mount-time `provider_list` call. Every `invoke` is answered by its COMMAND NAME via the route table.

```ts
import { vi } from "vitest";

const { inputInvokeMock } = vi.hoisted(() => ({
  inputInvokeMock: vi.fn(async (_cmd: string, _args?: unknown): Promise<unknown> => {
    throw new Error(`unexpected invoke ${_cmd}`);
  }),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: inputInvokeMock }));

/**
 * rev-6-5: wire `invoke` to a route table keyed by command name. Every invoke
 * is answered by its command, regardless of call order (provider_list at mount,
 * translate_session on Enter). NO mockResolvedValueOnce anywhere.
 */
function routeInputInvoke(routes: Record<string, (args?: unknown) => unknown>): void {
  inputInvokeMock.mockImplementation(async (cmd: string, args?: unknown) => {
    const fn = routes[cmd];
    if (!fn) throw new Error(`unexpected invoke ${cmd}`);
    return fn(args);
  });
}

it("renders multi-success ResultCards with friendly engine labels", async () => {
  routeInputInvoke({
    provider_list: () => [
      { uuid: "u1", name: "My OpenAI", secret_ref: "provider/u1" },
      { uuid: "u2", name: "My Anthropic", secret_ref: "provider/u2" },
    ],
    translate_session: () => ({
      outcomes: [
        { uuid: "u1", ok: true, text: "你好", engine: "provider/u1" },
        { uuid: "u2", ok: true, text: "您好", engine: "provider/u2" },
      ],
      actual_engine: undefined,
    }),
  });
  const { findByText } = render(() => <InputPanel />);
  const textarea = document.querySelector("textarea")!;
  fireEvent.input(textarea, { target: { value: "hello" } });
  fireEvent.keyDown(textarea, { key: "Enter" });
  expect(await findByText("你好")).toBeTruthy();
  expect(await findByText("您好")).toBeTruthy();
  expect(await findByText("My OpenAI")).toBeTruthy();
  expect(await findByText("My Anthropic")).toBeTruthy();
  expect(document.body.textContent).not.toContain("provider/u1");
  cleanup();
});

it("renders all-failed InlineError when every engine fails", async () => {
  routeInputInvoke({
    provider_list: () => [],
    translate_session: () => ({
      outcomes: [{ uuid: "u1", ok: false, error: "network" }],
      actual_engine: undefined,
    }),
  });
  const { findByText } = render(() => <InputPanel />);
  const textarea = document.querySelector("textarea")!;
  fireEvent.input(textarea, { target: { value: "hello" } });
  fireEvent.keyDown(textarea, { key: "Enter" });
  expect(await findByText(/网络错误|Network error/)).toBeTruthy();
  cleanup();
});

it("renders a partial result (one ok, one failed)", async () => {
  routeInputInvoke({
    provider_list: () => [],
    translate_session: () => ({
      outcomes: [
        { uuid: "u1", ok: true, text: "你好", engine: "provider/u1" },
        { uuid: "u2", ok: false, error: "config-401" },
      ],
      actual_engine: undefined,
    }),
  });
  const { findByText } = render(() => <InputPanel />);
  const textarea = document.querySelector("textarea")!;
  fireEvent.input(textarea, { target: { value: "hello" } });
  fireEvent.keyDown(textarea, { key: "Enter" });
  expect(await findByText("你好")).toBeTruthy();
  expect(await findByText(/授权|auth/i)).toBeTruthy();
  cleanup();
});
```

> If `test/InputPanel.test.tsx` already has a `vi.mock("@tauri-apps/api/core", ...)` + a `routeInvoke`-style helper, reuse the existing hoisted mock name + helper rather than introducing a second. The structural requirement (rev-6-5 + P1-7) is: one hoisted mock per file, route-table style, NO `mockResolvedValueOnce`.

- [x] **Step 2: Run tests to verify they fail**

Run: `pnpm vitest run test/InputPanel.test.tsx`
Expected: FAIL — InputPanel today only renders `single-success` and passes the raw `s.engine` as `engineLabel`.

- [x] **Step 3: Create the inputController friendly-label map**

Create `src/features/translation/inputController.ts`:

```ts
import { onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

const nameMap = new Map<string, string>();
let loaded = false;

export function ensureProviderNameMap(): void {
  onMount(async () => {
    if (loaded) return;
    try {
      const profiles = await invoke<{ uuid: string; name: string }[]>("provider_list");
      for (const p of profiles) nameMap.set(p.uuid, p.name);
      loaded = true;
    } catch {
      // Best-effort; labels fall back below.
    }
  });
}

const PRESET_LABELS: Record<string, string> = {
  openai: "OpenAI",
  anthropic: "Anthropic",
  gemini: "Gemini",
  ollama: "Ollama",
};

export function engineLabel(raw: string): string {
  if (nameMap.has(raw)) return nameMap.get(raw)!;
  if (raw.startsWith("provider/")) {
    const uuid = raw.slice("provider/".length);
    if (nameMap.has(uuid)) return nameMap.get(uuid)!;
  }
  if (PRESET_LABELS[raw]) return PRESET_LABELS[raw];
  if (["google", "deepl", "microsoft", "baidu", "youdao", "tencent"].includes(raw)) {
    return "Fallback";
  }
  return "Unknown";
}
```

- [x] **Step 4: Extend InputPanel to render multi/partial/all-failed + friendly labels (rev-7-3: extract InputPanelView)**

Edit `src/InputPanel.tsx`. **rev-7-3 (load-bearing for D5):** extract the presentational body into a named export `InputPanelView` so the ui-lab visual fixture (D5) can render the SAME View with canned props, instead of an approximate redraw. The View is a pure function of its props (no signals, no `invoke`, no `localStorage`); the default export owns the controller state. **rev-8-6:** merge the existing solid-js import with the newly-needed `For` + `type JSX` into ONE import line (the existing line 1 is `import { createSignal, createMemo, Show, type Component } from "solid-js";` — add `For` + `type JSX`):

```ts
// rev-8-6: MERGED import (was `import { createSignal, createMemo, Show, type Component } from "solid-js";`).
// InputPanelView needs For (the multi grid) + JSX (the return type). Replace line 1 with:
import { createSignal, createMemo, Show, For, type Component, type JSX } from "solid-js";
import { ensureProviderNameMap, engineLabel } from "./features/translation/inputController";
```

Inside the component, before the return, call the name-map loader and add the multi + allFailed memos:

```ts
ensureProviderNameMap();

const multi = createMemo(() => {
  const s = state();
  return s.kind === "multi-success" || s.kind === "partial" ? s.results : null;
});
const allFailedMessage = createMemo(() => {
  const s = state();
  if (s.kind !== "error") return null;
  return s.sub === "network" ? t("selection.error.network")
    : s.sub === "config-key" ? t("selection.error.config.key")
    : s.sub === "config-401" ? t("selection.error.config.auth")
    : s.message;
});
```

Update the single `ResultCard` to use `engineLabel`, then add the multi grid + the all-failed InlineError inside the returned JSX (after the single-success Show):

```tsx
      <Show when={single()} keyed>
        {(s) => (
          <ResultCard
            engineId={s.engine}
            engineLabel={engineLabel(s.engine)}
            text={s.text}
            outcome={"success" as ResultOutcome}
          />
        )}
      </Show>

      <Show when={multi()} keyed>
        {(results) => (
          <div class="input-results" data-multi="true">
            <For each={results}>
              {(r) => (
                <ResultCard
                  engineId={r.uuid}
                  engineLabel={engineLabel(r.uuid)}
                  text={r.text ?? ""}
                  outcome={(r.ok ? "success" : "failure") as ResultOutcome}
                  errorText={r.errorText}
                />
              )}
            </For>
          </div>
        )}
      </Show>

      <Show when={allFailedMessage()} keyed>
        {(msg) => (
          <InlineError icon={<AlertTriangle size={16} />}>
            <span>{msg}</span>
          </InlineError>
        )}
      </Show>
```

(`AlertTriangle` is imported from `lucide-solid` at `src/InputPanel.tsx:3`; `InlineError` + `ResultCard` + `ResultOutcome` are imported from `@linguaray/ui` at `src/InputPanel.tsx:4` — all four symbols are already in the file's import list, so no import edit is needed.)

**rev-7-3: expose `InputPanelView` + its prop type** so D5's ui-lab fixture can import it. Add at the bottom of `src/InputPanel.tsx`. The View is a COMPLETE, compilable presentational body (no `// ...`): it renders the textarea + actions + the single/multi/all-failed Show blocks, reading entirely from `props.*`. `onTranslate` is the primary-button + Enter handler; `engineLabel` defaults to identity so the View never calls `invoke`; error-message copy is resolved locally via `t(...)` (pure derivation from `props.state` — no signals, no IPC, no localStorage).

```tsx
/** rev-7-3: pure presentational View. Shared by the production InputPanel mount
 * (src/InputPanel.tsx default export) + the ui-lab visual fixture
 * (apps/ui-lab/src/pages/InputPanel.tsx). No signals, no invoke, no localStorage. */
export type InputPanelViewProps = {
  text: string;
  state: TranslationState;
  idle: boolean;
  hasResult?: boolean;
  engineLabel?: (raw: string) => string;
  onText: (v: string) => void;
  onTranslate: () => void;
  onClear: () => void;
};

export function InputPanelView(props: InputPanelViewProps): JSX.Element {
  const labelOf = (raw: string) => (props.engineLabel ?? ((r: string) => r))(raw);
  // rev-8-6 (load-bearing): showClear is a DERIVATION (a function of props), not
  // a value read once at mount. `const showClear = props.hasResult ?? false;`
  // would capture the value at first render and never update when the parent
  // passes a new hasResult (e.g. after the first translation resolves). Making
  // it a function keeps it reactive in Solid's fine-grained model.
  const showClear = () => props.hasResult ?? false;

  // Derive the single-success snapshot, the multi/partial results, and the
  // all-failed message PURELY from props.state (no signals).
  const single = createMemo(() => {
    const s = props.state;
    return s.kind === "single-success" ? { engine: s.engine, text: s.text } : null;
  });
  const multi = createMemo(() => {
    const s = props.state;
    return s.kind === "multi-success" || s.kind === "partial" ? s.results : null;
  });
  const errorMessage = createMemo(() => {
    const s = props.state;
    if (s.kind === "error") {
      return s.sub === "network" ? t("selection.error.network")
        : s.sub === "config-key" ? t("selection.error.config.key")
        : s.sub === "config-401" ? t("selection.error.config.auth")
        : s.message;
    }
    if (s.kind === "offline") return t("input.error.offline");
    if (s.kind === "no-permission") return t("selection.error.noPermission");
    if (s.kind === "keystore-corrupt") return t("selection.error.keystore");
    return null;
  });

  const onKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      props.onTranslate();
    }
  };

  return (
    <main class="container" style={{ padding: "var(--space-3, 12px)" }}>
      <h2 class="input-title">{t("input.title")}</h2>
      <textarea
        rows={4}
        placeholder={t("input.placeholder")}
        value={props.text}
        disabled={!props.idle}
        onInput={(e) => props.onText(e.currentTarget.value)}
        onKeyDown={onKeyDown}
        aria-label={t("input.title")}
      />
      <div class="input-actions">
        <Button variant="secondary" size="md" onClick={props.onClear} disabled={!showClear()}>
          {t("input.action.clear")}
        </Button>
        <Button
          variant="primary"
          size="md"
          loading={!props.idle}
          loadingLabel={t("selection.loading")}
          onClick={props.onTranslate}
          disabled={!props.text.trim()}
        >
          {t("input.action.translate")}
        </Button>
      </div>

      <Show when={single()} keyed>
        {(s) => (
          <ResultCard
            engineId={s.engine}
            engineLabel={labelOf(s.engine)}
            text={s.text}
            outcome={"success" as ResultOutcome}
          />
        )}
      </Show>

      <Show when={multi()} keyed>
        {(results) => (
          <div class="input-results" data-multi="true">
            <For each={results}>
              {(r) => (
                <ResultCard
                  engineId={r.uuid}
                  engineLabel={labelOf(r.uuid)}
                  text={r.text ?? ""}
                  outcome={(r.ok ? "success" : "failure") as ResultOutcome}
                  errorText={r.errorText}
                />
              )}
            </For>
          </div>
        )}
      </Show>

      <Show when={errorMessage()} keyed>
        {(msg) => (
          <InlineError icon={<AlertTriangle size={16} />}>
            <span>{msg}</span>
          </InlineError>
        )}
      </Show>
    </main>
  );
}
```

The production default export `InputPanel` is the CONTROLLER — it owns the signals + IPC + autosave (B2) and renders `<InputPanelView .../>`. rev-7-3: it passes `onTranslate={translate}` so the View's primary button + Enter both drive the controller's `translate()`:

```tsx
// src/InputPanel.tsx — the default export is the CONTROLLER (owns state/IPC/autosave).
const InputPanel: Component = () => {
  detectLocale();
  const [text, setText] = createSignal("");
  const [state, setState] = createSignal<TranslationState>({ kind: "loading" });
  const [idle, setIdle] = createSignal(true);
  const [hasResult, setHasResult] = createSignal(false);
  ensureProviderNameMap();

  async function translate() {
    const value = text().trim();
    if (!value) return;
    setIdle(false);
    setState({ kind: "loading" });
    try {
      const res = await invoke<SessionResultFE>("translate_session", {
        req: { text: value, from: "auto", to: "" },
      });
      setState(decodeSessionResult(res));
      setHasResult(true);
    } catch (e) {
      setState({ kind: "error", sub: "generic", message: String(e) });
      setHasResult(true);
    } finally {
      setIdle(true);
    }
  }
  const clear = () => {
    setText("");
    setState({ kind: "loading" });
    setHasResult(false);
  };

  return (
    <InputPanelView
      text={text()}
      state={state()}
      idle={idle()}
      hasResult={hasResult()}
      engineLabel={engineLabel}
      onText={setText}
      onTranslate={translate}
      onClear={clear}
    />
  );
};

export default InputPanel;
```

(B2's autosave/restore is layered onto this controller — it does not change the View's prop surface.) This is a pure refactor (no behavior change) that makes the surface reusable in the lab.

- [x] **Step 5: Run tests to verify they pass**

Run: `pnpm vitest run test/InputPanel.test.tsx`
Expected: PASS (existing + 3 new tests).

- [x] **Step 6: Commit**

```bash
git diff --check
git add src/InputPanel.tsx src/features/translation/inputController.ts test/InputPanel.test.tsx
git commit -m "feat(input): render multi/partial/all-failed with friendly engine labels + extract InputPanelView for ui-lab reuse (rev-7-3)"
```

---

### Task B2: Surface 02 contract — autosave, restore, clear, focus, disabled-while-loading, space tokens

**Files:**
- Modify: `src/InputPanel.tsx` — autosave/restore, auto-focus, Clear purges draft, loading disables textarea.
- Test: `test/InputPanel.test.tsx` — extend.

**Interfaces:**
- Produces: no new exports. Behavior:
  - On mount, read `localStorage.getItem("linguaray.input-draft")`; if present, set `text()` and focus + move cursor to end.
  - On `text()` change (debounced 300ms), write to localStorage.
  - `clear()` removes the localStorage key.
  - `onMount` focuses the textarea.
  - `disabled={!idle()}` on the textarea.

- [x] **Step 1: Write the failing tests**

Append to `test/InputPanel.test.tsx`:

```ts
it("restores a saved draft on mount", async () => {
  localStorage.setItem("linguaray.input-draft", "saved draft");
  render(() => <InputPanel />);
  const textarea = document.querySelector("textarea") as HTMLTextAreaElement;
  expect(textarea.value).toBe("saved draft");
  localStorage.removeItem("linguaray.input-draft");
  cleanup();
});

it("persists the draft after 300ms debounce", async () => {
  vi.useFakeTimers();
  render(() => <InputPanel />);
  const textarea = document.querySelector("textarea")!;
  fireEvent.input(textarea, { target: { value: "typing" } });
  expect(localStorage.getItem("linguaray.input-draft")).toBeNull();
  vi.advanceTimersByTime(350);
  expect(localStorage.getItem("linguaray.input-draft")).toBe("typing");
  localStorage.removeItem("linguaray.input-draft");
  vi.useRealTimers();
  cleanup();
});

it("Clear purges the persisted draft", async () => {
  // rev-6-5: use the route table (command-name keyed), NOT mockResolvedValueOnce
  // (which would be consumed by the mount-time provider_list invoke).
  routeInputInvoke({
    provider_list: () => [],
    translate_session: () => ({
      outcomes: [{ uuid: "u1", ok: true, text: "你好", engine: "openai" }],
      actual_engine: "openai",
    }),
  });
  localStorage.setItem("linguaray.input-draft", "leftover");
  const { getByRole } = render(() => <InputPanel />);
  const textarea = document.querySelector("textarea")!;
  fireEvent.input(textarea, { target: { value: "hello" } });
  fireEvent.keyDown(textarea, { key: "Enter" });
  await Promise.resolve();
  fireEvent.click(getByRole("button", { name: /清除|Clear/ }));
  expect(localStorage.getItem("linguaray.input-draft")).toBeNull();
  cleanup();
});

it("focuses the textarea on mount", () => {
  render(() => <InputPanel />);
  const textarea = document.querySelector("textarea")!;
  expect(document.activeElement).toBe(textarea);
  cleanup();
});
```

- [x] **Step 2: Run tests to verify they fail**

Run: `pnpm vitest run test/InputPanel.test.tsx`
Expected: FAIL — no draft restore/persist/clear-purge/focus behavior exists.

- [x] **Step 3: Implement autosave/restore/focus in InputPanel**

Edit `src/InputPanel.tsx`. Add `onMount, onCleanup, createEffect` to the solid-js import. Replace the component's state setup + `clear` with autosave/restore:

```tsx
const DRAFT_KEY = "linguaray.input-draft";
const DEBOUNCE_MS = 300;

const InputPanel: Component = () => {
  detectLocale();
  const [text, setText] = createSignal("");
  const [state, setState] = createSignal<TranslationState>({ kind: "loading" });
  const [idle, setIdle] = createSignal(true);
  const [hasResult, setHasResult] = createSignal(false);

  let textareaRef: HTMLTextAreaElement | undefined;
  let debounceTimer: ReturnType<typeof setTimeout> | undefined;

  onMount(() => {
    const saved = localStorage.getItem(DRAFT_KEY);
    if (saved) setText(saved);
    if (textareaRef) {
      textareaRef.focus();
      const end = saved?.length ?? 0;
      textareaRef.setSelectionRange(end, end);
    }
  });

  createEffect(() => {
    const value = text();
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      if (value) localStorage.setItem(DRAFT_KEY, value);
      else localStorage.removeItem(DRAFT_KEY);
    }, DEBOUNCE_MS);
  });

  onCleanup(() => {
    if (debounceTimer) clearTimeout(debounceTimer);
  });

  async function translate() {
    // unchanged body
  }

  const clear = () => {
    setText("");
    setState({ kind: "loading" });
    setHasResult(false);
    if (debounceTimer) clearTimeout(debounceTimer);
    localStorage.removeItem(DRAFT_KEY);
  };
```

Wire `ref={textareaRef}` on the textarea. Replace the inline `var(--space-3, 12px)` on `<main>` with `var(--space-lg)`:

```tsx
      <main class="container" style={{ padding: "var(--space-lg)" }}>
```

- [x] **Step 4: Run tests to verify they pass**

Run: `pnpm vitest run test/InputPanel.test.tsx`
Expected: PASS.

- [x] **Step 5: Commit**

```bash
git diff --check
git add src/InputPanel.tsx test/InputPanel.test.tsx
git commit -m "feat(input): autosave draft + restore + Clear purge + auto-focus + space token"
```

---

### Task B3: Popup hides `secret_ref` and shows friendly engine labels

**Files:**
- Modify: `src-tauri/src/db/providers.rs` — `ActiveSelection` gains `serde::Serialize`.
- Modify: `src-tauri/src/lib.rs` — add `#[tauri::command] async fn provider_get_active_selection(...)`; register in `invoke_handler!`.
- Modify: `src/features/settings/provider-ipc.ts` — add `providerGetActiveSelection()`.
- Modify: `src/features/settings/provider-types.ts` — add `ActiveSelectionFE`.
- Modify: `src/features/translation/popupController.ts` — load `{ uuid -> name }` map; expose `engineLabel(uuid)`.
- Modify: `src/Popup.tsx` — use `ctrl.engineLabel(uuid)`.
- Test: `test/Popup.test.tsx` — extend.

**Interfaces:**
- Produces (backend): `provider_get_active_selection() -> ActiveSelection { primary: Option<String>, parallel: Vec<String>, fallback: Option<String> }`.
- Produces (frontend): `providerGetActiveSelection(): Promise<ActiveSelectionFE>`. The popup controller builds a `Map<uuid, name>` from `providerList()` and labels unknown uuids as `"Fallback"` or `"Unknown"`.

- [x] **Step 1: Write the failing test (vi.hoisted + invokeMock pattern)**

Append to `test/Popup.test.tsx`. If the file already has a hoisted `invokeMock`, reuse it; otherwise add one consistent with `ProviderCenter.test.tsx`:

```ts
it("renders friendly engine label, not secret_ref/uuid", async () => {
  const { invoke } = await import("@tauri-apps/api/core");
  vi.mocked(invoke).mockImplementation(async (cmd: string) => {
    if (cmd === "provider_list") {
      return [
        { uuid: "u1", name: "My OpenAI", secret_ref: "provider/u1", template_id: "openai", protocol: "openai_chat", endpoint: "", model: null, enabled: true, sort_order: 0, is_local: false, needs_key: true, status: "active", capabilities: { balance: false, quota: false, model_list: false } },
      ];
    }
    if (cmd === "provider_get_active_selection") {
      return { primary: "u1", parallel: [], fallback: null };
    }
    return { outcomes: [], actual_engine: undefined };
  });

  const { findByText } = render(() => <Popup />);
  await emitEvent("popup-state", { status: "result", text: "你好", engine: "provider/u1", source_text: "hello" });
  expect(await findByText("My OpenAI")).toBeTruthy();
  expect(document.body.textContent).not.toContain("provider/u1");
  cleanup();
});
```

- [x] **Step 2: Run test to verify it fails**

Run: `pnpm vitest run test/Popup.test.tsx`
Expected: FAIL — the popup passes `engine` straight through.

- [x] **Step 3: Add the backend read-active-selection IPC**

In `src-tauri/src/db/providers.rs`, `ActiveSelection` (line 649) gains `serde::Serialize`:

```rust
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub struct ActiveSelection {
    pub primary: Option<String>,
    pub parallel: Vec<String>,
    pub fallback: Option<String>,
}
```

In `src-tauri/src/lib.rs`, add below `provider_list` (line 1070):

```rust
/// Read the active selection (primary / parallel / fallback UUIDs). Read-only.
#[tauri::command]
async fn provider_get_active_selection(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<db::providers::ActiveSelection, String> {
    let app = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _gate = app.data_gate.read();
        let db = require_ready_gated(&app, &_gate)?;
        db.with_conn(|conn| db_providers::read_active_selection(conn))
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}
```

Register in `invoke_handler!` after `provider_list`.

- [x] **Step 4: Build + run the Rust suite**

Run: `cargo build --manifest-path src-tauri/Cargo.toml --features xproc-test-helper && cargo test --manifest-path src-tauri/Cargo.toml --features xproc-test-helper`
Expected: clean build; all tests pass. Verify `provider_get_active_selection.toml` exists.

- [x] **Step 5: Add the frontend IPC wrapper + type**

In `src/features/settings/provider-types.ts`:

```ts
export type ActiveSelectionFE = {
  primary: string | null;
  parallel: string[];
  fallback: string | null;
};
```

In `src/features/settings/provider-ipc.ts`:

```ts
import type { ActiveSelectionFE } from "./provider-types";

export const providerGetActiveSelection = (): Promise<ActiveSelectionFE> =>
  invoke<ActiveSelectionFE>("provider_get_active_selection");
```

- [x] **Step 6: Build the friendly-label map in popupController**

Edit `src/features/translation/popupController.ts`. Add the `nameMap` load on mount + the `engineLabel` function (the SOURCE-text saving + `retrySelection` land in B4; here only the label map). Add inside `createPopupController`:

```ts
  const [state, setState] = createSignal<TranslationState>({ kind: "loading" });
  const [pinned, setPinned] = createSignal(false);
  /** uuid → friendly provider name. Loaded once on mount from provider_list. */
  const nameMap = new Map<string, string>();
  const unlisteners: UnlistenFn[] = [];

  onMount(async () => {
    try {
      const profiles = await invoke<{ uuid: string; name: string }[]>("provider_list");
      for (const p of profiles) nameMap.set(p.uuid, p.name);
    } catch {
      // Best-effort: leave the map empty; labels fall back below.
    }
    // existing popup-state / popup-multi-result / onFocusChanged listeners
  });

  const engineLabel = (raw: string): string => {
    if (nameMap.has(raw)) return nameMap.get(raw)!;
    if (raw.startsWith("provider/")) {
      const uuid = raw.slice("provider/".length);
      if (nameMap.has(uuid)) return nameMap.get(uuid)!;
    }
    const presetLabels: Record<string, string> = {
      openai: "OpenAI",
      anthropic: "Anthropic",
      gemini: "Gemini",
      ollama: "Ollama",
    };
    if (presetLabels[raw]) return presetLabels[raw];
    if (["google", "deepl", "microsoft", "baidu", "youdao", "tencent"].includes(raw)) {
      return "Fallback";
    }
    return "Unknown";
  };

  return { state, pinned, pin, unpin, dismiss, engineLabel };
```

- [x] **Step 7: Use engineLabel in Popup.tsx**

In `src/Popup.tsx`, update the two `engineLabel` props to use `ctrl.engineLabel(...)`.

- [x] **Step 8: Run the test to verify it passes**

Run: `pnpm vitest run test/Popup.test.tsx`
Expected: PASS.

- [x] **Step 9: Commit**

```bash
git diff --check
git add src-tauri/src/db/providers.rs src-tauri/src/lib.rs src/features/settings/provider-ipc.ts src/features/settings/provider-types.ts src/features/translation/popupController.ts src/Popup.tsx src-tauri/permissions/autogenerated/provider_get_active_selection.toml test/Popup.test.tsx
git commit -m "fix(popup): show friendly engine labels via provider name map, hide secret_ref"
```

---

### Task B4: Popup action buttons — Copy (Tauri clipboard), Retry (saved SOURCE text available in every state), settings nav, recovery CTA, aria-disabled TTS/Favorite

> **P1-3 (load-bearing):** Retry must be available in the error AND loading states too, not just the result state. So the backend loading/error payloads carry `source_text`, the popup controller saves `lastSource` on loading/error/result/multi, a new session clears the stale `lastSource`, and `buildActions` only renders Retry when `lastSource` is non-empty.

**Files:**
- Modify: `src/features/translation/types.ts` + `copy.ts` — new copy keys.
- Modify: `packages/ui/src/components/ResultCard.tsx` — forward `aria-disabled` (focusable).
- Modify: `package.json` — add `@tauri-apps/plugin-clipboard-manager` (npm; Cargo dep landed in A4).
- Modify: `src/features/translation/popupController.ts` — save `payload.source_text` as `lastSource` on loading/error/result/multi; clear on new session; `retrySelection()` calls `translateSelection(lastSource)`; clipboard translate saves raw clipboard text.
- Modify: `src/Popup.tsx` — Copy via Tauri clipboard + Copied feedback; Retry → `ctrl.retrySelection()` (only when `lastSource` non-empty); config error → open settings; keystore-corrupt → recovery CTA; TTS/Favorite `aria-disabled`.
- Test: `test/Popup.test.tsx` — extend.

**Interfaces:**
- Produces (frontend):
  - Copy uses `writeText` from `@tauri-apps/plugin-clipboard-manager`; on success sets `copiedUuid` for 1.2s.
  - Retry calls `ctrl.retrySelection()` → `translateSelection(lastSource)`. Only rendered when `lastSource` is non-empty (P1-3).
  - Config error → `invoke("open_settings_window", { section: "provider-center" })`.
  - Keystore-corrupt → `invoke("open_settings_window", { section: "keystore-recovery" })`.
  - TTS/Favorite render with `aria-disabled="true"` (focusable, NOT native `disabled`).

- [x] **Step 0: Add the clipboard plugin npm dep (REQUIRED — no navigator.clipboard fallback, P1-6)**

```bash
pnpm add @tauri-apps/plugin-clipboard-manager
```

The Cargo dep + `.plugin(tauri_plugin_clipboard_manager::init())` + the `clipboard-manager:allow-write-text` capability all landed in A4. Verify `popup.json` has the capability.

**There is no navigator.clipboard fallback branch.** Delete any existing `navigator.clipboard?.writeText` call in `src/Popup.tsx` (if present).

- [x] **Step 1: Write the failing tests**

Append to `test/Popup.test.tsx`. The file currently mocks `@tauri-apps/api/event` (line 6), `@tauri-apps/api/window` (line 9), and `@tauri-apps/api/core` (line 20), but does NOT mock the clipboard plugin yet. Add the clipboard plugin mock at file scope, immediately after the `core` mock (after line 22):

```diff
 vi.mock("@tauri-apps/api/core", () => ({
   invoke: vi.fn(async () => ({ outcomes: [], actual_engine: undefined })),
 }));
+vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({
+  writeText: vi.fn(async () => undefined),
+}));
```

```ts
it("Copy action flips to Copied feedback for 1.2s via Tauri clipboard", async () => {
  const { writeText } = await import("@tauri-apps/plugin-clipboard-manager");
  const { findByRole } = render(() => <Popup />);
  await emitEvent("popup-state", { status: "result", text: "你好", engine: "openai", source_text: "hello" });
  const copyBtn = await findByRole("button", { name: /复制|Copy/ });
  fireEvent.click(copyBtn);
  expect(await findByRole("button", { name: /已复制|Copied/ })).toBeTruthy();
  expect(writeText).toHaveBeenCalledWith("你好");
  cleanup();
});

it("Retry reuses the saved SOURCE text, not the translation result and not the clipboard", async () => {
  const { invoke } = await import("@tauri-apps/api/core");
  vi.mocked(invoke).mockClear();
  const { findByRole } = render(() => <Popup />);
  await emitEvent("popup-state", { status: "result", text: "你好", engine: "openai", source_text: "hello" });
  const retryBtn = await findByRole("button", { name: /重试|Retry/ });
  fireEvent.click(retryBtn);
  await Promise.resolve();
  const selCall = vi.mocked(invoke).mock.calls.find((c) => c[0] === "translate_selection_ipc");
  expect(selCall, "translate_selection_ipc must be called").toBeTruthy();
  expect(selCall![1]).toEqual({ text: "hello" });
  expect(vi.mocked(invoke).mock.calls.some((c) => c[0] === "translate_clipboard")).toBe(false);
  cleanup();
});

it("Retry is available in the error state because the error payload carries source_text (P1-3)", async () => {
  const { invoke } = await import("@tauri-apps/api/core");
  vi.mocked(invoke).mockClear();
  const { findByRole } = render(() => <Popup />);
  await emitEvent("popup-state", { status: "error", text: "network down", engine: "", source_text: "hello" });
  const retryBtn = await findByRole("button", { name: /重试|Retry/ });
  fireEvent.click(retryBtn);
  await Promise.resolve();
  const selCall = vi.mocked(invoke).mock.calls.find((c) => c[0] === "translate_selection_ipc");
  expect(selCall, "translate_selection_ipc must be called from the error-state Retry").toBeTruthy();
  expect(selCall![1]).toEqual({ text: "hello" });
  cleanup();
});

it("Retry is hidden when there is no source text (P1-3)", async () => {
  const { queryByRole } = render(() => <Popup />);
  await emitEvent("popup-state", { status: "error", text: "no source", engine: "", source_text: undefined });
  await Promise.resolve();
  expect(queryByRole("button", { name: /重试|Retry/ })).toBeNull();
  cleanup();
});

it("Retry for a multi-result reuses the joined SOURCE text", async () => {
  const { invoke } = await import("@tauri-apps/api/core");
  vi.mocked(invoke).mockClear();
  const { findByRole } = render(() => <Popup />);
  await emitEvent("popup-multi-result", {
    outcomes: [{ uuid: "u1", ok: true, text: "你好", engine: "provider/u1" }],
    source_text: "hello world",
  });
  const retryBtn = await findByRole("button", { name: /重试|Retry/ });
  fireEvent.click(retryBtn);
  await Promise.resolve();
  const selCall = vi.mocked(invoke).mock.calls.find((c) => c[0] === "translate_selection_ipc");
  expect(selCall).toBeTruthy();
  expect(selCall![1]).toEqual({ text: "hello world" });
  cleanup();
});

it("config-401 error offers a settings navigation button", async () => {
  const { invoke } = await import("@tauri-apps/api/core");
  vi.mocked(invoke).mockClear();
  const { findByRole } = render(() => <Popup />);
  await emitEvent("popup-state", { status: "error", text: "401 Unauthorized", engine: "" });
  const settingsBtn = await findByRole("button", { name: /打开设置|Open Settings/ });
  fireEvent.click(settingsBtn);
  await Promise.resolve();
  expect(vi.mocked(invoke).mock.calls.some((c) => c[0] === "open_settings_window")).toBe(true);
  cleanup();
});

it("TTS and Favorite are aria-disabled but focusable (not native disabled)", async () => {
  const { findAllByRole } = render(() => <Popup />);
  await emitEvent("popup-state", { status: "result", text: "你好", engine: "openai", source_text: "hi" });
  const tts = (await findAllByRole("button", { name: /朗读|Speak/ }))[0];
  const fav = (await findAllByRole("button", { name: /收藏|Favorite/ }))[0];
  expect(tts.getAttribute("aria-disabled")).toBe("true");
  expect(tts.hasAttribute("disabled")).toBe(false);
  expect(fav.getAttribute("aria-disabled")).toBe("true");
  expect(fav.hasAttribute("disabled")).toBe(false);
  cleanup();
});

it("rev-5-7: a clipboard-origin result carries source_text so Retry re-translates the clipboard text via translate_selection_ipc (NOT translate_clipboard)", async () => {
  // The backend translate_clipboard now emits source_text (Step 6b). The popup
  // controller saves it as lastSource; Retry calls translate_selection_ipc with
  // the saved source — it does NOT re-read the clipboard.
  const { invoke } = await import("@tauri-apps/api/core");
  vi.mocked(invoke).mockClear();
  const { findByRole } = render(() => <Popup />);
  // Simulate a clipboard-translate result payload carrying the clipboard source.
  await emitEvent("popup-state", { status: "result", text: "你好", engine: "openai", source_text: "clipboard text here" });
  const retryBtn = await findByRole("button", { name: /重试|Retry/ });
  fireEvent.click(retryBtn);
  await Promise.resolve();
  const selCall = vi.mocked(invoke).mock.calls.find((c) => c[0] === "translate_selection_ipc");
  expect(selCall, "Retry must call translate_selection_ipc with the saved clipboard source").toBeTruthy();
  expect(selCall![1]).toEqual({ text: "clipboard text here" });
  // rev-5-7: Retry does NOT re-read the clipboard.
  expect(vi.mocked(invoke).mock.calls.some((c) => c[0] === "translate_clipboard")).toBe(false);
  cleanup();
});
```

- [x] **Step 2: Run tests to verify they fail**

Run: `pnpm vitest run test/Popup.test.tsx`
Expected: FAIL — no Copied feedback, no settings button, Retry calls `translate_clipboard` (or nothing), TTS/Favorite use native `disabled`, and error-state Retry is unavailable.

- [x] **Step 3: Add the `open_settings_window` + `translate_selection_ipc` backend commands**

The `Payload.source_text` + `result_with_source`/`multi_result_with_source`/`error_with_source` emitters landed in A2/A3. Here add the two commands to `src-tauri/src/lib.rs`:

```rust
/// Show the main settings window and (optionally) navigate it to a section.
#[tauri::command]
fn open_settings_window(app: tauri::AppHandle, section: Option<String>) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.set_focus();
        if let Some(sec) = section {
            let _ = w.emit("navigate", sec);
        }
        Ok(())
    } else {
        Err("main window unavailable".into())
    }
}

/// Translate the live OS selection (fresh capture) or a caller-supplied SOURCE
/// text (Retry) via the session pipeline. DISTINCT from translate_clipboard.
#[tauri::command]
async fn translate_selection_ipc(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<Session>>,
    app_state: tauri::State<'_, Arc<AppState>>,
    text: Option<String>,
) -> Result<(), ()> {
    let state = state.inner().clone();
    let app_state = app_state.inner().clone();
    let gen = state.gen.next();
    // Retry (text=Some) skips capture; tray (text=None) captures fresh. The cursor
    // is read inside capture_and_translate when needed.
    let (x, y) = {
        let _g = state.gen.selection_lock();
        let pos = cursor::position();
        (pos.0 as f64, pos.1 as f64)
    };
    capture_and_translate(&app, &state, &app_state, text, x, y, gen).await;
    Ok(())
}
```

Register both in `invoke_handler!`. Build to confirm: `cargo build --manifest-path src-tauri/Cargo.toml --features xproc-test-helper`. Verify `open_settings_window.toml` and `translate_selection_ipc.toml` exist.

- [x] **Step 4: Add the new copy keys**

In `src/features/translation/types.ts`, extend the `CopyKey` union:

```ts
  | "selection.action.retry"
  | "selection.action.openSettings"
  | "selection.action.recovery"
  | "selection.action.comingTts"
  | "selection.action.comingFavorite"
  | "selection.action.copied"
```

In `src/features/translation/copy.ts`, add to BOTH `zh` and `en`:

```ts
// zh
"selection.action.openSettings": "打开设置",
"selection.action.recovery": "恢复密钥库",
"selection.action.comingTts": "朗读（Coming later）",
"selection.action.comingFavorite": "收藏（Coming later）",
"selection.action.copied": "已复制",
// en
"selection.action.openSettings": "Open Settings",
"selection.action.recovery": "Recover Keystore",
"selection.action.comingTts": "Speak (Coming later)",
"selection.action.comingFavorite": "Favorite (Coming later)",
"selection.action.copied": "Copied",
```

Also extend `PopupStatePayload` (and `PopupMultiPayload`) in `src/features/translation/types.ts` with `source_text?: string`.

- [x] **Step 5: Make ResultCard forward aria-disabled (focusable disabled actions)**

Edit `packages/ui/src/components/ResultCard.tsx`. The `ResultAction` type carries `ariaDisabled?: boolean`; the `IconButton` renders `aria-disabled` instead of `disabled` when set:

```tsx
// ResultAction type: add
  ariaDisabled?: boolean;

// In the actions render:
              <IconButton
                variant="ghost"
                size="sm"
                aria-label={a.label}
                aria-pressed={a.active ? "true" : undefined}
                disabled={a.disabled && !a.ariaDisabled}
                aria-disabled={a.ariaDisabled ? "true" : undefined}
                onClick={() => a.onClick?.()}
              >
```

When `ariaDisabled` is true, the button has `aria-disabled="true"` and NO native `disabled`, so it stays in the tab order. `onClick` is `() => {}` for aria-disabled actions.

- [x] **Step 6: Save the SOURCE text on every state + Retry via translateSelection(sourceText) (P1-3)**

Edit `src/features/translation/popupController.ts`. Add `lastSource` (the SOURCE, not the translation), saved on loading/error/result/multi, cleared on a new session:

```ts
  /** P1-3: the SOURCE text of the last session — Retry re-translates THIS.
   *  Saved from loading/error/result/multi payloads. Cleared when a brand-new
   *  session starts (detected by a loading payload with a NEW source_text). */
  let lastSource = "";

  // In the popup-state listener:
      await listen<PopupStatePayload>("popup-state", (e) => {
        const decoded = decodePopupState(e.payload);
        // P1-3: save the SOURCE on EVERY state (loading, result, error) so Retry
        // is available even when the result never arrived.
        if (e.payload.status === "loading") {
          // A loading payload marks a new session: clear any stale source, then
          // adopt the fresh source if the payload carries one.
          if (e.payload.source_text !== undefined) {
            lastSource = e.payload.source_text ?? "";
          } else {
            lastSource = "";
          }
        } else if (e.payload.source_text !== undefined) {
          lastSource = e.payload.source_text ?? lastSource;
        }
        setState(decoded);
      }),

  // In the popup-multi-result listener:
      await listen<PopupMultiPayload>("popup-multi-result", (e) => {
        const decoded = decodePopupMultiResult(e.payload);
        if (e.payload.source_text !== undefined) lastSource = e.payload.source_text ?? lastSource;
        setState(decoded);
      }),

  /**
   * Retry: re-translate the saved SOURCE text via the selection pipeline.
   * P1-3: passes lastSource (the original text), NOT a translation result.
   * MUST NOT call translate_clipboard.
   */
  const retrySelection = async () => {
    if (!lastSource) return;
    setState({ kind: "loading" });
    try {
      await translateSelection(lastSource);
    } catch (e) {
      setState({ kind: "error", sub: "generic", message: String(e) });
    }
  };
```

**rev-5-7 (clipboard source-text save, load-bearing):** the clipboard translate path MUST also carry `source_text` so Retry works for clipboard-translated text. Edit `translate_clipboard` (lib.rs:329-410) explicitly. Today it reads the clipboard text (L336-339), shows `popup::show_at` (no source), then emits `popup::result` / `popup::multi_result` / `popup::error` (none carry source). Replace each emit with the source-aware variant so the popup controller can save `lastSource` from the clipboard event payloads:

- [x] **Step 6b: Carry source_text through translate_clipboard (rev-5-7)**

Edit `src-tauri/src/lib.rs`, `translate_clipboard` (lib.rs:329). The clipboard text is read at L336-339 into `text`. Replace the emit sites so each carries `&text`:

```rust
    // L336-339 (unchanged): read the clipboard text under the selection lock.
    let text = {
        let _g = state.gen.selection_lock();
        clipboard::get_text()?
    };
    if text.trim().is_empty() {
        return Err("clipboard empty".into());
    }
    let (x, y) = cursor::position();

    // rev-5-7: show the loading popup carrying the clipboard SOURCE text so the
    // popup controller can save lastSource and Retry can re-run it. Replaces the
    // plain popup::show_at call (which carried no source). Use loading_with_source
    // + the anchor from build_popup_anchor (introduced in A3/A2).
    let anchor = match build_popup_anchor(&app, x as f64, y as f64) {
        Some(a) => a,
        None => return Ok(()),
    };
    if !state.gen.is_latest(gen) {
        return Ok(());
    }
    let _ = popup::loading_with_source(&app, &anchor, Some(&text));
```

Then every error path inside `translate_clipboard` (the client/keystore/db guards + the session-error fallthrough) calls `popup::error_with_source(&app, &msg, &text)` instead of `popup::error(&app, &msg)`. And the result routing (L399-410) uses the source-aware variants:

```rust
    match session_result {
        Ok(r) => match decide_clipboard_popup(&r) {
            ClipboardPopupDecision::SingleSuccess { text: t, engine } => {
                let _ = popup::set_popup_mode(&app, popup::PopupMode::Single, &anchor);
                let _ = popup::result_with_source(&app, &t, &engine, &text);
            }
            ClipboardPopupDecision::Multi => {
                let _ = popup::set_popup_mode(&app, popup::PopupMode::Multi, &anchor);
                let _ = popup::multi_result_with_source(&app, &r.outcomes, &text);
            }
            ClipboardPopupDecision::Error(msg) => {
                let _ = popup::set_popup_mode(&app, popup::PopupMode::Error, &anchor);
                let _ = popup::error_with_source(&app, &msg, &text);
            }
        },
        Err(msg) => {
            if state.gen.is_latest(gen) {
                let _ = popup::set_popup_mode(&app, popup::PopupMode::Error, &anchor);
                let _ = popup::error_with_source(&app, &msg, &text);
            }
        }
    }
    Ok(())
```

Frontend side (rev-5-7): the popup controller's `popup-state` / `popup-multi-result` listeners (B4 Step 6) ALREADY save `payload.source_text` into `lastSource` for EVERY payload — they do not distinguish selection-origin from clipboard-origin. So once the clipboard payloads carry `source_text`, `lastSource` is populated automatically and Retry re-runs via `translateSelection(lastSource)` — which calls `translate_selection_ipc({ text: lastSource })` and does NOT re-read the clipboard. No additional frontend change is needed beyond B4 Step 6; the only edit is the backend `translate_clipboard` emit sites above.

- [x] **Step 7: Wire Copy feedback, settings nav, recovery CTA, aria-disabled TTS/Favorite, conditional Retry in Popup.tsx**

Edit `src/Popup.tsx`. Add the clipboard import + `copiedUuid` signal + `openSettings` helper:

```tsx
import { writeText } from "@tauri-apps/plugin-clipboard-manager";

  const [copiedUuid, setCopiedUuid] = createSignal<string | null>(null);
  let copiedTimer: ReturnType<typeof setTimeout> | undefined;

  const openSettings = (section?: string) => {
    import("@tauri-apps/api/core")
      .then(({ invoke }) => invoke("open_settings_window", { section: section ?? null }))
      .catch(() => {});
  };
```

Add `buildActions` (Copy uses `writeText(translationText)`, NOT the source; Retry uses the source via `ctrl.retrySelection()`; **P1-3: Retry only appears when `ctrl.lastSource` is non-empty**):

```tsx
  const buildActions = (uuid: string): ResultAction[] => {
    const isPinned = ctrl.pinned();
    const isCopied = copiedUuid() === uuid;
    const actions: ResultAction[] = [
      {
        label: isCopied ? t("selection.action.copied") : t("selection.action.copy"),
        icon: <Copy size={14} />,
        onClick: async () => {
          const value = textFor(uuid) ?? "";
          try {
            await writeText(value);
            setCopiedUuid(uuid);
            if (copiedTimer) clearTimeout(copiedTimer);
            copiedTimer = setTimeout(() => setCopiedUuid(null), 1200);
          } catch {
            // Best-effort: clipboard may be unavailable.
          }
        },
      },
      {
        label: t("selection.action.comingTts"),
        icon: <Volume2 size={14} />,
        ariaDisabled: true,
        onClick: () => {},
      },
      {
        label: isPinned ? t("selection.action.unpin") : t("selection.action.pin"),
        icon: isPinned ? <PinOff size={14} /> : <Pin size={14} />,
        active: isPinned,
        onClick: () => (isPinned ? ctrl.unpin() : ctrl.pin()),
      },
      {
        label: t("selection.action.comingFavorite"),
        icon: <Star size={14} />,
        ariaDisabled: true,
        onClick: () => {},
      },
    ];
    return actions;
  };
```

The error-shell Retry is gated on `ctrl.hasSource()` (P1-3). Expose `hasSource` from the controller: `const hasSource = () => lastSource.length > 0;` and add it to the returned object. Then the error shell:

```tsx
              <Show
                when={errorState()?.sub === "network" && ctrl.hasSource()}
                fallback={
                  <Show
                    when={
                      errorState()?.sub === "config-key" ||
                      errorState()?.sub === "config-401"
                    }
                    fallback={<span></span>}
                  >
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => openSettings("provider-center")}
                    >
                      {t("selection.action.openSettings")}
                    </Button>
                  </Show>
                }
              >
                <Button variant="secondary" size="sm" onClick={() => void ctrl.retrySelection()}>
                  {t("selection.action.retry")}
                </Button>
              </Show>
```

For the keystore-corrupt state, add a dedicated Show before the closing `</main>`:

```tsx
  const isErrorShell = createMemo(() => {
    const k = state().kind;
    return k === "error" || k === "offline" || k === "no-selection" ||
      k === "no-permission";
  });
```

```tsx
      <Show when={state().kind === "keystore-corrupt"}>
        <div class="popup-error" role="alert">
          <EmptyState
            icon={<AlertTriangle size={32} />}
            title={t("selection.error.keystore")}
            action={
              <Button
                variant="secondary"
                size="sm"
                onClick={() => openSettings("keystore-recovery")}
              >
                {t("selection.action.recovery")}
              </Button>
            }
          />
        </div>
      </Show>
```

**Critical distinction (P1-3):**
- Copy writes the TRANSLATION (`textFor(uuid)`, the result) to the clipboard — that is what the user wants to paste.
- Retry re-translates the SOURCE (`lastSource`) — that is what the user wants to re-run. Retry is hidden when `lastSource` is empty.

- [x] **Step 8: Run tests to verify they pass**

Run: `pnpm vitest run test/Popup.test.tsx && pnpm --filter @linguaray/ui test`
Expected: PASS.

- [x] **Step 9: Commit**

```bash
git diff --check
git add src-tauri/src/lib.rs src/Popup.tsx src/features/translation/types.ts src/features/translation/copy.ts src/features/translation/popupController.ts packages/ui/src/components/ResultCard.tsx package.json pnpm-lock.yaml src-tauri/permissions/autogenerated/open_settings_window.toml src-tauri/permissions/autogenerated/translate_selection_ipc.toml test/Popup.test.tsx
git commit -m "feat(popup): source_text in every state payload + Tauri clipboard Copy + saved-SOURCE Retry (hidden when no source) + settings/recovery CTAs + aria-disabled TTS/Favorite (P1-3)"
```

---

### Task B5: Stable input order for `translate_parallel` (strict pre-failed interleaving)

**Files:**
- Modify: `src-tauri/src/service.rs` — `translate_parallel` emits outcomes in STRICT input order.
- Test: `src-tauri/tests/translate_parallel.rs` — add strict-order-with-pre-failure + ready-order tests.

**Interfaces:**
- Consumes: the input `profiles: Vec<ProviderProfile>` order is the contract order.
- Produces: `translate_parallel` returns outcomes in the SAME order as `profiles`.

**Verified fact:** `join_all` preserves INPUT order for READY futures. The bug: pre-failed outcomes are pushed BEFORE `outcomes.append(&mut all)` (service.rs:246), so a later pre-failed profile appears earlier in the result.

- [x] **Step 1: Write the failing tests**

Append to `src-tauri/tests/translate_parallel.rs`:

```rust
#[tokio::test]
async fn outcomes_preserve_input_order_with_pre_failed_middle() {
    // u1 = valid (ready), u2 = pre-fails preset conversion, u3 = valid (ready).
    // The result MUST be [u1, u2, u3] (input order), NOT [u2, u1, u3].
    let s1 = MockServer::start().await;
    let s3 = MockServer::start().await;
    let port1: u16 = s1.uri().rsplit(':').next().unwrap().parse().unwrap();
    let port3: u16 = s3.uri().rsplit(':').next().unwrap().parse().unwrap();
    mount_ok(&s1, "first").await;
    mount_ok(&s3, "third").await;

    let profiles = vec![
        profile("u1", &format!("http://lvh.me:{port1}/v1/chat/completions")),
        profile_unsupported("u2"),
        profile("u3", &format!("http://lvh.me:{port3}/v1/chat/completions")),
    ];
    let client = direct_client();
    let keystore = empty_keystore();

    let outcomes = translate_parallel(
        &client, &keystore, profiles, "x", "auto", "zh",
        AppOptions::default(), None,
    ).await;

    let got: Vec<&str> = outcomes.iter().map(|o| o.uuid.as_str()).collect();
    assert_eq!(got, vec!["u1", "u2", "u3"],
        "outcomes must preserve STRICT input order including the pre-failed middle entry");
    assert!(outcomes[1].result.is_err(), "u2 must be the pre-failed entry");
}
```

Add a `profile_unsupported` helper (build a `ProviderProfile` whose protocol `profile_to_preset` rejects — read `adapter.rs::profile_to_preset` to find a rejected protocol like `google_translate`/`custom_http`).

```rust
#[tokio::test]
async fn ready_outcomes_preserve_input_order_under_completion_jitter() {
    use std::time::Duration;
    let s1 = MockServer::start().await;
    let s2 = MockServer::start().await;
    let s3 = MockServer::start().await;
    let port1: u16 = s1.uri().rsplit(':').next().unwrap().parse().unwrap();
    let port2: u16 = s2.uri().rsplit(':').next().unwrap().parse().unwrap();
    let port3: u16 = s3.uri().rsplit(':').next().unwrap().parse().unwrap();
    Mock::given(any()).respond_with(ResponseTemplate::new(200).set_body_json(json!({
        "choices": [{"message": {"content": "slow"}}]
    })).insert_delay(Duration::from_millis(150))).mount(&s1).await;
    mount_ok(&s2, "fast").await;
    Mock::given(any()).respond_with(ResponseTemplate::new(200).set_body_json(json!({
        "choices": [{"message": {"content": "medium"}}]
    })).insert_delay(Duration::from_millis(50))).mount(&s3).await;

    let profiles = vec![
        profile("u1", &format!("http://lvh.me:{port1}/v1/chat/completions")),
        profile("u2", &format!("http://lvh.me:{port2}/v1/chat/completions")),
        profile("u3", &format!("http://lvh.me:{port3}/v1/chat/completions")),
    ];
    let client = direct_client();
    let keystore = empty_keystore();

    let outcomes = translate_parallel(
        &client, &keystore, profiles, "x", "auto", "zh",
        AppOptions::default(), None,
    ).await;

    let got: Vec<&str> = outcomes.iter().map(|o| o.uuid.as_str()).collect();
    assert_eq!(got, vec!["u1", "u2", "u3"]);
}
```

- [x] **Step 2: Run tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --features xproc-test-helper --test translate_parallel outcomes_preserve_input_order_with_pre_failed_middle`
Expected: FAIL — order is `[u2, u1, u3]`.

Run the ready-order test; expected PASS (locks join_all).

- [x] **Step 3: Implement strict input-order emission (B5 only fixes ORDERING; B6 changes the fallback policy)**

In `src-tauri/src/service.rs`, replace the body of `translate_parallel` from line 200 (`let mut ready...`) through the end of the fn (line 248) with an input-index-tagged version. Pre-failed outcomes keep their input position:

```rust
    // Collect (input_index, Option<uuid+preset>, Option<pre-failed outcome>).
    let mut entries: Vec<(usize, Option<(String, ProviderPreset)>, Option<TranslationOutcome>)> =
        Vec::with_capacity(profiles.len());
    for (idx, p) in profiles.into_iter().enumerate() {
        match profile_to_preset(&p) {
            Ok(preset) => entries.push((idx, Some((p.uuid.clone(), preset)), None)),
            Err(reason) => entries.push((
                idx,
                None,
                Some(TranslationOutcome {
                    uuid: p.uuid.clone(),
                    result: Err(Error::Config(ConfigKind::Unsupported {
                        provider: p.uuid.clone(),
                        reason,
                    })),
                }),
            )),
        }
    }

    // Drive all ready entries concurrently, tagging each result with its index.
    // (B5 only fixes ORDERING. B6 will change the fallback arg to None and add
    // the session-level fallback policy. For now, pass fallback.as_deref() to
    // preserve per-engine behavior until B6 lands.)
    let futs: Vec<_> = entries
        .iter()
        .filter_map(|(idx, ready, _)| ready.as_ref().map(|(uuid, preset)| (*idx, uuid.clone(), preset)))
        .map(|(idx, uuid, preset)| {
            let options = options.clone();
            let fb_ref: Option<&dyn TraditionalEngine> = fallback.as_deref();
            async move {
                let input = TranslateInput { text, from, to, options };
                let result =
                    translate_with_fallback_ref(client, keystore, preset, input, fb_ref).await;
                (idx, TranslationOutcome { uuid, result })
            }
        })
        .collect();
    let mut ready_results = futures::future::join_all(futs).await;

    // Build the final vec in strict input order: walk entries by index.
    ready_results.sort_by_key(|(idx, _)| *idx);
    let mut ready_iter = ready_results.into_iter();
    let mut outcomes: Vec<TranslationOutcome> = Vec::with_capacity(entries.len());
    for (_idx, ready, pre_failed) in entries {
        if let Some(o) = pre_failed {
            outcomes.push(o);
        } else if let Some((_idx, o)) = ready_iter.next() {
            outcomes.push(o);
        }
    }
    outcomes
```

- [x] **Step 4: Run the tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --features xproc-test-helper --test translate_parallel`
Expected: PASS.

- [x] **Step 5: Commit**

```bash
git diff --check
git add src-tauri/src/service.rs src-tauri/tests/translate_parallel.rs
git commit -m "fix(parallel): strict input-order outcomes incl. pre-failed entries (locks join_all guarantee)"
```

---

### Task B6: Bounded fallback — eligibility can actually hit + local-sacred-aware + error-class-aware (single call + single card per session)

> **P1-4 (load-bearing):** the current `translate_with_fallback_ref(..., None)` converts a non-local `FallbackEligible` into `LocalNoFallback`, so the session-level check that pattern-matches `Error::FallbackEligible` can NEVER hit. The fix: a new `translate_primary_only` runs the primary and PRESERVES the original `Error` (no conversion). `translate_parallel` uses it. A pure `eligible_for_session_fallback` decides whether to fire the single fallback call.

**Files:**
- Modify: `src-tauri/src/service.rs`:
  - **(new)** `pub async fn translate_primary_only(client, keystore, preset, input) -> Result<Translation, Error>` — runs `translate` (the primary) only; returns its `Result` verbatim (no `LocalNoFallback` conversion).
  - **(new)** `pub fn eligible_for_session_fallback(outcomes) -> bool` — pure decision (P1-4).
  - `translate_parallel` (service.rs:190) — use `translate_primary_only`; session-level fallback via `eligible_for_session_fallback` + a single `translate_with_fallback_ref`.
- Test: `src-tauri/tests/translate_parallel.rs` — fallback-call-count + error-class matrix tests.

**Interfaces:**
- Consumes: `fallback: Option<Arc<dyn TraditionalEngine>>`; `Error::FallbackEligible(FallbackKind)` / `Error::Config(ConfigKind)` / `Error::Keystore` / `Error::LocalNoFallback`; `is_local(&ProviderPreset) -> bool` (service.rs:255).
- Produces:
  - `translate_primary_only` returns the primary's raw `Result<Translation, Error>`. A network/5xx failure stays `Error::FallbackEligible(_)` (not converted to `LocalNoFallback`), so the session-level check can see it.
  - `eligible_for_session_fallback(outcomes: &[TranslationOutcome], locality: &[bool], local_primary_failed: bool) -> bool` (rev-6-4 signature) returns true iff:
    1. `local_primary_failed` is false (a LOCAL primary failure blocks the session), AND
    2. NO outcome succeeded, AND
    3. AT LEAST one failure is `Error::FallbackEligible(_)` from a NON-local provider (`locality[i] == false`).
    - `Config`/`MissingKey`/`AuthFailed`(401/403)/`Keystore`/`Unsupported`/`LocalNoFallback` do NOT count toward eligibility.
    - **rev-6-4 locality rule:** a LOCAL provider's `FallbackEligible` NEVER counts toward session fallback (local-sacred extends to EVERY local provider, not just the primary). The `locality` slice is aligned 1:1 with `outcomes` and built from the per-future `was_local = is_local(&preset)` flag captured before the join.
    - **Mixed rule (P1-4 refined):** if a LOCAL provider is the PRIMARY (input position 0) AND it failed, the session does NOT trigger a remote fallback (the `local_primary_failed` arg short-circuits to false).
  - When eligible, `translate_parallel` calls `eng.translate(...)` ONCE (the fallback engine directly) and appends the result as an independent outcome.

- [x] **Step 1: Write the failing tests (the full matrix, P1-4; fixed mock URLs on lvh.me, NOT 127.0.0.1:11434)**

Append to `src-tauri/tests/translate_parallel.rs`:

```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use linguaray_lib::engines::TraditionalEngine;
use linguaray_lib::error::{Error, FallbackKind};
use linguaray_lib::service::Translation;

struct CountingFallback { calls: AtomicUsize }
impl CountingFallback {
    fn new() -> Self { Self { calls: AtomicUsize::new(0) } }
    fn calls(&self) -> usize { self.calls.load(Ordering::SeqCst) }
}
#[async_trait::async_trait]
impl TraditionalEngine for CountingFallback {
    fn id(&self) -> &str { "counting" }
    fn label(&self) -> &str { "Counting" }
    fn needs_key(&self) -> bool { false }
    async fn translate(&self, _client: &reqwest::Client, _text: &str, _from: &str, _to: &str)
        -> Result<String, Error> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok("fallback-text".into())
    }
}

/// Helper: build a profile whose endpoint is loopback (is_local == true).
/// P1-4: tests use lvh.me for REMOTE profiles, never 127.0.0.1:11434 for the
/// fallback-eligibility cases (that's the local-sacred case).
fn profile_local(uuid: &str) -> ProviderProfile {
    profile(uuid, "http://127.0.0.1:11434/v1/chat/completions")
}

#[tokio::test]
async fn local_primary_failure_does_not_trigger_remote_fallback() {
    // P1-4 local-sacred: a LOCAL primary failing must NOT degrade to a remote
    // fallback. The fallback must be called ZERO times.
    let profiles = vec![profile_local("u1")];
    let client = direct_client();
    let keystore = empty_keystore();
    let counter = Arc::new(CountingFallback::new());
    let fallback: Arc<dyn TraditionalEngine> = counter.clone();

    let outcomes = translate_parallel(
        &client, &keystore, profiles, "x", "auto", "zh",
        AppOptions::default(), Some(fallback),
    ).await;

    assert_eq!(counter.calls(), 0,
        "local primary failure must NOT trigger remote fallback (local-sacred)");
    let ok = outcomes.iter().filter(|o| o.result.is_ok()).count();
    assert_eq!(ok, 0, "no fallback card for a local failure");
}

#[tokio::test]
async fn config_failure_does_not_trigger_fallback() {
    // P1-4: Config errors (Unsupported/401) never trigger fallback.
    let s2 = MockServer::start().await;
    let port2: u16 = s2.uri().rsplit(':').next().unwrap().parse().unwrap();
    Mock::given(any()).respond_with(ResponseTemplate::new(401)).mount(&s2).await;

    let profiles = vec![
        profile_unsupported("u1"),
        profile("u2", &format!("http://lvh.me:{port2}/v1/chat/completions")),
    ];
    let client = direct_client();
    let keystore = empty_keystore();
    let counter = Arc::new(CountingFallback::new());
    let fallback: Arc<dyn TraditionalEngine> = counter.clone();

    let outcomes = translate_parallel(
        &client, &keystore, profiles, "x", "auto", "zh",
        AppOptions::default(), Some(fallback),
    ).await;

    assert_eq!(counter.calls(), 0,
        "Config/Unsupported/401 failures must NOT trigger fallback");
    let ok = outcomes.iter().filter(|o| o.result.is_ok()).count();
    assert_eq!(ok, 0);
}

#[tokio::test]
async fn two_remote_transient_failures_trigger_at_most_one_fallback() {
    // P1-4: two NON-local FallbackEligible failures (500s) → fallback called
    // EXACTLY ONCE, ONE fallback result card. (This test FAILS today because
    // translate_with_fallback_ref(None) converts FallbackEligible→LocalNoFallback
    // so eligible_for_session_fallback can never see a FallbackEligible.)
    let s1 = MockServer::start().await;
    let s2 = MockServer::start().await;
    let port1: u16 = s1.uri().rsplit(':').next().unwrap().parse().unwrap();
    let port2: u16 = s2.uri().rsplit(':').next().unwrap().parse().unwrap();
    mount_500(&s1).await;
    mount_500(&s2).await;

    let profiles = vec![
        profile("u1", &format!("http://lvh.me:{port1}/v1/chat/completions")),
        profile("u2", &format!("http://lvh.me:{port2}/v1/chat/completions")),
    ];
    let client = direct_client();
    let keystore = empty_keystore();
    let counter = Arc::new(CountingFallback::new());
    let fallback: Arc<dyn TraditionalEngine> = counter.clone();

    let outcomes = translate_parallel(
        &client, &keystore, profiles, "x", "auto", "zh",
        AppOptions::default(), Some(fallback),
    ).await;

    assert_eq!(counter.calls(), 1, "fallback called once for the session");
    let ok = outcomes.iter().filter(|o| o.result.is_ok()).count();
    assert_eq!(ok, 1, "exactly one fallback result card");
}

#[tokio::test]
async fn eligible_for_session_fallback_pure_function_detects_non_local_fallback_eligible() {
    // P1-4 + rev-6-4: the pure decision function. A NON-local FallbackEligible
    // failure + no success → eligible. locality[0]=false (remote).
    use linguaray_lib::service::eligible_for_session_fallback;
    let outcomes = vec![
        TranslationOutcome {
            uuid: "u1".into(),
            result: Err(Error::FallbackEligible(FallbackKind::Network)),
        },
    ];
    assert!(eligible_for_session_fallback(&outcomes, &[false], false),
        "a non-local FallbackEligible failure with no success must be eligible");
}

#[tokio::test]
async fn eligible_for_session_fallback_rejects_all_config_errors() {
    use linguaray_lib::service::eligible_for_session_fallback;
    use linguaray_lib::error::ConfigKind;
    let outcomes = vec![
        TranslationOutcome {
            uuid: "u1".into(),
            result: Err(Error::Config(ConfigKind::Unsupported { provider: "u1".into(), reason: "no".into() })),
        },
        TranslationOutcome { uuid: "u2".into(), result: Err(Error::Keystore) },
    ];
    assert!(!eligible_for_session_fallback(&outcomes, &[false, false], false),
        "Config/Keystore failures must NOT be eligible");
}

#[tokio::test]
async fn eligible_for_session_fallback_ignores_local_fallback_eligible_rev6_4() {
    // rev-6-4: a LOCAL provider's FallbackEligible does NOT count. locality[0]
    // = true (local) + FallbackEligible → NOT eligible (local-sacred).
    use linguaray_lib::service::eligible_for_session_fallback;
    let outcomes = vec![
        TranslationOutcome {
            uuid: "u1".into(),
            result: Err(Error::FallbackEligible(FallbackKind::Network)),
        },
    ];
    assert!(!eligible_for_session_fallback(&outcomes, &[true], false),
        "a LOCAL FallbackEligible must NOT trigger remote fallback (local-sacred, rev-6-4)");
}

#[tokio::test]
async fn eligible_for_session_fallback_local_primary_failed_blocks_rev6_4() {
    // rev-6-4: local_primary_failed=true blocks the session fallback even if a
    // non-local parallel provider has a FallbackEligible failure.
    use linguaray_lib::service::eligible_for_session_fallback;
    let outcomes = vec![
        TranslationOutcome { uuid: "u1".into(), result: Err(Error::LocalNoFallback) },
        TranslationOutcome {
            uuid: "u2".into(),
            result: Err(Error::FallbackEligible(FallbackKind::Network)),
        },
    ];
    // locality: u1 local (primary), u2 remote.
    assert!(!eligible_for_session_fallback(&outcomes, &[true, false], true),
        "local_primary_failed must block the session fallback (rev-6-4)");
}

#[tokio::test]
async fn mixed_local_primary_and_remote_transient_does_not_trigger_fallback() {
    // P1-4 refined: local provider at PRIMARY (position 0) failing blocks the
    // session fallback even if a remote provider also has a FallbackEligible
    // failure. Local-sacred extends to the session level when local is primary.
    let s2 = MockServer::start().await;
    let port2: u16 = s2.uri().rsplit(':').next().unwrap().parse().unwrap();
    mount_500(&s2).await;

    let profiles = vec![
        profile_local("u1"), // local primary, will fail (nothing on 127.0.0.1:11434)
        profile("u2", &format!("http://lvh.me:{port2}/v1/chat/completions")), // remote 500
    ];
    let client = direct_client();
    let keystore = empty_keystore();
    let counter = Arc::new(CountingFallback::new());
    let fallback: Arc<dyn TraditionalEngine> = counter.clone();

    let outcomes = translate_parallel(
        &client, &keystore, profiles, "x", "auto", "zh",
        AppOptions::default(), Some(fallback),
    ).await;

    assert_eq!(counter.calls(), 0,
        "local primary failing must NOT trigger remote fallback (local-sacred at session level)");
    let ok = outcomes.iter().filter(|o| o.result.is_ok()).count();
    assert_eq!(ok, 0);
}

#[tokio::test]
async fn remote_primary_config_fail_plus_local_parallel_fallback_eligible_no_fallback_rev6_4() {
    // rev-6-4 matrix (a): remote primary fails with Config (401 → NOT
    // FallbackEligible), and a LOCAL parallel provider has a FallbackEligible
    // failure. The local parallel FallbackEligible must NOT trigger a remote
    // fallback (local-sacred extends to the session level for local providers).
    let s1 = MockServer::start().await;
    let port1: u16 = s1.uri().rsplit(':').next().unwrap().parse().unwrap();
    Mock::given(any()).respond_with(ResponseTemplate::new(401)).mount(&s1).await;

    let profiles = vec![
        profile("u1", &format!("http://lvh.me:{port1}/v1/chat/completions")), // remote primary, 401 → Config
        profile_local("u2"), // local parallel, will fail (FallbackEligible, nothing on 127.0.0.1:11434)
    ];
    let client = direct_client();
    let keystore = empty_keystore();
    let counter = Arc::new(CountingFallback::new());
    let fallback: Arc<dyn TraditionalEngine> = counter.clone();

    let outcomes = translate_parallel(
        &client, &keystore, profiles, "x", "auto", "zh",
        AppOptions::default(), Some(fallback),
    ).await;

    assert_eq!(counter.calls(), 0,
        "local parallel FallbackEligible must NOT trigger remote fallback (rev-6-4 local-sacred)");
    let ok = outcomes.iter().filter(|o| o.result.is_ok()).count();
    assert_eq!(ok, 0);
}

#[tokio::test]
async fn primary_pre_failed_locality_identified_correctly_rev6_4() {
    // rev-6-4 matrix (c): the primary (idx 0) pre-fails at profile_to_preset
    // time (Config::Unsupported). Its locality is recorded as false (it never
    // reached is_local because there was no preset). This is correct: a
    // pre-failed entry is Config, not FallbackEligible, so it cannot be the
    // trigger anyway. The test asserts the decision does not panic and the
    // primary's pre-failure does not block a legit remote parallel
    // FallbackEligible from firing the fallback.
    let s2 = MockServer::start().await;
    let port2: u16 = s2.uri().rsplit(':').next().unwrap().parse().unwrap();
    mount_500(&s2).await;

    let profiles = vec![
        profile_unsupported("u1"), // primary pre-fails (Config::Unsupported)
        profile("u2", &format!("http://lvh.me:{port2}/v1/chat/completions")), // remote parallel 500 → FallbackEligible
    ];
    let client = direct_client();
    let keystore = empty_keystore();
    let counter = Arc::new(CountingFallback::new());
    let fallback: Arc<dyn TraditionalEngine> = counter.clone();

    let outcomes = translate_parallel(
        &client, &keystore, profiles, "x", "auto", "zh",
        AppOptions::default(), Some(fallback),
    ).await;

    // The remote parallel FallbackEligible IS eligible (primary pre-failed with
    // Config, not local-primary-failed). Fallback fires exactly once.
    assert_eq!(counter.calls(), 1,
        "remote parallel FallbackEligible with a Config-pre-failed primary must trigger ONE fallback (rev-6-4)");
    let ok = outcomes.iter().filter(|o| o.result.is_ok()).count();
    assert_eq!(ok, 1, "exactly one fallback result card");
}
```

- [x] **Step 2: Run tests to verify they fail**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --features xproc-test-helper --test translate_parallel`
Expected: FAIL — the `two_remote_transient_failures_trigger_at_most_one_fallback` test fails because `translate_with_fallback_ref(None)` converts `FallbackEligible`→`LocalNoFallback`, so the eligibility check never sees a `FallbackEligible`.

- [x] **Step 3: Implement `translate_primary_only` + `eligible_for_session_fallback` + the session fallback policy**

In `src-tauri/src/service.rs`, add `translate_primary_only` (runs the primary only, preserves the original Error):

```rust
/// P1-4: run the PRIMARY engine only and return its raw Result. Unlike
/// translate_with_fallback_ref, this does NOT convert a non-local
/// FallbackEligible into LocalNoFallback when no fallback is configured — the
/// original Error is preserved so eligible_for_session_fallback can see it and
/// the session-level fallback can fire.
pub async fn translate_primary_only(
    client: &reqwest::Client,
    keystore: &Keystore,
    primary_preset: &ProviderPreset,
    input: TranslateInput<'_>,
) -> Result<Translation, Error> {
    translate(
        client,
        keystore,
        primary_preset,
        TranslateInput {
            text: input.text,
            from: input.from,
            to: input.to,
            options: input.options,
        },
    )
    .await
}
```

Add `eligible_for_session_fallback` (pure decision, P1-4 + rev-6-4 refined mixed rule):

```rust
/// P1-4 + rev-6-4: pure decision — does this set of outcomes warrant a
/// session-level fallback? Returns true iff:
///   (a) NO outcome succeeded, AND
///   (b) AT LEAST one failure is `Error::FallbackEligible(_)` from a NON-local
///       provider (rev-6-4: the locality flag is threaded through the parallel
///       execution as a parallel slice; a LOCAL provider's FallbackEligible
///       never counts toward session fallback — local-sacred extends to the
///       session level for every local provider, not just the primary).
/// Config/MissingKey/401/Keystore/Unsupported/LocalNoFallback never count.
///
/// `locality` is a slice aligned 1:1 with `outcomes` (same index space):
/// `locality[i] = true` iff outcome `i`'s provider `is_local()` was true. The
/// caller (translate_parallel) builds this slice from the per-future
/// `was_local` flag it captured before the join.
///
/// The LOCAL-PRIMARY gate is ALSO applied here (rev-6-4 folds it in): if
/// `local_primary_failed` is true, the session does NOT fire a remote fallback
/// even if a non-local parallel provider has a FallbackEligible failure.
pub fn eligible_for_session_fallback(
    outcomes: &[TranslationOutcome],
    locality: &[bool],
    local_primary_failed: bool,
) -> bool {
    if local_primary_failed {
        return false;
    }
    let any_success = outcomes.iter().any(|o| o.result.is_ok());
    if any_success {
        return false;
    }
    // rev-6-4: only a NON-local FallbackEligible counts. A local provider's
    // FallbackEligible is local-sacred and must not trigger remote fallback.
    outcomes.iter().enumerate().any(|(i, o)| {
        let was_local = locality.get(i).copied().unwrap_or(false);
        !was_local && matches!(o.result, Err(Error::FallbackEligible(_)))
    })
}
```

Now replace the `futs` + post-join logic of `translate_parallel` with the complete, compile-clean body below. **rev-5-3 + rev-6-4 (load-bearing):** the locality flag is captured per-future (`was_local = is_local(&preset)` BEFORE the await, so it is stable) and threaded through as `Vec<(usize, bool, Result<Translation, Error>)>` (idx, was_local, result). The reassembly builds BOTH the strict-order outcomes Vec AND a parallel `locality: Vec<bool>` slice aligned 1:1 with the outcomes, so `eligible_for_session_fallback` can check `(!was_local && FallbackEligible(_))`. There is NO iterator-consumption pattern that moves `entries` and then indexes it.

```rust
    let primary_idx = 0usize;
    let futs: Vec<_> = entries
        .iter()
        .filter_map(|(idx, ready, _)| {
            ready.as_ref().map(|(uuid, preset)| (*idx, uuid.clone(), preset.clone()))
        })
        .map(|(idx, uuid, preset)| {
            let options = options.clone();
            // rev-6-4: capture locality BEFORE the await (is_local is a pure fn
            // of the preset, so this is stable). Threaded through as the 2nd
            // tuple element so it survives the join.
            let was_local = is_local(&preset);
            async move {
                let input = TranslateInput { text, from, to, options };
                let result = translate_primary_only(client, keystore, &preset, input).await;
                (idx, was_local, uuid, result)
            }
        })
        .collect();
    let ready_results = futures::future::join_all(futs).await;

    // Build (idx -> (was_local, outcome)) so the locality flag survives the walk.
    let entry_count = entries.len();
    let mut by_idx: std::collections::HashMap<usize, (bool, TranslationOutcome)> = ready_results
        .into_iter()
        .map(|(idx, was_local, uuid, result)| {
            (idx, (was_local, TranslationOutcome { uuid, result }))
        })
        .collect();

    // Read the local-primary gate BEFORE the consuming walk drains by_idx.
    let primary_was_local = by_idx
        .get(&primary_idx)
        .map(|(wl, _)| *wl)
        .unwrap_or(false);

    // Strict input-order Vec + parallel locality slice (rev-6-4). For a
    // pre-failed entry, locality is derived from the preset that would have
    // been used; since pre-failure happens at profile_to_preset time (before we
    // have a preset), a pre-failed entry's locality is recorded as false (it
    // never reached is_local). This is correct: a pre-failed entry cannot be
    // FallbackEligible (it is Config::Unsupported), so its locality is moot.
    let mut outcomes: Vec<TranslationOutcome> = Vec::with_capacity(entry_count);
    let mut locality: Vec<bool> = Vec::with_capacity(entry_count);
    for (idx, _ready, pre_failed) in entries.into_iter() {
        if let Some(o) = pre_failed {
            outcomes.push(o);
            locality.push(false);
        } else if let Some((was_local, o)) = by_idx.remove(&idx) {
            outcomes.push(o);
            locality.push(was_local);
        }
    }

    // P1-4 + rev-6-4 session-fallback decision. local_primary_failed is true
    // iff the primary (idx 0) was local AND its outcome is an error.
    let local_primary_failed = primary_was_local
        && outcomes
            .get(primary_idx)
            .map(|o| o.result.is_err())
            .unwrap_or(false);
    let eligible = eligible_for_session_fallback(&outcomes, &locality, local_primary_failed);

    if eligible {
        if let Some(eng) = fallback.as_deref() {
            // Call the fallback engine directly to produce ONE independent
            // outcome (translate_with_fallback_ref with Some(fallback) would
            // re-run the primary, which we already did above).
            let input = TranslateInput { text, from, to, options: options.clone() };
            match eng.translate(client, input.text, input.from, input.to).await {
                Ok(fb_text) => {
                    // rev-5-3: APPEND the single fallback outcome at the end of
                    // the strict-order Vec (one card per session, P1-4).
                    outcomes.push(TranslationOutcome {
                        uuid: eng.id().to_string(),
                        result: Ok(Translation {
                            text: fb_text,
                            engine: eng.id().to_string(),
                        }),
                    });
                }
                Err(_) => { /* fallback itself failed; leave all-failed as-is */ }
            }
        }
    }
    outcomes
```

**Note:** `eligible_for_session_fallback` is exposed for unit testing with the full `(outcomes, locality, local_primary_failed)` signature (rev-6-4 folds the local-primary gate into the pure fn so the test matrix can exercise it directly). The live decision in `translate_parallel` builds the `locality` Vec in lockstep with the outcomes Vec and reads `primary_was_local` from `by_idx` before the consuming walk.

- [x] **Step 4: Run the tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --features xproc-test-helper --test translate_parallel`
Expected: PASS (all existing + the 6 new matrix tests). Any pre-existing test that asserted per-engine fallback MUST be updated to expect the session-level policy — re-read and fix, documenting each change.

- [x] **Step 5: Commit**

```bash
git diff --check
git add src-tauri/src/service.rs src-tauri/tests/translate_parallel.rs
git commit -m "fix(parallel): translate_primary_only preserves FallbackEligible so session fallback can hit; locality-aware eligibility (local providers never count; local-primary blocks) (one call + one card; rev-6-4) (P1-4)"
```

---

### Stage B Verification

Run before Stage C:

```bash
cargo build --manifest-path src-tauri/Cargo.toml --features xproc-test-helper
cargo test --manifest-path src-tauri/Cargo.toml --features xproc-test-helper
cargo clippy --manifest-path src-tauri/Cargo.toml --features xproc-test-helper --all-targets -- -D warnings
pnpm test
pnpm typecheck
pnpm build
```

Confirm:
- `translate_parallel` tests pass including strict-order-with-pre-failure + the fallback matrix tests (local-sacred, Config, two-remote-transient, pure-decision, mixed-local-primary, rev-6-4 local-parallel-FallbackEligible, rev-6-4 remote-primary-Config+local-parallel, rev-6-4 primary-pre-failed-locality).
- Popup tests cover friendly labels, Tauri-clipboard Copy feedback, saved-SOURCE Retry in result + error states (P1-3), Retry hidden when no source, multi-result Retry, settings nav, recovery CTA, aria-disabled TTS/Favorite.
- InputPanel tests cover multi/partial/all-failed, friendly labels, autosave/restore/clear/focus.
- `provider_get_active_selection`, `open_settings_window`, `translate_selection_ipc` are registered in `invoke_handler!` AND `build.rs` AND the relevant capabilities.
- `Payload.source_text` is present on every state's payload (P1-3).
- The popup controller saves `payload.source_text` on loading/error/result/multi and clears on a new session.
- No `navigator.clipboard` call remains in `src/Popup.tsx` (grep).
- Clippy clean.

Stop here. Do not begin Stage C until the reviewer signs off.

---

## Stage C: Surface 05-06 and Settings Shell

Checkpoint goal: Provider Center cold-loads the active selection (fail-closed) and shows only the 4 supported presets; all Provider Center states render (C3 split into 8 sub-tasks with COMPLETE test code using `vi.hoisted + invokeMock + routeInvoke`); the settings shell entry is styled and sized; the sidebar is accessible in rail mode with real keyboard support (Playwright, P1-9); the macOS Accessibility status surfaces in settings with re-check (verified listener registration, P1-9).

### Task C1: Provider Center cold-start active selection (fail-closed)

**Files:**
- Modify: `src/features/settings/ProviderCenter.tsx` — read active selection on mount; FAIL-CLOSED on read error.
- Test: `test/ProviderCenter.test.tsx` — extend.

**Interfaces:**
- Consumes: `providerGetActiveSelection()` (B3), `loadProviders()` (existing).
- Produces: `selection()` seeded with stored primary/parallel/fallback on cold load. Fail-closed: BOTH reads must succeed before role badges are applied; on failure, an error UI + Retry render and `providerSetActive` is NOT called.

**rev-5-5 fixture note (load-bearing):** `test/ProviderCenter.test.tsx:68-71` calls `invokeMock.mockReset()` in `beforeEach`, which clears the `mockImplementation` set inside `vi.hoisted`. So putting the default route table INSIDE the `vi.hoisted` `invokeMock` does NOT survive to the next test — the second test onward throws `unexpected invoke provider_list`. The fix: define a module-level `DEFAULT_ROUTES` table (covers `provider_list`, `key_status`, AND `provider_get_active_selection` returning an empty selection), and re-install it in `beforeEach` right after `mockReset()`. Tests that need a custom route call `routeInvoke({ ...DEFAULT_ROUTES, ...custom })` so they inherit the cold-load defaults.

Update the top of `test/ProviderCenter.test.tsx`:

```ts
// rev-5-5: the DEFAULT route table. Re-installed in beforeEach after mockReset()
// so every test gets provider_list/key_status/provider_get_active_selection
// satisfied without per-test overrides. This is the cold-load contract.
const DEFAULT_ROUTES: Record<string, (args?: unknown) => unknown> = {
  provider_list: () => [],
  key_status: () => ({}),
  provider_get_active_selection: () => ({
    primary: null,
    parallel: [],
    fallback: null,
  }),
};
```

Then update `beforeEach` (test/ProviderCenter.test.tsx:68) to re-install the defaults right after `mockReset()`:

```ts
beforeEach(() => {
  localeMock.current = "en";
  invokeMock.mockReset();
  // rev-5-5: re-install the default route table AFTER mockReset() (mockReset
  // clears the mockImplementation set by vi.hoisted, so the defaults must be
  // re-attached here or the next test throws "unexpected invoke provider_list").
  routeInvoke(DEFAULT_ROUTES);
});
```

The existing `routeInvoke` helper (test/ProviderCenter.test.tsx:60-66) already calls `invokeMock.mockImplementation(...)` on the route table it receives, so `routeInvoke(DEFAULT_ROUTES)` re-points the mock at the defaults. A test that needs a custom route merges: `routeInvoke({ ...DEFAULT_ROUTES, provider_list: () => [profile()] })`.

This satisfies rev-5-5: every test inherits the cold-load defaults, and the `vi.hoisted` `invokeMock` no longer needs a default `mockImplementation` (it can stay a bare `vi.fn(async () => undefined)` since `beforeEach` always installs `DEFAULT_ROUTES` before any render).

- [x] **Step 1: Write the failing tests**

Append to `test/ProviderCenter.test.tsx`. Use the `routeInvoke` helper that already exists in the file:

```ts
it("cold-loads the stored active selection into role badges", async () => {
  routeInvoke({
    provider_list: () => [
      { uuid: "u1", name: "OpenAI", secret_ref: "provider/u1", template_id: "openai", protocol: "openai_chat", endpoint: "", model: null, enabled: true, sort_order: 0, is_local: false, needs_key: true, status: "active", capabilities: { balance: false, quota: false, model_list: false } },
      { uuid: "u2", name: "Anthropic", secret_ref: "provider/u2", template_id: "anthropic", protocol: "anthropic", endpoint: "", model: null, enabled: true, sort_order: 1, is_local: false, needs_key: true, status: "active", capabilities: { balance: false, quota: false, model_list: false } },
    ],
    key_status: () => ({ "provider/u1": true, "provider/u2": true }),
    provider_get_active_selection: () => ({ primary: "u1", parallel: ["u2"], fallback: null }),
  });

  const { findAllByText } = render(() => <ProviderCenter />);
  expect((await findAllByText(/Primary|主引擎/)).length).toBeGreaterThan(0);
  expect((await findAllByText(/Parallel|并[行联]/)).length).toBeGreaterThan(0);
  cleanup();
});

it("fail-closed: shows error + Retry and does NOT call providerSetActive when reads fail", async () => {
  routeInvoke({
    provider_list: () => { throw new Error("db locked"); },
    provider_get_active_selection: () => { throw new Error("db locked"); },
  });

  const { findByText } = render(() => <ProviderCenter />);
  expect(await findByText(/加载失败|load failed/i)).toBeTruthy();
  expect(
    invokeMock.mock.calls.some((c) => c[0] === "provider_set_active"),
  ).toBe(false);
  cleanup();
});
```

- [x] **Step 2: Run test to verify it fails**

Run: `pnpm vitest run test/ProviderCenter.test.tsx`
Expected: FAIL — ProviderCenter does not call `provider_get_active_selection`; the fail-closed path does not exist.

- [x] **Step 3: Wire cold-load selection (fail-closed) into ProviderCenter**

Edit `src/features/settings/ProviderCenter.tsx`. Replace the `refresh` + `onMount` block with a fail-closed version:

```tsx
  const [selectionLoading, setSelectionLoading] = createSignal(true);
  const [selectionError, setSelectionError] = createSignal(false);

  const refresh = async () => {
    setSelectionLoading(true);
    setSelectionError(false);
    // P1-8 fail-closed: BOTH reads must resolve before roles are applied. If
    // either throws, show an error + Retry and do NOT call providerSetActive.
    try {
      const [list, active] = await Promise.all([
        loadProviders(),
        providerGetActiveSelection(),
      ]);
      setProviders(list);
      setSelection({
        primaryUuid: active.primary,
        parallelUuids: active.parallel,
        fallbackUuid: active.fallback,
      });
      setLoadError(false);
    } catch (e) {
      setLoadError(true);
      setSelectionError(true);
      pushToast("destructive", t.saveFailed);
    } finally {
      setSelectionLoading(false);
    }
  };

  onMount(() => {
    void refresh();
  });
```

Add the import `import { providerGetActiveSelection } from "./provider-ipc";`. Gate role mutations in every handler that calls `providerSetActive`:

```tsx
if (selectionLoading() || selectionError()) return;
```

Render an error + Retry banner when `selectionError()` is true:

```tsx
      <Show when={selectionError()}>
        <div class="provider-center__error" role="alert">
          <span>{t.loadFailed}</span>
          <Button variant="secondary" size="sm" onClick={() => void refresh()}>
            {t.retry}
          </Button>
        </div>
      </Show>
```

Add `loadFailed` and `retry` copy keys to `src/features/settings/copy.ts` (both zh + en).

- [x] **Step 4: Run test to verify it passes**

Run: `pnpm vitest run test/ProviderCenter.test.tsx`
Expected: PASS.

- [x] **Step 5: Commit**

```bash
git diff --check
git add src/features/settings/ProviderCenter.tsx src/features/settings/copy.ts test/ProviderCenter.test.tsx
git commit -m "feat(provider-center): fail-closed cold-load of active selection + default mock fixture (P1-7)"
```

---

### Task C2: Drop Google/DeepL from the Provider Center preset list

**Files:**
- Modify: `src/features/settings/ProviderCenter.tsx` — remove the `google` and `deepl` entries from `PRESETS` (lines 85-86).
- Test: `test/ProviderCenter.test.tsx` — extend.

- [x] **Step 1: Write the failing test (deterministic RED, P1-7: proper destructuring)**

Append to `test/ProviderCenter.test.tsx`:

```ts
it("preset list contains only the 4 supported AI presets", async () => {
  routeInvoke({
    provider_list: () => [],
    key_status: () => {},
    provider_get_active_selection: () => ({ primary: null, parallel: [], fallback: null }),
  });
  const { findByRole, queryByText } = render(() => <ProviderCenter />);
  const addBtn = await findByRole("button", { name: /添加|Add/ });
  fireEvent.click(addBtn);
  expect(await findByText("OpenAI")).toBeTruthy();
  expect(await findByText("Anthropic")).toBeTruthy();
  expect(await findByText("Gemini")).toBeTruthy();
  expect(await findByText("Ollama")).toBeTruthy();
  expect(queryByText(/Google Translate/)).toBeNull();
  expect(queryByText(/^DeepL$/)).toBeNull();
  cleanup();
});
```

If the preset menu toggle is not a button named "Add", read `ProviderCenter.tsx` around line 629 for the actual toggle and click that element. The RED must come from the assertions, not a wrong selector.

- [x] **Step 2: Run test to verify it fails**

Run: `pnpm vitest run test/ProviderCenter.test.tsx preset`
Expected: FAIL — Google/DeepL appear.

- [x] **Step 3: Remove google + deepl from PRESETS**

Delete lines 84-86 (the comment + two entries).

- [x] **Step 4: Run test to verify it passes**

Run: `pnpm vitest run test/ProviderCenter.test.tsx`
Expected: PASS.

- [x] **Step 5: Commit**

```bash
git diff --check
git add src/features/settings/ProviderCenter.tsx test/ProviderCenter.test.tsx
git commit -m "fix(provider-center): drop Google/DeepL presets (4 AI presets only)"
```

---

### Task C3: Provider Center state coverage (split into 8 COMPLETE sub-tasks)

> **P1-7 + P1-8 (rev-11 latency amendment):** every sub-task below uses the `routeInvoke` helper from `ProviderCenter.test.tsx`. **C3c (Connection, rev-11 A-path):** `provider_test_connection` returns `{ ok, message, latency_ms? }` and the UI displays `message` PLUS `{latency}ms` when `latency_ms` is present (latency IS implemented this stage per user-approved scope decision; the pre-rev-11 "no latency" wording is superseded). **C3f (Balance, B-path deferred):** shows "Balance check not yet available" and calls NO IPC (no `provider_get_balance`) — Balance states are deferred to R4/S3 per user-approved scope decision. C3 adds ZERO new backend commands — C3c's `latency_ms` is an ADDITIVE FIELD on the existing `ConnectionResult`, not a new command.

**Files (shared):**
- Modify: `src/features/settings/ProviderCenter.tsx`.
- Modify: `src/features/settings/copy.ts` for new copy keys.
- Test: `test/ProviderCenter.test.tsx` — each sub-task appends its test(s).

**Shared default route table** — the `invokeMock` default from C1 already routes `provider_list`/`key_status`/`provider_get_active_selection`. Sub-task tests override per-need via `routeInvoke`.

#### Task C3a: Duplicate provider

- [x] **Step 1: Write the failing test**

```ts
it("Duplicate button creates a new keyless provider copy", async () => {
  routeInvoke({
    provider_list: () => [
      { uuid: "u1", name: "OpenAI", secret_ref: "provider/u1", template_id: "openai", protocol: "openai_chat", endpoint: "https://api.openai.com/v1/chat/completions", model: "gpt-4o-mini", enabled: true, sort_order: 0, is_local: false, needs_key: true, status: "active", capabilities: { balance: false, quota: false, model_list: false } },
    ],
    key_status: () => ({ "provider/u1": true }),
    provider_get_active_selection: () => ({ primary: "u1", parallel: [], fallback: null }),
  });
  const { findByRole } = render(() => <ProviderCenter />);
  const dupBtn = await findByRole("button", { name: /复制|Duplicate/ });
  fireEvent.click(dupBtn);
  await flush();
  expect(invokeMock.mock.calls.some((c) => c[0] === "provider_duplicate")).toBe(true);
  cleanup();
});
```

- [x] **Step 2: Run → RED** (`pnpm vitest run test/ProviderCenter.test.tsx` — fails: no Duplicate button).
- [x] **Step 3: Implement** the Duplicate button in the provider row actions; wire `providerDuplicate(uuid)`.
- [x] **Step 4: Run → GREEN**.
- [x] **Step 5: Commit**:

```bash
git diff --check
git add src/features/settings/ProviderCenter.tsx test/ProviderCenter.test.tsx
git commit -m "feat(provider-center): Duplicate provider action (C3a)"
```

#### Task C3b: Empty Key / Save disabled / Save conflict

- [x] **Step 1: Write the failing tests (COMPLETE bodies, P1-7)**

```ts
it("Save button is disabled when the key field is empty for a needs-key provider", async () => {
  routeInvoke({
    provider_list: () => [
      { uuid: "u1", name: "OpenAI", secret_ref: "provider/u1", template_id: "openai", protocol: "openai_chat", endpoint: "https://api.openai.com/v1/chat/completions", model: "gpt-4o-mini", enabled: true, sort_order: 0, is_local: false, needs_key: true, status: "active", capabilities: { balance: false, quota: false, model_list: false } },
    ],
    key_status: () => ({ "provider/u1": false }),
    provider_get_active_selection: () => ({ primary: null, parallel: [], fallback: null }),
  });
  const { findByRole, getByRole } = render(() => <ProviderCenter />);
  const editBtn = await findByRole("button", { name: /编辑|Edit/ });
  fireEvent.click(editBtn);
  const saveBtn = getByRole("button", { name: /保存|Save/ });
  expect(saveBtn.hasAttribute("disabled")).toBe(true);
  cleanup();
});

it("Save shows a conflict error when the backend returns a unique-violation", async () => {
  routeInvoke({
    provider_list: () => [
      { uuid: "u1", name: "OpenAI", secret_ref: "provider/u1", template_id: "openai", protocol: "openai_chat", endpoint: "https://api.openai.com/v1/chat/completions", model: "gpt-4o-mini", enabled: true, sort_order: 0, is_local: false, needs_key: true, status: "active", capabilities: { balance: false, quota: false, model_list: false } },
    ],
    key_status: () => ({ "provider/u1": false }),
    provider_get_active_selection: () => ({ primary: null, parallel: [], fallback: null }),
    provider_update: () => { throw new Error("UNIQUE constraint failed: providers.name"); },
  });
  const { findByRole, getByRole, findByText } = render(() => <ProviderCenter />);
  const editBtn = await findByRole("button", { name: /编辑|Edit/ });
  fireEvent.click(editBtn);
  const nameInput = document.querySelector("input[name='name']") as HTMLInputElement;
  if (nameInput) fireEvent.input(nameInput, { target: { value: "OpenAI" } });
  const keyInput = document.querySelector("input[name='key'], input[type='password']") as HTMLInputElement;
  if (keyInput) fireEvent.input(keyInput, { target: { value: "sk-test" } });
  const saveBtn = getByRole("button", { name: /保存|Save/ });
  if (!saveBtn.hasAttribute("disabled")) fireEvent.click(saveBtn);
  await flush();
  expect(await findByText(/已存在|already exists|UNIQUE/i)).toBeTruthy();
  cleanup();
});
```

- [x] **Step 2: Run → RED.**
- [x] **Step 3: Implement** the disabled-until-valid logic (Save `disabled` when `needs_key && keyText().length === 0`) + the conflict error mapping (detect "UNIQUE constraint" in the error and surface a localized "name already exists" message).
- [x] **Step 4: Run → GREEN.**
- [x] **Step 5: Commit**:

```bash
git diff --check
git add src/features/settings/ProviderCenter.tsx src/features/settings/copy.ts test/ProviderCenter.test.tsx
git commit -m "feat(provider-center): Save disabled on empty key + conflict error (C3b)"
```

#### Task C3c: Connection message + latency (rev-11: user-approved A-path — latency IS implemented this stage; rev-12: saturation + Instant-probe test)

> **rev-11 (supersedes P1-8 "no latency"); rev-12 (P2-5 hardens the conversion + adds a real-Instant test):** `provider_test_connection` returns `{ ok: bool, message: String, latency_ms: Option<u32> }`. The `latency_ms` field is a rev-11 user-approved ADDITIVE field, set on the reachable HTTP path via `Instant::now()`/`elapsed()` timing inside the existing `client.get(...).send().await` probe; it is `None` on the early-return failure arms (empty endpoint, invalid endpoint, no HTTP client) and on the transport-error arm. The UI displays `message` PLUS `{latency}ms` when `latency_ms` is `Some`; it shows `message` alone when `latency_ms` is `None`. C3c still adds ZERO new backend commands — `latency_ms` is a field on the existing `ConnectionResult`, not a new command. **rev-12 (P2-5):** the timing is factored into a `pub fn measure_latency_ms(start: Instant) -> u32` pure helper (`u32::try_from(start.elapsed().as_millis()).unwrap_or(u32::MAX)` — saturating, clippy-clean; was an inline `as u32` cast in rev-11), and `connection_latency.rs` gains a third test asserting `latency_ms` reflects a REAL `Instant` probe (not a hardcoded constant).

**Files:**
- Modify: `src-tauri/src/lib.rs` — `ConnectionResult` (line 1449) gains `latency_ms: Option<u32>`; `provider_test_connection` (line 1507) wraps the probe in timing; **rev-12 (P2-5):** add `pub fn measure_latency_ms(start: std::time::Instant) -> u32` (saturating u128→u32 conversion) next to `provider_test_connection` and call it from the reachable arm.
- Modify: `src/features/settings/ProviderCenter.tsx` — render `{message} · {latency}ms` when `latency_ms` is present.
- Test: `test/ProviderCenter.test.tsx` — extend the connection-test tests with latency assertions.
- Test: `src-tauri/tests/connection_latency.rs` — assert `ConnectionResult` carries `latency_ms`; **rev-12:** assert `measure_latency_ms` reflects a real `Instant` probe.

**Interfaces:**
- `ConnectionResult { ok: bool, message: String, latency_ms: Option<u32> }` — `#[derive(Debug, Clone, Serialize)]` (already derived; the new field rides the existing serde).
- The frontend `ConnectionResultFE` mirror type (in `src/features/settings/provider-types.ts`) gains `latency_ms?: number | null`.

- [x] **Step 1: Write the failing backend test**

Create `src-tauri/tests/connection_latency.rs`:

```rust
//! Task C3c (rev-11): `provider_test_connection` returns a `ConnectionResult`
//! carrying `latency_ms: Option<u32>`. This integration test asserts the field
//! EXISTS on the serialized shape (set on the reachable path, None on the
//! early-return failure paths). The HTTP probe itself is not exercised here —
//! that requires a live socket; the field's presence + serde serialization is
//! the contract this test pins.
//!
//! We construct a `ConnectionResult` directly via the public type and assert
//! serde emits `latency_ms`. The reachable-vs-failure branching is verified by
//! the unit logic of the command (the `Instant` timing is inline in the command
//! body and cannot be exercised without a live socket).
use linguaray_lib::ConnectionResult;
use serde_json::json;

#[test]
fn connection_result_serializes_latency_ms_some() {
    let r = ConnectionResult {
        ok: true,
        message: "reachable (HTTP 200)".into(),
        latency_ms: Some(42),
    };
    let v = serde_json::to_value(&r).expect("serialize");
    assert_eq!(v["ok"], json!(true));
    assert_eq!(v["message"], json!("reachable (HTTP 200)"));
    assert_eq!(v["latency_ms"], json!(42));
}

#[test]
fn connection_result_serializes_latency_ms_none() {
    // rev-11: the early-return failure arms + transport-error arm return None.
    let r = ConnectionResult {
        ok: false,
        message: "endpoint not configured".into(),
        latency_ms: None,
    };
    let v = serde_json::to_value(&r).expect("serialize");
    assert_eq!(v["ok"], json!(false));
    assert_eq!(v["latency_ms"], json!(null));
}

// ─── rev-12 (P2-5): assert latency_ms reflects a REAL Instant probe ─────────
// rev-11's two tests above only pin the SERIALIZED SHAPE — they do not verify
// `provider_test_connection` actually times the HTTP round trip. rev-12 adds
// this test against a pure timing helper (`measure_latency_ms`) that the
// command body calls (instead of inlining the cast), so the timing logic is
// unit-testable without a live socket. The helper is added to lib.rs in Step 3
// (right next to `provider_test_connection`); it is `pub` so this integration
// test can reach it via `linguaray_lib::measure_latency_ms`.
use linguaray_lib::measure_latency_ms;

#[test]
fn measure_latency_ms_reflects_real_instant_elapsed() {
    // A fresh Instant has near-zero elapsed → well under 5ms (allows scheduler
    // jitter). This pins that the helper reads `elapsed()`, not a constant.
    let fresh = std::time::Instant::now();
    let measured = measure_latency_ms(fresh);
    assert!(
        measured <= 5,
        "a fresh Instant should measure ~0ms, got {measured}"
    );

    // An Instant from 2 seconds ago → >= 1900ms (allows minor scheduling delay,
    // but definitely in the seconds range, NOT a tiny constant).
    let two_seconds_ago = std::time::Instant::now() - std::time::Duration::from_secs(2);
    let measured_old = measure_latency_ms(two_seconds_ago);
    assert!(
        measured_old >= 1900,
        "a 2s-old Instant should measure ~2000ms, got {measured_old}"
    );
}
```

If `ConnectionResult` is not currently re-exported at the crate root, expose it in `lib.rs` (`pub use ...::ConnectionResult;` or declare the struct `pub` at its existing location — it is already `pub` at lib.rs:1449, so the integration test reaches it via `linguaray_lib::ConnectionResult` once the `latency_ms` field is added in Step 3).

- [x] **Step 2: Run the backend test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --features xproc-test-helper --test connection_latency`
Expected: FAIL — `no field \`latency_ms\` on type \`ConnectionResult\`` (the struct at lib.rs:1449 currently has only `ok` + `message`) AND `cannot find function \`measure_latency_ms\` in crate \`linguaray_lib\`` (rev-12 helper not yet defined).

- [x] **Step 3: Add latency_ms to ConnectionResult + time the probe**

Edit `src-tauri/src/lib.rs`. The `ConnectionResult` struct (lib.rs:1449) currently is:

```rust
#[derive(Debug, Clone, Serialize)]
pub struct ConnectionResult {
    pub ok: bool,
    pub message: String,
}
```

Change it to:

```rust
/// Result of a connection probe (P1 #8, rev-11 latency amendment).
///
/// `ok` + a human-readable message + the measured round-trip latency in
/// milliseconds. `latency_ms` is `Some` ONLY on the reachable HTTP-200 path
/// (rev-11); it is `None` on the early-return failure arms and on the
/// transport-error arm (no successful round-trip was measured).
#[derive(Debug, Clone, Serialize)]
pub struct ConnectionResult {
    pub ok: bool,
    pub message: String,
    /// rev-11: round-trip latency of the reachability probe, in milliseconds.
    /// `Some` only when the HTTP request completed (reachable path); `None`
    /// otherwise.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u32>,
}
```

Then update `provider_test_connection` (lib.rs:1507). The current reachable/failure arms (lib.rs:1525-1560) construct `ConnectionResult { ok, message }` WITHOUT `latency_ms`. Every construction site must be updated. The full replacement for the body from the `client.get(...)` probe onward (lib.rs:1538-1560) is:

```rust
    // Best-effort reachability probe. We don't care about the response body —
    // any HTTP response (even a 401/404) means the endpoint is reachable; only
    // a transport-level failure (connect/timeout/TLS) counts as "not ok".
    // rev-11: time the round trip so the UI can show {latency}ms on success.
    // rev-12: the timing is factored into `measure_latency_ms` (defined just
    // below this command) so the integration test can verify it uses a real
    // `Instant` probe without a live socket.
    let client = match session.client.as_ref() {
        Some(c) => c,
        None => {
            return Ok(ConnectionResult {
                ok: false,
                message: "HTTP client unavailable: startup build failed".into(),
                latency_ms: None,
            })
        }
    };
    let start = std::time::Instant::now();
    let req = client.get(&profile.endpoint).send().await;
    match req {
        Ok(resp) => {
            // rev-12: `measure_latency_ms` does the saturating u128→u32 conversion
            // (was `as u32` in rev-11, which risks clippy `cast_possible_truncation`).
            let latency_ms = Some(measure_latency_ms(start));
            Ok(ConnectionResult {
                ok: true,
                message: format!("reachable (HTTP {})", resp.status().as_u16()),
                latency_ms,
            })
        }
        Err(e) => Ok(ConnectionResult {
            ok: false,
            message: format!("connection failed: {e}"),
            latency_ms: None,
        }),
    }
```

**rev-12 (P2-5):** define the `measure_latency_ms` helper near `provider_test_connection` (same file, lib.rs). This is the pure timing function the command above calls and the integration test exercises:

```rust
/// rev-12 (P2-5): measure the elapsed milliseconds since `start` as a `u32`,
/// saturating at `u32::MAX` (clippy-clean; `as_millis()` returns u128). Used by
/// `provider_test_connection` so the timing logic is unit-testable without a
/// live socket. Exposed as `pub` so `tests/connection_latency.rs` can reach it
/// via `linguaray_lib::measure_latency_ms`.
pub fn measure_latency_ms(start: std::time::Instant) -> u32 {
    u32::try_from(start.elapsed().as_millis()).unwrap_or(u32::MAX)
}
```

> **rev-12 (P2-5, also fix in this task):** add a third test to `src-tauri/tests/connection_latency.rs` (Step 1's file) that asserts `latency_ms` reflects a REAL `Instant::now()`/`elapsed()` measurement rather than a hardcoded constant. Because `provider_test_connection` hits a live socket (cannot run in CI without a network), expose the timing logic as a tiny testable helper OR assert the field is `Some` and `<= elapsed` against a measured boundary. The simplest verifiable form: add a `pub fn measure_latency_ms(start: std::time::Instant) -> u32` pure helper (returns `u32::try_from(start.elapsed().as_millis()).unwrap_or(u32::MAX)`) and assert in the test that `measure_latency_ms(Instant::now()) <= 5` (a fresh `Instant` has near-zero elapsed) and that `measure_latency_ms(Instant::now() - Duration::from_secs(2)) >= 1900`. This pins that the field is derived from `Instant`, not a constant.

The THREE early-return arms before the probe (lib.rs:1525-1537 — empty endpoint, invalid endpoint) also need `latency_ms: None` added to their `ConnectionResult` literals. The `spawn_blocking` that reads the profile (lib.rs:1515-1523) is unchanged.

- [x] **Step 4: Run the backend test to verify it passes**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --features xproc-test-helper --test connection_latency`
Expected: PASS (3 tests — 2 serialization-shape from rev-11 + 1 rev-12 `measure_latency_ms_reflects_real_instant_elapsed`). Then run the full backend suite + clippy to confirm no regression:

```bash
cargo build --manifest-path src-tauri/Cargo.toml --features xproc-test-helper
cargo test --manifest-path src-tauri/Cargo.toml --features xproc-test-helper
cargo clippy --manifest-path src-tauri/Cargo.toml --features xproc-test-helper --all-targets -- -D warnings
```

- [x] **Step 5: Write the failing frontend test**

In `test/ProviderCenter.test.tsx`, REPLACE the two connection-test cases (the success + failure cases from the pre-rev-11 C3c) with cases that assert the latency renders. The new test bodies:

```ts
it("Connection test shows the success message with latency (rev-11)", async () => {
  routeInvoke({
    provider_list: () => [
      { uuid: "u1", name: "OpenAI", secret_ref: "provider/u1", template_id: "openai", protocol: "openai_chat", endpoint: "https://api.openai.com/v1/chat/completions", model: "gpt-4o-mini", enabled: true, sort_order: 0, is_local: false, needs_key: true, status: "active", capabilities: { balance: false, quota: false, model_list: false } },
    ],
    key_status: () => ({ "provider/u1": true }),
    provider_get_active_selection: () => ({ primary: null, parallel: [], fallback: null }),
    // rev-11: latency_ms is now part of the ConnectionResult shape.
    provider_test_connection: () => ({ ok: true, message: "reachable (HTTP 200)", latency_ms: 42 }),
  });
  const { findByRole, findByText } = render(() => <ProviderCenter />);
  const testBtn = await findByRole("button", { name: /测试连接|Test Connection/ });
  fireEvent.click(testBtn);
  await flush();
  expect(await findByText(/reachable \(HTTP 200\)/i)).toBeTruthy();
  // rev-11: the latency MUST render alongside the message.
  expect(await findByText(/42\s*ms/i)).toBeTruthy();
  cleanup();
});

it("Connection test shows the failure message WITHOUT latency when latency_ms is absent (rev-11)", async () => {
  routeInvoke({
    provider_list: () => [
      { uuid: "u1", name: "OpenAI", secret_ref: "provider/u1", template_id: "openai", protocol: "openai_chat", endpoint: "https://api.openai.com/v1/chat/completions", model: "gpt-4o-mini", enabled: true, sort_order: 0, is_local: false, needs_key: true, status: "active", capabilities: { balance: false, quota: false, model_list: false } },
    ],
    key_status: () => ({ "provider/u1": true }),
    provider_get_active_selection: () => ({ primary: null, parallel: [], fallback: null }),
    // rev-11: transport failures return latency_ms: null — the UI must NOT
    // render a latency chip in that case.
    provider_test_connection: () => ({ ok: false, message: "401 Unauthorized", latency_ms: null }),
  });
  const { findByRole, findByText } = render(() => <ProviderCenter />);
  const testBtn = await findByRole("button", { name: /测试连接|Test Connection/ });
  fireEvent.click(testBtn);
  await flush();
  expect(await findByText(/401 Unauthorized/i)).toBeTruthy();
  const latencyChip = document.body.textContent?.match(/\d+\s*ms/);
  expect(latencyChip, "latency must NOT render when latency_ms is null").toBeNull();
  cleanup();
});
```

- [x] **Step 6: Run the frontend test to verify it fails**

Run: `pnpm vitest run test/ProviderCenter.test.tsx`
Expected: FAIL — the connection row does not yet render `{latency}ms` (Step 5 implements the backend field but the UI still shows `message` only).

- [x] **Step 7: Implement the frontend latency display**

In `src/features/settings/ProviderCenter.tsx`, find the connection-test result render (the row that shows the `message` from `provider_test_connection`). Extend it to render `{latency}ms` when `latency_ms` is a non-null number. The TypeScript mirror type for the connection result (in `src/features/settings/provider-types.ts`) gains:

```ts
export interface ConnectionResultFE {
  ok: boolean;
  message: string;
  /** rev-11: round-trip latency in ms. Null on early-return/transport failures. */
  latency_ms?: number | null;
}
```

The render uses a conditional chip — when `result.latency_ms != null`, render a span with `{latency_ms} ms` (use the semantic `--color-muted` token family for the chip text, NOT a hardcoded hex). When `latency_ms` is `null`/`undefined`, render ONLY the `message` (no chip).

- [x] **Step 8: Run the frontend test to verify it passes**

Run: `pnpm vitest run test/ProviderCenter.test.tsx`
Expected: PASS — both connection cases pass (latency renders on success; no latency chip on null).

- [x] **Step 9: Commit**

```bash
git diff --check
git add src-tauri/src/lib.rs src-tauri/tests/connection_latency.rs src/features/settings/ProviderCenter.tsx src/features/settings/provider-types.ts test/ProviderCenter.test.tsx
git commit -m "feat(provider-center): connection test message + latency_ms + saturating measure_latency_ms helper (rev-11/rev-12, C3c A-path)"
```

> **rev-11 note:** the pre-rev-11 C3c was "message only, no latency (P1-8)". rev-11 supersedes that: latency is now a user-approved implemented field. The Global Constraints "No invented backend contracts (P1-8, rev-11 latency amendment)" line and the P1-8 summary line at the top of this plan reflect the new shape; the earlier "no latency" wording in those lines is replaced, NOT retained alongside.

#### Task C3d: Delete focus / deleting / retry

- [x] **Step 1: Write the failing tests (COMPLETE bodies, P1-7)**

```ts
it("Delete confirm-cancel returns focus to the delete trigger", async () => {
  routeInvoke({
    provider_list: () => [
      { uuid: "u1", name: "OpenAI", secret_ref: "provider/u1", template_id: "openai", protocol: "openai_chat", endpoint: "", model: null, enabled: true, sort_order: 0, is_local: false, needs_key: true, status: "active", capabilities: { balance: false, quota: false, model_list: false } },
    ],
    key_status: () => ({ "provider/u1": true }),
    provider_get_active_selection: () => ({ primary: null, parallel: [], fallback: null }),
  });
  const { findByRole } = render(() => <ProviderCenter />);
  const deleteBtn = await findByRole("button", { name: /删除|Delete/ });
  deleteBtn.focus();
  fireEvent.click(deleteBtn);
  const cancelBtn = await findByRole("button", { name: /取消|Cancel/ });
  fireEvent.click(cancelBtn);
  await flush();
  expect(document.activeElement).toBe(deleteBtn);
  cleanup();
});

it("Delete failure offers a Retry and stays in the deleting state until done", async () => {
  let deleteAttempts = 0;
  routeInvoke({
    provider_list: () => [
      { uuid: "u1", name: "OpenAI", secret_ref: "provider/u1", template_id: "openai", protocol: "openai_chat", endpoint: "", model: null, enabled: true, sort_order: 0, is_local: false, needs_key: true, status: "active", capabilities: { balance: false, quota: false, model_list: false } },
    ],
    key_status: () => ({ "provider/u1": true }),
    provider_get_active_selection: () => ({ primary: null, parallel: [], fallback: null }),
    provider_delete: () => {
      deleteAttempts += 1;
      if (deleteAttempts === 1) throw new Error("db locked");
      return {};
    },
  });
  const { findByRole } = render(() => <ProviderCenter />);
  const deleteBtn = await findByRole("button", { name: /删除|Delete/ });
  fireEvent.click(deleteBtn);
  const confirmBtn = await findByRole("button", { name: /确认删除|Confirm Delete/ });
  fireEvent.click(confirmBtn);
  await flush();
  const retryBtn = await findByRole("button", { name: /重试|Retry/ });
  fireEvent.click(retryBtn);
  await flush();
  expect(deleteAttempts).toBe(2);
  cleanup();
});
```

- [x] **Step 2: Run → RED.**
- [x] **Step 3: Implement** focus restore (store the trigger ref, restore on cancel) + the deleting/retry state.
- [x] **Step 4: Run → GREEN.**
- [x] **Step 5: Commit**:

```bash
git diff --check
git add src/features/settings/ProviderCenter.tsx test/ProviderCenter.test.tsx
git commit -m "feat(provider-center): delete focus restore + deleting/retry state (C3d)"
```

#### Task C3e: Disabled provider roles

- [x] **Step 1: Write the failing test (COMPLETE body, P1-7)**

```ts
it("a disabled provider's role badge is hidden / not settable", async () => {
  routeInvoke({
    provider_list: () => [
      { uuid: "u1", name: "OpenAI", secret_ref: "provider/u1", template_id: "openai", protocol: "openai_chat", endpoint: "", model: null, enabled: false, sort_order: 0, is_local: false, needs_key: true, status: "active", capabilities: { balance: false, quota: false, model_list: false } },
    ],
    key_status: () => ({ "provider/u1": true }),
    provider_get_active_selection: () => ({ primary: null, parallel: [], fallback: null }),
  });
  const { findByText, queryByText } = render(() => <ProviderCenter />);
  expect(await findByText("OpenAI")).toBeTruthy();
  expect(queryByText(/设为主引擎|Set as Primary/i)).toBeNull();
  expect(queryByText(/设为并行|Set as Parallel/i)).toBeNull();
  cleanup();
});
```

- [x] **Step 2: Run → RED.**
- [x] **Step 3: Implement** the disabled-role gating: when `provider.enabled === false`, do not render the role-assign buttons (or render them with `aria-disabled`).
- [x] **Step 4: Run → GREEN.**
- [x] **Step 5: Commit**:

```bash
git diff --check
git add src/features/settings/ProviderCenter.tsx test/ProviderCenter.test.tsx
git commit -m "feat(provider-center): hide role controls for disabled providers (C3e)"
```

#### Task C3f: Balance placeholder only (B-path: deferred to R4/S3 per user-approved scope decision, rev-11)

> **rev-11 (B-path):** Balance states (loading/unsupported/rate-limited/error) are deferred to R4/S3 per user-approved scope decision. `provider_get_balance` does NOT exist in the backend. The Balance UI shows a static "Balance check not yet available" string and calls NO IPC. This test asserts the placeholder renders, NOT a fetched value. (rev-10 framed this as "P1-8 frozen Scheme A"; rev-11 re-frames it as an explicit user-approved deferral — the implementation is unchanged, only the governance note.)

- [x] **Step 1: Write the failing test (COMPLETE body, P1-8: placeholder, no IPC)**

```ts
it("balance shows a 'not yet available' placeholder and calls no balance IPC (P1-8)", async () => {
  routeInvoke({
    provider_list: () => [
      { uuid: "u1", name: "OpenAI", secret_ref: "provider/u1", template_id: "openai", protocol: "openai_chat", endpoint: "", model: null, enabled: true, sort_order: 0, is_local: false, needs_key: true, status: "active", capabilities: { balance: true, quota: false, model_list: false } },
    ],
    key_status: () => ({ "provider/u1": true }),
    provider_get_active_selection: () => ({ primary: null, parallel: [], fallback: null }),
  });
  const { findByText } = render(() => <ProviderCenter />);
  expect(await findByText(/余额.*暂不可用|Balance.*not yet available/i)).toBeTruthy();
  // P1-8: NO provider_get_balance IPC exists and none is called.
  expect(invokeMock.mock.calls.some((c) => c[0] === "provider_get_balance")).toBe(false);
  cleanup();
});
```

- [x] **Step 2: Run → RED.**
- [x] **Step 3: Implement** a static placeholder row for the balance field: when `capabilities.balance === true`, render the localized "Balance check not yet available" string. **P1-8: do NOT invoke any IPC.**
- [x] **Step 4: Run → GREEN.**
- [x] **Step 5: Commit**:

```bash
git diff --check
git add src/features/settings/ProviderCenter.tsx src/features/settings/copy.ts test/ProviderCenter.test.tsx
git commit -m "feat(provider-center): balance placeholder only — no IPC (C3f, P1-8)"
```

#### Task C3g: Model fetch (loading / error / manual entry)

- [x] **Step 1: Write the failing tests (COMPLETE bodies, P1-7)**

```ts
it("model fetch error falls back to manual entry", async () => {
  routeInvoke({
    provider_list: () => [
      { uuid: "u1", name: "OpenAI", secret_ref: "provider/u1", template_id: "openai", protocol: "openai_chat", endpoint: "", model: null, enabled: true, sort_order: 0, is_local: false, needs_key: true, status: "active", capabilities: { balance: false, quota: false, model_list: true } },
    ],
    key_status: () => ({ "provider/u1": true }),
    provider_get_active_selection: () => ({ primary: null, parallel: [], fallback: null }),
    provider_get_models: () => { throw new Error("network"); },
  });
  const { findByRole } = render(() => <ProviderCenter />);
  const editBtn = await findByRole("button", { name: /编辑|Edit/ });
  fireEvent.click(editBtn);
  const modelInput = await findByRole("textbox", { name: /模型|model/i });
  expect(modelInput.tagName).toBe("INPUT");
  cleanup();
});

it("model fetch loading shows a spinner", async () => {
  routeInvoke({
    provider_list: () => [
      { uuid: "u1", name: "OpenAI", secret_ref: "provider/u1", template_id: "openai", protocol: "openai_chat", endpoint: "", model: null, enabled: true, sort_order: 0, is_local: false, needs_key: true, status: "active", capabilities: { balance: false, quota: false, model_list: true } },
    ],
    key_status: () => ({ "provider/u1": true }),
    provider_get_active_selection: () => ({ primary: null, parallel: [], fallback: null }),
    provider_get_models: () => new Promise(() => {}),
  });
  const { findByRole, container } = render(() => <ProviderCenter />);
  const editBtn = await findByRole("button", { name: /编辑|Edit/ });
  fireEvent.click(editBtn);
  const spinner = await waitFor(() =>
    container.querySelector(".spinner, [aria-busy='true'], [role='status']"),
  );
  expect(spinner).toBeTruthy();
  cleanup();
});
```

- [x] **Step 2: Run → RED.**
- [x] **Step 3: Implement** the model-fetch state machine: `loading` (spinner/`aria-busy`) → `select` (dropdown of fetched models) | `error` (manual `<input>`). Only fetch when `capabilities.model_list === true`.
- [x] **Step 4: Run → GREEN.**
- [x] **Step 5: Commit**:

```bash
git diff --check
git add src/features/settings/ProviderCenter.tsx test/ProviderCenter.test.tsx
git commit -m "feat(provider-center): model fetch loading/error->manual entry (C3g)"
```

#### Task C3h: Toast aria-label + Tooltip

- [x] **Step 1: Write the failing tests (COMPLETE bodies, P1-7; NO conditional assertions)**

```ts
it("toast has role=status and a non-empty aria-label", async () => {
  routeInvoke({
    provider_list: () => [],
    key_status: () => {},
    provider_get_active_selection: () => ({ primary: null, parallel: [], fallback: null }),
    provider_create: () => ({ uuid: "u-new" }),
  });
  const { findByRole } = render(() => <ProviderCenter />);
  const addBtn = await findByRole("button", { name: /添加|Add/ });
  fireEvent.click(addBtn);
  const createBtn = await findByRole("button", { name: /创建|Create/ });
  fireEvent.click(createBtn);
  await flush();
  const toast = await findByRole("status");
  const label = (toast.getAttribute("aria-label") ?? toast.textContent ?? "").trim();
  expect(label.length).toBeGreaterThan(0);
  cleanup();
});

it("Tooltip trigger carries an accessible label", async () => {
  const { container } = render(() => <SettingsShell><div /></SettingsShell>);
  const triggers = container.querySelectorAll("[data-tooltip], [aria-label]");
  expect(triggers.length).toBeGreaterThan(0);
  const trigger = triggers[0] as HTMLElement;
  trigger.focus();
  await flush();
  const describedBy = trigger.getAttribute("aria-describedby");
  if (describedBy) {
    const tip = document.getElementById(describedBy);
    expect(tip, "tooltip content element must exist for aria-describedby").toBeTruthy();
    expect(((tip as HTMLElement)?.textContent ?? "").trim().length).toBeGreaterThan(0);
  } else {
    const label = (trigger.getAttribute("aria-label") ?? trigger.getAttribute("title") ?? "").trim();
    expect(label.length).toBeGreaterThan(0);
  }
  cleanup();
});
```

> The C3h tooltip test's `if (describedBy)` branch is a structural variant (the component may use either aria-describedby or aria-label), NOT a "skip if absent" — both branches assert a non-empty accessible label. This satisfies P1-7 (no vacuous pass: each branch has a hard assertion).

- [x] **Step 2: Run → RED.**
- [x] **Step 3: Implement** the toast `role="status"` + `aria-label` (the toast message), and wire the Tooltip so its trigger carries `aria-describedby` pointing at the tooltip content element when open.
- [x] **Step 4: Run → GREEN.**
- [x] **Step 5: Commit**:

```bash
git diff --check
git add src/features/settings/ProviderCenter.tsx packages/ui/src/components/Toast.tsx packages/ui/src/components/Tooltip.tsx test/ProviderCenter.test.tsx
git commit -m "feat(provider-center): toast role=status + aria-label + tooltip accessible label (C3h)"
```

**C3 final step:** run `pnpm vitest run test/ProviderCenter.test.tsx` and confirm the full suite (C3a–C3h) passes.

---

### Task C4: Settings entry styling + window sizing + theme-color meta variants

**Files:**
- Modify: `index.html` — fix title/favicon; theme-color meta with light + dark `media` variants (P2: dark token `#020617`).
- Verify: `src/index.tsx` already imports `@linguaray/ui/styles` (A1 Step 5).
- Verify: `src-tauri/tauri.conf.json` main window is 800×600 default + 600×400 min (A4 Step 10).
- Test: `test/entry-styling.test.ts` **(new)**.

- [x] **Step 1: Write the failing test**

Create `test/entry-styling.test.ts`:

```ts
import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";

describe("settings entry styling", () => {
  it("index.html title is LinguaRay, not the Tauri default", () => {
    const html = readFileSync("index.html", "utf-8");
    expect(html).not.toContain("Tauri + Solid + Typescript App");
    expect(html).toMatch(/<title>LinguaRay/);
  });

  it("index.html has BOTH light and dark theme-color metas (P2)", () => {
    const html = readFileSync("index.html", "utf-8");
    expect(html).toMatch(/theme-color[^>]*media="\(prefers-color-scheme: light\)"/);
    expect(html).toMatch(/theme-color[^>]*media="\(prefers-color-scheme: dark\)"/);
  });

  it("index.html dark theme-color is the canvas token #020617 (P2)", () => {
    const html = readFileSync("index.html", "utf-8");
    expect(html).toMatch(/theme-color[^>]*#020617/);
    expect(html).not.toMatch(/theme-color[^>]*#0B1120/);
  });

  it("index.html theme-color is not the placeholder #000000", () => {
    const html = readFileSync("index.html", "utf-8");
    expect(html).not.toMatch(/theme-color.*#000000/);
  });

  it("index.html favicon points at a logo asset", () => {
    const html = readFileSync("index.html", "utf-8");
    expect(html).toMatch(/rel="icon"[^>]*logo/);
  });
});
```

- [x] **Step 2: Run test to verify it fails**

Run: `pnpm vitest run test/entry-styling.test.ts`
Expected: FAIL — the title is still the Tauri default; theme-color is `#000000` with no media variants.

- [x] **Step 3: Fix index.html**

Replace `index.html`'s `<head>` (P2: two theme-color metas, dark token `#020617`):

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <meta name="theme-color" media="(prefers-color-scheme: light)" content="#F8FAFC" />
    <meta name="theme-color" media="(prefers-color-scheme: dark)" content="#020617" />
    <link rel="icon" type="image/svg+xml" href="/src/assets/logo.svg" />
    <title>LinguaRay — Settings</title>
  </head>
  <body>
    <noscript>You need to enable JavaScript to run this app.</noscript>
    <div id="root"></div>
    <script src="/src/index.tsx" type="module"></script>
  </body>
</html>
```

Confirm `/src/assets/logo.svg` exists. The theme-color values are meta-tag content (not `src/` CSS), so the no-hex scan does not apply.

- [x] **Step 4: Run test to verify it passes**

Run: `pnpm vitest run test/entry-styling.test.ts`
Expected: PASS (5 tests).

- [x] **Step 5: Commit**

```bash
git diff --check
git add index.html test/entry-styling.test.ts
git commit -m "fix(entry): settings title/favicon + light/dark theme-color metas (P2: dark token #020617)"
```

---

### Task C5: Settings sidebar rail-mode accessibility — aria-disabled + real Playwright keyboard test (P1-9)

> **P1-9 (load-bearing):** disabled placeholders use `aria-disabled="true"` + `tabindex={0}` on a FOCUSABLE control. The keyboard test uses Playwright (installed in D5) against an isolated `?page=settings-keyboard` route with real `page.keyboard.press("Tab")` + `"Enter"`. jsdom cannot compute native Tab focus movement, so the keyboard contract is verified in Playwright, not Vitest.

**Files (rev-7-6: keyboard spec in apps/ui-lab, port 1421; CSS :focus locator):**
- Modify: `src/features/settings/SettingsShell.tsx` — rail mode keeps the accessible name; disabled placeholder aria-label uses the REAL copy value `t.nav.placeholderHint` ("Coming in R3b" / "将在 R3b 中提供", verified at `src/features/settings/copy.ts:206` + `:325`); disabled items are `aria-disabled` (focusable), NOT native `disabled`. (rev-8-1 applied the minimal additive edit in A4 — the existing WindowChrome/Tooltip/matchMedia/close/minimize are all kept; C5 only verifies the aria/label contracts.)
- Modify: `packages/ui/src/components/SidebarItem.tsx` — accept + forward `ariaLabel`; render disabled items with `aria-disabled` + `tabindex={0}` (NOT native `disabled`).
- Modify: `apps/ui-lab/src/App.tsx` — **rev-8-2 + rev-8-3:** add a `?nav=settings-keyboard` route that renders the REAL `SettingsShell` (imported from `@app/features/settings/SettingsShell` via the `@app` alias) with a local `activePage` signal + `onNavigate` handler; the shell root carries `data-testid="shell"` + `data-page`. A `FIXTURE_NAV_KEYS` array (containing `settings-keyboard` + `confirm-isolated`) is merged into `validNav` so the route is accepted without appearing in the gallery `NAV_ITEMS`. This route is the Playwright fixture target. NO root `playwright.config.ts` and NO root `e2e/` directory are created.
- **(new)** `apps/ui-lab/e2e/keyboard.spec.ts` — **rev-8-4:** the Playwright keyboard spec lives in the ui-lab (port 1421, the existing `apps/ui-lab/playwright.config.ts`), NOT in a root `e2e/keyboard.spec.ts` against port 1420. Reuses the lab's committed Playwright install + webServer config. Locator: the PRECISE selector `page.locator('[data-testid="shell"] .settings-shell__nav .sidebar-item:focus')` (rev-8-4: NOT the ambiguous `button:focus`).
- Test: `test/SettingsShell.test.tsx` — Vitest for the aria/label contracts (rail accessible name, aria-disabled focusable, disabled aria-label).

- [x] **Step 1: Write the failing Vitest tests (aria/label contracts)**

Append to `test/SettingsShell.test.tsx`. The rev-9-2 test uses `createSignal` (from `solid-js`) + `SettingsSection` (named type from the `SettingsShell` module). The existing file imports `SettingsShell` (default) + `{ render, cleanup }` from `@solidjs/testing-library` + the file-local `installMatchMedia` helper. Merge `createSignal` into the existing `solid-js` import line and `SettingsSection` into the existing `SettingsShell` import line so the rev-9-2 test type-checks:

```diff
- import { render, cleanup } from "@solidjs/testing-library";
- import SettingsShell from "../../src/features/settings/SettingsShell";
+ import { createSignal } from "solid-js";
+ import { render, cleanup } from "@solidjs/testing-library";
+ import SettingsShell, { type SettingsSection } from "../../src/features/settings/SettingsShell";
```

```ts
// rev-8-5 (load-bearing): the rail/wide mode is driven by matchMedia
// ("(min-width: 700px)"), NOT by window.innerWidth. Setting window.innerWidth
// does NOT fire a matchMedia change in jsdom, so the wide() signal would never
// re-evaluate. Use the existing installMatchMedia(matchesWide) helper from
// test/SettingsShell.test.tsx (mirrors ProviderCenter.interactions.test.tsx):
// installMatchMedia(false) => rail mode; installMatchMedia(true) => wide mode.

it("rail mode (matchMedia wide=false) keeps an accessible name on every nav item", () => {
  installMatchMedia(false); // rail mode — NOT window.innerWidth
  const { getAllByRole } = render(() => (
    <SettingsShell>
      <div />
    </SettingsShell>
  ));
  const items = getAllByRole("button");
  expect(items.length).toBeGreaterThan(0);
  for (const item of items) {
    const name = item.getAttribute("aria-label") ?? item.textContent ?? "";
    expect(name.trim().length, `nav item missing accessible name`).toBeGreaterThan(0);
  }
  cleanup();
});

it("disabled nav items are aria-disabled AND focusable (NOT native disabled) — P1-9", () => {
  installMatchMedia(true); // wide mode
  const { getAllByRole } = render(() => (
    <SettingsShell>
      <div />
    </SettingsShell>
  ));
  const disabled = getAllByRole("button").filter((b) => b.getAttribute("aria-disabled") === "true");
  expect(disabled.length).toBeGreaterThan(0);
  for (const b of disabled) {
    expect(b.hasAttribute("disabled")).toBe(false);
    expect(b.getAttribute("tabindex")).not.toBe("-1");
  }
  cleanup();
});

// rev-8-5 (load-bearing): the disabled placeholder aria-label uses the REAL
// copy value t.nav.placeholderHint = "Coming in R3b" (en) / "将在 R3b 中提供"
// (zh), verified at src/features/settings/copy.ts:206 + :325. The ariaLabel is
// `${item.label} — ${t.nav.placeholderHint}` (rev-8-1 edit 4). Assert the REAL
// value, NOT an invented "Coming later" string.
it("disabled placeholder nav item announces the real placeholderHint copy (Coming in R3b)", () => {
  installMatchMedia(true); // wide mode
  const { getAllByRole } = render(() => (
    <SettingsShell>
      <div />
    </SettingsShell>
  ));
  const disabled = getAllByRole("button").filter((b) => b.getAttribute("aria-disabled") === "true");
  expect(disabled.length).toBeGreaterThan(0);
  for (const b of disabled) {
    const name = b.getAttribute("aria-label") ?? b.textContent ?? "";
    // rev-8-5: the real en copy is "Coming in R3b" (copy.ts:206). The aria-label
    // is `${label} — Coming in R3b`. Assert the real substring.
    expect(name).toMatch(/Coming in R3b/);
  }
  cleanup();
});

// rev-9-2 (load-bearing): SettingsShell is a TRUE controlled component. A plain
// createSignal(props.activePage ?? ...) initializer would read props.activePage
// ONCE and then go stale when the parent passes a new value. This test renders
// the REAL (un-mocked) shell, drives `activePage` via a parent-owned signal, and
// asserts BOTH the root data-page AND the sidebar highlight track the new value.
// This is the test that would have FAILED against rev-8-1's read-once initializer.
it("controlled activePage prop reactively updates data-page + sidebar highlight (rev-9-2)", () => {
  installMatchMedia(true); // wide mode
  const [page, setPage] = createSignal<SettingsSection>("provider-center");
  const { container, getByTestId } = render(() => (
    <SettingsShell activePage={page()} onNavigate={(p) => setPage(p)}>
      <div />
    </SettingsShell>
  ));

  // Initial: data-page + highlight reflect the initial controlled value.
  const shell = getByTestId("shell");
  expect(shell.getAttribute("data-page")).toBe("provider-center");
  let activeItem = container.querySelector('.sidebar-item[aria-current="page"]');
  expect(activeItem?.textContent ?? "").toContain("Provider");

  // rev-9-2: parent changes activePage — the shell MUST follow reactively.
  setPage("keystore-recovery");
  expect(shell.getAttribute("data-page")).toBe("keystore-recovery");
  activeItem = container.querySelector('.sidebar-item[aria-current="page"]');
  expect(activeItem?.textContent ?? "").toContain("Keystore");

  cleanup();
});
```

- [x] **Step 2: Write the failing Playwright keyboard test (real Tab + Enter, P1-9 + rev-8-4)**

**rev-8-4 (load-bearing):** the keyboard spec lives at `apps/ui-lab/e2e/keyboard.spec.ts` (port 1421, the existing `apps/ui-lab/playwright.config.ts` which already wires the webServer + committed baselines). It targets the `?nav=settings-keyboard` route added to `apps/ui-lab/src/App.tsx` (Step 4). NO root `e2e/keyboard.spec.ts`, NO root Playwright config, NO port 1420. The shell root locator is `[data-testid="shell"]`; the focused-nav-item locator is the PRECISE `[data-testid="shell"] .settings-shell__nav .sidebar-item:focus` (rev-8-4: NOT `[data-testid="shell"] button:focus`, which is ambiguous — the WindowChrome close/minimize buttons also live inside the shell).

Create `apps/ui-lab/e2e/keyboard.spec.ts`:

```ts
import { test, expect } from "@playwright/test";

/**
 * rev-8-4: runs in apps/ui-lab (port 1421) against the ?nav=settings-keyboard
 * route, which renders the REAL SettingsShell with data-testid="shell" +
 * data-page. Reuses apps/ui-lab/playwright.config.ts (webServer + chromium).
 *
 * Tab past the OS/browser window-control focus trap. In a real Chromium window
 * the FIRST Tab can land on the window's traffic-light / tabbar controls (not
 * the sidebar). Loop Tab until a sidebar nav item is focused, with a sane cap
 * so the test fails (not hangs) if the sidebar never appears.
 *
 * rev-8-4 (load-bearing): the focused item is located via the PRECISE selector
 * `page.locator('[data-testid="shell"] .settings-shell__nav .sidebar-item:focus')`.
 * Do NOT use `[data-testid="shell"] button:focus` — the shell also contains the
 * WindowChrome close/minimize buttons (`<button class="window-chrome__...">`),
 * so `button:focus` is ambiguous and can match a window control. The
 * `.settings-shell__nav` scope (verified at SettingsShell.tsx:123:
 * `<nav class="settings-shell__nav">`) + `.sidebar-item` (the SidebarItem root
 * class) together identify ONLY nav items. The :focus pseudo-class is evaluated
 * live by the browser.
 */
async function tabUntilSidebarItemFocused(page: import("@playwright/test").Page): Promise<import("@playwright/test").Locator> {
  // rev-8-4: scoped to the settings nav — NOT all buttons in the shell.
  const sidebarItemFocused = '[data-testid="shell"] .settings-shell__nav .sidebar-item:focus';
  for (let i = 0; i < 12; i++) {
    await page.keyboard.press("Tab");
    const focused = page.locator(sidebarItemFocused);
    if ((await focused.count()) > 0) {
      return focused.first();
    }
  }
  throw new Error("Tab never reached a sidebar nav item within 12 presses (window-control focus trap not escaped)");
}

test.describe("settings sidebar keyboard nav (P1-9, rev-8-4)", () => {
  test("Tab moves focus between enabled nav items; Enter activates", async ({ page }) => {
    await page.setViewportSize({ width: 1000, height: 800 });
    await page.addInitScript(() => localStorage.setItem("linguaray.theme", "light"));
    // rev-8-4: ui-lab route on port 1421.
    await page.goto("http://localhost:1421/?nav=settings-keyboard&theme=light");
    await page.waitForSelector("[data-testid='shell']", { timeout: 10_000 });

    // Tab past the OS window-control trap onto the FIRST enabled sidebar item.
    const firstItem = await tabUntilSidebarItemFocused(page);
    await expect(firstItem).toBeVisible();
    // The first enabled item must NOT be aria-disabled (it is a real nav target).
    await expect(firstItem).not.toHaveAttribute("aria-disabled", "true");

    // rev-8-4: read the aria-label (or data-section) to identify the item by a
    // concrete string value — NOT by comparing Locator objects.
    const firstLabel = await firstItem.getAttribute("aria-label");

    // Tab again. Focus must land on the NEXT sidebar nav item. Assert by
    // comparing aria-label VALUES (a concrete string), not Locator references.
    await page.keyboard.press("Tab");
    const secondFocused = page.locator('[data-testid="shell"] .settings-shell__nav .sidebar-item:focus');
    await expect(secondFocused).toHaveCount(1);
    const secondLabel = await secondFocused.first().getAttribute("aria-label");
    expect(secondLabel, "second Tab must move focus to a different item (different aria-label)").not.toBe(firstLabel);
    expect(secondLabel, "second focused item must have a non-empty aria-label").toBeTruthy();

    // rev-8-4: Tab back to the first enabled item, then Enter — the navigate
    // handler fires and the shell root's data-page changes. (If the first
    // enabled item is already active, Tab to the next ENABLED item first.)
    const firstIsDisabled = await firstItem.getAttribute("aria-disabled");
    if (firstIsDisabled === "true") {
      // Re-focus an enabled item via Shift+Tab back, then Enter on it.
      await page.keyboard.press("Shift+Tab");
    }
    const before = await page.getAttribute("[data-testid='shell']", "data-page");
    await page.keyboard.press("Enter");
    await page.waitForTimeout(50);
    const after = await page.getAttribute("[data-testid='shell']", "data-page");
    expect(after, "data-page must be set on the shell root").not.toBeNull();
    // rev-8-4: Enter on an ENABLED item MUST change data-page (the navigate
    // handler fired). This is the load-bearing assertion — comparing the before
    // vs after data-page VALUES (strings), not Locator objects.
    expect(after, "Enter on an enabled item must navigate (data-page changed)").not.toBe(before);
  });

  test("disabled nav items are focusable (tabindex=0) but Enter is a no-op", async ({ page }) => {
    await page.setViewportSize({ width: 1000, height: 800 });
    await page.addInitScript(() => localStorage.setItem("linguaray.theme", "light"));
    await page.goto("http://localhost:1421/?nav=settings-keyboard&theme=light");
    await page.waitForSelector("[data-testid='shell']", { timeout: 10_000 });

    // rev-8-4: scope the disabled-item query to the settings nav so it cannot
    // match a WindowChrome control. A disabled item carries aria-disabled="true"
    // + tabindex="0" (focusable).
    const disabled = page.locator('[data-testid="shell"] .settings-shell__nav .sidebar-item[aria-disabled="true"]').first();
    await expect(disabled).toHaveAttribute("tabindex", "0");
    await disabled.focus();
    const before = await page.getAttribute("[data-testid='shell']", "data-page");
    await page.keyboard.press("Enter");
    await page.waitForTimeout(50);
    const after = await page.getAttribute("[data-testid='shell']", "data-page");
    // Enter on a disabled item does NOT navigate.
    expect(after).toBe(before);
  });
});
```

- [x] **Step 3: Run tests to verify they fail**

Run Vitest: `pnpm vitest run test/SettingsShell.test.tsx`
Expected: FAIL — rail-mode items have no persistent accessible name; disabled items use native `disabled` (not focusable); the rev-9-2 controlled-component test fails because `active` is still a read-once `createSignal` (changing the parent's `activePage` does not update `data-page` or the sidebar highlight).

Run Playwright (rev-6-7: ui-lab): `pnpm --filter @linguaray/ui-lab test:visual keyboard`
Expected: FAIL — the `?nav=settings-keyboard` route does not exist in `apps/ui-lab/src/App.tsx`; `[data-testid="shell"]` is not found; Tab does not move focus.

- [x] **Step 4: Add aria-disabled focusable controls + the ui-lab `?nav=settings-keyboard` route (rev-6-7)**

Edit `packages/ui/src/components/SidebarItem.tsx` — accept `ariaLabel`; render disabled items with `aria-disabled="true"` + `tabindex={0}` + NO native `disabled`:

```tsx
export type SidebarItemProps = {
  label: string;
  ariaLabel?: string;
  icon: JSX.Element;
  active?: boolean;
  badge?: string;
  onClick?: () => void;
  disabled?: boolean;
};

const SidebarItem: Component<SidebarItemProps> = (props) => {
  // P1-9: a "disabled" item is aria-disabled (focusable via tabindex=0), NOT
  // native disabled. It stays in the tab order so AT users can perceive it.
  return (
    <button
      type="button"
      class="sidebar-item"
      classList={{ "sidebar-item--active": !!props.active }}
      aria-current={props.active ? "page" : undefined}
      aria-label={props.ariaLabel ?? props.label}
      aria-disabled={props.disabled ? "true" : undefined}
      tabindex={props.disabled ? 0 : undefined}
      onClick={() => {
        if (props.disabled) return;
        props.onClick?.();
      }}
    >
      <span class="sidebar-item__icon" aria-hidden="true">{props.icon}</span>
      <span class="sidebar-item__label">{props.label}</span>
      <Show when={props.badge}>
        <span class="sidebar-item__badge">{props.badge}</span>
      </Show>
    </button>
  );
};
```

In `src/features/settings/SettingsShell.tsx` (rev-8-1 + rev-9-2 diff-instruction edits landed in A4), the existing `navItems` array marks `shortcuts` + `privacy` with `disabled: true` and passes `disabled={item.disabled}` to each `SidebarItem`; the updated `SidebarItem` (Step 4) renders those as `aria-disabled="true"` + `tabindex={0}`. The existing `renderItem` already wraps every disabled (and every rail) item in a `Tooltip` whose content is `t.nav.placeholderHint` ("Coming in R3b" / "将在 R3b 中提供"), and **rev-8-1 edit 4** passes `ariaLabel = \`${item.label} — ${t.nav.placeholderHint}\`` so the focusable disabled button announces both. **rev-9-2 (load-bearing):** `active()` is the DERIVATION `() => props.activePage ?? internalActive()` (A4 Step 7 edit 2), NOT a read-once `createSignal`, so when the ui-lab `settings-keyboard` fixture's parent `onNavigate` handler calls `setActivePage(p)`, the shell's `data-page` + sidebar highlight update REACTIVELY (this is what the rev-9-2 Vitest asserts, and what makes the Playwright Enter→`data-page` change observable). The shell's ROOT element carries `data-testid="shell"` + `data-page={active()}` (added by the A4 diff edit), so Playwright + real-DOM Vitest can read the active page WITHOUT mocking SettingsShell. C5 does NOT introduce a second SettingsShell body — the existing component (WindowChrome + Tooltip + matchMedia + close/minimize, all kept) is the single implementation. The keyboard spec reads `data-page` off this root to assert navigation.

**rev-8-2 + rev-8-3:** add the `?nav=settings-keyboard` route to `apps/ui-lab/src/App.tsx` (NOT the root `src/App.tsx`). The lab already has a `?nav=` router (verified at `apps/ui-lab/src/App.tsx:127-133`: `validNav = NAV_ITEMS.map(i => i.key)`, then `validNav.includes(params.get("nav"))`). **rev-8-3 (load-bearing):** `settings-keyboard` must be routable via `?nav=` but must NOT appear in the gallery navigation list. So add a separate `FIXTURE_NAV_KEYS` array and merge it into `validNav`, leaving `NAV_ITEMS` (the gallery sidebar source) untouched. The `SettingsSection` type is imported from the SAME module so no new type is created. **rev-8-2:** the import uses the `@app` alias (configured at `apps/ui-lab/{vite,vitest}.config.ts` + `tsconfig.json` → `../../src`), NOT a relative `../../../src/...` path.

```tsx
// apps/ui-lab/src/App.tsx — add to the NavKey union (line 28):
//   | "settings-keyboard"

// rev-8-3: fixture-only nav keys — routable via ?nav= but NOT shown in the
// gallery sidebar (they are not added to NAV_ITEMS). Merged into validNav below.
const FIXTURE_NAV_KEYS: NavKey[] = ["settings-keyboard", "confirm-isolated"];

// Add the imports (top of file, alongside the other page imports).
// rev-8-2: the `@app` alias resolves to <repo>/src (apps/ui-lab configs).
import SettingsShell, { type SettingsSection } from "@app/features/settings/SettingsShell";

// In the App component body, merge the fixture keys into validNav so the
// router accepts ?nav=settings-keyboard (verified current code at line 130-131):
//   const validNav = (NAV_ITEMS.map((i) => i.key) as NavKey[]);
//   const initialNav = validNav.includes(params.get("nav") as NavKey) ? ... : "selection-popup";
// Change to:
//   const validNav = ([...NAV_ITEMS.map((i) => i.key), ...FIXTURE_NAV_KEYS] as NavKey[]);
// (NAV_ITEMS itself is UNCHANGED, so the gallery sidebar does not list the fixture routes.)

// Inside the App component, add the isolated-fixture early return (after the
// confirm-isolated block, before the main <div class="lab"> return):
if (nav() === "settings-keyboard") {
  // rev-8-1 + rev-8-2: render the REAL SettingsShell (existing component —
  // WindowChrome + SidebarItem + Tooltip + matchMedia) with the controlled
  // activePage prop. The shell root carries data-testid="shell" + data-page
  // (added by the A4 minimal edit) so the Playwright keyboard spec reads the
  // active page. SettingsSection is the existing union (no new type).
  const [activePage, setActivePage] = createSignal<SettingsSection>("provider-center");
  return (
    <div class="gallery__iso" data-testid="lab-root">
      <SettingsShell
        activePage={activePage()}
        onNavigate={(p: SettingsSection) => setActivePage(p)}
      >
        <div data-testid="lab-shell-content">
          {activePage() === "provider-center"
            ? "Provider Center (lab fixture)"
            : activePage() === "keystore-recovery"
            ? "Keystore Recovery (lab fixture)"
            : activePage() === "shortcuts"
            ? "Shortcuts (Coming later)"
            : "Privacy (Coming later)"}
        </div>
      </SettingsShell>
    </div>
  );
}
```

This route renders ONLY the SettingsShell (the shell's own `WindowChrome` provides the frame; no lab header/nav/controls) so the keyboard spec's first Tab lands on a rail item. The root `src/App.tsx` is NOT modified for this route — the keyboard fixture lives in the lab, not the main app.

- [x] **Step 5: Run tests to verify they pass**

Run Vitest: `pnpm vitest run test/SettingsShell.test.tsx && pnpm --filter @linguaray/ui test`
Expected: PASS.

Run Playwright (rev-6-7: ui-lab): `pnpm --filter @linguaray/ui-lab test:visual keyboard`
Expected: PASS.

- [x] **Step 6: Commit**

```bash
git diff --check
git add packages/ui/src/components/SidebarItem.tsx test/SettingsShell.test.tsx apps/ui-lab/src/App.tsx apps/ui-lab/e2e/keyboard.spec.ts
git commit -m "fix(a11y): rail nav accessible name + aria-disabled focusable items + real Playwright keyboard test in apps/ui-lab (rev-6-7, P1-9)"
```

---

### Task C6: macOS Accessibility permission status in settings (with verified listener registration + Re-check + onFocus re-check, P1-9)

> **P1-9:** first assert `expect(listen).toHaveBeenCalledTimes(1)` (or the onFocusChanged equivalent) to prove the listener registered, THEN test the focus behavior. `onCleanup` calls `unlisten`.

**Files:**
- Modify: `src/features/settings/SettingsShell.tsx` — surface `a11y_status`; Re-check button; onFocus re-check; Open System Settings.
- Test: extend `test/SettingsShell.test.tsx`.

**Interfaces:**
- Consumes: `invoke<boolean>("a11y_status")`; `getCurrentWindow().onFocusChanged()`.
- Produces: a banner "Accessibility: Required" when `false`, with a Re-check button + an "Open System Settings" link. On window focus, re-run `a11y_status` (P1-9). The `onFocusChanged` listener is stored and called in `onCleanup`.

- [x] **Step 1: Write the failing tests (P1-9: assert listener registration first)**

Append to `test/SettingsShell.test.tsx`. Add the file-scope `@tauri-apps/api/window` + `invoke` mocks (via `vi.hoisted`), the `routeInvokeSettings` helper, and a `beforeEach` that resets + installs defaults (rev-5-8: complete helper, NOT a "mirror routeInvoke" instruction).

```ts
// rev-5-8: SettingsShell test scaffolding. The invoke mock + the route table
// helper live at module scope so every test can call routeInvokeSettings(...).
const { invokeMock, windowMock, onFocusChangedMock, unlistenMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(async (_cmd: string, _args?: unknown): Promise<unknown> => undefined),
  windowMock: { getCurrentWindow: vi.fn() },
  onFocusChangedMock: vi.fn(async () => (() => {}) as () => void),
  unlistenMock: vi.fn(() => {}),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => windowMock.getCurrentWindow(),
}));
vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: vi.fn(async () => undefined) }));

// rev-5-8: the DEFAULT route table for SettingsShell tests. a11y_status defaults
// to true (granted) so the warning banner is hidden unless a test overrides it.
const DEFAULT_SETTINGS_ROUTES: Record<string, (args?: unknown) => unknown> = {
  a11y_status: () => true,
};

/** Wire `invoke` to a route table merged over the settings defaults. */
function routeInvokeSettings(
  routes: Record<string, (args?: unknown) => unknown>,
): void {
  const merged = { ...DEFAULT_SETTINGS_ROUTES, ...routes };
  invokeMock.mockImplementation(async (cmd: string, args?: unknown) => {
    const fn = merged[cmd];
    if (!fn) throw new Error(`unexpected invoke ${cmd}`);
    return fn(args);
  });
}

beforeEach(() => {
  invokeMock.mockReset();
  onFocusChangedMock.mockReset();
  onFocusChangedMock.mockResolvedValue(unlistenMock);
  windowMock.getCurrentWindow.mockReturnValue({ onFocusChanged: onFocusChangedMock });
  // rev-5-8: re-install the default route table AFTER mockReset() (mockReset
  // clears the mockImplementation; same fix as ProviderCenter.test.tsx rev-5-5).
  routeInvokeSettings({});
});
```

```ts
it("shows the macOS Accessibility permission warning when not granted", async () => {
  routeInvokeSettings({ a11y_status: () => false });
  const { findByText, findByRole } = render(() => (
    <SettingsShell>
      <div />
    </SettingsShell>
  ));
  expect(await findByText(/Accessibility|辅助功能/)).toBeTruthy();
  expect(await findByRole("button", { name: /重新检查|Re-?check/i })).toBeTruthy();
  expect(await findByText(/系统设置|System Settings/)).toBeTruthy();
  cleanup();
});

it("Re-check re-invokes a11y_status", async () => {
  routeInvokeSettings({ a11y_status: () => false });
  const { findByRole } = render(() => (
    <SettingsShell>
      <div />
    </SettingsShell>
  ));
  const recheck = await findByRole("button", { name: /重新检查|Re-?check/i });
  invokeMock.mockClear();
  fireEvent.click(recheck);
  await flush();
  expect(invokeMock.mock.calls.some((c) => c[0] === "a11y_status")).toBe(true);
  cleanup();
});

it("registers exactly one onFocusChanged listener and re-checks on focus (P1-9)", async () => {
  onFocusChangedMock.mockResolvedValue(unlistenMock);
  windowMock.getCurrentWindow.mockReturnValue({ onFocusChanged: onFocusChangedMock });
  routeInvokeSettings({ a11y_status: () => false });
  render(() => (
    <SettingsShell>
      <div />
    </SettingsShell>
  ));
  await flush();
  // P1-9: prove the listener registered exactly once before testing behavior.
  expect(onFocusChangedMock).toHaveBeenCalledTimes(1);
  invokeMock.mockClear();
  const cb = onFocusChangedMock.mock.calls[0][0] as (e: { payload: boolean }) => void;
  cb({ payload: true });
  await flush();
  expect(invokeMock.mock.calls.some((c) => c[0] === "a11y_status")).toBe(true);
  cleanup();
});

it("onCleanup calls the unlisten returned by onFocusChanged (P1-9)", async () => {
  onFocusChangedMock.mockResolvedValue(unlistenMock);
  windowMock.getCurrentWindow.mockReturnValue({ onFocusChanged: onFocusChangedMock });
  routeInvokeSettings({ a11y_status: () => false });
  const { unmount } = render(() => (
    <SettingsShell>
      <div />
    </SettingsShell>
  ));
  await flush();
  unmount();
  expect(unlistenMock).toHaveBeenCalled();
  cleanup();
});
```

(`routeInvokeSettings` is the COMPLETE helper defined in Step 1 above — it merges the custom routes over `DEFAULT_SETTINGS_ROUTES` and re-installs them on the `invokeMock`. The C6 tests pass `{ a11y_status: () => false }` to force the warning-banner state.)

- [x] **Step 2: Run test to verify it fails**

Run: `pnpm vitest run test/SettingsShell.test.tsx`
Expected: FAIL — SettingsShell does not query `a11y_status` or listen to focus.

- [x] **Step 3: Surface a11y_status with Re-check + onFocus re-check + System Settings**

Edit `src/features/settings/SettingsShell.tsx`:

```tsx
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { openUrl } from "@tauri-apps/plugin-opener";

const [a11yGranted, setA11yGranted] = createSignal<boolean | null>(null);
let unlistenFocus: (() => void) | undefined;

const recheckA11y = async () => {
  try {
    setA11yGranted(await invoke<boolean>("a11y_status"));
  } catch {
    setA11yGranted(null);
  }
};

onMount(async () => {
  void recheckA11y();
  // P1-9: re-check when the window regains focus (user may have just granted).
  const un = await getCurrentWindow().onFocusChanged(({ payload: focused }) => {
    if (focused) void recheckA11y();
  });
  unlistenFocus = un;
});

onCleanup(() => {
  // P1-9: call the unlisten returned by onFocusChanged.
  unlistenFocus?.();
});
```

Render the banner (warning state only) with Re-check + Open System Settings:

```tsx
  <Show when={a11yGranted() === false}>
    <div class="settings-a11y-banner" role="status">
      <span>{t.a11y.title}: {t.a11y.hint}</span>
      <Button variant="ghost" size="sm" onClick={() => void recheckA11y()}>
        {t.a11y.recheck}
      </Button>
      <Button
        variant="ghost"
        size="sm"
        onClick={() => void openUrl("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")}
      >
        {t.a11y.openSettings}
      </Button>
    </div>
  </Show>
```

Add the `a11y.*` copy keys to `src/features/settings/copy.ts` (both zh + en):

```ts
// zh
a11y: {
  title: "辅助功能权限",
  hint: "未授予。打开系统设置 → 隐私与安全性 → 辅助功能。",
  recheck: "重新检查",
  openSettings: "打开系统设置",
},
// en
a11y: {
  title: "Accessibility Permission",
  hint: "Not granted. Open System Settings → Privacy & Security → Accessibility.",
  recheck: "Re-check",
  openSettings: "Open System Settings",
},
```

- [x] **Step 4: Run test to verify it passes**

Run: `pnpm vitest run test/SettingsShell.test.tsx`
Expected: PASS.

- [x] **Step 5: Commit**

```bash
git diff --check
git add src/features/settings/SettingsShell.tsx src/features/settings/copy.ts test/SettingsShell.test.tsx
git commit -m "feat(settings): a11y status + Re-check + verified onFocus re-check + unlisten on cleanup (P1-9)"
```

---

### Stage C Verification

Run before Stage D:

```bash
cargo build --manifest-path src-tauri/Cargo.toml --features xproc-test-helper
cargo test --manifest-path src-tauri/Cargo.toml --features xproc-test-helper
cargo clippy --manifest-path src-tauri/Cargo.toml --features xproc-test-helper --all-targets -- -D warnings
pnpm test
pnpm typecheck
pnpm build
pnpm --filter @linguaray/ui test
```

Confirm:
- ProviderCenter tests cover cold-load (success + fail-closed), 4-preset-only, and all 8 C3 sub-task states (COMPLETE test bodies via `routeInvoke`, no `// ...`). C3c (rev-11) shows `message` + `{latency}ms` when `latency_ms` is present (latency IS implemented this stage — the `connection_latency.rs` backend test pins the field, and `ProviderCenter.test.tsx` asserts the chip renders on success and is absent on null); C3f shows the placeholder and calls no balance IPC (Balance states deferred to R4/S3 per user-approved scope decision, rev-11).
- SettingsShell tests cover rail accessible name (650px), aria-disabled focusable items, disabled aria-label, a11y status + Re-check + verified onFocus listener registration + unlisten on cleanup (P1-9).
- index.html has the correct title/favicon + light/dark theme-color metas (dark token `#020617`).
- Clippy clean.

Stop here. Do not begin Stage D until the reviewer signs off.

---

## Stage D: Unified Verification, Visual Baselines, and Documentation Closure

Checkpoint goal: no legacy space aliases in `src/`; the test runner's `test:src` and a new `test:all` work; Playwright visual baselines capture ALL real surfaces (Provider Center, Keystore Recovery, Selection Popup, InputPanel) at 600/699/700/800 × light/dark with fixed IPC mock data; the R2a/R2b/R3a plan checkboxes reflect what actually shipped (via a retroactive table); the final verification sweep is green with `--features xproc-test-helper`, `clippy --all-targets -- -D warnings`, and `git diff --check`.

### Task D1: Sweep `--space-N` aliases + ban regressions

**Files:**
- Modify: `src/Popup.css` — replace `--space-1`/`--space-2`/`--space-3` with `--space-sm`/`--space-md`/`--space-lg`.
- Modify: `src/InputPanel.tsx` — already done in B2 (`var(--space-lg)`); verify.
- Create: `test/no-space-alias.test.ts` — guard.

- [x] **Step 1: Write the failing guard test**

Create `test/no-space-alias.test.ts`:

```ts
import { describe, it, expect } from "vitest";
import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";

const ALIAS = /--space-[0-9]/;

function walk(dir: string): string[] {
  const out: string[] = [];
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const full = join(dir, entry.name);
    if (entry.isDirectory()) out.push(...walk(full));
    else if (/\.(css|tsx?|ts)$/.test(entry.name) && !entry.name.endsWith(".test.ts")) {
      out.push(full);
    }
  }
  return out;
}

describe("no --space-N legacy aliases in src/", () => {
  it("no src/ file uses --space-1/2/3/etc.", () => {
    const files = walk("src");
    const offenders: string[] = [];
    for (const f of files) {
      const src = readFileSync(f, "utf-8");
      const stripped = src
        .replace(/\/\*[\s\S]*?\*\//g, "")
        .replace(/\/\/.*$/gm, "");
      if (ALIAS.test(stripped)) offenders.push(f);
    }
    expect(offenders, `legacy --space-N alias found in: ${offenders.join(", ")}`).toEqual([]);
  });
});
```

- [x] **Step 2: Run test to verify it fails**

Run: `pnpm vitest run test/no-space-alias.test.ts`
Expected: FAIL — `src/Popup.css` still uses `--space-1/2/3` (lines 22, 25, 31, 38, 50).

- [x] **Step 3: Replace the aliases in Popup.css**

Edit `src/Popup.css`. Replace each occurrence (verified line numbers):
- Line 22: `padding: var(--space-2, 8px);` → `padding: var(--space-md);`
- Line 25: `gap: var(--space-2, 8px);` → `gap: var(--space-md);`
- Line 31: `padding: var(--space-1, 4px);` → `padding: var(--space-sm);`
- Line 38: `gap: var(--space-2, 8px);` → `gap: var(--space-md);`
- Line 50: `padding: var(--space-3, 12px);` → `padding: var(--space-lg);`

Verify `src/InputPanel.tsx` is already `var(--space-lg)` (Task B2). Grep `--space-` across `src/` to catch stragglers.

- [x] **Step 4: Run the guard + the no-hex test**

Run: `pnpm vitest run test/no-space-alias.test.ts test/no-hardcoded-hex.test.ts`
Expected: PASS (both).

- [x] **Step 5: Commit**

```bash
git diff --check
git add src/Popup.css test/no-space-alias.test.ts
git commit -m "refactor(tokens): replace --space-N aliases with --space-sm/md/lg + guard test"
```

---

### Task D2: Fix Tooltip close behavior + Provider Center focus restore + `test:src`/`test:all`

**Ordering note:** fix the Tooltip close behavior and Provider Center focus restoration BEFORE touching the test environment.

**Files:**
- Modify: `packages/ui/src/components/Tooltip.tsx` — close on blur/escape.
- Modify: `src/features/settings/ProviderCenter.tsx` — focus restore on dialog close (overlaps with C3d; skip if already covered).
- Modify: `package.json` — fix `test:src`; add `test:all`.

- [x] **Step 1: Fix the Tooltip close behavior (RED → GREEN)**

Write a failing test that focuses the trigger, asserts the tooltip opens, blurs/presses Escape, asserts it closes. Implement `onBlur`/`onKeyDown(Escape)` handlers. (C3h's tooltip test already covers part of this; if C3h covered close-on-blur, skip and note it.)

- [x] **Step 2: Fix Provider Center focus restore**

If C3d's delete-focus test did not cover the general dialog-close restore, add it here. Otherwise skip and note "covered by C3d."

- [x] **Step 3: Inspect + fix the broken `test:src` script**

`package.json` line 14: `"test:src": "vitest run --root test",`. The `--root test` constrains the search root so `src/**/*.test.*` can't match. Drop the flag.

- [x] **Step 4: Edit package.json**

```json
  "scripts": {
    "start": "vite",
    "dev": "vite",
    "build": "vite build",
    "serve": "vite preview",
    "tauri": "tauri",
    "typecheck": "tsc --noEmit && pnpm -r typecheck",
    "test": "vitest run",
    "test:watch": "vitest",
    "test:src": "vitest run",
    "test:ui": "pnpm --filter @linguaray/ui test",
    "test:ui-lab": "pnpm --filter @linguaray/ui-lab test",
    "test:all": "pnpm test && pnpm test:ui && pnpm test:ui-lab"
  },
```

- [x] **Step 5: Run the fixed scripts**

Run: `pnpm test:src`, `pnpm test:ui`, `pnpm test:ui-lab`, `pnpm test:all`.
Expected: each runs the right suite.

- [x] **Step 6: Address remaining focus-related failures (no skips)**

Restructure any failing focus test to assert the programmatic contract (`aria-describedby`, `document.activeElement`), NOT the visual `:focus-visible` pseudo-class. No `.skip`, no threshold loosening.

- [x] **Step 7: Commit**

```bash
git diff --check
git add packages/ui/src/components/Tooltip.tsx packages/ui/src/components/Tooltip.test.tsx package.json
git commit -m "fix(ui+tests): Tooltip close + test:src/test:all scripts"
```

---

### Task D3: Sync R2a/R2b/R3a plan checkboxes via a retroactive table

**Files:**
- Modify: `docs/superpowers/plans/2026-08-09-r2a-parallel-translation.md`, `docs/superpowers/plans/2026-08-09-r2b-frontend-surfaces.md`, `docs/superpowers/plans/2026-08-09-r3a-settings-provider.md`.

**Method:** do NOT retro-edit historical RED states. Append a "Rev-4 retroactive status" table to each plan mapping each original task to its actual shipped state and where the audit plan covers it.

**Contract documents (rev-8-9 governance, load-bearing, rev-11/rev-12 scope resolution): the four design documents (MASTER.md, handoff-manifest.md, pages/04-tray-menu.md, pages/05-provider-center.md) are FROZEN — this plan does NOT modify them.** rev-11 resolved the previously-pending scope decisions (all five are now user-approved); rev-12 corrected the IMPLEMENTATION of two A-paths (Tray Error red-dot + Active pulse) to actually meet the frozen contract. The A-paths (Tray Error red-dot, Tray Active pulse, Connection latency) ARE implemented this stage; the B-paths (Update badge → R5/R6, Balance states → R4/S3) are deferred per user-approved scope decision. The differences are noted here so the reviewer can see the gap between the frozen contract and the shipped stage:
- **pages/04 Error icon (rev-11/rev-12: MET this stage):** the contract specified a red-dot overlay on error. rev-11 Task A5 first implemented it via a build-time-generated 32×32 PNG, BUT rev-11 filled the whole 32×32 buffer red (a solid-red square, NOT an overlay). **rev-12 (P1-2) corrects this:** `build.rs` now composites the red-dot ON TOP OF the app default base icon — `image::open("src-tauri/icons/32x32.png")` loads the base, then a ~10px-diameter `#DC2626` = `[220, 38, 38, 255]` dot is drawn at the top-right via a manual `put_pixel` circle test (`dx*dx + dy*dy <= r*r`). The runtime decodes it via `Image::from_bytes(include_bytes!(concat!(env!("OUT_DIR"), "/tray-error-32.png")))`. The frozen pages/04 red-dot requirement IS met this stage (rev-12 — a true overlay); the retroactive table records "implemented (rev-11/rev-12 A5; rev-12 overlay fix)".
- **pages/04 Active pulse (rev-11/rev-12: MET this stage):** the contract specified a pulse during in-flight translation. rev-11 Task A5 first implemented it via the "Translating…" tooltip ONLY (the icon was left at the app default; a live icon-pulse animation was declared out of scope). **rev-12 (P1-1) corrects this to a VISIBLE pulse:** `TrayStateController` spawns a background `tokio::task` timer (every 800ms) that swaps `tray.set_icon(normal)` ↔ `tray.set_icon(dimmed)` — the dimmed variant (`tray-active-32.png`, a ~60%-brightness version of the base icon) is also build-time-generated. The localized tooltip ("Translating…"/"翻译中…") remains as an auxiliary signal. The frozen pages/04 Active-pulse requirement IS met this stage (rev-12 — a real icon-level pulse, NOT just a tooltip).
- **pages/04 Update badge (rev-11/rev-12: deferred per user-approved scope decision):** the contract specified an update-available badge. This stage does NOT implement the badge — deferred to R5/R6 (the updater backend does not exist). The `TrayVisualState::UpdateAvailable` variant is RETAINED in the enum so the priority ordering is testable, but `recompute` NEVER produces it this stage. The retroactive table records "deferred to R5/R6 per user-approved scope decision (rev-11)".
- **pages/04 OCR/History:** disabled with the real copy ("Coming later" tray label; the settings disabled-item copy is the real `t.nav.placeholderHint` = "Coming in R3b"). Not removed.
- **pages/05 Connection latency (rev-11/rev-12: MET this stage):** the contract implied a latency readout. rev-11 Task C3c implements it: `ConnectionResult.latency_ms: Option<u32>` (set on the reachable HTTP path via `Instant` timing, `None` on failure) + frontend `{latency}ms` chip. **rev-12 (P2-5) hardens:** the `as_millis() as u32` truncation → `u32::try_from(...).unwrap_or(u32::MAX)` (saturation, clippy-clean) + a test asserting `latency_ms` reflects a real `Instant` probe (not a constant). The frozen pages/05 latency requirement IS met this stage.
- **pages/05 Balance (rev-11: deferred per user-approved scope decision):** "not yet available" placeholder, no IPC. Balance states are deferred to R4/S3 per user-approved scope decision (the balance IPC backend does not exist). The retroactive table records "deferred to R4/S3 per user-approved scope decision (rev-11)".
- **MASTER.md / handoff-manifest.md:** the scope-reduction differences (OCR/History/Update-badge deferred, Balance placeholder) are recorded in THIS plan's retroactive table — the frozen docs are NOT edited (rev-8-9 reverses the earlier "carry the rev-5 scope-reduction note into the docs" instruction).

Each annotation is a row in the retroactive table with the column "Design-doc difference" noting the proposal status + the rev-8-9 governance rule ("frozen — not modified; separate approval required").

- [x] **Step 1: For each plan, build the retroactive table**

Open each of the three plans. For every original task, determine from git history + the current code whether it shipped, shipped differently, or did not ship. Append a table at the END of each plan:

```markdown
---

## Rev-4 Retroactive Status (2026-08-09)

Appended by the R2/R3a contract audit (docs/superpowers/plans/2026-08-09-r2-r3-contract-audit-fixes.md).
Historical RED states are preserved as-written; this table records the actual
shipped state and where gaps are closed.

| Original task | Shipped? | Gap closed in (audit task) |
|---|---|---|
| ... | partial | A2 |
| ... | yes | — |
```

Fill each row from the audit's task list (A1-D5).

- [x] **Step 2: Spot-check against the code**

For each row, grep the cited file/function to confirm the claim. Do not mark a row "shipped" you cannot verify.

- [x] **Step 3: Commit**

```bash
git diff --check
git add docs/superpowers/plans/2026-08-09-r2a-parallel-translation.md docs/superpowers/plans/2026-08-09-r2b-frontend-surfaces.md docs/superpowers/plans/2026-08-09-r3a-settings-provider.md
git commit -m "docs(plans): append rev-4 retroactive status tables (no historical RED edits)"
```

---

### Task D4: Final verification sweep (explicit file lists, no `git add -A`, --features + clippy --all-targets)

**Files:**
- No code changes by default. This task runs the full matrix and confirms green.

- [x] **Step 0: Add the commit-guard helper (one-time)**

Before any D4 commit, verify the staged file list excludes `.mimosa/`, `dist/`, and `test-results/` (guard):

```bash
git diff --cached --name-only | grep -E '(^|/)(\.mimosa|dist|test-results)(/|$)' && { echo "ERROR: staged build/test artifact — unstage it"; git restore --staged .mimosa dist test-results 2>/dev/null; exit 1; } || true
```

Do NOT use `git add -A` or `git add .` anywhere in D4.

- [x] **Step 1: Run the full Rust suite**

Run:
```bash
cargo build --manifest-path src-tauri/Cargo.toml --features xproc-test-helper
cargo test --manifest-path src-tauri/Cargo.toml --features xproc-test-helper
cargo clippy --manifest-path src-tauri/Cargo.toml --features xproc-test-helper --all-targets -- -D warnings
```
Expected: clean build, all tests pass, no clippy warnings.

- [x] **Step 2: Run the full frontend suite**

Run:
```bash
pnpm typecheck
pnpm test:all
```
Expected: typecheck clean; all tests pass across root + ui + ui-lab.

- [x] **Step 3: Run the static acceptance scans**

Run:
```bash
pnpm vitest run test/no-hardcoded-hex.test.ts test/no-space-alias.test.ts
```
Expected: both pass.

- [x] **Step 4: Re-check the new IPC commands are fully permission-authorized**

Run:
```bash
grep -E "translate_session|translate_selection_ipc|provider_get_active_selection|open_settings_window" src-tauri/build.rs
grep -E "allow-translate-session|allow-translate-selection-ipc|allow-provider-get-active-selection|allow-open-settings-window|clipboard-manager:allow-write-text" src-tauri/capabilities/*.json
cargo test --manifest-path src-tauri/Cargo.toml --features xproc-test-helper --test capabilities
```
Expected: every command appears in `build.rs` AND in the capability that the calling window uses AND the capabilities integration test passes (P1-6).

- [x] **Step 5: Manual contract smoke (documented)**

Document the expected outcomes for the reviewer:
- Alt+Space on a selection → popup shows at 200×40 (loading) then resizes to 400×300 (single) or 600×400 (multi) with a friendly engine name (no `provider/<uuid>`). On Retina, the physical size is 2x; no clamping overflow.
- Tray → Translate Selection → popup shows the SELECTION translation (not the clipboard).
- Tray → Translate Clipboard → popup shows the CLIPBOARD translation.
- Tray → Switch Provider → submenu lists enabled providers; clicking one sets it primary (calls `set_active_primary_core`, P1-5).
- Tray → Settings → settings window opens on Provider Center.
- Settings cold-loads stored primary/parallel/fallback; on read failure, error + Retry, no role mutation.
- Input window: type, wait 300ms, reload → draft restored. Input cards show friendly engine names.
- Popup Copy → "Copied" 1.2s (Tauri clipboard; copies the translation). Retry → re-translates the saved SOURCE (not the translation, not the clipboard). Retry is available in the error state too (P1-3); hidden when no source.
- Local primary failure → no remote fallback. Config/401 → no fallback. Two remote 500s → exactly one fallback call + one card (P1-4: eligibility actually hits).

- [x] **Step 6: Commit any doc-only fixups**

```bash
git diff --check
git add <explicit files only>
git diff --cached --name-only | grep -E '(^|/)(\.mimosa|dist|test-results)(/|$)' && { echo "ERROR"; exit 1; } || true
git commit -m "chore: final verification sweep green"
```

If the sweep is fully green with no fixups, no commit is needed.

---

### Task D5: Playwright visual baselines (P1-10) — `toHaveScreenshot` at 600/699/700/800 × light/dark across REAL surfaces

> **P1-10:** capture visual baselines for EVERY real surface, not just the default Settings page. Each Surface gets a stable isolated fixture/route with fixed IPC mock data (NOT real Tauri calls). The tray is native UI — documented as "manual screenshot acceptance required," not browser-screenshotted.

**Files:**
- **(new)** `apps/ui-lab/e2e/surfaces.visual.spec.ts` — the Playwright `toHaveScreenshot` suite (4 widths × 2 themes × surfaces). Lives alongside the existing `apps/ui-lab/e2e/component-gallery.visual.spec.ts` so it reuses `apps/ui-lab/playwright.config.ts` + its committed baselines directory.
- **(new)** `apps/ui-lab/src/pages/InputPanel.tsx` — **rev-7-3 + rev-8-2 + rev-7-7:** the InputPanel fixture IMPORTS the pure presentational `InputPanelView` extracted from the production `src/InputPanel.tsx` (B1/B2 produce the View) via the `@app` alias (`@app/InputPanel`) and feeds it canned state + no-op handlers. ResultEntry fixtures use `engine` (REQUIRED) + `errorText` (NOT `error`); "idle" = `{ kind: "loading" }` + `idle:true`. The fixture does NOT re-draw an approximate InputPanel.
- **(new)** `apps/ui-lab/src/pages/KeystoreRecovery.tsx` — **rev-7-4 + rev-8-2:** the Keystore fixture IMPORTS the COMPLETE presentational `KeystoreRecoveryView` extracted from the production `src/features/settings/KeystoreRecovery.tsx` via the `@app` alias (`@app/features/settings/KeystoreRecovery`) and feeds it canned state + no-op handlers. The production View reproduces the FULL surface (destructive Banner + Confirm + Toast + busy + resetTriggerRef) — NOTHING is simplified. `BannerProps` has NO `icon` and NO `children` (verified at `packages/ui/src/components/Banner.tsx:7`); the Archive + Reset buttons go in the `action` slot as a `<span>` wrapper.
- Modify: `apps/ui-lab/src/App.tsx` — wire the InputPanel + KeystoreRecovery pages into the `?nav=` router + add an `input` state map (mirrors the existing `selectionStateMap` pattern). Handle the `fixture=` query param for the Provider Center empty/populated + Keystore healthy/corrupt variants. Seed `provState` with a REAL `ProviderState` value (`"empty"` or `"key-saved"`, NOT the non-existent `"configured"`).
- Modify: `apps/ui-lab/package.json` — ensure `@playwright/test` is a devDep (it already is, per the existing `component-gallery.visual.spec.ts`); the script is `test:visual` (verified at `apps/ui-lab/package.json`).
- Modify: `src/InputPanel.tsx` (B1/B2) — **rev-7-3:** extract the pure presentational body into `export function InputPanelView(props: InputPanelViewProps)` and have the default export be a thin controller that owns state + handlers and renders `<InputPanelView ... />`.
- Modify: `src/features/settings/KeystoreRecovery.tsx` — **rev-7-4:** extract `export function KeystoreRecoveryView(props: KeystoreRecoveryViewProps)` (pure presentational, FULL surface) and have the default export own the IPC/handlers + render `<KeystoreRecoveryView ... />`.
- **NOT created:** `src/ui-lab/` (does not exist; rev-4 invented it), root `playwright.config.ts` (the lab has its own at `apps/ui-lab/playwright.config.ts`), root `e2e/visual.spec.ts`.

**rev-5-9 (load-bearing):** the `@linguaray/ui-lab` workspace package at `apps/ui-lab` ALREADY has: a Vite dev server on port **1421** (`apps/ui-lab/vite.config.ts`), a Playwright config (`apps/ui-lab/playwright.config.ts`), a working visual spec (`apps/ui-lab/e2e/component-gallery.visual.spec.ts` with committed baselines under `apps/ui-lab/e2e/component-gallery.visual.spec.ts-snapshots/`), a `?nav=`/`?theme=`/`?state=` routing convention in `apps/ui-lab/src/App.tsx`, and fixture pages (`ComponentGallery.tsx`, `ProviderCenter.tsx`, `SelectionPopup.tsx`) under `apps/ui-lab/src/pages/`. The `SelectionPopup` fixture already covers loading/success-single/success-multi/partial/error-network/keystore-corrupt via `apps/ui-lab/src/pages/selectionStateMap.ts`. D5 EXTENDS this existing lab — it does NOT create a parallel `src/ui-lab/`.

**Interfaces:**
- Produces: a Playwright suite (`apps/ui-lab/e2e/surfaces.visual.spec.ts`) that, for each of `{600, 699, 700, 800}` width × `{light, dark}` theme, renders each Surface and calls `toHaveScreenshot` with `maxDiffPixelRatio: 0.01`. Surfaces:
  - **Provider Center** — empty + populated (`?nav=provider-center&fixture=empty|populated`).
  - **Keystore Recovery** — healthy + corrupt (`?nav=keystore&fixture=healthy|corrupt`).
  - **Selection Popup** — loading/single/multi/partial/error (`?nav=selection-popup&state=<state>` — states already in `selectionStateMap.ts`).
  - **InputPanel** — idle/multi/partial/error (`?nav=input-window&state=<state>` — new fixture + state map).
  - **Tray** — NOT browser-screenshotted; documented as "manual screenshot acceptance required."
- Baselines are committed under `apps/ui-lab/e2e/surfaces.visual.spec.ts-snapshots/`.

- [x] **Step 0: Verify + extend the ui-lab fixtures (rev-5-9)**

Verify the existing lab runs and the popup fixtures render. Run (from the repo root; rev-6-8: script is `test:visual`):

```bash
pnpm --filter @linguaray/ui-lab exec playwright --version
pnpm --filter @linguaray/ui-lab test:visual component-gallery
```

Expected: Playwright is installed; the existing component-gallery visual spec passes against its committed baselines (this proves the `?nav=`/`?theme=`/`?state=` routing + webServer config work).

Create `apps/ui-lab/src/pages/InputPanel.tsx` — **rev-7-3 + rev-8-2 + rev-7-7:** the fixture IMPORTS the pure presentational `InputPanelView` extracted from the production `src/InputPanel.tsx` (B1/B2 extract the View) via the `@app` alias (rev-8-2: configured at `apps/ui-lab/{vite,vitest}.config.ts` + `tsconfig.json` → `../../src`), and feeds it canned state + no-op handlers. The fixture does NOT re-draw an approximate InputPanel. ResultEntry fields are `engine` (REQUIRED) + `errorText` (NOT `error`); there is NO `idle` kind on `TranslationState`, so the "idle" fixture uses `{ kind: "loading" }` with `idle: true` (no in-flight request).

```tsx
import { type Component } from "solid-js";
// rev-8-2: the `@app` alias resolves to <repo>/src (see apps/ui-lab configs).
import { InputPanelView, type InputPanelViewProps } from "@app/InputPanel";
import "./InputPanel.css";

export type InputState = "idle" | "multi" | "partial" | "error";

export type InputPanelProps = {
  state: InputState;
};

// rev-7-7: FIXED canned data — NO invoke calls. The lab is a pure renderer.
// The shape matches InputPanelViewProps (the production View's prop type).
// ResultEntry = { uuid, engine (REQUIRED), text?, errorText?, ok }.
const SAMPLE_TEXT = "The quick brown fox jumps over the lazy dog.";

const STATE_PROPS: Record<InputState, InputPanelViewProps> = {
  // rev-7-7: there is NO `idle` kind on TranslationState. "idle" (no in-flight
  // request) is represented as { kind: "loading" } + idle: true.
  idle: {
    text: SAMPLE_TEXT,
    state: { kind: "loading" },
    idle: true,
    hasResult: false,
    onText: () => {},
    onTranslate: () => {},
    onClear: () => {},
  },
  multi: {
    text: SAMPLE_TEXT,
    state: {
      kind: "multi-success",
      results: [
        { uuid: "openai", engine: "OpenAI", ok: true, text: "你好" },
        { uuid: "anthropic", engine: "Claude", ok: true, text: "您好" },
      ],
    },
    idle: true,
    hasResult: true,
    engineLabel: (raw: string) => raw,
    onText: () => {},
    onTranslate: () => {},
    onClear: () => {},
  },
  partial: {
    text: SAMPLE_TEXT,
    state: {
      kind: "partial",
      results: [
        { uuid: "openai", engine: "OpenAI", ok: true, text: "你好" },
        { uuid: "anthropic", engine: "Claude", ok: false, errorText: "config-401" },
      ],
    },
    idle: true,
    hasResult: true,
    engineLabel: (raw: string) => raw,
    onText: () => {},
    onTranslate: () => {},
    onClear: () => {},
  },
  error: {
    text: SAMPLE_TEXT,
    state: { kind: "error", sub: "network", message: "Network error — all engines failed" },
    idle: true,
    hasResult: true,
    onText: () => {},
    onTranslate: () => {},
    onClear: () => {},
  },
};

const InputPanel: Component<InputPanelProps> = (props) => {
  return (
    <div class="input-shell" data-testid="lab-root">
      <InputPanelView {...STATE_PROPS[props.state]} />
    </div>
  );
};

export default InputPanel;
```

**rev-7-3: extract `InputPanelView` from the production component.** In B1/B2, refactor `src/InputPanel.tsx` so the presentational body (the textarea + ResultCard grid + InlineError) is a named export `InputPanelView` taking `(text, state, idle, hasResult?, engineLabel?, onText, onTranslate, onClear)` as props, and the default export owns the signals + IPC + autosave (B2) and renders `<InputPanelView ... />`. The View is a pure function of its props — no `createSignal`, no `invoke`, no `localStorage`. This is what the ui-lab fixture imports (via `../../../src/InputPanel`).

Create `apps/ui-lab/src/pages/InputPanel.css` (mirror the existing `SelectionPopup.css` import shape — the lab uses local CSS so the fixture is self-contained; consume `@linguaray/ui` tokens for color/spacing).

Create `apps/ui-lab/src/pages/KeystoreRecovery.tsx` — **rev-7-4 + rev-8-2:** the fixture IMPORTS the COMPLETE presentational `KeystoreRecoveryView` extracted from the production component (see below) via the `@app` alias (`@app/features/settings/KeystoreRecovery`), and feeds it canned state + no-op handlers. No IPC, no effects. `BannerProps` has NO `icon` and NO `children` (verified at `packages/ui/src/components/Banner.tsx:7`) — the Archive + Reset buttons go in the `action` slot as a `<span>` wrapper, exactly as the production component does.

```tsx
import { type Component } from "solid-js";
// rev-8-2: the `@app` alias (configured in apps/ui-lab/vite.config.ts:16,
// vitest.config.ts:19, tsconfig.json:21 -> ../../src) resolves production src.
import {
  KeystoreRecoveryView,
  type KeystoreRecoveryViewProps,
} from "@app/features/settings/KeystoreRecovery";
import "./KeystoreRecovery.css";

export type KeystoreState = "healthy" | "corrupt";

export type KeystoreRecoveryProps = {
  state: KeystoreState;
};

const KeystoreRecovery: Component<KeystoreRecoveryProps> = (props) => {
  // rev-7-4: canned props for the COMPLETE production View (Banner + Confirm +
  // Toast + busy). No IPC — the lab is a pure renderer.
  const viewProps: KeystoreRecoveryViewProps = {
    state: props.state,
    reason: props.state === "corrupt" ? "Keystore unlock failed (lab fixture)" : "",
    resetOpen: false,
    busy: null,
    toasts: [],
    onArchive: () => {},
    onReset: () => {},
    onOpenReset: () => {},
    onCloseReset: () => {},
    onDismissToast: () => {},
  };
  return (
    <div class="keystore-shell" data-testid="lab-root">
      <KeystoreRecoveryView {...viewProps} />
    </div>
  );
};

export default KeystoreRecovery;
```

**rev-7-4: extract the COMPLETE `KeystoreRecoveryView` from the production component.** Edit `src/features/settings/KeystoreRecovery.tsx` so the presentational body is a named export `KeystoreRecoveryView` and the default export is a thin controller that owns the IPC + state. The View reproduces the FULL production surface — Banner (corrupt: destructive, archived: info) + Confirm (destructive, Cancel-focused, `triggerRef`) + Toast stack + `busy` states — NOTHING is simplified. All three `KsState` values (`healthy` | `corrupt` | `archived`) are supported; `healthy` renders no Banner (settings normal). The View is pure: it receives state + handlers as props, calls NO `invoke`, has NO effects.

```tsx
// src/features/settings/KeystoreRecovery.tsx
// rev-8-7 (load-bearing): the View AND the controller live in the SAME file, so
// the import line must cover BOTH. The View needs Show + For + Component; the
// controller needs createSignal + onMount + onCleanup + invoke. JSX is used by
// the View's return type. Merged into ONE import block (verified against the
// current src/features/settings/KeystoreRecovery.tsx:1-12).
import {
  createSignal,
  onMount,
  onCleanup,
  Show,
  For,
  type Component,
} from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { Banner, Confirm, Button, Toast } from "@linguaray/ui";
import { SETTINGS_COPY } from "./copy";
import { detectLocale } from "../../i18n";
import "./KeystoreRecovery.css";

export type KsState = "healthy" | "corrupt" | "archived";

export type KsToastEntry = {
  id: number;
  variant: "info" | "success" | "warning" | "destructive";
  message: string;
};

/** rev-7-4: pure presentational View. Shared by the production mount + the ui-lab
 * visual fixture. Renders the FULL surface: Banner (corrupt/archived) + Confirm
 * (destructive, Cancel-focused) + Toast stack + busy. No IPC, no effects. */
export type KeystoreRecoveryViewProps = {
  state: KsState;
  reason: string;
  resetOpen: boolean;
  busy: "archive" | "reset" | null;
  toasts: KsToastEntry[];
  onArchive: () => void;
  onReset: () => void;        // Confirm "Confirm" -> reset_keystore
  onOpenReset: () => void;    // Reset trigger button -> open the Confirm
  onCloseReset: () => void;   // Confirm "Cancel" + backdrop -> close
  onDismissToast: (id: number) => void;
  /** Optional override ref for the Reset trigger (focus restore on Confirm close).
   *  The production mount supplies one; the lab fixture omits it. */
  resetTriggerRef?: { current?: HTMLElement };
};

export const KeystoreRecoveryView: Component<KeystoreRecoveryViewProps> = (props) => {
  const locale = detectLocale();
  const t = SETTINGS_COPY[locale].keystore;
  const description = () => t.description.replace("{reason}", props.reason);

  return (
    <section class="keystore-recovery" aria-label={t.title}>
      <Show when={props.state === "corrupt"}>
        <Banner
          variant="destructive"
          title={t.title}
          description={description()}
          action={
            <span class="keystore-recovery__banner-actions">
              <Button
                variant="primary"
                size="sm"
                loading={props.busy === "archive"}
                onClick={props.onArchive}
              >
                {t.archive}
              </Button>
              <Button
                variant="destructive"
                size="sm"
                ref={(el: HTMLButtonElement) => {
                  if (props.resetTriggerRef) props.resetTriggerRef.current = el;
                }}
                onClick={props.onOpenReset}
              >
                {t.reset}
              </Button>
            </span>
          }
        />
      </Show>

      <Show when={props.state === "archived"}>
        <Banner variant="info" title={t.archivedTitle} description={t.archivedPrompt} />
      </Show>

      {/* Healthy: no banner; settings normal. */}

      <Confirm
        open={props.resetOpen}
        // rev-8-7 (load-bearing): Confirm's onOpenChange passes (open: boolean).
        // Route open=true -> onOpenReset (opens the Confirm), open=false ->
        // onCloseReset (Cancel/backdrop). The previous `() => props.onCloseReset()`
        // form ignored the boolean and would close on every change.
        onOpenChange={(open) => (open ? props.onOpenReset() : props.onCloseReset())}
        variant="destructive"
        title={t.resetConfirmTitle}
        message={t.resetConfirmMessage}
        confirmLabel={t.resetConfirmConfirmLabel}
        cancelLabel={t.resetConfirmCancelLabel}
        onConfirm={props.onReset}
        onCancel={() => props.onCloseReset()}
        triggerRef={props.resetTriggerRef ?? {}}
      />

      <Show when={props.toasts.length > 0}>
        <div class="keystore-recovery__toasts" aria-live="polite">
          <For each={props.toasts}>
            {(entry) => (
              <Toast
                variant={entry.variant}
                message={entry.message}
                onDismiss={() => props.onDismissToast(entry.id)}
              />
            )}
          </For>
        </div>
      </Show>
    </section>
  );
};
```

> **rev-7-4 Reset-trigger wiring:** in the production mount, the Reset button's `onClick` calls `setResetOpen(true)` (the controller owns `resetOpen`). Because the View must stay presentational, the production controller passes `onOpenReset` too; to keep the prop surface above minimal, the production controller overrides the Reset button's `onClick` by passing `resetTriggerRef` AND binding the open-state through a dedicated prop. The clean production wiring (controller owns `resetOpen` + the open handler) is:

```tsx
// src/features/settings/KeystoreRecovery.tsx — PRODUCTION MOUNT (controller).
// Owns the IPC + state + the resetOpen signal; renders <KeystoreRecoveryView .../>.
const KeystoreRecovery: Component = () => {
  const locale = detectLocale();
  const t = SETTINGS_COPY[locale].keystore;

  const [state, setState] = createSignal<KsState>("healthy");
  const [reason, setReason] = createSignal("");
  const [resetOpen, setResetOpen] = createSignal(false);
  const [busy, setBusy] = createSignal<"archive" | "reset" | null>(null);
  const [toasts, setToasts] = createSignal<KsToastEntry[]>([]);
  const resetTriggerRef: { current?: HTMLElement } = {};
  let toastSeq = 0;
  const pushToast = (variant: KsToastEntry["variant"], message: string) => {
    const id = ++toastSeq;
    setToasts((prev) => [...prev, { id, variant, message }]);
  };
  const dismissToast = (id: number) =>
    setToasts((prev) => prev.filter((e) => e.id !== id));

  onMount(() => {
    invoke<string>("keystore_health")
      .then((h) => {
        if (h === "" || h == null) setState("healthy");
        else {
          setState("corrupt");
          setReason(h);
        }
      })
      .catch((e: unknown) => {
        setState("corrupt");
        setReason(String(e));
      });
  });

  const onArchive = async () => {
    setBusy("archive");
    try {
      await invoke<string>("archive_keystore");
      setState("archived");
    } catch (e: unknown) {
      pushToast("destructive", `${t.archiveFailed}: ${String(e)}`);
    } finally {
      setBusy(null);
    }
  };
  const onReset = async () => {
    setBusy("reset");
    try {
      await invoke<string | null>("reset_keystore");
      setState("archived");
      setResetOpen(false);
    } catch (e: unknown) {
      pushToast("destructive", `${t.resetFailed}: ${String(e)}`);
    } finally {
      setBusy(null);
    }
  };

  onCleanup(() => setToasts([]));

  return (
    <KeystoreRecoveryView
      state={state()}
      reason={reason()}
      resetOpen={resetOpen()}
      busy={busy()}
      toasts={toasts()}
      onArchive={onArchive}
      onReset={onReset}
      onOpenReset={() => setResetOpen(true)}
      onCloseReset={() => setResetOpen(false)}
      onDismissToast={dismissToast}
      resetTriggerRef={resetTriggerRef}
    />
  );
};

export default KeystoreRecovery;
```

The View's Reset button calls `props.onOpenReset` (opens the Confirm); the Confirm's `onOpenChange` routes `open=true`→`onOpenReset` / `open=false`→`onCloseReset` (rev-8-7); the Confirm/Cancel actions call `props.onReset`/`props.onCloseReset`. The production controller passes `onOpenReset={() => setResetOpen(true)}` + `onCloseReset={() => setResetOpen(false)}`; the lab fixture passes `onOpenReset={() => {}}` + `onCloseReset={() => {}}`. The full surface — destructive Banner (corrupt) + info Banner (archived) + destructive Cancel-focused Confirm + Toast stack + busy + resetTriggerRef focus restore — is reproduced verbatim; NOTHING is simplified.

- [x] **Step 1: Wire the new fixtures into the lab router**

Edit `apps/ui-lab/src/App.tsx`. The existing router reads `?nav=` + `?state=` + `?theme=`. Add: import `InputPanel` + `KeystoreRecovery`, extend the `nav` handling so `?nav=input-window&state=<idle|multi|partial|error>` renders `<InputPanel state={...} />` and `?nav=keystore&fixture=<healthy|corrupt>` renders `<KeystoreRecovery state={...} />`. For the Provider Center empty/populated variant, pass the existing `provState` signal seeded from a new `?fixture=` param read (the existing App already has a `provState` signal defaulting to `"empty"`; wire `?fixture=populated` to seed it with the populated mock).

```tsx
// apps/ui-lab/src/App.tsx — add to the existing imports + param reads:
import InputPanel from "./pages/InputPanel";
import KeystoreRecovery from "./pages/KeystoreRecovery";

// Inside the App component, alongside the existing params reads:
const initialFixture =
  params.get("fixture") === "populated" || params.get("fixture") === "empty"
    ? (params.get("fixture") as "populated" | "empty")
    : "empty";
const initialKeystoreFixture =
  params.get("fixture") === "healthy" || params.get("fixture") === "corrupt"
    ? (params.get("fixture") as "healthy" | "corrupt")
    : "healthy";
const initialInputState =
  params.get("state") === "idle" ||
  params.get("state") === "multi" ||
  params.get("state") === "partial" ||
  params.get("state") === "error"
    ? (params.get("state") as "idle" | "multi" | "partial" | "error")
    : "idle";

// Seed the existing provState signal + add keystore/input signals:
// rev-6-8: ProviderState has NO "configured" value (verified at
// apps/ui-lab/src/i18n/index.ts:40). The populated variant seeds "key-saved"
// (a real ProviderState); the empty variant seeds "empty".
const [provState, setProvState] = createSignal<ProviderState>(initialFixture === "populated" ? "key-saved" : "empty");
const [keystoreState, setKeystoreState] = createSignal<"healthy" | "corrupt">(initialKeystoreFixture);
const [inputState, setInputState] = createSignal<"idle" | "multi" | "partial" | "error">(initialInputState);
```

Then add two `<Match>` branches to the EXISTING `<Switch>` block in `apps/ui-lab/src/App.tsx`. The `<Switch>` (verified at `apps/ui-lab/src/App.tsx:314`) currently has `<Match when={nav() === "selection-popup"}>` (line 315), `<Match when={nav() === "provider-center"}>` (line 341, closing `</Match>` at line 373), `<Match when={nav() === "component-gallery"}>` (line 375), and the fallback `<Match when={!IMPLEMENTED.includes(nav())}>` (line 379). Insert the two new branches BETWEEN the `provider-center` close (line 373) and the `component-gallery` open (line 375):

```diff
            </Match>

+            <Match when={nav() === "input-window"}>
+              <InputPanel state={inputState()} />
+            </Match>
+            <Match when={nav() === "keystore"}>
+              <KeystoreRecovery state={keystoreState()} />
+            </Match>
+
            <Match when={nav() === "component-gallery"}>
```

(Both `input-window` and `keystore` must also be added to the `NavKey` union + `IMPLEMENTED` array at `apps/ui-lab/src/App.tsx:96` so the fallback `<Match when={!IMPLEMENTED.includes(nav())}>` does not swallow them — Step 1 covers that alongside the fixture imports.)

(The existing `nav() === "provider-center"` branch already renders `<ProviderCenter state={provState()} ... />`; seeding `provState` from `?fixture=` makes the empty/populated screenshots deterministic.)

- [x] **Step 2: Write the visual test (P1-10: every real surface)**

Create `apps/ui-lab/e2e/surfaces.visual.spec.ts` (lives next to the existing `component-gallery.visual.spec.ts` so it reuses `apps/ui-lab/playwright.config.ts`, which already points `webServer` at the lab dev server on port **1421**):

```ts
import { test, expect } from "@playwright/test";

const WIDTHS = [600, 699, 700, 800] as const;
const THEMES = ["light", "dark"] as const;
const BASE = "http://localhost:1421";

const SETTINGS_SURFACES = [
  { nav: "provider-center", fixture: "populated", label: "provider-center-populated" },
  { nav: "provider-center", fixture: "empty", label: "provider-center-empty" },
  { nav: "keystore", fixture: "healthy", label: "keystore-recovery-healthy" },
  { nav: "keystore", fixture: "corrupt", label: "keystore-recovery-corrupt" },
] as const;

const POPUP_SURFACES = [
  { state: "loading", label: "popup-loading" },
  { state: "success-single", label: "popup-single" },
  { state: "success-multi", label: "popup-multi" },
  { state: "partial", label: "popup-partial" },
  { state: "error-network", label: "popup-error" },
] as const;

const INPUT_SURFACES = [
  { state: "idle", label: "input-idle" },
  { state: "multi", label: "input-multi" },
  { state: "partial", label: "input-partial" },
  { state: "error", label: "input-error" },
] as const;

for (const width of WIDTHS) {
  for (const theme of THEMES) {
    for (const s of SETTINGS_SURFACES) {
      test(`visual: ${s.label} @ ${width}x ${theme}`, async ({ page }) => {
        await page.setViewportSize({ width, height: 800 });
        await page.goto(`${BASE}/?nav=${s.nav}&fixture=${s.fixture}&theme=${theme}`);
        await page.waitForSelector("[data-testid='lab-root'], .settings-shell, .provider-center", { timeout: 10_000 });
        await expect(page).toHaveScreenshot(`${s.label}-${width}-${theme}.png`, {
          maxDiffPixelRatio: 0.01,
          fullPage: true,
        });
      });
    }
    for (const s of POPUP_SURFACES) {
      test(`visual: ${s.label} @ ${width}x ${theme}`, async ({ page }) => {
        await page.setViewportSize({ width, height: 800 });
        await page.goto(`${BASE}/?nav=selection-popup&state=${s.state}&theme=${theme}`);
        await page.waitForSelector("[data-testid='lab-root'], .popup-shell", { timeout: 10_000 });
        await expect(page).toHaveScreenshot(`${s.label}-${width}-${theme}.png`, {
          maxDiffPixelRatio: 0.01,
          fullPage: true,
        });
      });
    }
    for (const s of INPUT_SURFACES) {
      test(`visual: ${s.label} @ ${width}x ${theme}`, async ({ page }) => {
        await page.setViewportSize({ width, height: 800 });
        await page.goto(`${BASE}/?nav=input-window&state=${s.state}&theme=${theme}`);
        await page.waitForSelector("[data-testid='lab-root'], .input-shell", { timeout: 10_000 });
        await expect(page).toHaveScreenshot(`${s.label}-${width}-${theme}.png`, {
          maxDiffPixelRatio: 0.01,
          fullPage: true,
        });
      });
    }
  }
}

test("no horizontal overflow at 699px (provider center)", async ({ page }) => {
  await page.setViewportSize({ width: 699, height: 800 });
  await page.goto(`${BASE}/?nav=provider-center&fixture=populated&theme=light`);
  await page.waitForSelector("[data-testid='lab-root'], .provider-center", { timeout: 10_000 });
  const scrollWidth = await page.evaluate(() => document.documentElement.scrollWidth);
  const clientWidth = await page.evaluate(() => document.documentElement.clientWidth);
  expect(scrollWidth, "horizontal overflow at 699px").toBeLessThanOrEqual(clientWidth);
});
```

> The tray is native OS UI and cannot be browser-screenshotted. D5 documents it as "manual screenshot acceptance required" in the commit message body (NOT a new `.md` file — per the project's no-new-docs convention). The manual capture steps: open the running app, click the tray icon, screenshot the menu (Translate Selection / Translate Clipboard / Switch Provider submenu / Settings / Quit) in light + dark, and attach the two PNGs to the PR.

- [x] **Step 3: Generate baselines**

Run (rev-6-8: the ui-lab script is `test:visual`, NOT `test:e2e`):
```bash
pnpm --filter @linguaray/ui-lab exec playwright test surfaces.visual --update-snapshots
```
Expected: baselines generated under `apps/ui-lab/e2e/surfaces.visual.spec.ts-snapshots/`.

- [x] **Step 4: Run the visual suite**

Run (rev-6-8):
```bash
pnpm --filter @linguaray/ui-lab test:visual surfaces.visual
```
Expected: all surface/width/theme combinations pass + the no-overflow test passes. If a baseline needs updating (intentional UI change), re-run with `--update-snapshots` and document why in the commit.

- [x] **Step 5: Commit baselines + fixtures**

```bash
git diff --check
git add src/InputPanel.tsx src/features/settings/KeystoreRecovery.tsx apps/ui-lab/src/pages/InputPanel.tsx apps/ui-lab/src/pages/InputPanel.css apps/ui-lab/src/pages/KeystoreRecovery.tsx apps/ui-lab/src/pages/KeystoreRecovery.css apps/ui-lab/src/App.tsx apps/ui-lab/e2e/surfaces.visual.spec.ts apps/ui-lab/e2e/surfaces.visual.spec.ts-snapshots/
git commit -m "test(visual): Playwright toHaveScreenshot baselines at 600/699/700/800 x light/dark across real surfaces in apps/ui-lab (Provider Center empty/populated, Keystore healthy/corrupt, Selection Popup 5 states, InputPanel 4 states); fixtures reuse production InputPanelView/KeystoreRecoveryView via @app alias (rev-8-2); ResultEntry uses engine/errorText; tray = manual capture (P1-10, rev-7-3, rev-7-4, rev-8-2, rev-7-7)"
```

---

### Stage D Verification

The Stage D Verification IS the final sweep (D4) + the visual baselines (D5). The plan is complete when:
- D4 Steps 1-4 are green (`--features xproc-test-helper`, `clippy --all-targets -- -D warnings`, `git diff --check`, capabilities integration test passes).
- D4 Step 5's documented outcomes match the running app.
- D5's visual baselines (every real surface × 4 widths × 2 themes) + the no-overflow test pass.
- D5's keyboard spec (from C5) passes.
- D4 Step 0's guard passed on every D4/D5 commit.

---

## Self-Review

Run after the plan is complete, before handing off.

**1. Spec coverage — every P1 (1-10) + every P2 maps to a task:**

| Requirement item | Where addressed (rev-7) |
|---|---|
| **P1-1** capture_and_translate complete + generation token + multi-monitor scale | A2 Step 5 (full helper, `gen: u64` checked at every await boundary, full HWND/capture/client/keystore/db), A2 Step 6 (`on_hotkey` passes `gen`), A2 Step 5 `build_popup_anchor` (**rev-7-1: resolves the cursor's Monitor via `monitor_from_point` and uses THAT monitor's `scale_factor()` for work_area + cursor; popup window's `scale_factor()` is only the `None` fallback; `sf > 0.0 && sf.is_finite()` guard**) |
| **P1-2** Geometry unified units (PopupAnchor, work_area, set_max_size before set_size, Error mode 400×300, NoSelection via show_at_sized, A3 before A2, A2 not dependent on B4) | A3 (`PopupAnchor`, `compute_popup_geometry_logical(mode, &anchor)`, `set_popup_mode` recomputes position, `loading_with_source` ships in A3), A2 Step 5 (NoSelection + Error via `show_at_sized`/`set_popup_mode`; **rev-7-1: `build_popup_anchor` uses the REAL `Monitor::work_area() -> &PhysicalRect<i32, u32>` + the target Monitor's `scale_factor()`**; `system_bar_deduction()` is DELETED) |
| **P1-3** Retry always has source (loading/error carry source_text, popup saves lastSource on every state, new session clears, Retry hidden when no source, clipboard translate saves raw text) | A3 (`loading_with_source`), A2 (`error_with_source`, `result_with_source`, `multi_result_with_source` **rev-6-2: single body, `Some(source_text.to_owned())`, duplicate deleted**), B4 Step 6 (`lastSource` saved on loading/error/result/multi, cleared on new session), B4 Step 6b (rev-5-7: `translate_clipboard` carries source_text), B4 Step 7 (`buildActions` Retry gated on `ctrl.hasSource()`), B4 Step 1 (test: error-state Retry + hidden-when-no-source + clipboard-origin Retry) |
| **P1-4** Fallback eligibility can hit (translate_primary_only, eligible_for_session_fallback, single call, local-sacred at session level, fixed mock URLs) | B6 Step 3 (`translate_primary_only` preserves Error; `eligible_for_session_fallback(outcomes, locality, local_primary_failed)` **rev-6-4: locality slice threaded through; local providers' FallbackEligible never counts**; `eng.translate` once; rev-5-3 indexed-Vec reassembly + parallel `locality` Vec), B6 Step 1 (tests use lvh.me for remote, 127.0.0.1:11434 only for local-sacred; **rev-6-4 matrix: remote-primary-Config+local-parallel, local-parallel-FallbackEligible, primary-pre-failed-locality**) |
| **P1-5** Tray Normal/Active/Error states implemented; Update badge deferred (rev-18) (set_active_primary_core, spawn_blocking, full SubmenuBuilder, status from db, refresh hook, navigate event, + rev-15/rev-16/rev-17 sync `TrayStateController` reducer driving Normal/Active/Error icon+tooltip with `parking_lot::Mutex` + PulseWorker(channel-quit + PulseEvent notify), Error red-dot OVERLAY on base icon via build-time-composited PNG, Active = real icon frame-swap pulse via `PulseWorker` worker) | A4 Step 9 (`set_active_primary_core` + `db_set_active_primary` (rev-5-4 real helpers) + `build_switch_provider_submenu` + `read_primary_status` + `build_tray_menu` + `refresh_tray` via `tray_by_id` (`tray.set_menu(Some(menu))`; `read_enabled_providers` maps `DbErr`; tray settings navigate = `"provider-center"`) + full `handle_tray_menu_event` with failure-preserves-old-primary), A4 Step 9b (**rev-8-8: each of the EIGHT provider mutation commands (`provider_create`/`provider_update`/`provider_delete`/`provider_toggle`/`provider_reorder`/`provider_set_active`/`provider_duplicate`/`provider_confirm_and_set_active`) gains `app_handle: tauri::AppHandle`, renames `app`→`app_state`, and calls `refresh_tray_if_available(&app_handle)` on the success path**), A4 Step 7 (App.tsx navigate listener + controlled `activePage` + **rev-8-1 + rev-9-2 minimal SettingsShell diff-instruction edit (controlled component derivation)**), **rev-15/rev-16/rev-17 Task A5 (`src-tauri/src/tray_state.rs` + `TrayStateController` reducer held in `Arc<parking_lot::Mutex<..>>` on `AppState` — synchronous, NOT `tokio::sync::Mutex`): rev-14 P1-1 (retained) all methods SYNC; rev-15 P1-1 `PulseWorker` channel-quit + rev-16 P2-1 / rev-17-2 notify (carries `PulseEvent`) — `mpsc::channel()` + worker loops on `recv_timeout(interval)` (`Ok`/`Disconnected`→emit `PulseEvent::Stopped`+return, `Timeout`→toggle frame + `notify.send(PulseEvent::Tick)`); `PulseWorker::stop()` = `stop_tx.send(())` + `handle.take().join()` (worker returns from `recv_timeout` on the signal so `join` completes — NO infinite-loop + join deadlock, the rev-14 bug); `impl Drop for PulseWorker { fn drop(&mut self) { self.stop(); } }`; controller holds `pulse_worker: Option<PulseWorker>`, leaving Active = `pulse_worker.take()` (Drop → stop); rev-15 P1-2 `RecordingRenderer` + `RenderedIcon` are `#[cfg(any(test, feature = "xproc-test-helper"))]`-gated (NOT `#[cfg(test)]` — invisible to the integration-test crate; the `lib.rs` re-export splits into a cfg-gated test-only block + an always-on block that ALSO re-exports `PulseEvent`/`PulseWorker`); **rev-16 P1-1 NO function overloading** — `record_translation_error(gen)` (translation, gen-tagged, `error_gen`) + `begin_switch()`/`finish_switch(rev, success)` (switch, NO gen, `switch_revision`/`switch_error_rev`) are DISTINCT method names (rev-15's two `record_error` overloads do not compile — `E0592`); **rev-17-4: `record_switch_error()`/`clear_switch_error()` DELETED** — `finish_switch(rev, false)`/`finish_switch(rev, true)` are the sole switch mutators (they carry the stale-revision guard the low-level methods lacked); **rev-17 P2-3: `clear_translation_error(gen)` DELETED** — it was never called (`finish_translation(gen, true)` merges the clear); **rev-16-2 / rev-17-3 gen guards** — `finish_translation(gen, true)` clears `error_gen` ONLY if `error_gen <= gen`; `record_translation_error(gen)` sets it ONLY if `gen >= latest_translation_gen` (rev-17-3 NEW field) AND `gen >= error_gen` (rev-16-2); a stale OLDER gen cannot clobber a NEWER gen; **rev-16-3 switch revision replaces rev-15's sticky `has_error: bool`** — `switch_revision: u64` (monotonic, `begin_switch()` bumps it) + `switch_error_rev: Option<u64>`; `finish_switch(rev, success)` IGNORES stale `rev != switch_revision` (re-ordered concurrent switches cannot clobber the latest); `recompute_pure` ORs `error_gen.is_some() || switch_error_rev.is_some()` → `Error`; **rev-18-1: the switch handler is split into a SYNC core `pub fn handle_switch_provider_core(app_state, uuid)` (NO AppHandle — testable) + a SYNC wrapper `pub fn handle_switch_provider(app, app_state, uuid)`** (rev-17-1's `async` was based on the wrong premise that `set_active_primary_core` was async — it is SYNC; rev-18-1 reverts to SYNC) — both acquire ONLY `app_state` (the wrapper via `app.state::<Arc<AppState>>()`) (NOT `Session`, NOT `gen.next()` — calling `next()` stales in-flight translations, verified concurrency.rs) and use `begin_switch()` → `set_active_primary_core(app_state.clone(), uuid.to_string())` (SYNC) → `finish_switch(rev, success)`; the tray.switch arm runs the wrapper via `tauri::async_runtime::spawn_blocking` (offload SYNC SQLite I/O — NOT `spawn(async move { ... .await })`); rev-15 P1-4 SINGLE timer model — `PulseWorker` only; `visual_epoch`, `tick_render()`, `stop_timer()` DELETED (rev-14 prose described an in-timer epoch check the code never performed; rev-15 keeps only the code model — the worker holds an independent renderer clone, the channel-quit is the sole barrier); rev-15 housekeeping `finish_translation(gen, success)` merges `end_translation + clear-on-success + recompute` into ONE method; `TranslationGuard::drop` calls it once (true RAII); rev-14 P1-2 (retained) `recompute` only swaps the worker when `new_state != current_state` (Active → Active counter bump does NOT churn the worker); rev-13 P1-3 `error_gen: Option<u64>` generation-tagged translation error (a same-or-newer-gen Retry success clears the prior red dot via `finish_translation(_, true)` — rev-16-2 guard); rev-13 P1-2 Error overlays a build-time-**composited** red-dot-on-base-icon PNG — `image::open("src-tauri/icons/32x32.png")` + top-right ~10px `#DC2626` dot via manual `put_pixel` circle test (NOT a solid-red square); rev-15/rev-14 P1-1 ActiveTranslation drives a **real icon frame-switch pulse** via `PulseWorker` every 800ms (`tray-active-32.png`, a ~60%-brightness dimmed variant); rev-14 P2 localized tooltip via `tray_tooltip_text(state, locale)` (en/zh); rev-14 P2 `detect_system_locale()` uses `sys_locale::get_locale()` (cross-platform, NOT `std::env::var("LANG")`); Normal restores `app.default_window_icon()` + drops the `PulseWorker` (→ stop → send + join); `UpdateAvailable` retained in the enum for priority-ordering tests but NEVER produced by `recompute` — deferred to R5/R6 per user-approved scope decision; `TrayStateController` does NOT derive `Debug` (holds `Arc<dyn TrayRenderer>`); the pixel-diff test `panic!`s if the generated PNG is missing (rev-14 P2 — does NOT silently skip); `tray_state.rs` integration test (33 tests, rev-18 — ALL `#[test]` SYNC; the functional switch test was rewritten from `#[tokio::test]`/async to `#[test]`/SYNC against a real DB) asserts priority ordering + reducer concurrency + RAII guard + generation-aware error + gen-guard + latest_translation_gen (rev-17-3: `stale_gen_error_ignored_after_newer_begin`) + PulseWorker alternating-frames (rev-17-2 `PulseEvent::Tick` notify, NO `thread::sleep`) + PulseWorker channel-quit (`stop_signal_joins_the_worker`, `drop_stops_the_worker` — rev-17 P2-4 assert `PulseEvent::Stopped` then join) + worker-stop barrier (`leaving_active_stops_the_worker_no_stale_frames` — rev-17-2 `PulseEvent::Stopped`) + worker no-churn on second begin + switch-flow switch_error_rev independence (rev-16-3 renamed, rev-17-4 uses `finish_switch`) + switch-revision ordering (rev-16-3: `two_concurrent_switches_second_wins`, `stale_switch_result_ignored`) + switch-does-not-bump-generation functional (rev-18-3: `#[test]`, calls the REAL SYNC core `handle_switch_provider_core(&app_state, &uuid)` — NO AppHandle — against a real temp DB + inserted provider) + structural grep (rev-16 P2-2 / rev-18 P2-4: `switch_arm_source_has_no_gen_next_call` ALSO asserts no `.await`/`spawn(async move`/`pub async fn handle_switch_provider`) + localization + pixel-diff; A5 Steps 8-9 wire the controller into `capture_and_translate` + `translate_clipboard` via `TranslationGuard::new(&app_state.tray, gen)` (sync) on entry, `guard.mark_success()` on success (→ `finish_translation(gen, true)`), `app_state.tray.lock().record_translation_error(gen)` (rev-16-1 renamed) on failure (→ `finish_translation(gen, false)`) — all sync, no `.await`; Step 10 wires switch-provider via the extracted **SYNC** `handle_switch_provider(app, app_state, uuid)` (rev-18-1: `pub fn` SYNC, offloaded via `spawn_blocking`; rev-17-1's `async` superseded) which uses `begin_switch()`/`finish_switch(rev, success)` (rev-16-3, rev-17-4) — NO `session.gen.next()`; `pub mod tray_state` (rev-12 P1-4) so the test path resolves; **rev-15: `src-tauri/Cargo.lock` added to the A5 commit** (git-tracked, updated by `sys-locale = "0.3"`); **rev-16 P2-4: `cargo build` (NO feature) succeeds** — the cfg-gated re-export does NOT leak `RecordingRenderer` into production; the frozen pages/04 red-dot + Active-pulse requirements ARE met this stage (rev-14/rev-15/rev-16/rev-17 — real overlay + real pulse via PulseWorker); Update badge remains deferred per user-approved scope decision, frozen doc NOT edited)**) |
| **P1-6** Permissions complete + clipboard plugin (Cargo before capabilities, lib.rs init, build.rs all commands, input.json + provider-list, main.json all, popup.json, permission TOMLs in git add, integration test) | A4 Step 1 (Cargo.toml + lib.rs plugin), A4 Step 2 (build.rs), A4 Step 3 (input.json + main.json + popup.json), A4 Step 4 (`capabilities.rs` integration test), every commit lists the autogenerated TOMLs |
| **P1-7** C3 tests rewritten (vi.hoisted + invokeMock + routeInvoke, default mock has provider_get_active_selection, consumption order, proper destructuring, no conditional tests) | C1 Step 1 (rev-5-5: `DEFAULT_ROUTES` + `beforeEach` re-installs after `mockReset()`), C2/C3a-C3h (every test uses `routeInvoke`, `findByText`/`queryByRole`/`findByRole` destructured from render), **B1 rev-6-5: `routeInputInvoke` route-table helper, NO `mockResolvedValueOnce`** |
| **P1-8** C3 contract fidelity (rev-11/rev-12: latency IS implemented as an additive field; Balance still deferred) (provider_test_connection = `{ok, message, latency_ms?: Option<u32>}` — latency_ms set on the reachable HTTP path via `Instant` timing, None on failure; **rev-12: `u32::try_from(elapsed.as_millis()).unwrap_or(u32::MAX)` saturation + test asserting real Instant probe**; provider_get_balance does not exist; Balance UI placeholder; Balance states deferred to R4/S3 per user-approved scope decision; zero NEW backend commands — `latency_ms` is an additive field on the existing `ConnectionResult`) | C3c (rev-11: message + latency_ms backend field + frontend `{latency}ms` chip — `connection_latency.rs` integration test + `ProviderCenter.test.tsx` latency assertion; **rev-12 P2-5: saturating conversion + Instant-probe test**), C3f (placeholder + assert no IPC — Balance states deferred to R4/S3 per user-approved scope decision, rev-11), Global Constraints (P1-8 line, rev-11 latency amendment + rev-12 saturation hardening) |
| **P1-9** C5/C6 real keyboard tests (Playwright, ?nav= route in ui-lab, real Tab/Enter, aria-disabled+tabindex=0; C6 assert listener called once then behavior; onCleanup unlisten) | C5 Step 2 (**rev-8-4: `apps/ui-lab/e2e/keyboard.spec.ts` port 1421, `?nav=settings-keyboard` route, PRECISE locator `page.locator('[data-testid="shell"] .settings-shell__nav .sidebar-item:focus')` (NOT ambiguous `button:focus`), compare `aria-label` VALUE after Tab + `data-page` VALUE change after Enter**), C5 Step 4 (SidebarItem aria-disabled + tabindex=0 + **rev-8-1 SettingsShell diff-instruction edit with `data-testid="shell"` + `data-page`** + ui-lab `?nav=settings-keyboard` route via **rev-8-3 `FIXTURE_NAV_KEYS`** rendering the REAL SettingsShell imported via **rev-8-2 `@app/...` alias**), C6 Step 1 (COMPLETE `routeInvokeSettings` + `beforeEach` defaults; `expect(onFocusChangedMock).toHaveBeenCalledTimes(1)` before behavior; `onCleanup` unlisten test) |
| **P1-10** D5 real surfaces (Provider Center empty+populated, Keystore Recovery healthy+corrupt, Selection Popup 5 states in ui-lab, InputPanel 4 states in ui-lab, 600/699/700/800 × light/dark, fixed IPC mocks, tray = manual) | D5 Step 0-5 (extends `apps/ui-lab` port 1421; **rev-7-4: the Keystore fixture imports the COMPLETE production `KeystoreRecoveryView` (Banner+Confirm+Toast+busy)**; **rev-7-7: `ResultEntry` fixtures use `engine`/`errorText` (not `error`), InputPanel "idle" = `{ kind: "loading" }` + `idle:true`**; **rev-7-3: `InputPanelView` is the complete presentational body**; **rev-8-2: fixtures import via the `@app` alias (NOT relative `../../../src/...`)**; `ProviderState` seed uses real `"key-saved"`/`"empty"`; script is `test:visual`; `surfaces.visual.spec.ts` reuses existing `?nav=`/`?state=`/`?theme=` routing) |
| **P2** Dark canvas token #020617 | A1 Step 3 (`DARK_THEME_COLOR = "#020617"`), C4 Step 3 (index.html dark meta), entry-styling test asserts `#020617` + rejects `#0B1120` |
| **P2** theme-color meta: force current to media=all, disable other (rev-5-6) | A1 Step 3 (`syncThemeColorMetas` sets current `media="all"` + non-current `media="disabled"`), A1 test asserts `media="all"` on current + forced-vs-OS test, Global Constraints theme-color (rev-5-6) |
| **P2** Task count (rev-12) 22 ### + 8 #### = 30 headings, minus C3 umbrella = 29 executable (rev-12 keeps the same count as rev-11 — A5 fixes are within-task refinements, not new tasks; rev-16 likewise adds NO new tasks, only within-task test additions) | Global Constraints (task count line), Self-Review item 2 |
| **P2** Contract sync (MASTER.md, pages/04, pages/05, handoff-manifest) | **rev-8-9 + rev-11/rev-12 governance:** the four design docs are FROZEN — this plan does NOT modify them. D3 records the design-doc differences in THIS plan's retroactive table only (MASTER.md + pages/04-tray-menu.md + pages/05-provider-center.md + handoff-manifest.md are NOT edited). Per the rev-12 "审核快照（rev-12 用户已批准）" Surface status table: the pages/04 red-dot Error state and Active-pulse state ARE implemented this stage (rev-11/rev-12 Task A5 — rev-12 makes the red-dot a true overlay on the base icon and the pulse a real icon frame-swap, correcting rev-11's solid-square + tooltip-only implementations); the pages/04 Update-badge state and the pages/05 Balance states are deferred per user-approved scope decision (Update badge → R5/R6; Balance → R4/S3). The earlier "pending Range decision" framing is superseded by rev-11 — all five decisions (Tray Error red-dot = A, Tray Active pulse = A, Tray Update badge = B→R5/R6, Connection latency = A, Balance states = B→R4/S3) are user-approved. A design-doc edit would require a separate proposal. |

**2. Task count (rev-11):** A(5: A1, A2, A3, A4, **A5 (rev-11)**) + B(6: B1, B2, B3, B4, B5, B6) + C(13: C1, C2, C3a-h (8), C4, C5, C6) + D(5: D1, D2, D3, D4, D5) = **29 executable**. The C3 umbrella (`### Task C3`) is a heading, not an executable task. 22 `### Task` headings + 8 `#### Task` sub-headings = 30 task headings total.

**3. Placeholder scan (rev-8):** searched for "TBD"/"fill in"/"similar to"/"re-read and adapt"/"mirror helper"/"wherever"/"if backend does not"/"/* ... */"/"// ... existing". rev-8 contains ZERO such placeholders in code blocks. Every Rust block is valid Rust against the current crate; every TSX block is valid TSX. **rev-8-1 (load-bearing): the `SettingsShell` edit is expressed as PRECISE `diff`-style instructions** (5 numbered edits with `+`/`-` lines), NOT a full-file code block — so there is no `// ... the existing matchMedia signal ... UNCHANGED` or `{/* The existing <WindowChrome ...> block is UNCHANGED. */}` placeholder (the rev-7-2 block's placeholders are deleted). **rev-8-2:** ui-lab fixtures import via the `@app` alias (NOT relative `../../../src/...`). **rev-8-4:** the Playwright locator is the PRECISE `[data-testid="shell"] .settings-shell__nav .sidebar-item:focus` (NOT the ambiguous `button:focus`). **rev-8-5:** disabled-item assertions use the REAL copy value `/Coming in R3b/` (NOT an invented `/coming later/`); rail/wide mode is driven by `installMatchMedia(...)` (NOT `window.innerWidth`). **rev-8-6:** `InputPanelView.showClear` is a derivation `() => props.hasResult ?? false` (NOT a value read once); the solid-js import is ONE merged line. **rev-8-7:** the `KeystoreRecovery.tsx` import line covers BOTH the View + the controller (`createSignal`/`onMount`/`onCleanup`/`Show`/`For`/`Component` + `invoke` + `Banner`/`Confirm`/`Button`/`Toast` + `SETTINGS_COPY` + `detectLocale`); the `Confirm.onOpenChange` routes the boolean. **rev-8-8:** the tray refresh covers EIGHT provider mutation commands (rev-7-8's six + `provider_duplicate` + `provider_confirm_and_set_active`). **rev-8-9:** the four design docs are FROZEN (not modified). **rev-7-3 writes the complete `InputPanelView` body** (the rev-6-9 `// ... (the JSX body...)` stub is deleted). **rev-7-4 writes the complete `KeystoreRecoveryView`** (Banner + Confirm + Toast + busy + resetTriggerRef — the rev-6-9 single-Banner simplification is deleted). **rev-7-7 corrects every ResultEntry fixture** to `engine` (required) + `errorText` (not `error`). The B6 fallback reassembly is a single compile-clean indexed-`Vec` block with a parallel `locality` slice. The B4 clipboard source-save is an explicit `translate_clipboard` edit. The C6 `routeInvokeSettings` is a COMPLETE helper. The D5 fixtures extend the real `apps/ui-lab` + import the production `InputPanelView`/`KeystoreRecoveryView` via the `@app` alias.

**4. Type consistency (rev-6):**
- **rev-7-1:** `Monitor::work_area(&self) -> &PhysicalRect<i32, u32>` (verified at `tauri-2.11.5/src/window/mod.rs:96`) — returns the REAL usable work area, not `Option`, not derived. `Monitor::scale_factor(&self) -> f64` returns `f64` DIRECTLY (only `WebviewWindow::scale_factor()` returns `Result<f64>`). `PhysicalRect<i32, u32>` exposes `position.x: i32` + `position.y: i32` + `size.width: u32` + `size.height: u32` (verified at `tauri-runtime-2.11.3/src/dpi.rs:28`). `build_popup_anchor` resolves the target Monitor via `monitor_from_point` and uses THAT monitor's `scale_factor()` for both `work_area()` + the cursor; the popup window's `win.scale_factor().unwrap_or(1.0)` is only the `None`-monitor fallback. The factor is guarded (`sf > 0.0 && sf.is_finite()`).
- `PopupAnchor { cursor_logical: (f64, f64), work_area: LogicalWorkArea, scale_factor: f64 }` — the single geometry source (P1-2). `build_popup_anchor` uses the TARGET MONITOR's `scale_factor()` (Monitor returns `f64` directly); `app.monitor_from_point(x, y)` takes `f64`; the popup window's `scale_factor()` (`Result<f64>`) is only the fallback.
- `PopupMode` is `pub enum PopupMode { Loading, Single, Multi, Error }` with `size_logical() -> (u32, u32)` — used consistently.
- `Payload<'a>` gains `source_text: Option<&'a str>` (borrowed, fine — the `<'a>` lifetime covers the borrow). **rev-5-1:** `PopupMultiPayload` gains `source_text: Option<String>` (OWNED — a runtime `&str` cannot be borrowed as `&'static str`); callers pass `Some(text.to_owned())`. `PopupStatePayload`/`PopupMultiPayload` (frontend) gain `source_text?: string`.
- `resolve_target_language(to, settings_target) -> String` is the central resolver, called inside `run_translate_session`.
- `capture_and_translate(app, state, app_state, supplied_text: Option<String>, x: f64, y: f64, gen: u64)` is the single shared pipeline. rev-5-2: every post-loading error path calls `popup::error_with_source(app, &msg, &text)` (not `popup::error`).
- `translate_primary_only(client, keystore, preset, input) -> Result<Translation, Error>` preserves the primary Error (P1-4).
- `eligible_for_session_fallback(outcomes: &[TranslationOutcome], locality: &[bool], local_primary_failed: bool) -> bool` is the pure decision (P1-4 + rev-6-4); the locality slice is built in `translate_parallel` from the per-future `was_local = is_local(&preset)` flag + the local-primary gate is folded into the pure fn via `local_primary_failed`. A LOCAL provider's `FallbackEligible` never counts (`!was_local && FallbackEligible(_)`).
- **rev-5-4:** `set_active_primary_core(app_state: Arc<AppState>, uuid: String) -> Result<SetActiveResult, String>` calls `db_set_active_primary(&app, &uuid)` which uses the REAL `set_active_slots(&tx, uuid, &[], None)` (lib.rs:1711) — there is NO `write_active_selection`. `SetActiveOutcome { Written, NeedsConsent { actual_scope } }` (lib.rs:1672) maps 1:1 to `SetActiveResult`.
- **rev-7-8 + rev-8-8:** `refresh_tray(app)` updates the existing `"main-tray"` via `app.tray_by_id("main-tray")` + `tray.set_menu(Some(menu))` (`TrayIcon::set_menu` takes `Option<M>`, verified at `tauri-2.11.5/src/tray/mod.rs:512`) + `tray.set_tooltip(...)` — does NOT rebuild a duplicate tray. `refresh_tray_if_available` wraps it (best-effort, logs on failure). The EIGHT provider mutation commands (`provider_create`/`provider_update`/`provider_delete`/`provider_toggle`/`provider_reorder`/`provider_set_active`/`provider_duplicate`/`provider_confirm_and_set_active`) each gain an `app_handle: tauri::AppHandle` parameter (they previously took only `state`) + rename `app`→`app_state`. `read_enabled_providers` maps `DbErr` (the `db/mod.rs:39` enum, aliased `DbErr` at `lib.rs:1606`), NOT `rusqlite::Error`.
- **rev-7-4:** `BannerProps = { variant, title, description?, action?, onDismiss?, dismissLabel?, class? }` (verified at `packages/ui/src/components/Banner.tsx:7`) — NO `icon`, NO `children`. The Keystore `KeystoreRecoveryView` reproduces the FULL surface (destructive Banner + Confirm + Toast + busy + resetTriggerRef); the Archive + Reset buttons go in `Banner.action` as a `<span>` wrapper, the message in `description`.
- **rev-7-3 + rev-7-4 + rev-8-2:** `InputPanelView(props: InputPanelViewProps)` + `KeystoreRecoveryView(props: KeystoreRecoveryViewProps)` are named exports of the production components, imported by the ui-lab fixtures via the `@app` alias (rev-8-2: `@app/InputPanel` + `@app/features/settings/KeystoreRecovery`) so the lab renders the SAME presentational surface (no approximate redraw). `InputPanelView` is the COMPLETE body (textarea + single/multi/partial ResultCard grid + InlineError) with **rev-8-6: `showClear = () => props.hasResult ?? false` (reactive derivation)**; `KeystoreRecoveryView` is the COMPLETE surface (no simplification) with **rev-8-7: the merged import line + the boolean `Confirm.onOpenChange`**.
- **rev-7-7:** `ResultEntry = { uuid, engine (REQUIRED), text?, errorText?, ok }` — fixtures use `engine` + `errorText` (NOT `error`); "idle" = `{ kind: "loading" }` + `idle:true` (no `idle` kind exists).
- `provider_test_connection` returns `ConnectionResult { ok: bool, message: String, latency_ms: Option<u32> }` (rev-11: `latency_ms` is a real additive field, set on the reachable HTTP path via `Instant::now()`/`elapsed()`; `None` on early-return failures + transport error). `provider_get_balance` does NOT exist (Balance states deferred to R4/S3 per user-approved scope decision, rev-11).
- **rev-11 (Task A5) tray controller APIs:** `tauri::tray::TrayIcon::set_icon(&self, icon: Option<Image<'_>>) -> crate::Result<()>` + `TrayIcon::set_tooltip<S: AsRef<str>>(&self, tooltip: Option<S>) -> crate::Result<()>` (both verified at `tauri-2.11.5/src/tray/mod.rs`); `tauri::image::Image::from_bytes(bytes: &[u8]) -> crate::Result<Image>` (verified at `tauri-2.11.5/src/image/mod.rs:76`); the crate's `tauri` feature list already includes `image-png` so PNG bytes decode without an extra runtime feature. `Manager::tray_by_id(&self, "main-tray") -> Option<TrayIcon>` (used in A4's `refresh_tray`). The Error red-dot PNG is generated at build time by `build.rs` (using the `image = "0.25"` build-dependency) into `OUT_DIR` and embedded via `include_bytes!(concat!(env!("OUT_DIR"), "/tray-error-32.png"))` — NO external file path, NO design-asset PNG checked into the repo.
- `engineLabel(raw: string): string` is defined in both `popupController.ts` and `inputController.ts`.
- `translateSelection(sourceText?)` (selection-ipc) vs `translateClipboard()` — distinct entries → distinct backend commands. rev-5-7: clipboard-origin payloads carry `source_text` so Retry re-uses the saved source (NOT a clipboard re-read).
- **rev-5-6:** `syncThemeColorMetas` sets the current meta to `media="all"` (always wins, even when forced-vs-OS) + the non-current to `media="disabled"`.

**5. Risk notes for the implementer (rev-5):**
- A2 Step 5 depends on A3's `PopupAnchor`/`set_popup_mode`/`show_at_sized`/`loading_with_source`. Order is A1 → A3 → A2 → A4 → **A5 (rev-11)**.
- **rev-7-1 (geometry):** `Monitor::work_area()` (verified at `tauri-2.11.5/src/window/mod.rs:96`) returns the REAL OS-reported usable work area — no approximation. The `system_bar_deduction()` approximation is DELETED. `Monitor::scale_factor()` returns `f64` directly (do NOT `.unwrap_or()` it) — and it is the factor `build_popup_anchor` uses for the cursor's monitor. `WebviewWindow::scale_factor()` returns `Result<f64>` and is ONLY the `None`-monitor fallback.
- A4 Step 9 `set_active_primary_core` uses the REAL `set_active_slots` (lib.rs:1711) + `db_set_active_primary` — there is NO `write_active_selection`. If `provider_set_active`'s closure is refactored, keep `db_set_active_primary` as the parallel-empty fast path.
- **rev-7-8 + rev-8-8 (tray):** `refresh_tray` MUST use `tray_by_id("main-tray")` + `set_menu(Some(menu))` (`Option<M>`) + `set_tooltip`; do NOT call `build_tray` (which would `TrayIconBuilder::with_id("main-tray")` again and panic on the duplicate id). Every provider mutation command calls `refresh_tray_if_available(&app_handle)` (the best-effort wrapper) on its success path (Step 9b) — each of the EIGHT commands (`provider_create`/`provider_update`/`provider_delete`/`provider_toggle`/`provider_reorder`/`provider_set_active`/`provider_duplicate`/`provider_confirm_and_set_active`) takes a NEW `app_handle: tauri::AppHandle` parameter + renames its local `app`→`app_state`. `read_enabled_providers` maps `DbErr`, not `rusqlite::Error`.
- B4 Step 0 adds the clipboard plugin npm dep; the Cargo dep + plugin init landed in A4. There is NO navigator.clipboard fallback — delete any existing `navigator.clipboard?.writeText`.
- B4 P1-3 + rev-5-7 are load-bearing: the popup controller saves `payload.source_text` on loading/error/result/multi (including clipboard-origin), clears on a new session, and `buildActions` hides Retry when `lastSource` is empty. Copy writes the translation; Retry re-translates the saved source via `translateSelection(lastSource)` (never re-reads the clipboard).
- B5 Step 1 needs a `profile_unsupported` helper; read `profile_to_preset` in `adapter.rs` first to build a profile it actually rejects.
- **rev-5-3 + rev-6-4 (B6):** the fallback reassembly is a single compile-clean indexed-`Vec` block; `primary_was_local` is read from `by_idx` BEFORE the consuming `for` loop drains it. The `locality: Vec<bool>` slice is built in lockstep with `outcomes` (rev-6-4) and passed to `eligible_for_session_fallback(outcomes, &locality, local_primary_failed)`. Do NOT reintroduce the `while let ready_iter.next()` iterator pattern (it moved `entries` then indexed it).
- B6 changes `translate_parallel` to use `translate_primary_only`; any pre-existing test asserting per-engine fallback MUST be updated and the change documented. B6's mixed rule (local-primary failure blocks session fallback) is the refined P1-4 reading. **rev-6-4:** the locality rule now extends local-sacred to EVERY local provider (not just the primary) — a local parallel provider's `FallbackEligible` never counts.
- **rev-5-5 (C1):** the `DEFAULT_ROUTES` table is re-installed in `beforeEach` AFTER `mockReset()` — without this, the second test throws `unexpected invoke provider_list`. Tests needing custom routes merge: `routeInvoke({ ...DEFAULT_ROUTES, ...custom })`.
- C3c shows `message` + `{latency}ms` (rev-11: latency is now implemented — `latency_ms: Option<u32>` on `ConnectionResult`, set on the reachable path; the "no latency" wording from P1-8 is superseded by rev-11-2). C3f shows the placeholder and calls no IPC (Balance states deferred to R4/S3 per user-approved scope decision, rev-11). Neither adds a NEW backend command — C3c's `latency_ms` is an additive field on the existing `provider_test_connection` return type, and C3f adds none.
- **rev-5-8 (C5/C6):** the Playwright keyboard spec reads `data-page` off the REAL SettingsShell root (which now carries `data-testid="shell"` + `data-page`), and the `tabUntilSidebarItemFocused` helper escapes the OS window-control focus trap (cap 12 Tabs). `routeInvokeSettings` is the COMPLETE helper defined in C6 Step 1 — it merges over `DEFAULT_SETTINGS_ROUTES` and re-installs in `beforeEach`.
- **rev-6-7 (C5 keyboard spec):** the Playwright keyboard spec lives at `apps/ui-lab/e2e/keyboard.spec.ts` (port 1421, the existing `apps/ui-lab/playwright.config.ts`), targeting the `?nav=settings-keyboard` route added to `apps/ui-lab/src/App.tsx`. NO root `e2e/` directory, NO root Playwright config, NO port 1420. The lab already has `@playwright/test` + the webServer config committed. Run with `pnpm --filter @linguaray/ui-lab test:visual keyboard`.
- C6 asserts `expect(onFocusChangedMock).toHaveBeenCalledTimes(1)` BEFORE testing behavior (P1-9), and `onCleanup` calls `unlisten`.
- **rev-5-9 (D5):** the visual suite runs in `apps/ui-lab` (port 1421), NOT the main app. The popup fixtures already exist (`SelectionPopup.tsx`); InputPanel + KeystoreRecovery are NEW fixtures. Verify the `?nav=input-window` + `?nav=keystore` + `?fixture=` routes are wired in `apps/ui-lab/src/App.tsx` before generating baselines.
- The two pre-existing `_for_test` names are KEPT (renaming would expand blast radius); no NEW `*ForTest` names are introduced.
