import 'package:linguaray_application/src/dictionary/models.dart';

abstract interface class DictionaryRepository {
  Future<List<String>> listCompatibleServiceIds();

  Future<DictionaryEntry> lookup(DictionaryLookupQuery query);
}
