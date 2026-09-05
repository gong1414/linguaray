import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../app/dependencies.dart';

/// Read-only previews mount the production screens against isolated data.
/// They never initialize the runtime or read the user's libraries/settings.
class CatalogSettingsFixture extends StatelessWidget {
  const CatalogSettingsFixture({
    required this.child,
    this.empty = false,
    super.key,
  });
  final Widget child;
  final bool empty;

  @override
  Widget build(BuildContext context) {
    final repository = _PreviewRepository(empty);
    return ProviderScope(
      overrides: [
        glossaryRepositoryProvider.overrideWithValue(repository),
        vocabularyRepositoryProvider.overrideWithValue(repository),
        workspaceSettingsRepositoryProvider.overrideWithValue(repository),
      ],
      child: ExcludeFocus(child: IgnorePointer(child: child)),
    );
  }
}

class _PreviewRepository
    implements
        GlossaryRepository,
        VocabularyRepository,
        WorkspaceSettingsRepository {
  _PreviewRepository(this.empty);
  final bool empty;

  @override
  Future<List<GlossaryBookRecord>> listBooks() async => empty
      ? []
      : const [
          GlossaryBookRecord(
            id: 'desktop',
            name: '桌面软件',
            enabled: true,
            entryCount: 2,
          ),
          GlossaryBookRecord(
            id: 'reading',
            name: '阅读笔记',
            enabled: true,
            entryCount: 0,
          ),
        ];

  @override
  Future<List<GlossaryEntryRecord>> listEntries({
    required String bookId,
    String query = '',
  }) async => bookId != 'desktop'
      ? []
      : const [
          GlossaryEntryRecord(
            id: '1',
            term: 'menu bar',
            translation: '菜单栏',
            forbidden: [],
            caseSensitive: false,
            wholeWord: true,
            hits: 12,
          ),
          GlossaryEntryRecord(
            id: '2',
            term: 'keyboard shortcut',
            translation: '快捷键',
            forbidden: [],
            caseSensitive: false,
            wholeWord: true,
            hits: 8,
          ),
        ];

  @override
  Future<VocabularySnapshot> load({
    VocabularyFilter filter = VocabularyFilter.all,
    String query = '',
  }) async => VocabularySnapshot(
    filter: filter,
    query: query,
    entries: empty
        ? []
        : const [
            VocabularyRecord(
              id: '1',
              word: 'intuitive',
              translation: '直观的；易于理解的',
              sourceLanguage: 'en',
              targetLanguage: 'zh-Hans',
              source: 'dictionary',
              favorite: true,
              createdAt: 0,
              updatedAt: 0,
            ),
            VocabularyRecord(
              id: '2',
              word: 'consistent',
              translation: '一致的；连贯的',
              sourceLanguage: 'en',
              targetLanguage: 'zh-Hans',
              source: 'dictionary',
              favorite: false,
              createdAt: 0,
              updatedAt: 0,
            ),
          ],
  );

  @override
  Future<ApiServerStatus> loadApiServer() async =>
      const ApiServerStatus(enabled: false, host: '127.0.0.1', port: 60828);

  @override
  Future<NetworkSettings> loadNetworkSettings() async => const NetworkSettings(
    proxyMode: NetworkProxyMode.system,
    proxyUrl: '',
    proxyBypass: '',
    checkUpdatesOnLaunch: true,
  );

  // A catalog fixture must fail if a preview accidentally invokes a write or
  // requests another capability, instead of falling through to real storage.
  @override
  dynamic noSuchMethod(Invocation invocation) => throw UnsupportedError(
    'Read-only catalog fixture: ${invocation.memberName}',
  );
}
