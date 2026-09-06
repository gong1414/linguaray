import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_desktop/src/features/services/services_settings_view.dart';
import 'package:linguaray_desktop/src/shared/settings_labels.dart';

void main() {
  testWidgets('translation services page exposes the built-in dictionary', (
    tester,
  ) async {
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: ServicesSettingsView(
            labels: _labels,
            pageTitle: 'Services',
            services: const [
              ServiceRecord(
                id: 'system+translation',
                name: 'system',
                providerId: 'system',
                providerName: 'System',
                kind: 'translation',
                enabled: false,
                isDefault: false,
              ),
              ServiceRecord(
                id: 'ecdict+dictionary',
                name: 'ecdict',
                providerId: 'ecdict',
                providerName: 'ECDICT',
                kind: 'dictionary',
                enabled: true,
                isDefault: true,
              ),
            ],
            serviceKind: 'translation',
            loading: false,
            onEnabledChanged: (_, _) {},
            onMakeDefault: (_) {},
            onDelete: (_) {},
            onConfigureProviders: () {},
          ),
        ),
      ),
    );

    expect(find.text('System Translation'), findsOneWidget);
    expect(find.text('Dictionary'), findsOneWidget);
    expect(find.text('ECDICT Dictionary'), findsOneWidget);
    expect(find.byIcon(Icons.remove_circle_outline_rounded), findsNothing);
  });
}

const _labels = ServicesSettingsLabels(
  title: 'Services',
  empty: 'Empty',
  loading: 'Loading',
  translation: 'Translation',
  dictionary: 'Dictionary',
  ocr: 'OCR',
  enabled: 'Enabled',
  makeDefault: 'Make default',
  isDefault: 'Default',
  configureProviders: 'Configure providers',
  commonLanguages: 'Common languages',
  defaultService: 'Default service',
  delete: 'Delete',
  deleteConfirm: 'Delete?',
);
