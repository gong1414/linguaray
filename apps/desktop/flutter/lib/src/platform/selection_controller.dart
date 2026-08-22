import 'package:nativeapi/nativeapi.dart';

import '../services/runtime.dart';
import 'permission_controller.dart';
import 'platform_types.dart';

class SelectionController {
  SelectionController({PermissionController? permissions})
    : _permissions = permissions ?? permissionController;

  final PermissionController _permissions;

  Future<SelectionResult> readSelection() async {
    final triggerPosition = DisplayManager.instance.getCursorPosition();
    final permission = await _permissions.refresh();
    if (permission.accessibility == PermissionState.denied) {
      throw const PlatformOperationException(
        action: TriggerAction.translateSelection,
        code: 'accessibilityDenied',
        message: 'Accessibility permission is required to read selected text.',
      );
    }

    final result = await runtime
        .textExtractor()
        .extractFromScreenSelectionDetailed();
    return SelectionResult(
      text: result.text,
      triggerPosition: triggerPosition,
      readMethod: switch (result.readMethod) {
        'primarySelection' => SelectionReadMethod.primarySelection,
        'clipboard' => SelectionReadMethod.clipboard,
        _ => SelectionReadMethod.simulatedCopy,
      },
      recoverableError: result.recoverableError,
    );
  }

  Future<String?> readClipboard() async {
    try {
      return await runtime.textExtractor().extractFromClipboard();
    } catch (_) {
      return null;
    }
  }
}

final selectionController = SelectionController();
