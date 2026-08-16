import { makeStyles, tokens } from "@fluentui/react-components";

/**
 * Shared layout styles for LinguaRay views.
 *
 * Visual controls, interaction states, focus rings, typography and colors are
 * owned by Fluent UI. These classes only compose those controls into the
 * desktop window layouts used by Ueli's renderer pattern.
 */
export const useUiStyles = makeStyles({
  page: {
    display: "flex",
    flexDirection: "column",
    gap: tokens.spacingVerticalM,
    minWidth: 0,
  },
  windowPage: {
    boxSizing: "border-box",
    height: "100vh",
    overflowY: "auto",
    display: "flex",
    flexDirection: "column",
    gap: tokens.spacingVerticalS,
    padding: tokens.spacingHorizontalM,
  },
  stack: {
    display: "flex",
    flexDirection: "column",
    gap: tokens.spacingVerticalS,
    minWidth: 0,
  },
  stackTight: {
    display: "flex",
    flexDirection: "column",
    gap: tokens.spacingVerticalXXS,
    minWidth: 0,
  },
  row: {
    display: "flex",
    alignItems: "center",
    gap: tokens.spacingHorizontalS,
    minWidth: 0,
  },
  rowWrap: {
    display: "flex",
    alignItems: "center",
    flexWrap: "wrap",
    gap: tokens.spacingHorizontalS,
    minWidth: 0,
  },
  rowBetween: {
    display: "flex",
    alignItems: "flex-start",
    justifyContent: "space-between",
    gap: tokens.spacingHorizontalM,
    minWidth: 0,
  },
  end: {
    display: "flex",
    justifyContent: "flex-end",
    alignItems: "center",
    flexWrap: "wrap",
    gap: tokens.spacingHorizontalS,
  },
  grow: {
    flex: "1 1 12rem",
    minWidth: 0,
  },
  fieldSmall: {
    width: "9rem",
  },
  fieldTiny: {
    width: "6rem",
  },
  card: {
    minWidth: 0,
  },
  selectedCard: {
    minWidth: 0,
    backgroundColor: tokens.colorNeutralBackground1Selected,
    border: `${tokens.strokeWidthThin} solid ${tokens.colorBrandStroke1}`,
  },
  empty: {
    minHeight: "8rem",
    display: "flex",
    flexDirection: "column",
    alignItems: "center",
    justifyContent: "center",
    gap: tokens.spacingVerticalXS,
    textAlign: "center",
    color: tokens.colorNeutralForeground2,
  },
  muted: {
    color: tokens.colorNeutralForeground2,
  },
  danger: {
    color: tokens.colorPaletteRedForeground1,
  },
  warning: {
    color: tokens.colorPaletteDarkOrangeForeground1,
  },
  success: {
    color: tokens.colorPaletteGreenForeground1,
  },
  title: {
    margin: 0,
  },
  preWrap: {
    whiteSpace: "pre-wrap",
    overflowWrap: "anywhere",
    margin: 0,
  },
  clamp: {
    minWidth: 0,
    overflow: "hidden",
    textOverflow: "ellipsis",
    whiteSpace: "nowrap",
  },
  monospace: {
    fontFamily: '"SF Mono", "Cascadia Code", Consolas, monospace',
  },
  code: {
    display: "block",
    flex: "1 1 16rem",
    minWidth: 0,
    padding: tokens.spacingVerticalXS,
    borderRadius: tokens.borderRadiusMedium,
    backgroundColor: tokens.colorNeutralBackground3,
    fontFamily: '"SF Mono", "Cascadia Code", Consolas, monospace',
    overflowWrap: "anywhere",
    userSelect: "all",
  },
  twoColumn: {
    display: "grid",
    gridTemplateColumns: "minmax(260px, 340px) minmax(0, 1fr)",
    alignItems: "start",
    gap: tokens.spacingHorizontalM,
    minWidth: 0,
    "@media (max-width: 700px)": {
      gridTemplateColumns: "minmax(0, 1fr)",
    },
  },
  list: {
    display: "flex",
    flexDirection: "column",
    gap: tokens.spacingVerticalXS,
    minWidth: 0,
  },
  dividerSpace: {
    marginTop: tokens.spacingVerticalXS,
    marginBottom: tokens.spacingVerticalXS,
  },
  dialogActions: {
    display: "flex",
    justifyContent: "flex-end",
    gap: tokens.spacingHorizontalS,
  },
  iconButtonText: {
    fontSize: tokens.fontSizeBase100,
  },
});
