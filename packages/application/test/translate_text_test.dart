import 'dart:async';

import 'package:linguaray_application/linguaray_application.dart';
import 'package:test/test.dart';

void main() {
  test('cancellation reaches all silent service subscriptions', () async {
    final repository = _SilentRepository();
    final stream = TranslateText(repository)(
      query: const TranslationQuery(
        text: 'hello',
        sourceLanguage: 'en',
        targetLanguage: 'zh-Hans',
      ),
      catalog: await repository.loadCatalog(),
    );
    final subscription = stream.listen((_) {});
    while (repository.started < 2) {
      await Future<void>.delayed(Duration.zero);
    }
    await subscription.cancel().timeout(const Duration(seconds: 1));
    expect(repository.cancelled, 2);
  });

  test('explicit language pair does not wait for language detection', () async {
    final repository = _FakeTranslationRepository(pendingDetection: true);
    final result = await TranslateText(repository)(
      query: const TranslationQuery(
        text: 'hello',
        sourceLanguage: 'en',
        targetLanguage: 'zh-Hans',
      ),
      catalog: await repository.loadCatalog(),
    ).last.timeout(const Duration(seconds: 1));
    expect(result.complete, isTrue);
    expect(repository.detections, 0);
  });

  test(
    'partial text and incomplete status survive a terminal failure',
    () async {
      final repository = _FakeTranslationRepository(incomplete: true);
      final result = await TranslateText(repository)(
        query: const TranslationQuery(
          text: 'hello',
          sourceLanguage: 'en',
          targetLanguage: 'zh-Hans',
        ),
        catalog: await repository.loadCatalog(),
      ).last;
      expect(result.results.last.text, '您好');
      expect(result.results.last.status, TranslationResultStatus.failed);
      expect(result.results.last.errorCode, 'translation_incomplete');
    },
  );

  test('streams independent service progress and completes', () async {
    final repository = _FakeTranslationRepository();
    final catalog = await LoadTranslationCatalog(repository)();

    final runs = await TranslateText(repository)(
      query: const TranslationQuery(
        text: 'hello',
        sourceLanguage: autoLanguageCode,
        targetLanguage: 'zh-Hans',
      ),
      catalog: catalog,
    ).toList();

    expect(runs, isNotEmpty);
    expect(runs.last.complete, isTrue);
    expect(runs.last.detectedLanguage, 'en');
    expect(repository.translatedSourceLanguages, everyElement('en'));
    expect(runs.last.targetLanguage, 'zh-Hans');
    expect(runs.last.results.map((result) => result.text), ['你好', '您好']);
    expect(
      runs.last.results.map((result) => result.status),
      everyElement(TranslationResultStatus.completed),
    );
  });

  test('one failed service does not discard successful results', () async {
    final repository = _FakeTranslationRepository(failSecond: true);
    final catalog = await LoadTranslationCatalog(repository)();

    final last = await TranslateText(repository)(
      query: const TranslationQuery(
        text: 'hello',
        sourceLanguage: autoLanguageCode,
        targetLanguage: 'zh-Hans',
      ),
      catalog: catalog,
    ).last;

    expect(last.results.first.text, '你好');
    expect(last.results.first.status, TranslationResultStatus.completed);
    expect(last.results.last.status, TranslationResultStatus.failed);
    expect(last.results.last.errorCode, 'network_failure');
  });
}

final class _FakeTranslationRepository implements TranslationRepository {
  _FakeTranslationRepository({
    this.failSecond = false,
    this.pendingDetection = false,
    this.incomplete = false,
  });

  final bool pendingDetection;
  final bool incomplete;
  int detections = 0;

  final bool failSecond;
  final List<String> translatedSourceLanguages = [];

  @override
  Future<TranslationCatalog> loadCatalog() async => const TranslationCatalog(
    languages: [
      LanguageOption(code: 'en', name: 'English'),
      LanguageOption(code: 'zh-Hans', name: '简体中文'),
    ],
    services: [
      TranslationServiceOption(
        id: 'system',
        name: 'System',
        isStreaming: false,
      ),
      TranslationServiceOption(id: 'cloud', name: 'Cloud', isStreaming: true),
    ],
    defaultSourceLanguage: autoLanguageCode,
    defaultTargetLanguage: 'zh-Hans',
  );

  @override
  Future<String?> detectLanguage({
    required String serviceId,
    required String text,
  }) async {
    detections++;
    return pendingDetection ? Completer<String?>().future : 'en';
  }

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
    translatedSourceLanguages.add(sourceLanguage);
    if (service.id == 'cloud' && failSecond) {
      throw const TranslationFailure('offline');
    }
    yield service.id == 'system' ? '你好' : '您';
    if (service.id == 'cloud') {
      yield '好';
      if (incomplete) throw const TranslationFailure('translation_incomplete');
    }
  }
}

final class _SilentRepository extends _FakeTranslationRepository {
  int started = 0;
  int cancelled = 0;
  @override
  Stream<String> translate({
    required TranslationServiceOption service,
    required String text,
    required String sourceLanguage,
    required String targetLanguage,
  }) {
    return StreamController<String>(
      onListen: () => started++,
      onCancel: () => cancelled++,
    ).stream;
  }
}
