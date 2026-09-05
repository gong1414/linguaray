import 'dart:async';

import 'package:flutter/widgets.dart';
import 'package:nativeapi/nativeapi.dart' as nativeapi;

import '../../platform/permission_controller.dart';
import '../../services/app_windows.dart';
import '../../services/window_positioning.dart' show fitPopoverToWorkArea;
import '../../utils/platform_util.dart';

final class QuickTranslateWindowCoordinator {
  QuickTranslateWindowCoordinator(this._window, this._isMounted);

  final nativeapi.Window Function() _window;
  final bool Function() _isMounted;
  final GlobalKey toolbarKey = GlobalKey();
  final GlobalKey contentKey = GlobalKey();

  Timer? _settledTimer;
  bool _resizeScheduled = false;
  int? _focusedListenerId;
  int? _blurredListenerId;

  nativeapi.Window get window => _window();

  void registerEvents() {
    if (!(kIsMacOS || kIsWindows)) return;
    _focusedListenerId = nativeapi.WindowManager.instance
        .on<nativeapi.WindowFocusedEvent>((event) {
          if (event.windowId == window.id) {
            unawaited(permissionController.refresh());
          }
        });
    _blurredListenerId = nativeapi.WindowManager.instance
        .on<nativeapi.WindowBlurredEvent>((event) {
          if (event.windowId == window.id && !window.isAlwaysOnTop) {
            hideMiniTranslatorWindow();
          }
        });
  }

  void scheduleResize() {
    if (!(kIsMacOS || kIsWindows) || _resizeScheduled) return;
    _resizeScheduled = true;
    WidgetsBinding.instance.endOfFrame.then((_) {
      _resizeScheduled = false;
      if (!_isMounted()) return;
      _resize();
      _settledTimer?.cancel();
      _settledTimer = Timer(const Duration(milliseconds: 120), () {
        if (_isMounted()) _resize();
      });
    });
  }

  void dispose() {
    _settledTimer?.cancel();
    if (_focusedListenerId != null) {
      nativeapi.WindowManager.instance.off(_focusedListenerId!);
    }
    if (_blurredListenerId != null) {
      nativeapi.WindowManager.instance.off(_blurredListenerId!);
    }
  }

  void _resize() {
    if (!canResizeMiniTranslatorWindow) return;
    try {
      final height =
          (_renderHeight(toolbarKey) + _renderHeight(contentKey) + 24.0).clamp(
            180.0,
            800.0,
          );
      final size = window.contentSize;
      final position = window.position;
      final displays = nativeapi.DisplayManager.instance.getAll();
      final display =
          displays
              .where(
                (display) =>
                    display.workArea.contains(position + const Offset(8, 8)),
              )
              .firstOrNull ??
          displays.firstOrNull;
      final fitted = display == null
          ? position & Size(size.width, height)
          : fitPopoverToWorkArea(
              position: position,
              desiredSize: Size(size.width, height),
              workArea: display.workArea,
            );
      if ((size.height - fitted.height).abs() >= 1 ||
          (size.width - fitted.width).abs() >= 1) {
        window.setContentSize(fitted.width, fitted.height);
      }
      if ((position - fitted.topLeft).distance >= 1) {
        window.setPosition(fitted.left, fitted.top);
      }
    } catch (_) {}
  }

  double _renderHeight(GlobalKey key) {
    final box = key.currentContext?.findRenderObject() as RenderBox?;
    return box?.size.height ?? 0;
  }
}
