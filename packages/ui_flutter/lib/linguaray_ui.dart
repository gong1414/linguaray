/// The LinguaRay design system for Flutter.
///
/// Material 3 is the foundation for every new LinguaRay screen. The
/// [LinguaRayMaterialTheme] projects the brand colors, typography, shapes, and
/// accessible desktop control sizes onto Flutter's maintained components.
///
/// The boundary is deliberate: nothing here knows a LinguaRay
/// concept. The product's own widgets that compose these — the language pair,
/// the glossary mark, the provider row, the floating ball — live in the app, at
/// `apps/desktop/flutter/lib/src/widgets/`.
///
/// The generic legacy atoms remain exported while existing settings and window
/// chrome migrate. Their names can overlap Material widgets, so code touching
/// those surfaces should import this package with an explicit `show` list or a
/// prefix:
///
/// ```dart
/// import 'package:linguaray_ui/linguaray_ui.dart' as ui;
/// ```
///
/// New surfaces establish their theme with:
///
/// ```dart
/// Theme(
///   data: ui.LinguaRayMaterialTheme.light(),
///   child: const TranslationView(),
/// )
/// ```
library;

export 'src/theme/material_theme.dart';
export 'src/theme/text_styles.dart';
export 'src/theme/theme.dart';
export 'src/theme/themes.dart';
export 'src/theme/tokens.dart';
export 'src/widgets/badge.dart';
export 'src/widgets/brand_logo.dart';
export 'src/widgets/browser_frame.dart';
export 'src/widgets/button.dart';
export 'src/widgets/callout.dart';
export 'src/widgets/checkbox.dart';
export 'src/widgets/data_table.dart';
export 'src/widgets/dialog.dart';
export 'src/widgets/empty_state.dart';
export 'src/widgets/field.dart';
export 'src/widgets/focus_ring.dart';
export 'src/widgets/icon_button.dart';
export 'src/widgets/kbd.dart';
export 'src/widgets/label.dart';
export 'src/widgets/menu.dart';
export 'src/widgets/option_card.dart';
export 'src/widgets/popover.dart';
export 'src/widgets/preference.dart';
export 'src/widgets/pressable.dart';
export 'src/widgets/progress.dart';
export 'src/widgets/radio.dart';
export 'src/widgets/search_field.dart';
export 'src/widgets/segmented_control.dart';
export 'src/widgets/shortcut_recorder.dart';
export 'src/widgets/sidebar.dart';
export 'src/widgets/stage.dart';
export 'src/widgets/step_list.dart';
export 'src/widgets/surface.dart';
export 'src/widgets/switch.dart';
export 'src/widgets/tabs.dart';
export 'src/widgets/toast.dart';
export 'src/widgets/window_controls.dart';
export 'src/widgets/window_frame.dart';
