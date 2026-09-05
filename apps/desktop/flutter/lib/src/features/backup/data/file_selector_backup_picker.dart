import 'package:file_selector/file_selector.dart';

import '../backup_port.dart';

final class FileSelectorBackupPicker implements BackupFilePicker {
  const FileSelectorBackupPicker();

  static const _zip = XTypeGroup(
    label: 'LinguaRay backup',
    extensions: ['zip'],
    uniformTypeIdentifiers: ['public.zip-archive'],
  );

  @override
  Future<String?> pickExportPath({required String suggestedName}) async {
    final destination = await getSaveLocation(
      suggestedName: suggestedName,
      acceptedTypeGroups: const [_zip],
    );
    return destination?.path;
  }

  @override
  Future<String?> pickRestorePath() async {
    final source = await openFile(acceptedTypeGroups: const [_zip]);
    return source?.path;
  }
}
