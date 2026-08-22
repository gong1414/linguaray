# 全新 UI 产品与设计规范

## 设计任务

为 LinguaRay 设计一套新的 macOS/Windows 桌面界面。它是“随时可调用的翻译工具”，
不是聊天机器人、网页后台、手机应用，也不是旧项目的换色版本。

现有代码中的主窗口、侧栏、欢迎页、设置页、快捷翻译窗和 golden 不是视觉参考。
执行者应从用户任务出发重新组织层级，同时保留已经验证的能力和架构边界。

## 设计基础

- Flutter 官方 Material 3 是唯一 UI 基础。
- 使用 `ThemeData`、`ColorScheme` 和少量 LinguaRay `ThemeExtension` 表达品牌；
  不再建立一套平行的按钮、卡片、开关、对话框和表单组件。
- 不引入 Ant Design、聊天组件库、第二套 Flutter UI framework 或 WebView UI。
- 复用 `assets/brand/linguaray/` 中的正式 Logo、图标和托盘资产，不重新画 Logo。
- 品牌气质：安静、清晰、专业、轻量、隐私优先。使用中性 surface 和克制的 ray teal
  作为操作色。禁止复刻旧界面的米色背景、深蓝胶囊侧栏和荧光绿色选中态。
- 不使用大面积渐变、发光、玻璃拟态、聊天气泡、营销型 dashboard 卡片海洋。

## 核心用户流程

### 1. 快捷翻译

1. 用户通过托盘或全局快捷键打开快捷窗。
2. 输入、粘贴、划词或 OCR 文本进入同一 source state。
3. 用户能立即看到源语言、目标语言和正在使用的服务。
4. 结果以紧凑、可复制的方式显示；多 provider 可切换或比较，但不能像聊天消息。
5. 非置顶时失焦关闭；置顶后保持；窗口随内容动态增高并限制在当前屏幕工作区。

快捷窗不得包含侧栏、欢迎页、设置表单或重复标题框。空白时强调输入；有结果时强调
译文；错误时给出可执行下一步，例如授权、配置服务、安装系统语言包或重试。

### 2. 输入翻译工作台

工作台是长文本、provider 对比和设置的稳定窗口。输入翻译页面至少包含：

- 源语言（含自动检测）和目标语言（含自动匹配）
- 语言交换动作及不可交换时的正确 disabled 状态
- 可输入/粘贴的原文区域、字符数、清空和翻译动作
- loading、streaming、成功、单服务失败、部分服务失败、全部失败状态
- 当前服务和多服务切换/比较
- 复制译文、已复制反馈、重试和“配置服务”入口
- 系统语言包缺失、网络错误、不支持语言对等可操作错误

主布局在正常桌面宽度可以双栏；窄窗口必须自然堆叠，不得依赖横向滚动或出现 overflow。
这不是 source/result 两张空白大卡片的机械复刻，执行者可以重新设计信息密度与层级。

### 3. 首次运行

首次运行是工作台内部的一条 route，不是第二个原生窗口。目标是完成配置，而不是展示
营销介绍。建议按以下任务呈现简短 checklist 或 step flow：

1. 解释并检查 macOS 辅助功能与屏幕录制权限；Windows 显示“无需授权”。
2. 展示或设置核心快捷键，并即时显示冲突。
3. 确认可用翻译/OCR 服务；没有服务时引导到配置。
4. 完成后进入输入翻译；允许跳过非阻塞步骤，之后可在设置中恢复。

所有权限状态必须来自实时 controller；从系统设置返回后一次 focus/resume 或“重新检查”
即可更新。已经授权时不得继续显示“未授权”。

### 4. 设置

设置面向任务组织，不沿用旧项目的三栏 deck。首版必须能找到：

- 常规：开机启动、菜单栏显示、界面语言、浅色/深色/跟随系统
- 服务：翻译与 OCR 服务、默认服务、启用状态、常用语言、自动目标
- Providers：添加、编辑、删除、endpoint/model 和密钥状态
- 快捷键：四个首版动作、录制、冲突原因、清除、恢复默认
- 权限：辅助功能和屏幕录制的实时状态、解释、授权/打开系统设置、重新检查
- 关于：版本、平台、开源协议、版权和第三方声明

服务和 provider 可以在信息架构中合并为一个上层目的地，但必须保持概念清晰：provider
表示“如何连接”，service 表示“哪种能力/模型被使用”。API 密钥始终遮蔽，保存后不把
明文重新填回 UI。

## 导航和窗口

- 工作台默认最小尺寸保持当前可支持的 `840 × 560`，设计必须在该尺寸无 overflow；
  同时为更宽和接近最小尺寸提供响应式方案。
- 快捷窗初始宽度约 `396`，高度由内容决定，最大约 `800`。不要把工作台缩进快捷窗。
- 桌面宽度使用简洁的 Material NavigationRail、NavigationDrawer 或等价官方组件；
  只有“翻译”和“设置”等真正的一级任务进入全局导航。
- 设置内的二级导航可以使用 rail/list/detail，但在较窄内容区应折叠为单列。
- macOS 和 Windows 每个平台只能有一种窗口 chrome 策略。不得同时显示原生标题栏与
  自绘标题栏，不得出现两套关闭按钮、两层边框或额外黑色宿主窗口。
- 继续使用现有 surface 互斥和窗口定位服务；视觉层不直接调用 `nativeapi`。

## 组件与视觉规则

- 优先使用官方 `NavigationRail`/`NavigationDrawer`、`DropdownMenu`、`TextField`、
  `FilledButton`、`OutlinedButton`、`IconButton`、`Card`、`ListTile`、`SwitchListTile`、
  `Dialog`、`SnackBar`、`ProgressIndicator` 和 `SegmentedButton`。
- 只有 Material 3 缺失且在多个页面重复的产品模式，才在 `packages/ui_flutter` 包装；
  包装必须是薄层，不得重新实现交互语义。
- 交互目标通常不小于 40–44 logical pixels；完整键盘导航、可见 focus、tooltip 和
  semantics label 必须存在。
- 间距采用一致的 4/8 基准；圆角、阴影和边框由主题统一决定，页面不得各写一套。
- 浅色/深色均满足文本与操作对比度；错误不能只靠颜色传达。
- 动画仅用于 surface、展开和流式状态，建议 120–200ms；尊重 reduced motion。
- 中文、英文、长德文/法文等本地化文案不得截断关键动作。

## 每个 surface 必须设计的状态

| Surface | 必备状态 |
| --- | --- |
| 工作台翻译 | empty、typing、catalog loading、translating、streaming、success、multiple providers、partial failure、all failure、no services、language pack missing、long text |
| 快捷翻译 | empty、prefilled selection、OCR result、translating、success、multiple providers、permission denied、capture cancelled、service error、pinned、long result |
| 首次运行/权限 | checking、granted、denied、not required、unknown、shortcut conflict、no provider、ready |
| Providers/服务 | loading、empty、list、add/edit validation、saving、test success、test failure、delete confirm、secret already stored |
| 快捷键 | registered、unregistered、recording、invalid、local duplicate、OS conflict、reset confirm |
| 通用 | offline/network error、unexpected error、disabled action、keyboard focus、light/dark、中文/英文、macOS/Windows |

Widgetbook 中必须先出现这些纯 UI 状态。它们使用 fixture/fake，不初始化 Rust、不读取
安全存储、不要求真实网络或系统权限。

## 文案规则

- 所有产品文案写入 `apps/desktop/flutter/lib/src/i18n/*.i18n.json`，禁止在 widget 中
  硬编码中英文。
- 错误文案包含“发生了什么”和“下一步”，不直接展示 Rust exception、provider 原始响应、
  文件路径或密钥。
- “权限未授予”“没有服务”“系统语言包未安装”是不同问题，不得统一显示“操作失败”。
- 功能命名统一使用：输入翻译、快捷翻译、划词翻译、截图 OCR、翻译服务、Provider/提供商。

