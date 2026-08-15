# bridge/ — Tauri API 唯一入口

`src/` 下只有本目录允许 `import "@tauri-apps/…"`（`test/bridge-boundary.test.ts`
强制；docs/UI-RULES.md 规则 3）。

- 业务代码一律 `import { invoke } from "../bridge/invoke"` 等，不允许直接
  引 `@tauri-apps/*`。
- View 组件不新增直接 `invoke` 调用：新命令封装为 `*-ipc.ts` 模块再供 View
  使用（现有 `features/**/*-ipc.ts` 均通过本目录转发）。
- 新增插件依赖时：在此目录加对应转发模块，并在边界测试的清单中登记。
