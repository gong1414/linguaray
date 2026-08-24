# LinguaRay UI

The reusable Flutter design system for LinguaRay, published internally as the
`linguaray_ui` package.

Primitives only: no translation, provider, glossary or language-pair concept
reaches this package, and it ships no sample data. The product's own widgets
and tokens live in the app, under `apps/desktop/flutter/lib/src/` — the
[widget map](#widget-map) below names both halves.

## Using it

```dart
import 'package:linguaray_ui/linguaray_ui.dart' as ui;

ui.DesignThemeProvider(
  theme: ui.DesignThemeName.studioLight,
  child: ui.Button(
    variant: ui.ButtonVariant.primary,
    onPressed: () {},
    child: const Text('翻译'),
  ),
);
```

Some widget names (`Badge`, `Button`, `Checkbox`, `Dialog`, `Divider`, `Radio`,
`Step`, `Switch`) are also Material's. This package never imports Material, so
nothing clashes internally — but an app that uses both should import one of them
with a prefix, as above.

## Tokens

`lib/src/theme/` splits into the `DesignTokens` defaults (`metrics` and the
type scale) and one entry per palette in `themes.dart`
(`DesignThemes.studioLight` and friends).

**A theme may change exactly three families: colour, radii and shadows.** The
structural constants — sidebar and titlebar sizes, the 11/12/13/15/17 type
scale, the hairline unit — are the same in all four. Every field defaults to the
**Studio Light** value, and each theme overrides only what it changes, so a
theme reads as "the baseline, plus these overrides":

```dart
DesignThemes.studioDark; // brightness, gradients, colours, shadows
DesignThemes.brightLight; // + pill radii
```

Reach for tokens through the context extension:

```dart
context.tokens;   // the whole set
context.colors;   // colours
context.radii;    // corner radii
context.metrics;  // sidebar / titlebar / window sizes
context.typography;
context.shadows;
context.themeName;     // which palette is active
context.hairlineWidth; // one device pixel
```

Type recipes come off `DesignTypography`: `sansStyle`, `displayStyle`,
`cjkStyle`, `monoStyle`, `labelStyle`, `numericStyle`.

Two unrelated things are called hairline: `context.hairlineWidth` is the
separator's *width*, while its colour comes from `colors.hairline` /
`hairlineStrong` / `hairlineSoft` (plus `accentHairline`, `warnHairline`,
`dangerHairline`).

## Widget map

The atoms this package ships:

`Badge`, `Button`, `Callout`, `Checkbox`, `Radio` (+ `RadioList`), `Switch`,
`SegmentedControl`, `Tabs` (+ `TabItem`), `OptionCard`, `Kbd`, `Label`,
`Surface` / `Divider`, `Dialog` / `DialogHeader` / `DialogBody` /
`DialogFooter`, `EmptyState`, `Field` / `Input` / `TextArea` / `Select` /
`FieldValue`, `SearchField`, `ProgressBar` / `Meter` / `Spinner`, `Step` /
`StepList`, `DataTable` family, `PopoverWindow` / `PopoverPanel`,
`BrowserFrame`, `FloatingToolbar` / `ToolbarSeparator` (in-page overlay bar),
`WindowFrame` / `WindowTitlebar` / `WindowBody` / `WindowMain` /
`WindowContent` / `WindowFooter` / `TrafficLights`, `Sidebar` /
`SidebarGroup` / `SidebarCard` / `NavItem` / `Rail` / `RailItem` / `Aside`,
`Stage` / `ActionBar`, and the interaction primitives `Pressable`,
`HoverRegion`, `FocusRing`.

### In the app, not here

The product's own widgets, for when you are looking one up and land here first.
They live in `apps/desktop/flutter/lib/src/widgets/`, and take generic slots the
same way the atoms do — the product words go in at the call site:

`Avatar` (colour + glyph; no provider table), `ListTile`, `ListCard`, `Mark`,
`TextBlock`, `HighlightBlock`, `TitledCard`, `InfoCard`, `DetailBlock`,
`SettingRow`, `Stat` / `SegmentGauge`, `Thumbnail`, `SwapPair`.

Their tokens moved with them, into
`apps/desktop/flutter/lib/src/theme/product_tokens.dart`. Provider brand
colours, the marker on a preferred translation, and the source / translation
type recipes are all product concepts. They layer on top of the tokens here and
are reached the same way, `context.product` beside `context.tokens`.

## Gallery

The desktop app owns the live component gallery. Run its Widgetbook entry point
so component work uses the same fonts, localization and desktop runner as the
product:

```bash
cd ../../apps/desktop/flutter
flutter run -d macos -t lib/widgetbook.dart
```

## Tests

`test/widget_metrics_test.dart` asserts the fixed geometry — the numbers a
font's own line box would otherwise drift off (control heights, the switch box,
the sidebar group's gaps).

`test/golden_test.dart` renders each block on its own at DPR 1 into
`test/goldens/<block>.png`, ~10 KB each, so a regression names the block it
broke and the image is small enough to read. It loads the real faces — SF,
PingFang SC, Apple Symbols and the Fluent icon font — so the images show the
typography, not a wall of placeholder boxes. A host without those faces skips
the suite rather than reporting false diffs, which makes this a macOS-local
guard; the metrics test above is the part that holds everywhere. Refresh after
a deliberate visual change:

```bash
flutter test --update-goldens
```

Both have twins on the app's side of the boundary, covering the widgets that
live there: `test/business_widget_golden_test.dart` and
`test/design_widget_alignment_test.dart`.

## Design notes

- **Hairlines.** `context.hairlineWidth` resolves to half a logical pixel on
  Retina so a separator is one device pixel.
- **Line height.** Every recipe sets
  `leadingDistribution: TextLeadingDistribution.even`; without it a
  tight-leading chip sits taller than its design.
- **Focus.** `FocusRing` is a 3px stroke just outside the box following its
  corner radius, shown for keyboard focus only.
- **Selection.** `--selection` resolves to the accent while the window is key;
  `WindowFrame(unfocused: true)` swaps in the unemphasized pair by re-scoping
  the tokens, so every row inside picks it up without threading state down.
