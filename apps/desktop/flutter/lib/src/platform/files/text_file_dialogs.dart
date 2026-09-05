import 'dart:convert';

import 'package:file_selector/file_selector.dart';

/// Native text-file exchange. Cancellation is represented explicitly; failures
/// propagate to the feature controller so the view can localize the outcome.
class TextFileDialogs {
  const TextFileDialogs();

  Future<String?> open({required String extension}) async {
    final file = await openFile(acceptedTypeGroups: [_type(extension)]);
    return file?.readAsString();
  }

  Future<bool> save({
    required String suggestedName,
    required String extension,
    required String mimeType,
    required Future<String> Function() content,
  }) async {
    final location = await getSaveLocation(
      suggestedName: suggestedName,
      acceptedTypeGroups: [_type(extension)],
    );
    if (location == null) return false;
    await XFile.fromData(
      utf8.encode(await content()),
      mimeType: mimeType,
      name: location.path.split(RegExp(r'[/\\]')).last,
    ).saveTo(location.path);
    return true;
  }

  XTypeGroup _type(String extension) =>
      XTypeGroup(label: extension.toUpperCase(), extensions: [extension]);
}
