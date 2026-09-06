import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_desktop/src/app/dependencies.dart';
import 'package:linguaray_desktop/src/features/backup/backup_port.dart';
import 'package:linguaray_desktop/src/features/backup/data_transfer_view_model.dart';

void main() {
  test('data transfer export and restore stay in the view model', () async {
    final archive = _FakeArchive();
    final picker = _FakePicker()
      ..exportPath = '/tmp/backup.zip'
      ..restorePath = '/tmp/restore.zip';
    var restored = 0;
    final container = ProviderContainer(
      overrides: [
        backupArchiveProvider.overrideWithValue(archive),
        backupFilePickerProvider.overrideWithValue(picker),
        backupRestoreEffectsProvider.overrideWithValue(() async {
          restored += 1;
        }),
      ],
    );
    addTearDown(container.dispose);
    final subscription = container.listen(
      dataTransferViewModelProvider,
      (_, _) {},
      fireImmediately: true,
    );
    addTearDown(subscription.close);

    await container.read(dataTransferViewModelProvider.notifier).exportBackup();
    expect(
      container.read(dataTransferViewModelProvider).operation,
      DataTransferOperation.exported,
    );
    expect(archive.exported, ['/tmp/backup.zip']);

    picker.exportPath = null;
    await container.read(dataTransferViewModelProvider.notifier).exportBackup();
    expect(
      container.read(dataTransferViewModelProvider).operation,
      DataTransferOperation.exported,
    );

    archive.exportError = StateError('disk');
    picker.exportPath = '/tmp/fail.zip';
    await container.read(dataTransferViewModelProvider.notifier).exportBackup();
    expect(
      container.read(dataTransferViewModelProvider).operation,
      DataTransferOperation.failed,
    );

    await container
        .read(dataTransferViewModelProvider.notifier)
        .restoreBackup();
    expect(
      container.read(dataTransferViewModelProvider).operation,
      DataTransferOperation.restored,
    );
    expect(archive.restored, ['/tmp/restore.zip']);
    expect(restored, 1);

    archive.restoreError = StateError('corrupt');
    picker.restorePath = '/tmp/bad.zip';
    await container
        .read(dataTransferViewModelProvider.notifier)
        .restoreBackup();
    expect(
      container.read(dataTransferViewModelProvider).operation,
      DataTransferOperation.failed,
    );
    expect(restored, 1);
  });
}

final class _FakeArchive implements BackupArchive {
  final exported = <String>[];
  final restored = <String>[];
  Object? exportError;
  Object? restoreError;

  @override
  Future<void> exportTo({required String destinationPath}) async {
    if (exportError != null) throw exportError!;
    exported.add(destinationPath);
  }

  @override
  Future<void> restoreFrom({required String sourcePath}) async {
    if (restoreError != null) throw restoreError!;
    restored.add(sourcePath);
  }
}

final class _FakePicker implements BackupFilePicker {
  String? exportPath;
  String? restorePath;

  @override
  Future<String?> pickExportPath({required String suggestedName}) async =>
      exportPath;

  @override
  Future<String?> pickRestorePath() async => restorePath;
}
