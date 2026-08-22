import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_desktop/src/ui/first_run/first_run_view.dart';
import 'package:linguaray_ui/linguaray_ui.dart' show LinguaRayMaterialTheme;

void main() {
  testWidgets('first-run actions wrap without overflowing a narrow viewport', (
    tester,
  ) async {
    tester.view.devicePixelRatio = 1;
    tester.view.physicalSize = const Size(360, 900);
    addTearDown(tester.view.reset);

    await tester.pumpWidget(
      MaterialApp(
        theme: LinguaRayMaterialTheme.light(),
        home: FirstRunView(
          labels: const FirstRunLabels(
            title: 'Set up LinguaRay',
            subtitle: 'A few checks, then you can translate from any app.',
            permissionsTitle: 'Permissions',
            permissionsBody: 'Selection and capture need access.',
            accessibility: 'Accessibility',
            screenRecording: 'Screen Recording',
            shortcutsTitle: 'Shortcuts',
            shortcutsBody: 'Ready.',
            servicesTitle: 'Services',
            servicesBody: 'Enable a service.',
            granted: 'Granted',
            denied: 'Not granted',
            notRequired: 'Not required',
            unknown: 'Unknown',
            checking: 'Checking…',
            conflict: 'Conflict',
            noProvider: 'No service',
            ready: 'Ready',
            grant: 'Grant access',
            recheck: 'Recheck',
            configureServices: 'Configure services',
            start: 'Get started',
            skip: 'Skip for now',
          ),
          permissions: const AccessSnapshot.notRequired(),
          shortcutsReady: true,
          shortcutConflict: false,
          hasServices: true,
          checkingPermissions: false,
          onGrantAccessibility: () {},
          onGrantScreenRecording: () {},
          onRecheck: () {},
          onConfigureServices: () {},
          onStart: () {},
          onSkip: () {},
        ),
      ),
    );
    await tester.pump();

    expect(find.text('Get started'), findsOneWidget);
    expect(find.text('Recheck'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });
}
