import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../../app/dependencies.dart';

final glossaryViewModelProvider =
    NotifierProvider<GlossaryViewModel, GlossaryViewState>(
      GlossaryViewModel.new,
    );

final class GlossaryViewState {
  const GlossaryViewState({
    this.books = const [],
    this.entries = const [],
    this.selectedBookId,
    this.query = '',
    this.loading = true,
    this.errorCode,
  });

  final List<GlossaryBookRecord> books;
  final List<GlossaryEntryRecord> entries;
  final String? selectedBookId;
  final String query;
  final bool loading;
  final String? errorCode;

  GlossaryBookRecord? get selectedBook =>
      books.where((book) => book.id == selectedBookId).firstOrNull;

  GlossaryViewState copyWith({
    List<GlossaryBookRecord>? books,
    List<GlossaryEntryRecord>? entries,
    Object? selectedBookId = _unset,
    String? query,
    bool? loading,
    Object? errorCode = _unset,
  }) {
    return GlossaryViewState(
      books: books ?? this.books,
      entries: entries ?? this.entries,
      selectedBookId: identical(selectedBookId, _unset)
          ? this.selectedBookId
          : selectedBookId as String?,
      query: query ?? this.query,
      loading: loading ?? this.loading,
      errorCode: identical(errorCode, _unset)
          ? this.errorCode
          : errorCode as String?,
    );
  }
}

const _unset = Object();

final class GlossaryViewModel extends Notifier<GlossaryViewState> {
  @override
  GlossaryViewState build() {
    scheduleMicrotask(reloadBooks);
    return const GlossaryViewState();
  }

  GlossaryRepository get _repository => ref.read(glossaryRepositoryProvider);

  Future<void> reloadBooks({String? select}) async {
    state = state.copyWith(loading: true);
    try {
      final loadedBooks = await _repository.listBooks();
      final books = loadedBooks
          .where((book) => book.errorCode == null)
          .toList(growable: false);
      final requested = select ?? state.selectedBookId;
      final selected = books.any((book) => book.id == requested)
          ? requested
          : books.firstOrNull?.id;
      state = state.copyWith(
        books: books,
        selectedBookId: selected,
        errorCode: loadedBooks.any((book) => book.errorCode != null)
            ? AppErrorCode.glossaryCorrupt.wireName
            : null,
      );
      await reloadEntries();
    } catch (_) {
      state = state.copyWith(
        loading: false,
        errorCode: AppErrorCode.glossaryCorrupt.wireName,
      );
    }
  }

  Future<void> reloadEntries() async {
    final bookId = state.selectedBookId;
    if (bookId == null) {
      state = state.copyWith(entries: const [], loading: false);
      return;
    }
    final query = state.query;
    try {
      final entries = await _repository.listEntries(
        bookId: bookId,
        query: query,
      );
      if (state.selectedBookId != bookId || state.query != query) return;
      state = state.copyWith(entries: entries, loading: false);
    } catch (_) {
      if (state.selectedBookId != bookId || state.query != query) return;
      state = state.copyWith(
        entries: const [],
        loading: false,
        errorCode: AppErrorCode.glossaryCorrupt.wireName,
      );
    }
  }

  Future<void> selectBook(String id) async {
    if (state.selectedBookId == id) return;
    state = state.copyWith(selectedBookId: id, loading: true);
    await reloadEntries();
  }

  Future<void> setQuery(String query) async {
    state = state.copyWith(query: query);
    await reloadEntries();
  }

  Future<List<LanguageOption>> loadLanguages() async {
    try {
      return (await ref.read(loadTranslationCatalogProvider)()).languages;
    } catch (_) {
      return const [];
    }
  }

  Future<void> upsertBook(GlossaryBookDraft draft) async {
    final saved = await _repository.upsertBook(draft);
    await reloadBooks(select: saved.id);
  }

  Future<void> toggleBook(GlossaryBookRecord book) {
    return upsertBook(
      GlossaryBookDraft(
        id: book.id,
        name: book.name,
        enabled: !book.enabled,
        sourceLanguage: book.sourceLanguage,
        targetLanguage: book.targetLanguage,
      ),
    );
  }

  Future<void> deleteBook(String bookId) async {
    await _repository.deleteBook(bookId);
    await reloadBooks();
  }

  Future<void> upsertEntry(GlossaryEntryDraft draft) async {
    final bookId = state.selectedBookId;
    if (bookId == null) return;
    await _repository.upsertEntry(bookId: bookId, draft: draft);
    await reloadEntries();
  }

  Future<void> deleteEntry(String entryId) async {
    final bookId = state.selectedBookId;
    if (bookId == null) return;
    await _repository.deleteEntry(bookId: bookId, entryId: entryId);
    await reloadEntries();
  }

  Future<GlossaryImportSummary?> importBook(
    GlossaryBookRecord book,
    GlossaryExchangeFormat format,
  ) async {
    final report = await ref
        .read(glossaryExchangeControllerProvider)
        .importBook(book.id, format);
    if (report == null) return null;
    await reloadBooks(select: book.id);
    return report;
  }

  Future<bool> exportBook(
    GlossaryBookRecord book,
    GlossaryExchangeFormat format,
  ) {
    return ref
        .read(glossaryExchangeControllerProvider)
        .exportBook(book, format);
  }
}
