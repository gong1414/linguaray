import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../app/dependencies.dart';

enum DataTransferOperation {
  idle,
  exporting,
  exported,
  restoring,
  restored,
  failed,
}

final dataTransferViewModelProvider =
    NotifierProvider<DataTransferViewModel, DataTransferViewState>(
      DataTransferViewModel.new,
    );

final class DataTransferViewState {
  const DataTransferViewState({
    this.operation = DataTransferOperation.idle,
    this.selectedPath,
    this.error,
  });

  final DataTransferOperation operation;
  final String? selectedPath;
  final String? error;

  bool get busy =>
      operation == DataTransferOperation.exporting ||
      operation == DataTransferOperation.restoring;
}

final class DataTransferViewModel extends Notifier<DataTransferViewState> {
  @override
  DataTransferViewState build() => const DataTransferViewState();

  Future<void> exportBackup() async {
    final destination = await ref
        .read(backupFilePickerProvider)
        .pickExportPath(suggestedName: _backupFileName());
    if (destination == null) return;
    state = DataTransferViewState(
      operation: DataTransferOperation.exporting,
      selectedPath: destination,
    );
    try {
      await ref
          .read(backupArchiveProvider)
          .exportTo(destinationPath: destination);
      state = DataTransferViewState(
        operation: DataTransferOperation.exported,
        selectedPath: destination,
      );
    } catch (caught) {
      state = DataTransferViewState(
        operation: DataTransferOperation.failed,
        selectedPath: destination,
        error: caught.toString(),
      );
    }
  }

  Future<void> restoreBackup({Future<void> Function()? afterRestore}) async {
    final source = await ref.read(backupFilePickerProvider).pickRestorePath();
    if (source == null) return;
    state = DataTransferViewState(
      operation: DataTransferOperation.restoring,
      selectedPath: source,
    );
    try {
      await ref.read(backupArchiveProvider).restoreFrom(sourcePath: source);
      if (afterRestore != null) {
        await afterRestore();
      } else {
        await ref.read(backupRestoreEffectsProvider).call();
      }
      state = DataTransferViewState(
        operation: DataTransferOperation.restored,
        selectedPath: source,
      );
    } catch (caught) {
      state = DataTransferViewState(
        operation: DataTransferOperation.failed,
        selectedPath: source,
        error: caught.toString(),
      );
    }
  }

  String _backupFileName() {
    final now = DateTime.now();
    String two(int value) => value.toString().padLeft(2, '0');
    return 'LinguaRay-backup-${now.year}${two(now.month)}${two(now.day)}-'
        '${two(now.hour)}${two(now.minute)}.zip';
  }
}
