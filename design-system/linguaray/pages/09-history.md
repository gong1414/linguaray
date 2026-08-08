# Surface 09: History

**Surface ID:** `surface.history`
**Penpot 页面:** 30 Knowledge
**Penpot 画板尺寸:** [待查询]
**生产窗口默认尺寸:** 设置子页
**生产窗口最小尺寸:** 同设置（600×400）
**尺寸冲突说明:** 以生产窗口尺寸为实施标准

## 状态矩阵

| 状态 | 描述 | 组件 |
|---|---|---|
| Initial (not opted in) | 隐私门："启用历史？" 附解释 | Confirm/隐私门 + Button |
| Empty | "暂无历史" | EmptyState |
| Loading | 骨架行 | Skeleton rows |
| Populated | 列表：源片段 → 翻译片段 + 引擎 + 时间戳 | History row × N |
| Search | 过滤结果；空时"无匹配" | Search field + History row（过滤）/ EmptyState |
| Export | 格式选择 → 文件保存对话框 | Select (格式) + 文件保存 |
| Retention cleanup | 后台清理运行；非侵入式 summary badge 显示"已清理 N 条" | Badge (info) |

## 本地化 copy key

| key | en | zh |
|---|---|---|
| `history.title` | History | 历史 |
| `history.privacyGate.title` | Enable history? | 启用历史？ |
| `history.privacyGate.description` | History is encrypted and stored locally only. | 历史经过加密，仅本地存储。 |
| `history.privacyGate.enable` | Enable | 启用 |
| `history.privacyGate.skip` | Skip | 跳过 |
| `history.empty.title` | No history yet | 暂无历史 |
| `history.search.placeholder` | Search history… | 搜索历史… |
| `history.search.noMatches` | No matches | 无匹配 |
| `history.export.title` | Export | 导出 |
| `history.export.format` | Format | 格式 |
| `history.cleanup.summary` | {count} items cleaned | 已清理 {count} 条 |
| `history.action.delete` | Delete | 删除 |
| `history.action.favorite` | Favorite | 收藏 |
| `history.action.unfavorite` | Unfavorite | 取消收藏 |

## 组件组合

- **隐私门（未启用时）：** 说明卡 + Button (enable) / Button (skip)
- **搜索栏：** TextField (search, leading Search icon)
- **列表：** History row × N
  - 源片段 → 翻译片段
  - 引擎标签（`--text-xs`）+ 时间戳（`--text-xs`）
  - 收藏按钮 + 删除按钮
- **加载：** Skeleton rows
- **导出：** Select (格式) → 文件保存对话框
- **清理 badge：** 非侵入式 summary badge（顶部或角落）

## 页面特有约束

- 设置子页，遵循设置窗口尺寸与自适应规则。
- 同意门：首次启动需显式同意才写入历史。
- 加密搜索：批量解密（固定 200 行/批）+ cursor 分页 + Unicode NFKC + 大小写折叠匹配；内存中仅一批。
- 单条损坏记录不中断查询，标记 `corrupt: true` 显示"损坏条目"。
- 数据库阻塞工作放 `spawn_blocking`。
- 收藏永不过期；保留期默认 30 天，可配置。
