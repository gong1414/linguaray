import 'package:linguaray_application/linguaray_application.dart';

import '../../../platform/files/text_file_dialogs.dart';

final class GlossaryExchangeController {
  const GlossaryExchangeController(this._repository, this._files);
  final GlossaryRepository _repository;
  final TextFileDialogs _files;

  Future<GlossaryImportSummary?> importBook(
    String bookId,
    GlossaryExchangeFormat format,
  ) async {
    final content = await _files.open(extension: format.name);
    if (content == null) return null;
    return _repository.importEntries(
      bookId: bookId,
      content: content,
      format: format,
    );
  }

  Future<bool> exportBook(
    GlossaryBookRecord book,
    GlossaryExchangeFormat format,
  ) {
    final safeName = book.name
        .replaceAll(RegExp(r'[<>:"/\\|?*\x00-\x1F]'), '-')
        .replaceAll(RegExp(r'^-+|-+$'), '');
    return _files.save(
      suggestedName:
          '${safeName.isEmpty ? 'linguaray-glossary' : safeName}.${format.name}',
      extension: format.name,
      mimeType: format == GlossaryExchangeFormat.csv
          ? 'text/csv'
          : 'application/xml',
      content: () => _repository.exportEntries(bookId: book.id, format: format),
    );
  }
}
