# bridge/ — Tauri API 唯一入口

`src/` 下只有本目录允许 `import "@tauri-apps/…"`（`test/bridge-boundary.test.ts`
强制；docs/UI-RULES.md 规则 3）。

- 业务命令一律使用 `import { commands } from "../bridge/invoke"` 提供的
  tauri-specta 生成包装器；不导出裸 `invoke`，也不允许直接引 `@tauri-apps/*`。
- View 组件不直接调用 `commands`：命令先封装为 feature 的 `*-ipc.ts` 模块，
  再供 View 使用（现有 `features/**/*-ipc.ts` 均通过本目录转发）。
- 新增 Rust 命令时：添加 `#[tauri::command]` / `#[specta::specta]`，登记到
  `collect_commands!`，运行 `pnpm bindings:generate`，再同步 capability 与
  feature IPC 封装；`pnpm bindings:check` 负责检查生成文件漂移。
- 新增插件依赖时：在此目录加对应转发模块，并在边界测试的清单中登记。
