import 'package:flutter/foundation.dart' show defaultTargetPlatform;
import 'package:flutter/material.dart';

/// Canonical LinguaRay brand colors. These are the only named product hues
/// used by the production Material 3 theme.
abstract final class LinguaRayPalette {
  static const Color actionOrange = Color(0xFFB94D30);
  static const Color linguaBlue = Color(0xFF2859D9);
  static const Color rayTeal = Color(0xFF18A6A6);
  static const Color rayTealDark = Color(0xFF34C0BE);
  static const Color navy = Color(0xFF13233F);
  static const Color graphite = Color(0xFF302D2B);
  static const Color paper = Color(0xFFF6F5F3);
  static const Color white = Color(0xFFFFFFFF);
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
    navy: LinguaRayPalette.graphite,
    ray: LinguaRayPalette.rayTeal,
    canvas: LinguaRayPalette.paper,
    ink: LinguaRayPalette.graphite,
    resultRule: LinguaRayPalette.actionOrange,
  );

  static const dark = LinguaRayBrandColors(
    navy: LinguaRayPalette.navy,
    ray: LinguaRayPalette.rayTealDark,
    canvas: Color(0xFF24211F),
    ink: Color(0xFFF5EDE7),
    resultRule: Color(0xFFFFB695),
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
    this.settingsNavWidth = 80,
    this.quickWidth = 720,
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
      primary: LinguaRayPalette.actionOrange,
      onPrimary: Color(0xFFFFFFFF),
      primaryContainer: Color(0xFFFFF0E9),
      onPrimaryContainer: Color(0xFF82351E),
      secondary: Color(0xFF7A6557),
      onSecondary: Color(0xFFFFFFFF),
      secondaryContainer: Color(0xFFF1ECE7),
      onSecondaryContainer: Color(0xFF504238),
      tertiary: Color(0xFF8A644B),
      onTertiary: Color(0xFFFFFFFF),
      tertiaryContainer: Color(0xFFF4E8DD),
      onTertiaryContainer: Color(0xFF4E3626),
      error: Color(0xFFBA1A1A),
      onError: Color(0xFFFFFFFF),
      errorContainer: Color(0xFFFFDAD6),
      onErrorContainer: Color(0xFF410002),
      surface: LinguaRayPalette.paper,
      onSurface: LinguaRayPalette.graphite,
      surfaceContainerLowest: Color(0xFFFFFFFF),
      surfaceContainerLow: Color(0xFFF6F5F3),
      surfaceContainer: Color(0xFFEEECE9),
      surfaceContainerHigh: Color(0xFFE8E4DF),
      surfaceContainerHighest: Color(0xFFDCD6D0),
      onSurfaceVariant: Color(0xFF766F69),
      outline: Color(0xFF9D948B),
      outlineVariant: Color(0xFFE7E2DC),
      inverseSurface: LinguaRayPalette.navy,
      onInverseSurface: LinguaRayPalette.paper,
      inversePrimary: Color(0xFFFFBEA5),
      scrim: Color(0xFF000000),
      shadow: Color(0xFF000000),
      surfaceTint: Colors.transparent,
    );
  }

  static ColorScheme _darkScheme() {
    return const ColorScheme(
      brightness: Brightness.dark,
      primary: Color(0xFFFFB695),
      onPrimary: Color(0xFF54200E),
      primaryContainer: Color(0xFF71321D),
      onPrimaryContainer: Color(0xFFFFDACA),
      secondary: Color(0xFFC6B4A9),
      onSecondary: Color(0xFF36291F),
      secondaryContainer: Color(0xFF453C35),
      onSecondaryContainer: Color(0xFFF1E4D9),
      tertiary: Color(0xFFDFC0A9),
      onTertiary: Color(0xFF403126),
      tertiaryContainer: Color(0xFF594435),
      onTertiaryContainer: Color(0xFFFBE8D8),
      error: Color(0xFFFFB4AB),
      onError: Color(0xFF690005),
      errorContainer: Color(0xFF93000A),
      onErrorContainer: Color(0xFFFFDAD6),
      surface: Color(0xFF24211F),
      onSurface: Color(0xFFF5EDE7),
      surfaceContainerLowest: Color(0xFF2E2925),
      surfaceContainerLow: Color(0xFF282420),
      surfaceContainer: Color(0xFF352F29),
      surfaceContainerHigh: Color(0xFF3C352E),
      surfaceContainerHighest: Color(0xFF453C34),
      onSurfaceVariant: Color(0xFFC2B4A8),
      outline: Color(0xFF9E8D7E),
      outlineVariant: Color(0xFF4E443A),
      inverseSurface: Color(0xFFF3ECE6),
      onInverseSurface: LinguaRayPalette.graphite,
      inversePrimary: LinguaRayPalette.actionOrange,
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
    final resolvedPlatform = platform ?? defaultTargetPlatform;
    final base = ThemeData(
      useMaterial3: true,
      brightness: brightness,
      colorScheme: scheme,
      scaffoldBackgroundColor: scheme.surface,
      visualDensity: VisualDensity.compact,
      splashFactory: NoSplash.splashFactory,
      hoverColor: scheme.onSurface.withValues(alpha: 0.035),
      highlightColor: scheme.onSurface.withValues(alpha: 0.045),
      platform: resolvedPlatform,
      fontFamily: resolvedPlatform == TargetPlatform.macOS
          ? 'CupertinoSystemText'
          : 'Segoe UI',
      fontFamilyFallback: resolvedPlatform == TargetPlatform.macOS
          ? const ['PingFang SC', 'Helvetica Neue', 'sans-serif']
          : const ['Microsoft YaHei UI', 'Microsoft YaHei', 'sans-serif'],
    );

    final displayFamily = resolvedPlatform == TargetPlatform.macOS
        ? 'CupertinoSystemDisplay'
        : 'Segoe UI';
    final textTheme = base.textTheme.copyWith(
      headlineMedium: base.textTheme.headlineMedium?.copyWith(
        fontFamily: displayFamily,
        fontSize: 28,
        height: 1.25,
        fontWeight: FontWeight.w500,
        letterSpacing: -0.3,
        color: scheme.onSurface,
      ),
      titleLarge: base.textTheme.titleLarge?.copyWith(
        fontFamily: displayFamily,
        fontSize: 26,
        height: 1.3,
        letterSpacing: -0.45,
        fontWeight: FontWeight.w500,
        color: scheme.onSurface,
      ),
      titleMedium: base.textTheme.titleMedium?.copyWith(
        fontSize: 13,
        height: 1.4,
        letterSpacing: 0,
        fontWeight: FontWeight.w500,
        color: scheme.onSurface,
      ),
      bodyLarge: base.textTheme.bodyLarge?.copyWith(
        fontSize: 14,
        height: 1.5,
        letterSpacing: 0,
        color: scheme.onSurface,
      ),
      bodyMedium: base.textTheme.bodyMedium?.copyWith(
        fontSize: 14,
        height: 1.5,
        letterSpacing: 0,
        color: scheme.onSurface,
      ),
      bodySmall: base.textTheme.bodySmall?.copyWith(
        fontSize: 12,
        height: 1.5,
        color: scheme.onSurfaceVariant,
        letterSpacing: 0,
      ),
      labelSmall: base.textTheme.labelSmall?.copyWith(
        fontSize: 11,
        height: 1.4,
        color: scheme.onSurfaceVariant,
        letterSpacing: 0.1,
      ),
      labelMedium: base.textTheme.labelMedium?.copyWith(
        fontSize: 12,
        height: 1.4,
        color: scheme.onSurfaceVariant,
        letterSpacing: 0,
      ),
      labelLarge: base.textTheme.labelLarge?.copyWith(
        letterSpacing: 0,
        fontWeight: FontWeight.w500,
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
        fillColor: isDark
            ? scheme.surfaceContainerLow
            : const Color(0xFFFAF9F7),
        floatingLabelBehavior: FloatingLabelBehavior.always,
        labelStyle: textTheme.bodySmall,
        border: outline,
        enabledBorder: outline,
        focusedBorder: outline.copyWith(
          borderSide: BorderSide(
            color: scheme.primary.withValues(alpha: 0.75),
            width: 1.25,
          ),
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
      switchTheme: SwitchThemeData(
        trackOutlineWidth: const WidgetStatePropertyAll(0),
        trackOutlineColor: const WidgetStatePropertyAll(Colors.transparent),
        thumbColor: WidgetStateProperty.resolveWith(
          (states) => states.contains(WidgetState.disabled)
              ? scheme.onSurface.withValues(alpha: 0.25)
              : Colors.white,
        ),
        trackColor: WidgetStateProperty.resolveWith(
          (states) => states.contains(WidgetState.selected)
              ? scheme.primary
              : scheme.outlineVariant,
        ),
      ),
      dropdownMenuTheme: DropdownMenuThemeData(textStyle: textTheme.bodyMedium),
      dialogTheme: DialogThemeData(
        backgroundColor: scheme.surfaceContainerLowest,
        surfaceTintColor: Colors.transparent,
        elevation: 8,
        shadowColor: Colors.black.withValues(alpha: 0.1),
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(14),
          side: BorderSide(color: scheme.outlineVariant),
        ),
      ),
      snackBarTheme: SnackBarThemeData(
        behavior: SnackBarBehavior.floating,
        backgroundColor: isDark
            ? scheme.surfaceContainerHighest
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
        thickness: WidgetStatePropertyAll(5),
        radius: Radius.circular(8),
        thumbVisibility: WidgetStatePropertyAll(true),
      ),
      focusColor: scheme.primary.withValues(alpha: 0.16),
      hoverColor: scheme.onSurface.withValues(alpha: 0.035),
      extensions: <ThemeExtension<dynamic>>[brand, LinguaRayMetrics.standard],
    );
  }
}

double? lerpDouble(double a, double b, double t) => a + (b - a) * t;
