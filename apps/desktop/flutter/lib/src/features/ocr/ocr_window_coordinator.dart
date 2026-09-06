import 'package:nativeapi/nativeapi.dart' as nativeapi;

import '../../app/windows/app_windows.dart';

/// Owns the OCR surface's native listener and visibility policy.
final class OcrWindowCoordinator {
  OcrWindowCoordinator({required this._keepVisible});
  final bool Function() _keepVisible;
  int? _blurredListener;

  void registerEvents() {
    _blurredListener ??= nativeapi.WindowManager.instance
        .on<nativeapi.WindowBlurredEvent>((event) {
          if (event.windowId == ocrWindowController.window.id &&
              !_keepVisible()) {
            close();
          }
        });
  }

  void setPinned(bool value) =>
      ocrWindowController.window.isAlwaysOnTop = value;
  void close() => hideOcrWindow();

  void dispose() {
    final id = _blurredListener;
    if (id != null) nativeapi.WindowManager.instance.off(id);
    _blurredListener = null;
  }
}
