import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_desktop/src/data/runtime_history_repository.dart';
import 'package:linguaray_runtime/linguaray_runtime.dart' as rt;

void main() {
  late Directory dataDirectory;
  late RuntimeHistoryRepository repository;

  setUp(() async {
    dataDirectory = await Directory.systemTemp.createTemp(
      'linguaray-history-repository-',
    );
    final runtime = rt.Runtime(dataDir: dataDirectory.path);
    repository = RuntimeHistoryRepository(history: runtime.history());
  });

  tearDown(() async {
    if (dataDirectory.existsSync()) {
      await dataDirectory.delete(recursive: true);
    }
  });

  test('maps history CRUD, filters, queries, and counts', () async {
    final first = await repository.upsert(
      const HistoryRecordDraft(
        source: 'self attention',
        translation: '自注意力',
        sourceLanguage: 'en',
        targetLanguage: 'zh-Hans',
        serviceId: 'system+translation',
        serviceName: 'System',
        edited: false,
      ),
    );
    await repository.upsert(
      const HistoryRecordDraft(
        source: 'build failed',
        translation: '构建失败',
        sourceLanguage: 'en',
        targetLanguage: 'zh-Hans',
        serviceId: 'system+translation',
        serviceName: 'System',
        edited: true,
      ),
    );

    final favorite = await repository.setFavorite(
      entryId: first.id,
      favorite: true,
    );
    expect(favorite?.favorite, isTrue);

    final favorites = await repository.load(filter: HistoryFilter.favorites);
    expect(favorites.entries.single.id, first.id);
    expect(favorites.counts.all, 2);
    expect(favorites.counts.favorites, 1);
    expect(favorites.counts.edited, 1);

    final search = await repository.load(query: '构建');
    expect(search.entries.single.source, 'build failed');
    expect(search.query, '构建');

    expect(await repository.delete([first.id]), 1);
    expect((await repository.load()).entries, hasLength(1));
    expect(await repository.clear(), 1);
    expect((await repository.load()).entries, isEmpty);
  });
}
