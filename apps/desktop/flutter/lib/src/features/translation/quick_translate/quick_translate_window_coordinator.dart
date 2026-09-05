import 'dart:async';

import 'package:flutter/widgets.dart';
import 'package:nativeapi/nativeapi.dart' as nativeapi;

import '../../../app/windows/app_windows.dart';
import '../../../platform/permissions/permission_controller.dart';
import '../../../platform/platform_util.dart';
import '../../../platform/windows/window_positioning.dart'
    show fitPopoverToWorkArea;

final class QuickTranslateWindowCoordinator {
  QuickTranslateWindowCoordinator(
    this._window,
    this._isMounted, {
    this.onDismiss,
  });

  final VoidCallback? onDismiss;

  final nativeapi.Window Function() _window;
  final bool Function() _isMounted;
  final GlobalKey toolbarKey = GlobalKey();
  final GlobalKey contentKey = GlobalKey();

  Timer? _settledTimer;
  bool _resizeScheduled = false;
  int? _resizedListenerId;
  int? _focusedListenerId;
  int? _blurredListenerId;

  nativeapi.Window get window => _window();

  void startDragging() => window.startDragging();

  void registerEvents() {
    if (!(kIsMacOS || kIsWindows)) return;
    _resizedListenerId = nativeapi.WindowManager.instance
        .on<nativeapi.WindowResizedEvent>((event) {
          if (event.windowId == window.id) scheduleResize();
        });
    _focusedListenerId = nativeapi.WindowManager.instance
        .on<nativeapi.WindowFocusedEvent>((event) {
          if (event.windowId == window.id) {
            unawaited(permissionController.refresh());
          }
        });
    _blurredListenerId = nativeapi.WindowManager.instance
        .on<nativeapi.WindowBlurredEvent>((event) {
          if (event.windowId == window.id && !window.isAlwaysOnTop) {
            onDismiss?.call();
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
    if (_resizedListenerId != null) {
      nativeapi.WindowManager.instance.off(_resizedListenerId!);
    }
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
      final outer = window.size;
      final frame = Size(
        (outer.width - size.width).clamp(0, double.infinity),
        (outer.height - size.height).clamp(0, double.infinity),
      );
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
          ? position & Size(size.width + frame.width, height + frame.height)
          : fitPopoverToWorkArea(
              position: position,
              desiredSize: Size(
                size.width + frame.width,
                height + frame.height,
              ),
              workArea: display.workArea,
            );
      final content = Size(
        (fitted.width - frame.width).clamp(1, double.infinity),
        (fitted.height - frame.height).clamp(1, double.infinity),
      );
      if ((size.height - content.height).abs() >= 1 ||
          (size.width - content.width).abs() >= 1) {
        window.setContentSize(content.width, content.height);
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
