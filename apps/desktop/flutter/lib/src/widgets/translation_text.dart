import 'package:flutter/foundation.dart'
    show TargetPlatform, defaultTargetPlatform;
import 'package:flutter/material.dart' show SelectableText;
import 'package:flutter/widgets.dart';

import 'native_text.dart';
import 'ui.dart' show DesignThemeContext;

const _selectionAlpha = 0.20;

/// Selectable translation output with a native macOS text surface and a
/// Flutter implementation on Windows.
class TranslationText extends StatelessWidget {
  const TranslationText(
    this.data, {
    super.key,
    this.style,
    this.textAlign = TextAlign.start,
    this.padding = EdgeInsets.zero,
    this.onTap,
    this.onDoubleTap,
  });

  final String data;
  final TextStyle? style;
  final TextAlign textAlign;
  final EdgeInsetsGeometry padding;
  final GestureTapCallback? onTap;
  final GestureTapCallback? onDoubleTap;

  @override
  Widget build(BuildContext context) {
    final tokens = context.tokens;
    final selectionColor = tokens.colors.accent.withValues(
      alpha: _selectionAlpha,
    );

    if (defaultTargetPlatform == TargetPlatform.macOS) {
      return NativeText(
        text: data,
        style: style,
        textAlign: textAlign,
        padding: padding,
        selectionColor: selectionColor,
        brightness: tokens.brightness,
        onTap: onTap,
        onDoubleTap: onDoubleTap,
      );
    }

    final selectable = Padding(
      padding: padding,
      child: DefaultSelectionStyle(
        selectionColor: selectionColor,
        child: SelectableText(data, style: style, textAlign: textAlign),
      ),
    );
    if (onTap == null && onDoubleTap == null) return selectable;

    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTap: onTap,
      onDoubleTap: onDoubleTap,
      child: selectable,
    );
  }
}
