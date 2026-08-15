# LinguaRay 界面强制规则（止血冻结令）

> 2026-08-16 起，所有修改 UI 的 AI 与人类贡献者必须遵守本文件。
> 背景：自制 Solid 组件库（packages/ui，26 个组件）与散落在页面里的直接
> IPC/窗口调用，导致界面持续失控。决策：停止扩建自制组件库，后续迁移
> React + Mantine；在此之前以下门禁由测试强制执行。

## 冻结规则（由测试强制）

1. **禁止新增自制 UI 组件。** `packages/ui/src/components/` 的组件清单已
   冻结（`test/ui-freeze.test.ts`）。要新增组件 = 先改冻结清单 + 说明理由。
2. **禁止在普通窗口自定义标题栏。** 设置（main）、引导（onboarding）、
   输入（input）窗口使用系统原生标题栏；只有翻译浮窗（popup）和 OCR
   浮层允许 `decorations: false`。生产代码禁止使用 `WindowChrome`
   （ui-lab 组件画廊除外）。
3. **`@tauri-apps/*` 只能出现在 `src/bridge/`。** 业务代码一律从
   `src/bridge/*` 导入（`test/bridge-boundary.test.ts` 强制）。
   View 组件不应新增直接 `invoke`；新命令封装进 bridge 模块。
4. **窗口权限最小授权。** 每个窗口 capability 必须覆盖、且只授予其前端
   实际调用的窗口操作（`test/window-permission-gate.test.ts` 强制）。
5. **Debug 与 Release 身份必须可区分。** `pnpm dev:app` 使用
   `tauri.debug.conf.json`（bundle id `…linguaray.debug`、产品名
   `LinguaRay Dev`、窗口标题带 Dev 指纹）。改 `tauri.conf.json` 窗口
   字段必须同步 debug 副本，否则 drift 测试失败。

## 约定（代码评审执行）

6. 颜色、间距、圆角、字体只能来自 design token（`@linguaray/ui/styles`），
   禁止硬编码 hex/px 魔法值（已有 lock 测试禁止 App.css 硬编码 hex）。
7. 禁止使用裸 `<button>`/`<input>`/`<select>` 模拟已有组件的能力。
8. 单个 View 超过 ~600 行必须按区块拆分；禁止再次出现 2,000 行页面。
9. ui-lab 必须渲染生产组件组合（真实 SettingsShell 等），不能只渲染
   简化替身。
10. 修改 UI 的 PR/提交必须附真实应用截图对照（见
    `scripts/real-app-screenshots.sh` 与 `docs/baselines/real-app/`）。

## 命令速查

- 开发（Dev 身份）：`pnpm dev:app`
- 本地免签名构建（无 updater 产物）：`pnpm build:local`
- 真实应用截图基线：`bash scripts/real-app-screenshots.sh`
- 门禁测试：`pnpm test`（含 freeze / bridge / window-permission / drift）
