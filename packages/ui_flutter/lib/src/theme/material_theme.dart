import 'package:flutter/foundation.dart' show defaultTargetPlatform;
import 'package:flutter/material.dart';

/// Canonical LinguaRay interface colors. These are the only named product hues
/// used by the production Material 3 theme.
abstract final class LinguaRayPalette {
  static const Color actionInk = Color(0xFF30343B);
  static const Color graphite = Color(0xFF22252B);
  static const Color paper = Color(0xFFF3F4F6);
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
      primary: LinguaRayPalette.actionInk,
      onPrimary: Color(0xFFFFFFFF),
      primaryContainer: Color(0xFFE9EBEF),
      onPrimaryContainer: Color(0xFF30343B),
      secondary: Color(0xFF626872),
      onSecondary: Color(0xFFFFFFFF),
      secondaryContainer: Color(0xFFECEEF2),
      onSecondaryContainer: Color(0xFF3E444E),
      tertiary: Color(0xFF805B24),
      onTertiary: Color(0xFFFFFFFF),
      tertiaryContainer: Color(0xFFFFF0D2),
      onTertiaryContainer: Color(0xFF563B12),
      error: Color(0xFFBA1A1A),
      onError: Color(0xFFFFFFFF),
      errorContainer: Color(0xFFFFDAD6),
      onErrorContainer: Color(0xFF410002),
      surface: LinguaRayPalette.paper,
      onSurface: LinguaRayPalette.graphite,
      surfaceContainerLowest: Color(0xFFFFFFFF),
      surfaceContainerLow: Color(0xFFF3F4F6),
      surfaceContainer: Color(0xFFECEEF2),
      surfaceContainerHigh: Color(0xFFE4E7EC),
      surfaceContainerHighest: Color(0xFFD8DCE3),
      onSurfaceVariant: Color(0xFF626872),
      outline: Color(0xFF858D99),
      outlineVariant: Color(0xFFDEE1E6),
      inverseSurface: LinguaRayPalette.graphite,
      onInverseSurface: LinguaRayPalette.paper,
      inversePrimary: Color(0xFFE4E7EC),
      scrim: Color(0xFF000000),
      shadow: Color(0xFF000000),
      surfaceTint: Colors.transparent,
    );
  }

  static ColorScheme _darkScheme() {
    return const ColorScheme(
      brightness: Brightness.dark,
      primary: Color(0xFFE4E7EC),
      onPrimary: Color(0xFF252930),
      primaryContainer: Color(0xFF3D434D),
      onPrimaryContainer: Color(0xFFF2F3F5),
      secondary: Color(0xFFB3B9C3),
      onSecondary: Color(0xFF252930),
      secondaryContainer: Color(0xFF383E47),
      onSecondaryContainer: Color(0xFFE3E6EC),
      tertiary: Color(0xFFE3C17D),
      onTertiary: Color(0xFF3B2C10),
      tertiaryContainer: Color(0xFF59451D),
      onTertiaryContainer: Color(0xFFFFE7B3),
      error: Color(0xFFFFB4AB),
      onError: Color(0xFF690005),
      errorContainer: Color(0xFF93000A),
      onErrorContainer: Color(0xFFFFDAD6),
      surface: Color(0xFF17191D),
      onSurface: Color(0xFFF2F3F5),
      surfaceContainerLowest: Color(0xFF202328),
      surfaceContainerLow: Color(0xFF24272D),
      surfaceContainer: Color(0xFF2A2E35),
      surfaceContainerHigh: Color(0xFF323740),
      surfaceContainerHighest: Color(0xFF3B414B),
      onSurfaceVariant: Color(0xFFB3B9C3),
      outline: Color(0xFF89929F),
      outlineVariant: Color(0xFF3B4049),
      inverseSurface: Color(0xFFF2F3F5),
      onInverseSurface: LinguaRayPalette.graphite,
      inversePrimary: LinguaRayPalette.actionInk,
      scrim: Color(0xFF000000),
      shadow: Color(0xFF000000),
      surfaceTint: Colors.transparent,
    );
  }

  static ThemeData _build(Brightness brightness, TargetPlatform? platform) {
    final isDark = brightness == Brightness.dark;
    final scheme = isDark ? _darkScheme() : _lightScheme();
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
        fontWeight: FontWeight.w600,
        letterSpacing: -0.3,
        color: scheme.onSurface,
      ),
      titleLarge: base.textTheme.titleLarge?.copyWith(
        fontFamily: displayFamily,
        fontSize: 28,
        height: 1.3,
        letterSpacing: -0.45,
        fontWeight: FontWeight.w600,
        color: scheme.onSurface,
      ),
      titleMedium: base.textTheme.titleMedium?.copyWith(
        fontSize: 15,
        height: 1.4,
        letterSpacing: 0,
        fontWeight: FontWeight.w600,
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
        color: scheme.surfaceContainerLow,
        surfaceTintColor: Colors.transparent,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(12),
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
            : const Color(0xFFF7F8FA),
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
      searchBarTheme: SearchBarThemeData(
        constraints: const BoxConstraints(minHeight: 40, maxHeight: 40),
        elevation: const WidgetStatePropertyAll(0),
        backgroundColor: WidgetStatePropertyAll(scheme.surfaceContainerLow),
        surfaceTintColor: const WidgetStatePropertyAll(Colors.transparent),
        side: WidgetStatePropertyAll(BorderSide(color: scheme.outlineVariant)),
        shape: WidgetStatePropertyAll(
          RoundedRectangleBorder(borderRadius: BorderRadius.circular(radius)),
        ),
        padding: const WidgetStatePropertyAll(
          EdgeInsets.symmetric(horizontal: 12),
        ),
        textStyle: WidgetStatePropertyAll(textTheme.bodyMedium),
        hintStyle: WidgetStatePropertyAll(
          textTheme.bodyMedium?.copyWith(color: scheme.onSurfaceVariant),
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
        minVerticalPadding: 14,
        contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
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
              : states.contains(WidgetState.selected)
              ? scheme.onPrimary
              : scheme.surfaceContainerLowest,
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
        elevation: 0,
        shadowColor: Colors.black.withValues(alpha: 0.1),
        titleTextStyle: textTheme.titleLarge?.copyWith(fontSize: 20),
        contentTextStyle: textTheme.bodyMedium,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(16),
          side: BorderSide(color: scheme.outlineVariant),
        ),
      ),
      popupMenuTheme: PopupMenuThemeData(
        color: scheme.surfaceContainerLowest,
        surfaceTintColor: Colors.transparent,
        textStyle: textTheme.bodyMedium,
        elevation: 4,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(radius),
          side: BorderSide(color: scheme.outlineVariant),
        ),
      ),
      snackBarTheme: SnackBarThemeData(
        behavior: SnackBarBehavior.floating,
        backgroundColor: isDark
            ? scheme.surfaceContainerHighest
            : LinguaRayPalette.graphite,
        contentTextStyle: textTheme.bodyMedium?.copyWith(
          color: isDark ? scheme.onSurface : scheme.onPrimary,
        ),
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
    );
  }
}
