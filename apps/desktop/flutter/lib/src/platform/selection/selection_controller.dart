import 'package:nativeapi/nativeapi.dart';

import '../../app/runtime.dart';
import '../permissions/permission_controller.dart';
import '../platform_types.dart';
import 'selection_replacement_controller.dart';

class SelectionController {
  SelectionController({
    PermissionController? permissions,
    SelectionReplacementController? replacement,
  }) : _permissions = permissions ?? permissionController,
       _replacement = replacement ?? selectionReplacementController;

  final PermissionController _permissions;
  final SelectionReplacementController _replacement;

  Future<SelectionResult> readSelection() async {
    final triggerPosition = DisplayManager.instance.getCursorPosition();
    final permission = await _permissions.refresh();
    if (permission.accessibility == PermissionState.denied) {
      throw const PlatformOperationException(
        action: TriggerAction.translateSelection,
        code: 'accessibility_denied',
        message: 'Accessibility permission is required to read selected text.',
      );
    }

    final target = await _replacement.capture();
    final result = await runtime
        .textExtractor()
        .extractFromScreenSelectionDetailed();
    return SelectionResult(
      text: result.text,
      replacementTarget: target?.text == result.text ? target : null,
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

final selectionController = SelectionController(
  permissions: permissionController,
  replacement: selectionReplacementController,
);
