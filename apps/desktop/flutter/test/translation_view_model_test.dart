import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_desktop/src/config/dependencies.dart';
import 'package:linguaray_desktop/src/ui/translation/view_models/translation_view_model.dart';

void main() {
  test('loads catalog and exposes only application models', () async {
    final container = ProviderContainer(
      overrides: [
        translationRepositoryProvider.overrideWithValue(
          _FakeTranslationRepository(),
        ),
      ],
    );
    addTearDown(container.dispose);
    final subscription = container.listen(
      translationViewModelProvider,
      (_, _) {},
      fireImmediately: true,
    );
    addTearDown(subscription.close);

    await _waitFor(
      () => !container.read(translationViewModelProvider).loadingCatalog,
    );

    final state = container.read(translationViewModelProvider);
    expect(state.catalog, isA<TranslationCatalog>());
    expect(state.languages.single.name, 'English');
    expect(state.services.single.name, 'Local stub');
    expect(state.selectedServiceId, 'stub');
    expect(state.targetLanguage, automaticTargetCode);
  });

  test('submits through the port and publishes completed result', () async {
    final container = ProviderContainer(
      overrides: [
        translationRepositoryProvider.overrideWithValue(
          _FakeTranslationRepository(),
        ),
      ],
    );
    addTearDown(container.dispose);
    final subscription = container.listen(
      translationViewModelProvider,
      (_, _) {},
      fireImmediately: true,
    );
    addTearDown(subscription.close);
    await _waitFor(
      () => !container.read(translationViewModelProvider).loadingCatalog,
    );

    final viewModel = container.read(translationViewModelProvider.notifier);
    viewModel.setSourceText('Hello');
    await viewModel.submit();

    final state = container.read(translationViewModelProvider);
    expect(state.submitting, isFalse);
    expect(state.run?.detectedLanguage, 'en');
    expect(state.run?.targetLanguage, 'zh-Hans');
    expect(state.selectedResult?.text, '你好');
    expect(state.selectedResult?.status, TranslationResultStatus.completed);
  });
}

Future<void> _waitFor(bool Function() condition) async {
  for (var attempt = 0; attempt < 50; attempt++) {
    if (condition()) return;
    await Future<void>.delayed(const Duration(milliseconds: 1));
  }
  fail('Timed out waiting for view-model state.');
}

final class _FakeTranslationRepository implements TranslationRepository {
  @override
  Future<TranslationCatalog> loadCatalog() async => const TranslationCatalog(
    languages: [LanguageOption(code: 'en', name: 'English')],
    services: [
      TranslationServiceOption(
        id: 'stub',
        name: 'Local stub',
        isStreaming: false,
      ),
    ],
    defaultSourceLanguage: autoLanguageCode,
    defaultTargetLanguage: 'zh-Hans',
  );

  @override
  Future<String?> detectLanguage({
    required String serviceId,
    required String text,
  }) async => 'en';

  @override
  Future<String> resolveTarget({
    required String? selectedTarget,
    required String fallbackTarget,
    required String? detectedLanguage,
  }) async => selectedTarget ?? fallbackTarget;

  @override
  Stream<String> translate({
    required TranslationServiceOption service,
    required String text,
    required String sourceLanguage,
    required String targetLanguage,
  }) async* {
    yield '你好';
  }
}
