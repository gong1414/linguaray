# LinguaRay Handoff Manifest

**状态：** R0 冻结 · **日期：** 2026-08-08
**设计源：** [RAYLINE-REDESIGN.md](RAYLINE-REDESIGN.md) · [token-map.md](token-map.md) · [MASTER.md](MASTER.md)

---

## Penpot File & Team

Team ID: 81f57451-85cc-819d-8008-72a1a7e76bb1
File ID: 3be9e5e1-190f-8090-8008-72a4d9868ce7

---

## Penpot 页面

| 页面 | Page ID |
|---|---|
| Foundations | 3be9e5e1-190f-8090-8008-72a4d9868ce8 |
| Components | c1f4df2c-22c2-800a-8008-72aae7746c8d |
| Core | c1f4df2c-22c2-800a-8008-72aae77494a8 |
| Settings | c1f4df2c-22c2-800a-8008-72aae774f806 |
| Knowledge | c1f4df2c-22c2-800a-8008-72aae775044f |
| OCR | c1f4df2c-22c2-800a-8008-72aae7755128 |
| System | c1f4df2c-22c2-800a-8008-72aae775512f |
| Handoff | c1f4df2c-22c2-800a-8008-72aae775a12b |

---

## Token 集合

- Core: 97 Token（Core / Primitives）
- Light: 28 Token（Semantic / Light）
- Dark: 28 Token（Semantic / Dark）
- 工程扩展: 21 Token（标注 [工程扩展]，未回写 Penpot；含 R1 新增 4 个 `--color-status-{success,warning,danger,info}-fg` 用于 StatusBadge 软背景 WCAG AA 配对）

---

## 16 Surface

| # | Surface ID | Penpot 页面 | Surface Node ID |
|---|---|---|---|
| 01 | surface.selection-popup | Core | c1f4df2c-22c2-800a-8008-72aeabc47e06 |
| 02 | surface.input-window | Core | c1f4df2c-22c2-800a-8008-72aeac1da766 |
| 03 | surface.multi-result | Core | c1f4df2c-22c2-800a-8008-72aeac83fb23 |
| 04 | surface.tray-menu | Core | c1f4df2c-22c2-800a-8008-72aeacd6d5bb |
| 05 | surface.provider-center | Settings | c1f4df2c-22c2-800a-8008-72af2cb703d8 |
| 06 | surface.keystore-recovery | Settings | c1f4df2c-22c2-800a-8008-72af2dc439df |
| 07 | surface.shortcuts | Settings | c1f4df2c-22c2-800a-8008-72af2e3c2c64 |
| 08 | surface.privacy-data | Settings | c1f4df2c-22c2-800a-8008-72af2ee2ea41 |
| 09 | surface.history | Knowledge | c1f4df2c-22c2-800a-8008-72b02c9695d8 |
| 10 | surface.vocabulary | Knowledge | c1f4df2c-22c2-800a-8008-72b02da02086 |
| 11 | surface.dictionary | Knowledge | c1f4df2c-22c2-800a-8008-72b09ce20184 |
| 12 | surface.ocr-overlay | OCR | c1f4df2c-22c2-800a-8008-72b1b033e028 |
| 13 | surface.text-to-speech | OCR | c1f4df2c-22c2-800a-8008-72b1b0ebe8a3 |
| 14 | surface.onboarding | System | c1f4df2c-22c2-800a-8008-72b25445c602 |
| 15 | surface.external-api | System | c1f4df2c-22c2-800a-8008-72b255056533 |
| 16 | surface.updater | System | c1f4df2c-22c2-800a-8008-72b2eecf7bb1 |

> 以上 Node ID 已通过 Penpot MCP 从对应 Surface 根画板读取，并通过严格 UUID 与全局唯一性校验。

---

## 18 Component

| 组件 | Penpot 页面 | Component Node ID |
|---|---|---|
| Button | Components | c1f4df2c-22c2-800a-8008-72adc60dd9de |
| Icon button | Components | 1b620aa9-34b8-80e9-8008-7379e6d0bd24 |
| Segmented control | Components | c1f4df2c-22c2-800a-8008-72adc6cf1ad2 |
| Shortcut chip | Components | c1f4df2c-22c2-800a-8008-72adc729c2fa |
| Text field | Components | c1f4df2c-22c2-800a-8008-72adc6bf6adb |
| Select | Components | 1b620aa9-34b8-80e9-8008-737a1c5e1087 |
| Toggle | Components | c1f4df2c-22c2-800a-8008-72adc71e5243 |
| Status badges | Components | 1b620aa9-34b8-80e9-8008-737b3d039f8d |
| Inline error | Components | c1f4df2c-22c2-800a-8008-72adc8dd10d8 |
| Toast | Components | c1f4df2c-22c2-800a-8008-72adc7a61dd1 |
| Confirmation dialog | Components | c1f4df2c-22c2-800a-8008-72adc8b1f21d |
| Empty state | Components | c1f4df2c-22c2-800a-8008-72adc88bb6f8 |
| Translation card | Components | c1f4df2c-22c2-800a-8008-72adc8228750 |
| Result card | Components | 1b620aa9-34b8-80e9-8008-737b1e601ace |
| Provider row | Components | c1f4df2c-22c2-800a-8008-72adc7c65a5b |
| History row | Components | c1f4df2c-22c2-800a-8008-72adc7ea7945 |
| Sidebar item | Components | c1f4df2c-22c2-800a-8008-72adc6fa0825 |
| Window chrome | Components | c1f4df2c-22c2-800a-8008-72ae2a593189 |

> Variant 组件族记录 Variant Container Node ID；单体组件记录主实例 Node ID。18 个根节点均位于 `01 Components`，并写入 `rayline.component-id`、`rayline.contract-version=rev-4.2` 与 `rayline.handoff-status=R0-frozen` 元数据。

---

## 最近批准修订

2026-08-08 rev-4.2：焦点色 #0891B2、品牌 Indigo #4F46E5 (light) / #818CF8 (dark)、neutral 命名、shadow.raised/overlay 原生值、border.control 工程扩展、strong-fill 两主题统一、Core 97 / Semantic 28+28。

2026-08-08 R1-1：tokens.css 三层结构（Core / Semantic / Aliases）落地；新增 4 个工程扩展 Token `--color-status-{success,warning,danger,info}-fg`（Penpot success/warning/danger.default 在对应 soft 背景上不达 AA 4.5，info.* 无 Penpot 源）。工程扩展由 17 → 21；Core 97 / Semantic 28+28 不变。
