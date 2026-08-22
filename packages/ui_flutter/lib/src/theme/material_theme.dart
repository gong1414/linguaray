import 'package:flutter/material.dart';

@immutable
final class LinguaRayBrandColors extends ThemeExtension<LinguaRayBrandColors> {
  const LinguaRayBrandColors({
    required this.navy,
    required this.ray,
    required this.resultSurface,
    required this.railSurface,
  });

  static const light = LinguaRayBrandColors(
    navy: Color(0xFF1A2433),
    ray: Color(0xFF0F766E),
    resultSurface: Color(0xFFE7F3F1),
    railSurface: Color(0xFFEEF2F2),
  );

  static const dark = LinguaRayBrandColors(
    navy: Color(0xFFE6EEF8),
    ray: Color(0xFF5EEAD4),
    resultSurface: Color(0xFF10211F),
    railSurface: Color(0xFF111618),
  );

  final Color navy;
  final Color ray;
  final Color resultSurface;
  final Color railSurface;

  @override
  LinguaRayBrandColors copyWith({
    Color? navy,
    Color? ray,
    Color? resultSurface,
    Color? railSurface,
  }) {
    return LinguaRayBrandColors(
      navy: navy ?? this.navy,
      ray: ray ?? this.ray,
      resultSurface: resultSurface ?? this.resultSurface,
      railSurface: railSurface ?? this.railSurface,
    );
  }

  @override
  LinguaRayBrandColors lerp(covariant LinguaRayBrandColors? other, double t) {
    if (other == null) return this;
    return LinguaRayBrandColors(
      navy: Color.lerp(navy, other.navy, t)!,
      ray: Color.lerp(ray, other.ray, t)!,
      resultSurface: Color.lerp(resultSurface, other.resultSurface, t)!,
      railSurface: Color.lerp(railSurface, other.railSurface, t)!,
    );
  }
}

@immutable
final class LinguaRayMetrics extends ThemeExtension<LinguaRayMetrics> {
  const LinguaRayMetrics({
    this.space = 8,
    this.railWidth = 80,
    this.quickWidth = 396,
    this.controlHeight = 44,
    this.captionHeight = 40,
    this.macTrafficInset = 78,
    this.workbenchMinSize = const Size(840, 560),
  });

  static const standard = LinguaRayMetrics();

  final double space;
  final double railWidth;
  final double quickWidth;
  final double controlHeight;
  final double captionHeight;
  final double macTrafficInset;
  final Size workbenchMinSize;

  @override
  LinguaRayMetrics copyWith({
    double? space,
    double? railWidth,
    double? quickWidth,
    double? controlHeight,
    double? captionHeight,
    double? macTrafficInset,
    Size? workbenchMinSize,
  }) {
    return LinguaRayMetrics(
      space: space ?? this.space,
      railWidth: railWidth ?? this.railWidth,
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
      railWidth: lerpDouble(railWidth, other.railWidth, t)!,
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
  static const Color _seed = Color(0xFF0F766E);

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

  static ThemeData _build(Brightness brightness, TargetPlatform? platform) {
    final isDark = brightness == Brightness.dark;
    final scheme =
        ColorScheme.fromSeed(
          seedColor: _seed,
          brightness: brightness,
          contrastLevel: 0.18,
        ).copyWith(
          primary: isDark ? const Color(0xFF5EEAD4) : const Color(0xFF0F766E),
          onPrimary: isDark ? const Color(0xFF042F2E) : Colors.white,
          primaryContainer: isDark
              ? const Color(0xFF134E4A)
              : const Color(0xFFCCFBF1),
          onPrimaryContainer: isDark
              ? const Color(0xFFCCFBF1)
              : const Color(0xFF134E4A),
          secondary: isDark ? const Color(0xFFB7C5D6) : const Color(0xFF3F4F63),
          onSecondary: isDark ? const Color(0xFF1C2836) : Colors.white,
          surface: isDark ? const Color(0xFF121618) : const Color(0xFFFBFCFC),
          onSurface: isDark ? const Color(0xFFE6EBEB) : const Color(0xFF1A2124),
          surfaceContainerLowest: isDark
              ? const Color(0xFF0C1012)
              : const Color(0xFFFFFFFF),
          surfaceContainerLow: isDark
              ? const Color(0xFF161B1D)
              : const Color(0xFFF3F6F6),
          surfaceContainer: isDark
              ? const Color(0xFF1B2123)
              : const Color(0xFFEEF2F2),
          outlineVariant: isDark
              ? const Color(0xFF2A3336)
              : const Color(0xFFD5DEDE),
        );

    final brand = isDark
        ? LinguaRayBrandColors.dark
        : LinguaRayBrandColors.light;
    final resolvedPlatform = platform ?? TargetPlatform.macOS;
    final base = ThemeData(
      useMaterial3: true,
      brightness: brightness,
      colorScheme: scheme,
      scaffoldBackgroundColor: isDark
          ? const Color(0xFF0C1012)
          : const Color(0xFFF4F6F6),
      visualDensity: VisualDensity.standard,
      splashFactory: InkSparkle.splashFactory,
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
        fontSize: 26,
        height: 1.2,
        fontWeight: FontWeight.w600,
        letterSpacing: -0.4,
      ),
      titleLarge: base.textTheme.titleLarge?.copyWith(
        fontSize: 17,
        height: 1.3,
        fontWeight: FontWeight.w600,
      ),
      titleMedium: base.textTheme.titleMedium?.copyWith(
        fontSize: 14,
        height: 1.3,
        fontWeight: FontWeight.w600,
      ),
      bodyLarge: base.textTheme.bodyLarge?.copyWith(fontSize: 15, height: 1.5),
      bodyMedium: base.textTheme.bodyMedium?.copyWith(
        fontSize: 13,
        height: 1.45,
      ),
      labelLarge: base.textTheme.labelLarge?.copyWith(
        fontWeight: FontWeight.w600,
      ),
    );

    const radius = 10.0;
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
        toolbarHeight: 40,
        titleTextStyle: textTheme.titleMedium,
      ),
      navigationRailTheme: NavigationRailThemeData(
        backgroundColor: brand.railSurface,
        elevation: 0,
        minWidth: 80,
        groupAlignment: -1,
        labelType: NavigationRailLabelType.all,
        useIndicator: true,
        indicatorColor: scheme.primaryContainer,
        selectedIconTheme: IconThemeData(
          color: scheme.onPrimaryContainer,
          size: 22,
        ),
        unselectedIconTheme: IconThemeData(
          color: scheme.onSurfaceVariant,
          size: 22,
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
          horizontal: 14,
          vertical: 12,
        ),
        hintStyle: textTheme.bodyMedium?.copyWith(
          color: scheme.onSurfaceVariant,
        ),
      ),
      filledButtonTheme: FilledButtonThemeData(
        style: FilledButton.styleFrom(
          minimumSize: const Size(0, 44),
          padding: const EdgeInsets.symmetric(horizontal: 18),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(radius),
          ),
          textStyle: textTheme.labelLarge,
        ),
      ),
      outlinedButtonTheme: OutlinedButtonThemeData(
        style: OutlinedButton.styleFrom(
          minimumSize: const Size(0, 44),
          padding: const EdgeInsets.symmetric(horizontal: 14),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(radius),
          ),
        ),
      ),
      textButtonTheme: TextButtonThemeData(
        style: TextButton.styleFrom(
          minimumSize: const Size(0, 40),
          padding: const EdgeInsets.symmetric(horizontal: 12),
        ),
      ),
      iconButtonTheme: IconButtonThemeData(
        style: IconButton.styleFrom(
          minimumSize: const Size.square(40),
          shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
        ),
      ),
      chipTheme: base.chipTheme.copyWith(
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
        side: BorderSide(color: scheme.outlineVariant),
        labelStyle: textTheme.labelMedium,
        selectedColor: scheme.primaryContainer,
      ),
      segmentedButtonTheme: SegmentedButtonThemeData(
        style: ButtonStyle(
          visualDensity: VisualDensity.compact,
          tapTargetSize: MaterialTapTargetSize.padded,
          shape: WidgetStatePropertyAll(
            RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
          ),
        ),
      ),
      listTileTheme: ListTileThemeData(
        minVerticalPadding: 10,
        iconColor: scheme.onSurfaceVariant,
        titleTextStyle: textTheme.titleMedium,
        subtitleTextStyle: textTheme.bodyMedium?.copyWith(
          color: scheme.onSurfaceVariant,
        ),
      ),
      dialogTheme: DialogThemeData(
        backgroundColor: scheme.surfaceContainerLowest,
        surfaceTintColor: Colors.transparent,
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
      ),
      snackBarTheme: SnackBarThemeData(
        behavior: SnackBarBehavior.floating,
        backgroundColor: isDark
            ? const Color(0xFF1F2A2C)
            : const Color(0xFF1A2433),
        contentTextStyle: textTheme.bodyMedium?.copyWith(color: Colors.white),
      ),
      tooltipTheme: const TooltipThemeData(
        waitDuration: Duration(milliseconds: 400),
        padding: EdgeInsets.symmetric(horizontal: 10, vertical: 6),
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
      scrollbarTheme: const ScrollbarThemeData(
        thickness: WidgetStatePropertyAll(8),
        radius: Radius.circular(8),
        thumbVisibility: WidgetStatePropertyAll(true),
      ),
      extensions: <ThemeExtension<dynamic>>[brand, LinguaRayMetrics.standard],
    );
  }
}

double? lerpDouble(double a, double b, double t) => a + (b - a) * t;
