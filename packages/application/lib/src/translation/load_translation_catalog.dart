import 'package:linguaray_application/src/translation/models.dart';
import 'package:linguaray_application/src/translation/ports.dart';

final class LoadTranslationCatalog {
  const LoadTranslationCatalog(this._repository);

  final TranslationRepository _repository;

  Future<TranslationCatalog> call() => _repository.loadCatalog();
}
