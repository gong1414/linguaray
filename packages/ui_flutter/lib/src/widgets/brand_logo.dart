import 'package:flutter/widgets.dart';

// Exact geometry from assets/brand/linguaray/dist/svg/linguaray-symbol.svg.
// Keep this painter in sync with that canonical asset through the brand asset
// generation workflow; do not redraw either path independently.
final Path _linguaPath = Path()
  ..moveTo(44, 38)
  ..lineTo(86, 88)
  ..lineTo(86, 159)
  ..cubicTo(86, 167, 92, 173, 100, 173)
  ..lineTo(138, 173)
  ..lineTo(158, 204)
  ..lineTo(92, 204)
  ..cubicTo(65, 204, 44, 183, 44, 156)
  ..close();

final Path _rayPath = Path()
  ..moveTo(91, 38)
  ..lineTo(156, 38)
  ..cubicTo(194, 38, 220, 61, 220, 95)
  ..cubicTo(220, 120, 205, 139, 181, 147)
  ..lineTo(220, 204)
  ..lineTo(196, 204)
  ..lineTo(147, 138)
  ..lineTo(147, 115)
  ..lineTo(156, 115)
  ..cubicTo(170, 115, 179, 110, 179, 91)
  ..cubicTo(179, 81, 170, 75, 156, 75)
  ..lineTo(91, 75)
  ..close();

/// The single-color LinguaRay LR monogram for system-controlled surfaces.
class BrandGlyph extends StatelessWidget {
  const BrandGlyph({super.key, required this.size, required this.color});

  final double size;
  final Color color;

  @override
  Widget build(BuildContext context) {
    return ExcludeSemantics(
      child: CustomPaint(
        size: Size.square(size),
        painter: _BrandGlyphPainter(color),
      ),
    );
  }
}

class _BrandGlyphPainter extends CustomPainter {
  const _BrandGlyphPainter(this.color);

  final Color color;

  @override
  void paint(Canvas canvas, Size size) {
    final scale = size.shortestSide / 240;
    canvas.scale(scale);
    final paint = Paint()..color = color;
    canvas.drawPath(_linguaPath, paint);
    canvas.drawPath(_rayPath, paint);
  }

  @override
  bool shouldRepaint(_BrandGlyphPainter oldDelegate) =>
      color != oldDelegate.color;
}

/// The production LinguaRay app mark.
///
/// Colors and placement match the flat app-icon master. The UI-only corner
/// radius keeps the mark aligned with surrounding controls; operating systems
/// apply their own masks to the exported application icon.
class BrandLogo extends StatelessWidget {
  const BrandLogo({super.key, this.size = 20});

  final double size;

  @override
  Widget build(BuildContext context) {
    return ExcludeSemantics(
      child: CustomPaint(
        size: Size.square(size),
        painter: const _BrandLogoPainter(),
      ),
    );
  }
}

class _BrandLogoPainter extends CustomPainter {
  const _BrandLogoPainter();

  @override
  void paint(Canvas canvas, Size size) {
    final edge = size.shortestSide;
    canvas.drawRRect(
      RRect.fromRectAndRadius(
        Offset.zero & Size.square(edge),
        Radius.circular(edge * 0.22),
      ),
      Paint()..color = const Color(0xFF13233F),
    );

    canvas.save();
    canvas.translate(edge * (142 / 1024), edge * (176 / 1024));
    canvas.scale(edge * (2.8 / 1024));
    canvas.drawPath(_linguaPath, Paint()..color = const Color(0xFFFFFFFF));
    canvas.drawPath(_rayPath, Paint()..color = const Color(0xFF34C0BE));
    canvas.restore();
  }

  @override
  bool shouldRepaint(_BrandLogoPainter oldDelegate) => false;
}
