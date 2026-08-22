import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_desktop/src/routes/workbench/welcome.dart';
import 'package:linguaray_desktop/src/widgets/ui.dart' show DesignThemeProvider;

void main() {
  testWidgets('welcome actions wrap without overflowing a narrow viewport', (
    tester,
  ) async {
    tester.view.devicePixelRatio = 1;
    // The Windows workbench draws its own caption controls, so its shell has a
    // wider irreducible minimum than the native macOS title bar. Both widths
    // remain well below the real 840 px workbench minimum and exercise the
    // responsive welcome content.
    tester.view.physicalSize = Size(Platform.isWindows ? 360 : 224, 900);
    addTearDown(tester.view.reset);

    await tester.pumpWidget(
      const DesignThemeProvider(
        child: MaterialApp(home: WorkbenchWelcomePage()),
      ),
    );
    await tester.pump();

    expect(find.text('Get started'), findsOneWidget);
    expect(find.text('Recheck access'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });
}
