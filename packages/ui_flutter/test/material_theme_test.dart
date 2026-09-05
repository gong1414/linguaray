import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_ui/linguaray_ui.dart';

void main() {
  test('theme follows the desktop host when no platform is supplied', () {
    addTearDown(() => debugDefaultTargetPlatformOverride = null);
    for (final target in [TargetPlatform.macOS, TargetPlatform.windows]) {
      debugDefaultTargetPlatformOverride = target;
      final theme = LinguaRayMaterialTheme.light();
      expect(theme.platform, target);
      expect(
        theme.textTheme.bodyMedium?.fontFamily,
        target == TargetPlatform.macOS ? 'CupertinoSystemText' : 'Segoe UI',
      );
    }
  });

  test('material theme projects the canonical brand palette', () {
    final light = LinguaRayMaterialTheme.light();
    final dark = LinguaRayMaterialTheme.dark();

    expect(light.colorScheme.primary, LinguaRayPalette.linguaBlue);
    expect(light.colorScheme.secondary, LinguaRayPalette.rayTeal);
    expect(light.scaffoldBackgroundColor, LinguaRayPalette.paper);
    expect(light.colorScheme.onSurface, LinguaRayPalette.graphite);
    expect(dark.colorScheme.secondary, LinguaRayPalette.rayTealDark);
    expect(dark.colorScheme.inversePrimary, LinguaRayPalette.linguaBlue);
  });

  test('core foreground pairs meet desktop contrast requirements', () {
    for (final theme in [
      LinguaRayMaterialTheme.light(),
      LinguaRayMaterialTheme.dark(),
    ]) {
      final scheme = theme.colorScheme;
      expect(_contrast(scheme.onSurface, scheme.surface), greaterThan(7));
      expect(
        _contrast(scheme.onSurfaceVariant, scheme.surfaceContainerLowest),
        greaterThan(4.5),
      );
      expect(_contrast(scheme.onPrimary, scheme.primary), greaterThan(4.5));
      expect(
        _contrast(scheme.onSecondaryContainer, scheme.secondaryContainer),
        greaterThan(4.5),
      );
    }
  });
}

double _contrast(Color a, Color b) {
  final lighter = a.computeLuminance() > b.computeLuminance() ? a : b;
  final darker = identical(lighter, a) ? b : a;
  return (lighter.computeLuminance() + 0.05) /
      (darker.computeLuminance() + 0.05);
}
