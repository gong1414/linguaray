# Surface 13: Text-to-Speech

**Surface ID:** `surface.text-to-speech`
**Penpot 页面:** 40 OCR & Media
**Penpot 画板尺寸:** [待查询]
**生产窗口默认尺寸:** 设置子页
**生产窗口最小尺寸:** 同设置（600×400）
**尺寸冲突说明:** 以生产窗口尺寸为实施标准

## 状态矩阵

| 状态 | 描述 | 组件 |
|---|---|---|
| Idle | 扬声器图标（点击朗读） | IconButton (Volume2) |
| Speaking | 动画扬声器图标 + "停止" | IconButton (animated Square) + Button (stop) |
| Error | "TTS 错误：{message}"（例如无可用语音） | Inline error |
| No voices | "未找到系统语音"（罕见） | EmptyState / Inline error |

## 本地化 copy key

| key | en | zh |
|---|---|---|
| `tts.title` | Text-to-Speech | 文本转语音 |
| `tts.action.speak` | Speak | 朗读 |
| `tts.action.stop` | Stop | 停止 |
| `tts.voice` | Voice | 语音 |
| `tts.speed` | Speed | 语速 |
| `tts.error` | TTS error: {message} | TTS 错误：{message} |
| `tts.noVoices` | No system voices found | 未找到系统语音 |
| `tts.queue.title` | Queue | 队列 |
| `tts.offline.notice` | System offline voices only | 仅系统离线语音 |

## 组件组合

- **触发（在翻译结果卡中）：** IconButton (Volume2) / IconButton (Square, 停止)
- **设置区：**
  - Select (voice) — 系统离线语音列表
  - Select/Slider (speed) — 语速
- **播放控制：** Button (speak) / Button (stop)
- **队列：** 队列列表（如支持）
- **离线状态：** 说明文本（仅系统离线语音）
- **错误：** Inline error / EmptyState（无语音）

## 页面特有约束

- 设置子页，遵循设置窗口尺寸与自适应规则。
- 仅系统离线语音（macOS `AVSpeechSynthesizer` / Windows `SpeechSynthesizer`）。
- 队列管理：`tts_speak(text, voice_id)` / `tts_stop()` / `tts_list_voices()`。
- TTS 控制也出现在 Selection Popup（Surface 01）和 Input Window（Surface 02）的结果卡中。
