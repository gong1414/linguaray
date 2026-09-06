import 'package:linguaray_application/src/dictionary/models.dart';
import 'package:linguaray_application/src/dictionary/ports.dart';

final class LookUpWord {
  const LookUpWord(this._repository);

  final DictionaryRepository _repository;

  Future<DictionaryEntry?> call(DictionaryLookupQuery query) async {
    final word = query.word.trim();
    if (word.isEmpty) return null;
    final services = await _repository.listCompatibleServiceIds();
    if (services.isEmpty) return null;
    return _repository.lookup(
      DictionaryLookupQuery(
        word: word,
        sourceLanguage: query.sourceLanguage,
        targetLanguage: query.targetLanguage,
        serviceId: query.serviceId ?? services.first,
      ),
    );
  }

  Future<bool> get isAvailable async =>
      (await _repository.listCompatibleServiceIds()).isNotEmpty;
}
