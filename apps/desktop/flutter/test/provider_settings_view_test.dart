import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_desktop/src/features/providers/provider_labels.dart';
import 'package:linguaray_desktop/src/features/providers/providers_settings_view.dart';
import 'package:linguaray_desktop/src/shared/settings_labels.dart';

void main() {
  testWidgets(
    'model search reaches IDs beyond the first sixteen without mixing references',
    (tester) async {
      tester.view.physicalSize = const Size(1000, 1000);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.reset);
      String? selected;
      await tester.pumpWidget(
        MaterialApp(
          home: ProviderEditorView(
            labels: _labels,
            types: _types,
            draftId: 'openrouter',
            typeId: 'openrouter',
            fields: const {},
            storedSecretKeys: const {},
            testing: false,
            testResult: null,
            saving: false,
            operationError: null,
            discovery: ProviderModelDiscovery(
              liveModels: List.generate(100, (i) => 'vendor/model-$i'),
              referenceModels: const ['offline/only'],
              queriedAt: DateTime(2026, 9, 5, 12),
            ),
            onFetchModels: () {},
            onIdChanged: (_) {},
            onTypeChanged: (_) {},
            onFieldChanged: (key, value) => selected = value,
            onTest: () {},
            onSave: () {},
            onCancel: () {},
          ),
        ),
      );
      final search = find.byKey(
        const ValueKey('provider-model-search-openrouter'),
      );
      await tester.ensureVisible(search);
      await tester.enterText(search, 'model-99');
      await tester.pump();
      await tester.ensureVisible(find.text('vendor/model-99'));
      await tester.tap(find.text('vendor/model-99'));
      expect(selected, 'vendor/model-99');
      expect(find.text('offline/only'), findsNothing);
      expect(tester.takeException(), isNull);
    },
  );

  test('new presets keep the model endpoint derived from the API root', () {
    const type = ProviderTypeOption(
      id: 'x',
      label: 'X',
      isLlm: true,
      fields: [],
      baseUrl: 'https://old.example/v1',
      modelsUrl: 'https://old.example/v1/models',
    );
    expect(providerPresetInitialFields(type), {
      'baseUrl': 'https://old.example/v1',
    });
  });

  testWidgets('active provider editor starts with the catalog picker', (
    tester,
  ) async {
    String? selected;
    await tester.pumpWidget(
      MaterialApp(
        home: ProviderEditorView(
          labels: _labels,
          types: _types,
          draftId: '',
          typeId: '',
          fields: const {},
          storedSecretKeys: const {},
          testing: false,
          testResult: null,
          saving: false,
          operationError: null,
          onIdChanged: (_) {},
          onTypeChanged: (value) => selected = value,
          onFieldChanged: (_, _) {},
          onTest: () {},
          onSave: () {},
          onCancel: () {},
        ),
      ),
    );

    expect(
      find.byKey(const ValueKey('provider-preset-search')),
      findsOneWidget,
    );
    expect(find.text('OpenRouter'), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('provider-preset-openrouter')));
    expect(selected, 'openrouter');
  });

  testWidgets('selected preset exposes model discovery and advanced fields', (
    tester,
  ) async {
    var fetches = 0;
    await tester.pumpWidget(
      MaterialApp(
        home: ProviderEditorView(
          labels: _labels,
          types: _types,
          draftId: 'openrouter',
          typeId: 'openrouter',
          fields: const {'baseUrl': 'https://openrouter.ai/api/v1'},
          storedSecretKeys: const {},
          testing: false,
          testResult: null,
          saving: false,
          operationError: null,
          discovery: const ProviderModelDiscovery(liveModels: ['model-a']),
          onFetchModels: () => fetches++,
          onIdChanged: (_) {},
          onTypeChanged: (_) {},
          onFieldChanged: (_, _) {},
          onTest: () {},
          onSave: () {},
          onCancel: () {},
        ),
      ),
    );

    expect(
      find.byKey(const ValueKey('provider-field-defaultModel')),
      findsOneWidget,
    );
    await tester.tap(find.byIcon(Icons.refresh_rounded));
    expect(fetches, 1);
    expect(find.byKey(const ValueKey('provider-field-baseUrl')), findsNothing);
    await tester.ensureVisible(find.byIcon(Icons.expand_more_rounded));
    await tester.tap(find.byIcon(Icons.expand_more_rounded));
    await tester.pump();
    expect(
      find.byKey(const ValueKey('provider-field-baseUrl')),
      findsOneWidget,
    );
    expect(find.text('model-a'), findsOneWidget);
  });

  testWidgets('catalog search filters the active provider picker', (
    tester,
  ) async {
    await tester.pumpWidget(
      MaterialApp(
        home: ProviderEditorView(
          labels: _labels,
          types: _types,
          draftId: '',
          typeId: '',
          fields: const {},
          storedSecretKeys: const {},
          testing: false,
          testResult: null,
          saving: false,
          operationError: null,
          onIdChanged: (_) {},
          onTypeChanged: (_) {},
          onFieldChanged: (_, _) {},
          onTest: () {},
          onSave: () {},
          onCancel: () {},
        ),
      ),
    );

    await tester.enterText(
      find.byKey(const ValueKey('provider-preset-search')),
      'openrouter',
    );
    await tester.pump();
    expect(find.text('OpenRouter'), findsOneWidget);
    expect(find.text('Google Web'), findsNothing);
  });

  test(
    'catalog helpers retain endpoint metadata and exact preset identity',
    () {
      const deeplFree = ProviderTypeOption(
        id: 'deepl-free',
        engineTypeId: 'deepl',
        label: 'DeepL Free',
        isLlm: false,
        baseUrl: 'https://api-free.deepl.com',
        fields: [],
      );
      expect(providerPresetInitialFields(deeplFree), {
        'baseUrl': 'https://api-free.deepl.com',
      });
      expect(kCatalogCategoryOrder, [
        'builtIn',
        'traditionalApi',
        'llmOfficial',
        'aggregator',
        'localOrSelfHosted',
      ]);
      expect(
        findProviderCatalogOption(
          _types,
          presetId: 'openrouter',
          engineTypeId: 'openai_compatible',
        )?.id,
        'openrouter',
      );
    },
  );
}

const _types = [
  ProviderTypeOption(
    id: 'google-web',
    engineTypeId: 'google_web',
    label: 'Google Web',
    isLlm: false,
    category: 'builtIn',
    fields: [],
  ),
  ProviderTypeOption(
    id: 'openrouter',
    engineTypeId: 'openai_compatible',
    label: 'OpenRouter',
    isLlm: true,
    category: 'aggregator',
    fields: [
      ProviderFieldSpec(
        key: 'apiKey',
        label: 'API key',
        secret: true,
        requiredField: true,
      ),
      ProviderFieldSpec(
        key: 'defaultModel',
        label: 'Model',
        secret: false,
        requiredField: true,
      ),
      ProviderFieldSpec(
        key: 'baseUrl',
        label: 'Base URL',
        secret: false,
        requiredField: false,
        advanced: true,
      ),
    ],
  ),
];

const _labels = ProvidersSettingsLabels(
  title: 'Providers',
  empty: 'Empty',
  loading: 'Loading',
  add: 'Add provider',
  edit: 'Edit provider',
  delete: 'Delete',
  deleteConfirmTitle: 'Delete?',
  deleteConfirmBody: 'Delete provider?',
  secretStored: 'Stored',
  secretPlaceholder: 'Stored secret',
  save: 'Save',
  cancel: 'Cancel',
  test: 'Test',
  testing: 'Testing',
  testPassed: 'Passed',
  testFailed: 'Failed',
  idLabel: 'Provider ID',
  typeLabel: 'Provider type',
  validationMissing: 'Missing',
  saveFailed: 'Save failed',
);
