import 'package:linguaray_application/src/translation/models.dart';

abstract interface class TranslationRepository {
  Future<TranslationCatalog> loadCatalog();

  Future<String?> detectLanguage({
    required String serviceId,
    required String text,
  });

  Future<String> resolveTarget({
    required String? selectedTarget,
    required String fallbackTarget,
    required String? detectedLanguage,
  });

  Stream<String> translate({
    required TranslationServiceOption service,
    required String text,
    required String sourceLanguage,
    required String targetLanguage,
  });
}
