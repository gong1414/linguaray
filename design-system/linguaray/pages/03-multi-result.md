# Surface 03: Multi-Result

**Surface ID:** `surface.multi-result`
**Penpot 页面:** 10 Core Translation
**Penpot 画板尺寸:** [待查询]
**生产窗口默认尺寸:** 600×400
**生产窗口最小尺寸:** 600×400
**尺寸冲突说明:** 以生产窗口尺寸为实施标准

## 状态矩阵

| 状态 | 描述 | 组件 |
|---|---|---|
| Loading | N 个 spinner 卡片（每个并行 provider 一个） | SpinnerCard × N |
| Partial success | 成功引擎的结果 + 失败引擎的错误 badge | ResultCard（成功）+ Inline error badge（失败） |
| All success | 所有卡片填充，按用户 provider 排序显示（非按耗时）；已完成卡片不随后续到达跳位 | ResultCard × N（按排序） |
| All failed | 所有卡片显示错误；回退（如配置）作为单独结果显示 | Inline error × N + ResultCard（回退） |
| Error (consent revoked) | "需要多引擎同意" — 链接重新同意 | EmptyState + Link/Button |
| Pinned | 固定结果卡片，保持可见 | ResultCard（pinned） |

## 本地化 copy key

| key | en | zh |
|---|---|---|
| `multiresult.loading` | Translating with {count} providers… | 正在用 {count} 个服务商翻译… |
| `multiresult.consent.title` | Multi-engine consent required | 需要多引擎同意 |
| `multiresult.consent.cta` | Re-consent | 重新同意 |
| `multiresult.consent.notice` | Your text is sent to multiple services. | 您的文本将发送到多个服务。 |
| `multiresult.fallback.label` | Fallback result | 回退结果 |

## 组件组合

- **容器：** 弹窗展开模式（600×400），`--color-bg-elevated`
- **卡片网格：** ResultCard 横向并排（按 provider 排序）
  - 2 providers：2 列，每列 ≥ 200px
  - 3+ providers：超宽时内部横向滚动
  - 每张卡片独立垂直滚动
- **加载：** 每张卡片内 Spinner (12px)
- **同意提示：** 进入多引擎前 Confirmation dialog（同意数据发送范围）
- **回退结果：** 单独 ResultCard（标注 fallback）

## 页面特有约束

- 卡片位置稳定：按 provider 排序，结果到达不跳位（不按耗时排序）。
- 进入展开模式通过 Tauri `setSize`/`setMaxSize` 切到 600×400；离开时还原。
- 工作区夹紧：四边夹紧，margin = 8px。
- 多引擎需显式 opt-in 并完成同意（`preferences.parallel_consent_version`）。
- 添加新 provider 到并行列表需重新确认数据发送范围。
