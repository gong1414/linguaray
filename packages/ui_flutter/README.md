# LinguaRay UI

The internal Flutter package contains the single desktop Material theme and the
canonical `BrandLogo`. Product views live in `apps/desktop/flutter/lib/src/ui/`.

```dart
import 'package:flutter/material.dart';
import 'package:linguaray_ui/linguaray_ui.dart';

MaterialApp(
  theme: LinguaRayMaterialTheme.light(),
  darkTheme: LinguaRayMaterialTheme.dark(),
  home: const MyScreen(),
);
```

Use Material controls and `Theme.of(context)` for colors and typography. The
light theme uses white content, warm gray group surfaces, graphite text, and
orange actions. Dark mode maps the same roles to dark surfaces. SF/PingFang on
macOS and Segoe UI/Microsoft YaHei on Windows follow the desktop host.

Settings pages use the shared `SettingsPage` in the desktop app: 32 point page
insets, a 26 point title, 24 points before content, and a consistent action row.
Controls use 8 point corners, 36 point buttons, and flat 40 point search fields.
Notifications, dialogs, and menus derive their styles from the same theme.
The logo follows `assets/brand/linguaray/`; its canonical colors are independent
of application control colors.

Run `flutter run -d macos -t lib/widgetbook.dart` in the desktop app to inspect
real product views. `test/catalog_surface_golden_test.dart` covers those views
in light and dark mode with native macOS and Windows font baselines. Refresh
only on the matching host and inspect each changed image before accepting it.
The UI package's tests verify theme contrast and platform font selection.
