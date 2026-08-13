// @linguaray/ui — LinguaRay design system primitives (MASTER-frozen).
// Component re-exports (SolidJS).
export { default as Button } from "./components/Button";
export type { ButtonProps, ButtonVariant, ButtonSize } from "./components/Button";

export { default as IconButton } from "./components/IconButton";
export type {
  IconButtonProps,
  IconButtonVariant,
  IconButtonSize,
} from "./components/IconButton";

export { default as Spinner } from "./components/Spinner";
export type { SpinnerProps } from "./components/Spinner";

export { default as TextField } from "./components/TextField";
export type { TextFieldProps, TextFieldSize } from "./components/TextField";

export { default as TextArea } from "./components/TextArea";
export type { TextAreaProps } from "./components/TextArea";

export { default as ResultCard } from "./components/ResultCard";
export type {
  ResultCardProps,
  ResultAction,
  ResultOutcome,
} from "./components/ResultCard";

export { default as VisuallyHidden } from "./components/VisuallyHidden";

export { default as Select } from "./components/Select";
export type { SelectProps, SelectOption } from "./components/Select";

export { default as Switch } from "./components/Switch";
export type { SwitchProps } from "./components/Switch";

export { default as Dialog } from "./components/Dialog";
export type { DialogProps } from "./components/Dialog";

export { default as Confirm } from "./components/Confirm";
export type { ConfirmProps, ConfirmVariant } from "./components/Confirm";

export { default as Banner } from "./components/Banner";
export type { BannerProps, BannerVariant } from "./components/Banner";

export { default as Toast } from "./components/Toast";
export type { ToastProps, ToastVariant } from "./components/Toast";

export { default as Tooltip } from "./components/Tooltip";
export type { TooltipProps, TooltipSide } from "./components/Tooltip";

export { default as EmptyState } from "./components/EmptyState";
export type { EmptyStateProps } from "./components/EmptyState";

export { default as ProviderCard } from "./components/ProviderCard";
export type {
  ProviderCardProps,
  ProviderRole,
  ProviderProfile,
  ProviderCardLabels,
} from "./components/ProviderCard";
export { defaultProviderCardLabels } from "./components/ProviderCard";

// R1 新增组件导出
export { default as SegmentedControl } from "./components/SegmentedControl";
export type { SegmentedControlProps, SegmentedOption } from "./components/SegmentedControl";

export { default as ShortcutChip } from "./components/ShortcutChip";
export type { ShortcutChipProps, ShortcutChipLabels } from "./components/ShortcutChip";

export { default as StatusBadge } from "./components/StatusBadge";
export type { StatusBadgeProps, StatusBadgeVariant } from "./components/StatusBadge";

export { default as InlineError } from "./components/InlineError";
export type { InlineErrorProps } from "./components/InlineError";

export { default as WindowChrome } from "./components/WindowChrome";
export type { WindowChromeProps, WindowChromeLabels } from "./components/WindowChrome";

export { default as SidebarItem } from "./components/SidebarItem";
export type { SidebarItemProps } from "./components/SidebarItem";

export { default as HistoryRow } from "./components/HistoryRow";
export type { HistoryRowProps, HistoryRowLabels } from "./components/HistoryRow";

export { default as ProviderRow } from "./components/ProviderRow";
export type { ProviderRowProps, ProviderRowLabels } from "./components/ProviderRow";

export { default as TranslationCard } from "./components/TranslationCard";
export type {
  TranslationCardProps,
  TranslationCardLabels,
  TranslationState,
} from "./components/TranslationCard";

// 共享逻辑
export { providerStatus } from "./components/providerPresentation";
export type { ProviderStatus } from "./components/providerTypes";
