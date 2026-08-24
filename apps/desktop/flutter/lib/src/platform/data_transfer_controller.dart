import 'package:file_selector/file_selector.dart';
import 'package:flutter/foundation.dart';

import '../services/runtime.dart';
import '../services/settings_store.dart';
import '../services/system_proxy.dart';
import 'secret_store.dart';

enum DataTransferOperation {
  idle,
  exporting,
  exported,
  restoring,
  restored,
  failed,
}

class DataTransferController extends ChangeNotifier {
  DataTransferOperation operation = DataTransferOperation.idle;
  String? selectedPath;
  String? error;

  Future<void> exportBackup() async {
    final destination = await getSaveLocation(
      suggestedName: _backupFileName(),
      acceptedTypeGroups: const [
        XTypeGroup(
          label: 'LinguaRay backup',
          extensions: ['zip'],
          uniformTypeIdentifiers: ['public.zip-archive'],
        ),
      ],
    );
    if (destination == null) return;
    operation = DataTransferOperation.exporting;
    selectedPath = destination.path;
    error = null;
    notifyListeners();
    try {
      await runtime.backup().exportTo(destinationPath: destination.path);
      operation = DataTransferOperation.exported;
    } catch (caught) {
      operation = DataTransferOperation.failed;
      error = caught.toString();
    }
    notifyListeners();
  }

  Future<void> restoreBackup() async {
    const type = XTypeGroup(
      label: 'LinguaRay backup',
      extensions: ['zip'],
      uniformTypeIdentifiers: ['public.zip-archive'],
    );
    final source = await openFile(acceptedTypeGroups: [type]);
    if (source == null) return;
    operation = DataTransferOperation.restoring;
    selectedPath = source.path;
    error = null;
    notifyListeners();
    try {
      await runtime.backup().restoreFrom(sourcePath: source.path);
      await providerCredentialsController.hydrateAll();
      await Future.wait([
        settingsStore.reloadAppearance(),
        settingsStore.reloadGeneral(),
        settingsStore.reloadShortcuts(),
        settingsStore.reloadAdvanced(),
        settingsStore.reloadProviders(),
        settingsStore.reloadServices(),
      ]);
      await initializeSystemProxy();
      operation = DataTransferOperation.restored;
    } catch (caught) {
      operation = DataTransferOperation.failed;
      error = caught.toString();
    }
    notifyListeners();
  }

  String _backupFileName() {
    final now = DateTime.now();
    String two(int value) => value.toString().padLeft(2, '0');
    return 'LinguaRay-backup-${now.year}${two(now.month)}${two(now.day)}-'
        '${two(now.hour)}${two(now.minute)}.zip';
  }
}

final dataTransferController = DataTransferController();
