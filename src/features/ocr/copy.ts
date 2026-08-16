import type { Locale } from "../../app/i18n";

/** OCR overlay copy. macOS uses the system picker; this overlay is the
 *  Windows on-demand region-select surface created by `ocr_capture`. */
export type OcrCopy = {
  openImage: string;
  clipboardImage: string;
  hint: string;
  cancel: string;
  recognizing: string;
  errorPrefix: string;
  overlayRole: string;
};

const en: OcrCopy = {
  openImage: "Open image",
  clipboardImage: "Clipboard image",
  hint: "Draw a region, drop a file, or press Esc to cancel",
  cancel: "Cancel (Esc)",
  recognizing: "Recognizing…",
  errorPrefix: "OCR failed",
  overlayRole: "OCR region selection",
};

const zh: OcrCopy = {
  openImage: "打开图片",
  clipboardImage: "剪贴板图片",
  hint: "拖选区域、放入文件，或按 Esc 取消",
  cancel: "取消（Esc）",
  recognizing: "正在识别…",
  errorPrefix: "识别失败",
  overlayRole: "OCR 区域选择",
};

export const OCR_COPY: Record<Locale, OcrCopy> = { en, zh };
