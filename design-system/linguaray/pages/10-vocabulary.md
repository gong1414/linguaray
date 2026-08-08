# Surface 10: Vocabulary

**Surface ID:** `surface.vocabulary`
**Penpot 页面:** 30 Knowledge
**Penpot 画板尺寸:** [待查询]
**生产窗口默认尺寸:** 设置子页
**生产窗口最小尺寸:** 同设置（600×400）
**尺寸冲突说明:** 以生产窗口尺寸为实施标准

## 状态矩阵

| 状态 | 描述 | 组件 |
|---|---|---|
| Empty | "暂无保存的单词" + 提示 | EmptyState |
| Populated | 列表：单词 + 释义片段 + 时间戳 + 删除 | ListRow × N |
| Export | 格式选择：CSV / JSON / AnkiConnect → 进度 → 完成/错误 | Select (格式) + 进度 + 结果 |

## 本地化 copy key

| key | en | zh |
|---|---|---|
| `vocabulary.title` | Vocabulary | 生词本 |
| `vocabulary.empty.title` | No saved words yet | 暂无保存的单词 |
| `vocabulary.empty.hint` | Save words from translations to build your list. | 从翻译中保存单词以建立您的列表。 |
| `vocabulary.action.add` | Add | 添加 |
| `vocabulary.action.delete` | Delete | 删除 |
| `vocabulary.action.export` | Export | 导出 |
| `vocabulary.export.format.csv` | CSV | CSV |
| `vocabulary.export.format.json` | JSON | JSON |
| `vocabulary.export.format.anki` | AnkiConnect | AnkiConnect |
| `vocabulary.export.progress` | Exporting… | 导出中… |
| `vocabulary.export.done` | Export complete | 导出完成 |
| `vocabulary.export.error` | Export failed: {reason} | 导出失败：{reason} |
| `vocabulary.field.word` | Word | 单词 |
| `vocabulary.field.definition` | Definition | 释义 |

## 组件组合

- **列表：** ListRow × N
  - title：单词
  - subtitle：释义片段（`--text-xs` `--color-fg-muted`）
  - trailing：时间戳 + IconButton (delete)
- **空状态：** EmptyState（icon + title + hint）
- **导出：** Select (格式：CSV/JSON/AnkiConnect) + Button (导出)
  - 进度指示（Spinner）
  - 完成/错误反馈（Toast）
- **添加：** 从翻译 popup 收藏（链接 Surface 01 的 favorite 动作）

## 页面特有约束

- 设置子页，遵循设置窗口尺寸与自适应规则。
- 单词 + 释义加密存储（AES-256-GCM + 域分离 AAD）。
- AnkiConnect 导出：解密内容仅在内存，不写临时明文文件；发送到 `127.0.0.1:8765`。
- UUID 在加密前生成并用于 AAD。
