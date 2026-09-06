import '../../../app/runtime.dart';
import '../backup_port.dart';

final class RuntimeBackupArchive implements BackupArchive {
  const RuntimeBackupArchive();

  @override
  Future<void> exportTo({required String destinationPath}) =>
      runtime.backup().exportTo(destinationPath: destinationPath);

  @override
  Future<void> restoreFrom({required String sourcePath}) =>
      runtime.backup().restoreFrom(sourcePath: sourcePath);
}
