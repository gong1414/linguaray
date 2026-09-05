import 'dart:async';

import 'package:linguaray_application/linguaray_application.dart';

const service = TranslationServiceOption(
  id: 'stub',
  name: 'Stub',
  isStreaming: false,
);
const catalog = TranslationCatalog(
  languages: [],
  services: [service],
  defaultSourceLanguage: 'en',
  defaultTargetLanguage: 'zh-Hans',
);

final class Stub implements TranslationRepository {
  final detection = Completer<String?>();
  int detections = 0;
  int translations = 0;
  @override
  Future<TranslationCatalog> loadCatalog() async => catalog;
  @override
  Future<String?> detectLanguage({
    required String serviceId,
    required String text,
  }) {
    detections++;
    return detection.future;
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
    translations++;
    yield '你好';
  }
}

Future<void> main() async {
  final repo = Stub();
  var events = 0;
  final done = Completer<void>();
  TranslateText(repo)(
    query: const TranslationQuery(
      text: 'Hello',
      sourceLanguage: 'en',
      targetLanguage: 'zh-Hans',
    ),
    catalog: catalog,
  ).listen((_) => events++, onDone: done.complete);
  await Future<void>.delayed(const Duration(milliseconds: 100));
  print(
    'Explicit en -> zh-Hans, detection pending: detections=${repo.detections}, translations=${repo.translations}, UI events=$events',
  );
  repo.detection.complete('en');
  await done.future;
  print(
    'After detection resolves: translations=${repo.translations}, UI events=$events',
  );
}
