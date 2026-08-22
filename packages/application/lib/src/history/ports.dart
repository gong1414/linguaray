import 'package:linguaray_application/src/history/models.dart';

abstract interface class HistoryRepository {
  Future<HistorySnapshot> load({
    HistoryFilter filter = HistoryFilter.all,
    String query = '',
  });

  Future<HistoryRecord> upsert(HistoryRecordDraft draft);

  Future<HistoryRecord?> setFavorite({
    required String entryId,
    required bool favorite,
  });

  Future<int> delete(List<String> entryIds);

  Future<int> clear();
}
