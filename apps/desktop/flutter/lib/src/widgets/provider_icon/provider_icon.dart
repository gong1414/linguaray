import 'package:flutter/widgets.dart';

import '../../theme/product_tokens.dart' show ProductTokens;
import '../ui.dart' show DesignThemeContext, DesignTypographyStyles;

/// A compact, neutral provider identifier.
///
/// LinguaRay deliberately does not bundle third-party logos. Provider rows use
/// a stable colour and short monogram, which keeps the UI consistent and avoids
/// treating vendor artwork as application-owned assets.
class ProviderIcon extends StatelessWidget {
  const ProviderIcon(
    this.type, {
    super.key,
    this.size = 22,
    this.color,
    this.border,
  });

  /// The provider type value as the runtime spells it — `deepl`, `anthropic`,
  /// `openai_compatible`.
  final String type;
  final double size;
  final Color? color;
  final Border? border;

  @override
  Widget build(BuildContext context) {
    final tokens = context.tokens;
    final radius = BorderRadius.circular(tokens.radii.avatar);
    return _LetterMark(
      type: type,
      size: size,
      radius: radius,
      color: color,
      border: border,
    );
  }
}

class _LetterMark extends StatelessWidget {
  const _LetterMark({
    required this.type,
    required this.size,
    required this.radius,
    this.color,
    this.border,
  });

  final String type;
  final double size;
  final BorderRadius radius;
  final Color? color;
  final Border? border;

  @override
  Widget build(BuildContext context) {
    final tokens = context.tokens;
    final background =
        color ??
        switch (type) {
          'system' => ProductTokens.providerBuiltin,
          'anthropic' => ProductTokens.providerClaude,
          'deepl' => ProductTokens.providerDeepl,
          _ => ProductTokens.providerDict,
        };

    return ExcludeSemantics(
      child: Container(
        width: size,
        height: size,
        alignment: Alignment.center,
        decoration: BoxDecoration(
          color: background,
          borderRadius: radius,
          border: border,
        ),
        child: Text(
          type.isEmpty ? '?' : type.substring(0, 1).toUpperCase(),
          style: tokens.typography.displayStyle(
            fontSize: size * 0.55,
            fontWeight: FontWeight.w700,
            height: 1,
            color: const Color(0xFFFFFFFF),
          ),
        ),
      ),
    );
  }
}
