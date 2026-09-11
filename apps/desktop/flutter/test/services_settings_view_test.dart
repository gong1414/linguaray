import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_desktop/src/features/services/services_settings_view.dart';
import 'package:linguaray_desktop/src/shared/settings_labels.dart';
import 'package:linguaray_ui/linguaray_ui.dart';

void main() {
  testWidgets(
    'service editor keeps provider selection and trims a custom name',
    (tester) async {
      tester.view
        ..devicePixelRatio = 1
        ..physicalSize = const Size(440, 520);
      addTearDown(tester.view.reset);
      ServiceDraft? saved;
      await tester.pumpWidget(
        MaterialApp(
          theme: LinguaRayMaterialTheme.light(),
          home: ServiceEditorView(
            providers: const [
              ProviderRecord(
                id: 'first',
                typeId: 'deepl',
                displayName: 'First',
                publicFields: {},
                storedSecretKeys: {},
              ),
              ProviderRecord(
                id: 'second',
                typeId: 'deepl',
                displayName: 'Second',
                publicFields: {},
                storedSecretKeys: {},
              ),
            ],
            serviceKind: 'ocr',
            onSave: (value) => saved = value,
            onCancel: () {},
          ),
        ),
      );
      await tester.tap(find.byType(DropdownButtonFormField<String>));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Second').last);
      await tester.pumpAndSettle();
      await tester.enterText(find.byType(TextField), '  My OCR  ');
      await tester.tap(find.byType(FilledButton));
      expect(saved?.providerId, 'second');
      expect(saved?.kind, 'ocr');
      expect(saved?.name, 'My OCR');
      await tester.enterText(find.byType(TextField), ' ');
      await tester.tap(find.byType(FilledButton));
      expect(saved?.name, 'second ocr');
      expect(tester.takeException(), isNull);
    },
  );

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
