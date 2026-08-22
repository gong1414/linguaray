import 'package:flutter/material.dart';

/// Canonical LinguaRay brand colors. These are the only named product hues
/// used by the production Material 3 theme.
abstract final class LinguaRayPalette {
  static const Color linguaBlue = Color(0xFF2859D9);
  static const Color rayTeal = Color(0xFF18A6A6);
  static const Color rayTealDark = Color(0xFF34C0BE);
  static const Color navy = Color(0xFF13233F);
  static const Color graphite = Color(0xFF172033);
  static const Color paper = Color(0xFFF7F9FC);
}

@immutable
final class LinguaRayBrandColors extends ThemeExtension<LinguaRayBrandColors> {
  const LinguaRayBrandColors({
    required this.navy,
    required this.ray,
    required this.canvas,
    required this.ink,
    required this.resultRule,
  });

  static const light = LinguaRayBrandColors(
    navy: LinguaRayPalette.navy,
    ray: LinguaRayPalette.rayTeal,
    canvas: LinguaRayPalette.paper,
    ink: LinguaRayPalette.graphite,
    resultRule: LinguaRayPalette.rayTeal,
  );

  static const dark = LinguaRayBrandColors(
    navy: LinguaRayPalette.navy,
    ray: LinguaRayPalette.rayTealDark,
    canvas: LinguaRayPalette.navy,
    ink: Color(0xFFE8EEF7),
    resultRule: LinguaRayPalette.rayTealDark,
  );

  final Color navy;
  final Color ray;
  final Color canvas;
  final Color ink;
  final Color resultRule;

  @override
  LinguaRayBrandColors copyWith({
    Color? navy,
    Color? ray,
    Color? canvas,
    Color? ink,
    Color? resultRule,
  }) {
    return LinguaRayBrandColors(
      navy: navy ?? this.navy,
      ray: ray ?? this.ray,
      canvas: canvas ?? this.canvas,
      ink: ink ?? this.ink,
      resultRule: resultRule ?? this.resultRule,
    );
  }

  @override
  LinguaRayBrandColors lerp(covariant LinguaRayBrandColors? other, double t) {
    if (other == null) return this;
    return LinguaRayBrandColors(
      navy: Color.lerp(navy, other.navy, t)!,
      ray: Color.lerp(ray, other.ray, t)!,
      canvas: Color.lerp(canvas, other.canvas, t)!,
      ink: Color.lerp(ink, other.ink, t)!,
      resultRule: Color.lerp(resultRule, other.resultRule, t)!,
    );
  }
}

@immutable
final class LinguaRayMetrics extends ThemeExtension<LinguaRayMetrics> {
  const LinguaRayMetrics({
    this.space = 8,
    this.commandBarHeight = 48,
    this.settingsNavWidth = 240,
    this.quickWidth = 420,
    this.controlHeight = 36,
    this.captionHeight = 32,
    this.macTrafficInset = 78,
    this.workbenchMinSize = const Size(840, 560),
  });

  static const standard = LinguaRayMetrics();

  final double space;
  final double commandBarHeight;
  final double settingsNavWidth;
  final double quickWidth;
  final double controlHeight;
  final double captionHeight;
  final double macTrafficInset;
  final Size workbenchMinSize;

  @override
  LinguaRayMetrics copyWith({
    double? space,
    double? commandBarHeight,
    double? settingsNavWidth,
    double? quickWidth,
    double? controlHeight,
    double? captionHeight,
    double? macTrafficInset,
    Size? workbenchMinSize,
  }) {
    return LinguaRayMetrics(
      space: space ?? this.space,
      commandBarHeight: commandBarHeight ?? this.commandBarHeight,
      settingsNavWidth: settingsNavWidth ?? this.settingsNavWidth,
      quickWidth: quickWidth ?? this.quickWidth,
      controlHeight: controlHeight ?? this.controlHeight,
      captionHeight: captionHeight ?? this.captionHeight,
      macTrafficInset: macTrafficInset ?? this.macTrafficInset,
      workbenchMinSize: workbenchMinSize ?? this.workbenchMinSize,
    );
  }

  @override
  LinguaRayMetrics lerp(covariant LinguaRayMetrics? other, double t) {
    if (other == null) return this;
    return LinguaRayMetrics(
      space: lerpDouble(space, other.space, t)!,
      commandBarHeight: lerpDouble(
        commandBarHeight,
        other.commandBarHeight,
        t,
      )!,
      settingsNavWidth: lerpDouble(
        settingsNavWidth,
        other.settingsNavWidth,
        t,
      )!,
      quickWidth: lerpDouble(quickWidth, other.quickWidth, t)!,
      controlHeight: lerpDouble(controlHeight, other.controlHeight, t)!,
      captionHeight: lerpDouble(captionHeight, other.captionHeight, t)!,
      macTrafficInset: lerpDouble(macTrafficInset, other.macTrafficInset, t)!,
      workbenchMinSize: Size.lerp(workbenchMinSize, other.workbenchMinSize, t)!,
    );
  }
}

extension LinguaRayThemeContext on BuildContext {
  LinguaRayBrandColors get brandColors =>
      Theme.of(this).extension<LinguaRayBrandColors>() ??
      (Theme.of(this).brightness == Brightness.dark
          ? LinguaRayBrandColors.dark
          : LinguaRayBrandColors.light);

  LinguaRayMetrics get metrics =>
      Theme.of(this).extension<LinguaRayMetrics>() ?? LinguaRayMetrics.standard;
}

abstract final class LinguaRayMaterialTheme {
  static ThemeData light({TargetPlatform? platform}) =>
      _build(Brightness.light, platform);

  static ThemeData dark({TargetPlatform? platform}) =>
      _build(Brightness.dark, platform);

  static ThemeData forBrightness(
    Brightness brightness, {
    TargetPlatform? platform,
  }) => brightness == Brightness.dark
      ? dark(platform: platform)
      : light(platform: platform);

  static ColorScheme _lightScheme() {
    return const ColorScheme(
      brightness: Brightness.light,
      primary: LinguaRayPalette.linguaBlue,
      onPrimary: Color(0xFFFFFFFF),
      primaryContainer: Color(0xFFD9E4FF),
      onPrimaryContainer: Color(0xFF0B1F66),
      secondary: LinguaRayPalette.rayTeal,
      onSecondary: Color(0xFFFFFFFF),
      secondaryContainer: Color(0xFFD3F1F1),
      onSecondaryContainer: Color(0xFF043838),
      tertiary: LinguaRayPalette.navy,
      onTertiary: Color(0xFFFFFFFF),
      tertiaryContainer: Color(0xFFD5DCEA),
      onTertiaryContainer: LinguaRayPalette.navy,
      error: Color(0xFFB3261E),
      onError: Color(0xFFFFFFFF),
      errorContainer: Color(0xFFF9DEDC),
      onErrorContainer: Color(0xFF410E0B),
      surface: LinguaRayPalette.paper,
      onSurface: LinguaRayPalette.graphite,
      surfaceContainerLowest: Color(0xFFFFFFFF),
      surfaceContainerLow: Color(0xFFEFF3F8),
      surfaceContainer: Color(0xFFE7EDF5),
      surfaceContainerHigh: Color(0xFFDFE6F0),
      surfaceContainerHighest: Color(0xFFD6DEEA),
      onSurfaceVariant: Color(0xFF445060),
      outline: Color(0xFF6B7686),
      outlineVariant: Color(0xFFC5CDD8),
      inverseSurface: LinguaRayPalette.navy,
      onInverseSurface: Color(0xFFE8EEF7),
      inversePrimary: Color(0xFFADC2FF),
      scrim: Color(0xFF000000),
      shadow: Color(0xFF000000),
      surfaceTint: Colors.transparent,
    );
  }

  static ColorScheme _darkScheme() {
    return const ColorScheme(
      brightness: Brightness.dark,
      primary: Color(0xFF9BB3FF),
      onPrimary: Color(0xFF0A1A4A),
      primaryContainer: LinguaRayPalette.linguaBlue,
      onPrimaryContainer: Color(0xFFFFFFFF),
      secondary: LinguaRayPalette.rayTealDark,
      onSecondary: Color(0xFF042424),
      secondaryContainer: Color(0xFF0E5C5C),
      onSecondaryContainer: Color(0xFFC8F4F3),
      tertiary: Color(0xFFC5D0E3),
      onTertiary: LinguaRayPalette.navy,
      tertiaryContainer: Color(0xFF243552),
      onTertiaryContainer: Color(0xFFD7E0EE),
      error: Color(0xFFFFB4AB),
      onError: Color(0xFF690005),
      errorContainer: Color(0xFF93000A),
      onErrorContainer: Color(0xFFFFDAD6),
      surface: LinguaRayPalette.navy,
      onSurface: Color(0xFFE8EEF7),
      surfaceContainerLowest: Color(0xFF0D182C),
      surfaceContainerLow: Color(0xFF18253C),
      surfaceContainer: Color(0xFF1C2A44),
      surfaceContainerHigh: Color(0xFF23324E),
      surfaceContainerHighest: Color(0xFF2B3B59),
      onSurfaceVariant: Color(0xFFB7C2D4),
      outline: Color(0xFF8A96A8),
      outlineVariant: Color(0xFF3A4860),
      inverseSurface: Color(0xFFE8EEF7),
      onInverseSurface: LinguaRayPalette.navy,
      inversePrimary: LinguaRayPalette.linguaBlue,
      scrim: Color(0xFF000000),
      shadow: Color(0xFF000000),
      surfaceTint: Colors.transparent,
    );
  }

  static ThemeData _build(Brightness brightness, TargetPlatform? platform) {
    final isDark = brightness == Brightness.dark;
    final scheme = isDark ? _darkScheme() : _lightScheme();
    final brand = isDark
        ? LinguaRayBrandColors.dark
        : LinguaRayBrandColors.light;
    final resolvedPlatform = platform ?? TargetPlatform.macOS;
    final base = ThemeData(
      useMaterial3: true,
      brightness: brightness,
      colorScheme: scheme,
      scaffoldBackgroundColor: scheme.surface,
      visualDensity: VisualDensity.compact,
      splashFactory: InkRipple.splashFactory,
      platform: resolvedPlatform,
      fontFamilyFallback: const [
        'MiSans',
        '.AppleSystemUIFont',
        'Segoe UI',
        'PingFang SC',
        'Microsoft YaHei UI',
      ],
    );

    final textTheme = base.textTheme.copyWith(
      headlineMedium: base.textTheme.headlineMedium?.copyWith(
        fontSize: 22,
        height: 1.25,
        fontWeight: FontWeight.w600,
        letterSpacing: -0.3,
        color: scheme.onSurface,
      ),
      titleLarge: base.textTheme.titleLarge?.copyWith(
        fontSize: 16,
        height: 1.3,
        fontWeight: FontWeight.w600,
        color: scheme.onSurface,
      ),
      titleMedium: base.textTheme.titleMedium?.copyWith(
        fontSize: 13,
        height: 1.3,
        fontWeight: FontWeight.w600,
        color: scheme.onSurface,
      ),
      bodyLarge: base.textTheme.bodyLarge?.copyWith(
        fontSize: 14,
        height: 1.5,
        color: scheme.onSurface,
      ),
      bodyMedium: base.textTheme.bodyMedium?.copyWith(
        fontSize: 13,
        height: 1.45,
        color: scheme.onSurface,
      ),
      labelLarge: base.textTheme.labelLarge?.copyWith(
        fontWeight: FontWeight.w600,
        fontSize: 13,
      ),
    );

    const radius = 8.0;
    final outline = OutlineInputBorder(
      borderRadius: BorderRadius.circular(radius),
      borderSide: BorderSide(color: scheme.outlineVariant),
    );

    return base.copyWith(
      textTheme: textTheme,
      dividerTheme: DividerThemeData(
        color: scheme.outlineVariant,
        thickness: 1,
        space: 1,
      ),
      cardTheme: CardThemeData(
        elevation: 0,
        margin: EdgeInsets.zero,
        color: scheme.surfaceContainerLowest,
        surfaceTintColor: Colors.transparent,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(radius),
          side: BorderSide(color: scheme.outlineVariant),
        ),
      ),
      appBarTheme: AppBarTheme(
        elevation: 0,
        scrolledUnderElevation: 0,
        backgroundColor: scheme.surface,
        foregroundColor: scheme.onSurface,
        surfaceTintColor: Colors.transparent,
        toolbarHeight: 48,
        titleTextStyle: textTheme.titleMedium,
      ),
      navigationBarTheme: NavigationBarThemeData(
        backgroundColor: scheme.surface,
        elevation: 0,
        height: 48,
        indicatorColor: scheme.primaryContainer,
        labelTextStyle: WidgetStateProperty.resolveWith((states) {
          final selected = states.contains(WidgetState.selected);
          return textTheme.labelLarge?.copyWith(
            color: selected ? scheme.primary : scheme.onSurfaceVariant,
          );
        }),
      ),
      navigationRailTheme: NavigationRailThemeData(
        backgroundColor: scheme.surface,
        elevation: 0,
        minWidth: 72,
        groupAlignment: -1,
        labelType: NavigationRailLabelType.all,
        useIndicator: true,
        indicatorColor: scheme.primaryContainer,
        selectedIconTheme: IconThemeData(color: scheme.primary, size: 20),
        unselectedIconTheme: IconThemeData(
          color: scheme.onSurfaceVariant,
          size: 20,
        ),
        selectedLabelTextStyle: textTheme.labelMedium?.copyWith(
          color: scheme.primary,
          fontWeight: FontWeight.w600,
        ),
        unselectedLabelTextStyle: textTheme.labelMedium?.copyWith(
          color: scheme.onSurfaceVariant,
        ),
      ),
      inputDecorationTheme: InputDecorationTheme(
        filled: true,
        fillColor: scheme.surfaceContainerLowest,
        border: outline,
        enabledBorder: outline,
        focusedBorder: outline.copyWith(
          borderSide: BorderSide(color: scheme.primary, width: 2),
        ),
        errorBorder: outline.copyWith(
          borderSide: BorderSide(color: scheme.error),
        ),
        contentPadding: const EdgeInsets.symmetric(
          horizontal: 12,
          vertical: 10,
        ),
        hintStyle: textTheme.bodyMedium?.copyWith(
          color: scheme.onSurfaceVariant,
        ),
      ),
      filledButtonTheme: FilledButtonThemeData(
        style: FilledButton.styleFrom(
          minimumSize: const Size(0, 36),
          padding: const EdgeInsets.symmetric(horizontal: 14),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(radius),
          ),
          textStyle: textTheme.labelLarge,
        ),
      ),
      outlinedButtonTheme: OutlinedButtonThemeData(
        style: OutlinedButton.styleFrom(
          minimumSize: const Size(0, 36),
          padding: const EdgeInsets.symmetric(horizontal: 12),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(radius),
          ),
        ),
      ),
      textButtonTheme: TextButtonThemeData(
        style: TextButton.styleFrom(
          minimumSize: const Size(0, 32),
          padding: const EdgeInsets.symmetric(horizontal: 10),
        ),
      ),
      iconButtonTheme: IconButtonThemeData(
        style: IconButton.styleFrom(
          minimumSize: const Size.square(32),
          shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(6)),
        ),
      ),
      chipTheme: base.chipTheme.copyWith(
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(6)),
        side: BorderSide(color: scheme.outlineVariant),
        labelStyle: textTheme.labelMedium,
        selectedColor: scheme.primaryContainer,
        showCheckmark: false,
      ),
      segmentedButtonTheme: SegmentedButtonThemeData(
        style: ButtonStyle(
          visualDensity: VisualDensity.compact,
          tapTargetSize: MaterialTapTargetSize.shrinkWrap,
          shape: WidgetStatePropertyAll(
            RoundedRectangleBorder(borderRadius: BorderRadius.circular(6)),
          ),
        ),
      ),
      listTileTheme: ListTileThemeData(
        minVerticalPadding: 8,
        iconColor: scheme.onSurfaceVariant,
        selectedTileColor: scheme.primaryContainer.withValues(alpha: 0.55),
        selectedColor: scheme.primary,
        titleTextStyle: textTheme.titleMedium,
        subtitleTextStyle: textTheme.bodyMedium?.copyWith(
          color: scheme.onSurfaceVariant,
        ),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
      ),
      dialogTheme: DialogThemeData(
        backgroundColor: scheme.surfaceContainerLowest,
        surfaceTintColor: Colors.transparent,
        elevation: 0,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(12),
          side: BorderSide(color: scheme.outlineVariant),
        ),
      ),
      snackBarTheme: SnackBarThemeData(
        behavior: SnackBarBehavior.floating,
        backgroundColor: isDark
            ? const Color(0xFF243552)
            : LinguaRayPalette.navy,
        contentTextStyle: textTheme.bodyMedium?.copyWith(color: Colors.white),
      ),
      tooltipTheme: const TooltipThemeData(
        waitDuration: Duration(milliseconds: 400),
        padding: EdgeInsets.symmetric(horizontal: 8, vertical: 6),
      ),
      progressIndicatorTheme: ProgressIndicatorThemeData(
        color: scheme.primary,
        linearTrackColor: scheme.surfaceContainerHighest,
      ),
      textSelectionTheme: TextSelectionThemeData(
        cursorColor: scheme.primary,
        selectionColor: scheme.primary.withValues(alpha: 0.22),
        selectionHandleColor: scheme.primary,
      ),
      tabBarTheme: TabBarThemeData(
        dividerColor: scheme.outlineVariant,
        indicatorColor: scheme.primary,
        labelColor: scheme.primary,
        unselectedLabelColor: scheme.onSurfaceVariant,
        overlayColor: WidgetStatePropertyAll(
          scheme.primary.withValues(alpha: 0.08),
        ),
      ),
      scrollbarTheme: const ScrollbarThemeData(
        thickness: WidgetStatePropertyAll(8),
        radius: Radius.circular(8),
        thumbVisibility: WidgetStatePropertyAll(true),
      ),
      focusColor: scheme.primary.withValues(alpha: 0.16),
      hoverColor: scheme.primary.withValues(alpha: 0.06),
      extensions: <ThemeExtension<dynamic>>[brand, LinguaRayMetrics.standard],
    );
  }
}

double? lerpDouble(double a, double b, double t) => a + (b - a) * t;
