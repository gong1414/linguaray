/** Shared structural class names; visual tokens come from Ant Design. */
const styles = {
  page: "lr-page",
  windowPage: "lr-window-page",
  stack: "lr-stack",
  stackTight: "lr-stack-tight",
  row: "lr-row",
  rowWrap: "lr-row-wrap",
  rowBetween: "lr-row-between",
  end: "lr-end",
  grow: "lr-grow",
  fieldSmall: "lr-field-small",
  fieldTiny: "lr-field-tiny",
  card: "lr-card",
  selectedCard: "lr-selected-card",
  empty: "lr-empty",
  muted: "lr-muted",
  danger: "lr-danger",
  warning: "lr-warning",
  success: "lr-success",
  title: "lr-title",
  preWrap: "lr-pre-wrap",
  clamp: "lr-clamp",
  monospace: "lr-monospace",
  code: "lr-code",
  twoColumn: "lr-two-column",
  list: "lr-list",
  dividerSpace: "lr-divider-space",
  dialogActions: "lr-dialog-actions",
  iconButtonText: "lr-icon-button-text",
} as const;

export function useUiStyles() {
  return styles;
}
