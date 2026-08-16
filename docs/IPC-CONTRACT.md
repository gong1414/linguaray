# IPC Contract（Rust 命令 × 前端调用 × capabilities 授权对账清单）

> 本文是 LinguaRay IPC 面的唯一对账基线。任何命令增删必须四方同步：
> **Rust 实现 → `collect_commands!` 注册/生成 bindings → capabilities 授权 → 前端 `commands.*` 调用**。
> 配套 lock 测试 `test/ipc-contract.test.ts` 在 CI 中强制执行下列不变量。

## 不变量（由 test/ipc-contract.test.ts 锁定）

| # | 不变量 | 含义 |
|---|--------|------|
| 1 | `called ⊆ registered` | 前端每个生成包装器调用必须映射到 `collect_commands!` 中的命令，杜绝幻影调用 |
| 2 | `called ⊆ authorized` | 前端每个调用的命令必须被至少一个 capability 授权（fail-closed） |
| 3 | `authorized ⊆ registered` | capabilities 里不存在指向未注册命令的 `allow-*`，杜绝过期授权 |
| 4 | `registered ⊆ called ∪ RETAINED` | 每个注册命令要么有前端调用方，要么显式列入下方保留清单并注明理由 |

## 命名规范

- Rust 命令：`snake_case`；capability 权限条目：`allow-kebab-case`（Tauri 自动转换）。
- 生成的 TypeScript 方法为 `camelCase`；对账测试从 `bindings.ts` 读取它对应的
  Rust `snake_case` 命令，再与注册和授权集合比较。

## 显式保留（registered 但前端零调用）

**当前为空。** 每个注册命令都有前端调用方（不变量 4 的硬约束）。

历史记录：迁移审计发现 5 个零调用命令（`translate`、`translate_default`、
`tts_list_voices`、`archive_database`、`get_data_readiness`），已于重构 P0.3
从 `collect_commands!`、capabilities 与 Rust 实现三处同步删除。此后新增
"注册但暂无调用方"的命令时，必须先加入 `test/ipc-contract.test.ts` 的
RETAINED 清单并在本表登记理由。

## 前置条件

- capabilities 仅使用 `core:default`、`dialog:*`、`process:*`、`opener:*` 等插件权限与 `allow-*` 自定义命令权限；插件权限不参与命令对账。
- 事件（`emit`/`listen`）面不在本清单范围，由各 feature 测试自行锁定。
