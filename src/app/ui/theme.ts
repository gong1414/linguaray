import { createTheme, type MantineColorsTuple, type MantineThemeOverride } from "@mantine/core";

/**
 * LinguaRay Mantine theme — the SINGLE theme for the React tree.
 *
 * Values are mapped 1:1 from the frozen design tokens
 * (packages/ui/src/styles/tokens.css → design-system token-map.md), so the
 * React windows inherit the approved visual language:
 *  - brand = core indigo scale (light uses 600 #4F46E5, dark uses 400 #818CF8
 *    via primaryShade)
 *  - radius: sm 6 / md 8 / lg 12 (core-radius-*)
 *  - fonts: Inter + Noto Sans SC + platform fallbacks
 * DO NOT invent colors/spacing here — change tokens.css/token-map.md first,
 * then update this mapping (docs/UI-RULES.md rule 6).
 */
const brand: MantineColorsTuple = [
  "#EEF2FF", // 50
  "#E0E7FF", // 100
  "#C7D2FE", // 200
  "#A5B4FC", // 300
  "#818CF8", // 400
  "#6366F1", // 500
  "#4F46E5", // 600 — light primary (--color-brand-default)
  "#4338CA", // 700 — light hover (--color-brand-hover)
  "#3730A3", // 800
  "#312E81", // 900
];

const neutral: MantineColorsTuple = [
  "#F8FAFC", // 50
  "#F1F5F9", // 100
  "#E2E8F0", // 200
  "#CBD5E1", // 300
  "#94A3B8", // 400
  "#64748B", // 500
  "#475569", // 600
  "#334155", // 700
  "#1E293B", // 800
  "#0F172A", // 900
];

const success: MantineColorsTuple = ["#F0FDF4", "#DCFCE7", "#BBF7D0", "#86EFAC", "#4ADE80", "#22C55E", "#16A34A", "#15803D", "#166534", "#14532D"];
const warning: MantineColorsTuple = ["#FFFBEB", "#FEF3C7", "#FDE68A", "#FCD34D", "#FBBF24", "#F59E0B", "#D97706", "#B45309", "#92400E", "#78350F"];
const danger: MantineColorsTuple = ["#FEF2F2", "#FEE2E2", "#FECACA", "#FCA5A5", "#F87171", "#EF4444", "#DC2626", "#B91C1C", "#991B1B", "#7F1D1D"];
const info: MantineColorsTuple = ["#EFF6FF", "#DBEAFE", "#BFDBFE", "#93C5FD", "#60A5FA", "#3B82F6", "#2563EB", "#1D4ED8", "#1E40AF", "#1E3A8A"];

export const linguaTheme: MantineThemeOverride = createTheme({
  colors: { brand, neutral, success, warning, danger, info },
  primaryColor: "brand",
  primaryShade: { light: 6, dark: 4 },
  black: "#0F172A",
  white: "#FFFFFF",
  defaultRadius: "md",
  radius: { xs: "4px", sm: "6px", md: "8px", lg: "12px", xl: "16px" },
  fontFamily:
    '"Inter", "Noto Sans SC", -apple-system, BlinkMacSystemFont, "Segoe UI", ' +
    '"Helvetica Neue", "PingFang SC", "Microsoft YaHei", sans-serif',
  fontFamilyMonospace:
    '"IBM Plex Mono", "SF Mono", "Cascadia Code", Consolas, "Noto Sans Mono CJK SC", monospace',
  headings: {
    fontWeight: "600",
    sizes: {
      h1: { fontSize: "24px", lineHeight: "1.33", fontWeight: "700" },
      h2: { fontSize: "20px", lineHeight: "1.40", fontWeight: "600" },
      h3: { fontSize: "16px", lineHeight: "1.50", fontWeight: "600" },
      h4: { fontSize: "14px", lineHeight: "1.43", fontWeight: "600" },
    },
  },
  components: {
    Button: { defaultProps: { radius: "md" } },
    ActionIcon: { defaultProps: { radius: "md" } },
    TextInput: { defaultProps: { radius: "md" } },
    PasswordInput: { defaultProps: { radius: "md" } },
    Textarea: { defaultProps: { radius: "md" } },
    Select: { defaultProps: { radius: "md" } },
    Switch: { defaultProps: { radius: "md" } },
    Modal: { defaultProps: { radius: "lg" } },
    Alert: { defaultProps: { radius: "md" } },
    Card: { defaultProps: { radius: "md", padding: "md" } },
    Paper: { defaultProps: { radius: "md" } },
    Tooltip: { defaultProps: { radius: "sm", withArrow: true } },
    Notifications: { defaultProps: { radius: "md" } },
  },
});

export default linguaTheme;
