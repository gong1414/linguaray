# Surface 11: Dictionary

**Surface ID:** `surface.dictionary`
**Penpot 页面:** 30 Knowledge
**Penpot 画板尺寸:** [待查询]
**生产窗口默认尺寸:** 设置子页
**生产窗口最小尺寸:** 同设置（600×400）
**尺寸冲突说明:** 以生产窗口尺寸为实施标准

## 状态矩阵

| 状态 | 描述 | 组件 |
|---|---|---|
| No packages | "未安装词典" + "浏览词典包" | EmptyState + Button |
| Package installing | 进度条 | Progress bar |
| Lookup result | 释义文本 + 来源词典名 | 释义区 + 来源标签 |
| Lookup no result | "未找到释义" | EmptyState |
| Lookup error | "词典错误：{message}" | Inline error |

## 本地化 copy key

| key | en | zh |
|---|---|---|
| `dictionary.title` | Dictionary | 词典 |
| `dictionary.lookup.placeholder` | Look up a word… | 查询单词… |
| `dictionary.lookup.action` | Look Up | 查询 |
| `dictionary.noPackages.title` | No dictionaries installed | 未安装词典 |
| `dictionary.noPackages.cta` | Browse packages | 浏览词典包 |
| `dictionary.installing.progress` | Installing {name}… | 正在安装 {name}… |
| `dictionary.result.source` | Source: {name} | 来源：{name} |
| `dictionary.result.noResult` | No definition found | 未找到释义 |
| `dictionary.error` | Dictionary error: {message} | 词典错误：{message} |

## 组件组合

- **查询栏：** TextField (lookup) + Button (look up)
- **结果区：** 释义文本（`--text-base`）+ 来源词典名标签（`--text-xs` `--color-fg-muted`）
- **无包：** EmptyState + Button (浏览词典包)
- **安装：** Progress bar
- **错误：** Inline error
- **无结果：** EmptyState

## 页面特有约束

- 设置子页，遵循设置窗口尺寸与自适应规则。
- macOS：系统词典 + 离线 StarDict/MDX；Windows：仅 StarDict/MDX（离线包格式一致）。
- 离线归因（source dictionary name 标注）。
- 查询使用 `dict_lookup(word)` / `dict_list_packages()` / `dict_install_package(path)`。
