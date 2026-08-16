# 已知问题登记（待裁决，不在重构范围内擅自变更）

## BUG-1: PopupView 收藏生词的 targetLanguage 硬编码 "zh"

- **位置**: `src/features/translation/PopupView.tsx`（收藏到生词本路径）
- **现状行为**: popup 窗口点收藏时 `targetLanguage` 固定传 `"zh"`。
- **不一致**: 输入窗口（`InputPanelView` → `inputController`）按
  `detectLocale()`（zh/en）取收藏目标语言；popup 是迁移端口时引入的偏差。
- **为什么没修**: 修它 = 用户可见行为变更（英文用户的收藏语言会从 zh 变为
  en），超出"保持行为的等价重构"边界，需产品决策。
- **裁决选项**: ① 跟随 locale（与输入窗一致，推荐）；② 保持 zh（如果生词本
  定位就是中英学习）；③ 跟随该次翻译的目标语言。
- **登记日期**: 2026-08-16（重构现代化审计）
