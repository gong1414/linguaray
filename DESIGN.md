---
name: LinguaRay
description: Neutral desktop surfaces for focused translation and text work.
colors:
  primary: "#30343B"
  on-primary: "#FFFFFF"
  primary-container: "#E9EBEF"
  secondary: "#626872"
  warning: "#805B24"
  warning-container: "#FFF0D2"
  error: "#BA1A1A"
  error-container: "#FFDAD6"
  surface: "#F3F4F6"
  surface-container-lowest: "#FFFFFF"
  surface-container-low: "#F3F4F6"
  surface-container: "#ECEEF2"
  surface-container-high: "#E4E7EC"
  surface-container-highest: "#D8DCE3"
  on-surface: "#22252B"
  on-surface-variant: "#626872"
  outline: "#858D99"
  outline-variant: "#DEE1E6"
  field-fill: "#F7F8FA"
  dark-primary: "#E4E7EC"
  dark-on-primary: "#252930"
  dark-primary-container: "#3D434D"
  dark-secondary: "#B3B9C3"
  dark-warning: "#E3C17D"
  dark-warning-container: "#59451D"
  dark-error: "#FFB4AB"
  dark-error-container: "#93000A"
  dark-surface: "#17191D"
  dark-surface-container-lowest: "#202328"
  dark-surface-container-low: "#24272D"
  dark-surface-container: "#2A2E35"
  dark-surface-container-high: "#323740"
  dark-surface-container-highest: "#3B414B"
  dark-on-surface: "#F2F3F5"
  dark-on-surface-variant: "#B3B9C3"
  dark-outline: "#89929F"
  dark-outline-variant: "#3B4049"
typography:
  headline-medium:
    fontSize: "28px"
    fontWeight: 600
    lineHeight: 1.25
    letterSpacing: "-0.3px"
  title-large:
    fontSize: "28px"
    fontWeight: 600
    lineHeight: 1.3
    letterSpacing: "-0.45px"
  title-medium:
    fontSize: "15px"
    fontWeight: 600
    lineHeight: 1.4
    letterSpacing: "0px"
  body:
    fontSize: "14px"
    lineHeight: 1.5
    letterSpacing: "0px"
  body-small:
    fontSize: "12px"
    lineHeight: 1.5
    letterSpacing: "0px"
  label-small:
    fontSize: "11px"
    lineHeight: 1.4
    letterSpacing: "0.1px"
  label-medium:
    fontSize: "12px"
    lineHeight: 1.4
    letterSpacing: "0px"
  label-large:
    fontSize: "13px"
    fontWeight: 600
    letterSpacing: "0px"
  translation-source:
    fontSize: "17px"
    lineHeight: 1.65
  translation-result:
    fontSize: "18px"
    lineHeight: 1.65
  ocr-editor:
    fontSize: "18px"
    lineHeight: 1.7
rounded:
  small-control: "6px"
  control: "8px"
  sheet: "12px"
  dialog: "16px"
spacing:
  tight: "4px"
  label-gap: "6px"
  small: "8px"
  medium: "12px"
  content: "16px"
  row: "20px"
  section: "24px"
  page: "28px"
components:
  button-primary:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.on-primary}"
    typography: "{typography.label-large}"
    rounded: "{rounded.control}"
    padding: "0px 14px"
  button-primary-dark:
    backgroundColor: "{colors.dark-primary}"
    textColor: "{colors.dark-on-primary}"
    typography: "{typography.label-large}"
    rounded: "{rounded.control}"
    padding: "0px 14px"
  input:
    backgroundColor: "{colors.field-fill}"
    rounded: "{rounded.control}"
    padding: "10px 12px"
  search:
    backgroundColor: "{colors.surface-container-low}"
    typography: "{typography.body}"
    rounded: "{rounded.control}"
    height: "40px"
    padding: "0px 12px"
  settings-destination-selected:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.on-primary}"
    rounded: "{rounded.control}"
    padding: "8px 10px"
  card:
    backgroundColor: "{colors.surface-container-low}"
    rounded: "{rounded.sheet}"
  reading-sheet:
    backgroundColor: "{colors.surface-container-lowest}"
    rounded: "{rounded.sheet}"
    padding: "16px"
  dialog:
    backgroundColor: "{colors.surface-container-lowest}"
    rounded: "{rounded.dialog}"
---

# Design System: LinguaRay

## Overview

**Creative North Star: "A quiet desktop reading instrument"**

White, cool gray and graphite keep the controls legible and let translated text
carry the page. Light and dark themes share the same hierarchy: a subdued window
canvas, a distinct reading surface and strong neutral actions. The canonical
LinguaRay mark remains an identity asset; its colors do not become interface accents.

This is a native Flutter Material 3 system. The frontmatter records implemented
tokens from `packages/ui_flutter/lib/src/theme/material_theme.dart` and the shared
desktop components. Values written as `px` are Flutter logical pixels, not CSS or
physical display pixels. Update this record with the executable source when tokens
change; unspecified Material properties retain their framework defaults.

**Key Characteristics:**

- Neutral action contrast in both themes.
- System typography with distinct display and text families on macOS.
- Flat sections, fine borders and readable text sheets.
- Compact desktop controls with responsive wrapping.

## Colors

**The Semantic Color Rule.** Product widgets consume the active ColorScheme;
reusable palette changes belong in LinguaRayMaterialTheme.

Primary ink marks the main action and selected settings destination. Secondary
ink supports descriptions, metadata and inactive controls. Tertiary colors in the
source scheme support amber warnings; errors use the error roles. These semantic
states do not establish additional decorative accents.

The neutral surface family separates canvas, fields and reading content. Use
`surfaceContainerLowest` for settings pages, reading results and dialogs;
`surfaceContainerLow` provides a quieter source sheet or card. Fine dividers use
`outlineVariant`. The light input fill is a dedicated theme override; dark fields
use `surfaceContainerLow`. Dark tokens are a paired scheme, not color inversion.

## Typography

macOS uses `CupertinoSystemDisplay` for `headlineMedium` and `titleLarge`, and
`CupertinoSystemText` elsewhere, with `PingFang SC`, `Helvetica Neue` and
`sans-serif` fallbacks. Windows uses `Segoe UI`, with `Microsoft YaHei UI`,
`Microsoft YaHei` and `sans-serif` fallbacks. There is no web-font dependency.

The large title anchors a settings page; medium titles label sections and rows.
Both body roles share the body token, while smaller body text describes controls.
Label roles distinguish actions, metadata and sidebar group labels. Roles with no
explicit weight or height in the frontmatter inherit their Material TextTheme
values. Dialog titles derive from `titleLarge` with a local size override (20).

**The Reading Role Rule.** Translation source, translation result and OCR editor
sizes are feature-specific overrides of body text, not global body replacements.

## Layout

Settings use `SettingsShellView` and `SettingsPage`. The grouped directory exposes
destinations in one sidebar (208 wide, or 184 when shell width is below 900).
The page insets are 28 on the top and sides and 24 at the bottom. The heading has
a minimum height of 48; actions wrap below it when the available heading width
is below 620. A divider separates the heading and content, with a 16 gap before
and a 24 gap after it. An optional toolbar precedes content by 20.

Open section blocks end with 28 spacing and separate their heading from controls
by 16. `SettingsActionRow` uses 20 vertical padding and moves its action below the
description when its available row width is below 540. These are local layout
thresholds, not screen-size or mobile breakpoints.

Quick translation gives source and result separate sheets. With visible source
and at least 568 available content width they share two equal columns; otherwise
they stack. The sheet gap is 12. OCR uses a toolbar above a full-width editor.
The translation window remains dynamically sized and constrained to the active
display's work area.

**The Density Rule.** `VisualDensity.compact` modifies component constraints.
Button minimum sizes are style inputs, not guaranteed painted or hit-target
dimensions; measure the native widget when verifying target size.

## Elevation & Depth

Pages, cards, search bars, app bars and dialogs are flat (elevation 0), using
tonal separation and borders. Surface tint is transparent. Popup menus retain
Material elevation 4. A dialog shadow color is configured but its zero elevation
does not imply a visible shadow. Floating snackbars use strong tonal contrast.

**The Flat Surface Rule.** Use spacing, surface roles and fine borders to group
content; reserve popup elevation for the implemented transient menu treatment.

## Shapes

Small controls and chips use the small-control radius; fields, filled/outlined
buttons, search, menus and selected sidebar rows use the control radius. Cards
and reading sheets use the sheet radius, and dialogs use the dialog radius.
Default text-button shape remains inherited unless a component overrides it.
Borders are normally one logical pixel; focused fields increase their stroke to
1.25 using primary ink at 75% opacity.

## Components

- **Buttons:** filled actions carry primary contrast. Outlined actions have 12
  horizontal padding; text actions have 10. Filled and outlined buttons declare
  a minimum height of 36, text buttons 32, and icon buttons a minimum 32 square,
  all subject to native density and layout. Preserve Material disabled states.
- **Inputs and search:** persistent floating labels keep fields identifiable.
  Errors use semantic error borders. Search constrains its height to the
  frontmatter value; its border and zero elevation align with ordinary fields.
- **Navigation:** the active destination uses primary fill and on-primary text;
  inactive destinations use secondary ink. Icons are 17, labels stay on one line
  with ellipsis, and the full directory scrolls. Selection has explicit semantics.
- **Chips and switches:** chips use a fine border and primary-container selection
  without a checkmark. Switches use primary tracks when selected and outline-variant
  tracks otherwise; disabled thumbs use reduced on-surface opacity.
- **Sections and records:** open settings groups avoid nested cards. Lists keep
  source, result and their actions aligned; cards remain available where a bounded
  group is part of the feature. The ListTile theme supplies medium titles and
  subdued body subtitles.
- **Reading sheets:** the source sheet recedes and the result sheet provides the
  clearest reading surface. OCR uses an expanding, borderless text field inside its
  editor container. Keep the feature-specific reading roles and editing behavior.
- **Feedback and focus:** the theme uses no splash, hover ink at 3.5%, highlight ink
  at 4.5%, focus primary at 16%, and selection primary at 22%. Individual Material
  controls still resolve their own state overlays. Tooltips wait 400 ms; visible
  scrollbars have thickness 5 and radius 8. Do not infer a global animation timing
  from these values.

## Do's and Don'ts

### Do:

- Do use LinguaRayMaterialTheme and the shared settings frame for new surfaces.
- Do preserve localized labels, focus behavior and loading, empty and error states.
- Do keep the canonical LinguaRay brand assets and native window contract.
- Do verify native layouts in Widgetbook, widget tests and host-specific goldens.
- Do recapture Windows baselines on Windows before claiming Windows visual validation.

### Don't:

- Don't add decorative gradients, colored status stripes or nested settings cards.
- Don't infer rendered hit-target dimensions from minimumSize alone.
- Don't add duplicate title chrome or an additional onboarding window.
- Don't treat a browser detector or successful golden generation as a native visual verdict.

The implementation run reports 92 rendered macOS goldens, 244 passing workspace
tests, a passing `dart run melos run analyze` and a successful macOS debug
production build. Its finish reviewer reported
no further macOS visual fixes. All 14 settings destinations passed layout checks at
780 × 520 and 1000 × 700 in both themes with macOS and Windows theme variants.
Native Windows CI subsequently rendered all 92 Windows goldens with Windows
fonts ([capture run](https://github.com/gong1414/linguaray/actions/runs/34597770235)). The former 84
baselines were replaced and eight dialog baselines added. Representative settings,
translation, OCR and service-dialog captures were visually checked. Full desktop
build and integration results are recorded by the pull request's required CI checks.
