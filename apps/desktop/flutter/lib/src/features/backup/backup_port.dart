abstract interface class BackupArchive {
  Future<void> exportTo({required String destinationPath});

  Future<void> restoreFrom({required String sourcePath});
}

abstract interface class BackupFilePicker {
  Future<String?> pickExportPath({required String suggestedName});

  Future<String?> pickRestorePath();
}
