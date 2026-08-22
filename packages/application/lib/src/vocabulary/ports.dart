import 'package:linguaray_application/src/vocabulary/models.dart';

abstract interface class VocabularyRepository {
  Future<VocabularySnapshot> load({
    VocabularyFilter filter = VocabularyFilter.all,
    String query = '',
  });

  Future<VocabularyRecord> upsert(VocabularyDraft draft);

  Future<VocabularyRecord?> setFavorite({
    required String entryId,
    required bool favorite,
  });

  Future<VocabularyRecord?> updateNote({required String entryId, String? note});

  Future<int> delete(List<String> entryIds);
}
