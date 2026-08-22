import 'package:flutter/widgets.dart';
import 'package:linguaray_ui/src/theme/tokens.dart';

/// CSS distributes leading evenly above and below the text box; Flutter's
/// default is proportional. Even distribution is what keeps a height-1 chip
/// exactly as tall as its text box.
const TextLeadingDistribution _kEvenLeading = TextLeadingDistribution.even;

TextStyle _style(
  DesignFont font, {
  required double fontSize,
  FontWeight? fontWeight,
  double? height,
  Color? color,
  double? letterSpacing,
  List<FontFeature>? fontFeatures,
}) => TextStyle(
  fontFamily: font.family,
  fontFamilyFallback: font.fallback.isEmpty ? null : font.fallback,
  fontSize: fontSize,
  fontWeight: fontWeight,
  height: height,
  color: color,
  letterSpacing: letterSpacing,
  fontFeatures: fontFeatures,
  leadingDistribution: _kEvenLeading,
);

/// Type recipes, one per face. `height` maps to CSS `line-height`; pass `1.0`
/// where a line-height of one is wanted.
extension DesignTypographyStyles on DesignTypography {
  TextStyle displayStyle({
    double? fontSize,
    FontWeight? fontWeight,
    double? height,
    Color? color,
    double? letterSpacing,
    List<FontFeature>? fontFeatures,
  }) => _style(
    display,
    fontSize: fontSize ?? body,
    fontWeight: fontWeight,
    height: height,
    color: color,
    letterSpacing: letterSpacing,
    fontFeatures: fontFeatures,
  );

  TextStyle sansStyle({
    double? fontSize,
    FontWeight? fontWeight,
    double? height,
    Color? color,
    double? letterSpacing,
  }) => _style(
    sans,
    fontSize: fontSize ?? body,
    fontWeight: fontWeight,
    height: height,
    color: color,
    letterSpacing: letterSpacing,
  );

  TextStyle cjkStyle({
    double? fontSize,
    FontWeight? fontWeight,
    double? height,
    Color? color,
  }) => _style(
    cjk,
    fontSize: fontSize ?? body,
    fontWeight: fontWeight,
    height: height,
    color: color,
  );

  TextStyle monoStyle({
    double? fontSize,
    FontWeight? fontWeight,
    double? height,
    Color? color,
  }) => _style(
    mono,
    fontSize: fontSize ?? body,
    fontWeight: fontWeight,
    height: height,
    color: color,
  );

  /// `.bt-label` — the micro section label ("原文 · 第 12 / 38 段", "质量信号").
  /// AppKit sets these at 11pt semibold in sentence case: no uppercasing, no
  /// added tracking.
  TextStyle labelStyle({Color? color}) => _style(
    label,
    fontSize: caption,
    fontWeight: FontWeight.w600,
    height: 1,
    color: color,
  );

  /// `.bt-numeric` — numerals and shortcut glyphs.
  TextStyle numericStyle({double? fontSize, Color? color, double? height}) =>
      _style(
        display,
        fontSize: fontSize ?? body,
        fontWeight: FontWeight.w600,
        height: height,
        color: color,
        letterSpacing: -0.01 * (fontSize ?? body),
        fontFeatures: const [FontFeature.tabularFigures()],
      );
}
