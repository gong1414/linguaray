import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';

/// Allows the small rasterisation drift Flutter produces across x64 and arm64
/// while still failing on any visible layout, colour, or typography change.
///
/// The tolerance is a fraction of changed pixels, not a per-channel colour
/// tolerance. At 0.05%, a 1000 x 700 catalog image may differ by at most 350
/// pixels. The largest observed cross-architecture drift is 20 pixels.
void installGoldenComparator({double precisionTolerance = 0.0005}) {
  final current = goldenFileComparator;
  if (current is! LocalFileComparator) return;

  goldenFileComparator = _TolerantGoldenFileComparator(
    current.basedir.resolve('_golden_test.dart'),
    precisionTolerance: precisionTolerance,
  );
}

class _TolerantGoldenFileComparator extends LocalFileComparator {
  _TolerantGoldenFileComparator(
    super.testFile, {
    required double precisionTolerance,
  }) : assert(precisionTolerance >= 0 && precisionTolerance <= 1),
       _precisionTolerance = precisionTolerance;

  final double _precisionTolerance;

  @override
  Future<bool> compare(Uint8List imageBytes, Uri golden) async {
    if (autoUpdateGoldenFiles) {
      await update(golden, imageBytes);
      return true;
    }
    final result = await GoldenFileComparator.compareLists(
      imageBytes,
      await getGoldenBytes(golden),
    );
    if (result.passed || result.diffPercent <= _precisionTolerance) {
      result.dispose();
      return true;
    }

    final error = await generateFailureOutput(result, golden, basedir);
    result.dispose();
    throw FlutterError(error);
  }
}
