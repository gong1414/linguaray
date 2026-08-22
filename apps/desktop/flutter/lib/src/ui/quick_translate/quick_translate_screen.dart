import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nativeapi/nativeapi.dart' as nativeapi;

import '../../platform/permission_controller.dart';
import '../../platform/platform_types.dart';
import '../../platform/trigger_controller.dart';
import '../../services/app_windows.dart';
import '../../utils/platform_util.dart';
import '../i18n_labels.dart';
import 'view_models/quick_translate_view_model.dart';
import 'widgets/quick_translate_view.dart';

class QuickTranslateScreen extends ConsumerStatefulWidget {
  const QuickTranslateScreen({super.key});

  @override
  ConsumerState<QuickTranslateScreen> createState() =>
      _QuickTranslateScreenState();
}

class _QuickTranslateScreenState extends ConsumerState<QuickTranslateScreen>
    with WidgetsBindingObserver {
  final GlobalKey _toolbarKey = GlobalKey();
  final GlobalKey _contentKey = GlobalKey();
  bool _pinned = false;
  bool _copied = false;
  QuickTranslateNotice _notice = QuickTranslateNotice.none;
  Timer? _copiedTimer;
  Timer? _resizeSettledTimer;
  bool _isWindowResizeScheduled = false;
  int? _windowFocusedListenerId;
  int? _windowBlurredListenerId;

  nativeapi.Window get _window => miniTranslatorWindowController.window;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    triggerController.quickWindowText.addListener(_consumeTriggeredText);
    triggerController.lastError.addListener(_showTriggerError);
    permissionController.addListener(_onPermissionChanged);
    if (kIsLinux || kIsMacOS || kIsWindows) {
      _registerWindowEvents();
    }
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _consumeTriggeredText();
      _scheduleWindowResize();
    });
  }

  @override
  void dispose() {
    triggerController.quickWindowText.removeListener(_consumeTriggeredText);
    triggerController.lastError.removeListener(_showTriggerError);
    permissionController.removeListener(_onPermissionChanged);
    WidgetsBinding.instance.removeObserver(this);
    _unregisterWindowEvents();
    _copiedTimer?.cancel();
    _resizeSettledTimer?.cancel();
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) {
      unawaited(permissionController.refresh());
    }
  }

  void _registerWindowEvents() {
    _windowFocusedListenerId = nativeapi.WindowManager.instance
        .on<nativeapi.WindowFocusedEvent>((event) {
          if (event.windowId == _window.id) {
            unawaited(permissionController.refresh());
          }
        });
    _windowBlurredListenerId = nativeapi.WindowManager.instance
        .on<nativeapi.WindowBlurredEvent>((event) {
          if (event.windowId == _window.id && !_window.isAlwaysOnTop) {
            hideMiniTranslatorWindow();
          }
        });
  }

  void _unregisterWindowEvents() {
    if (_windowFocusedListenerId != null) {
      nativeapi.WindowManager.instance.off(_windowFocusedListenerId!);
    }
    if (_windowBlurredListenerId != null) {
      nativeapi.WindowManager.instance.off(_windowBlurredListenerId!);
    }
  }

  void _consumeTriggeredText() {
    final value = triggerController.quickWindowText.value;
    if (value == null || value.trim().isEmpty) return;
    triggerController.quickWindowText.value = null;
    ref.read(quickTranslateViewModelProvider.notifier).setSourceText(value);
    unawaited(ref.read(quickTranslateViewModelProvider.notifier).submit());
    _scheduleWindowResize();
  }

  void _showTriggerError() {
    final error = triggerController.lastError.value;
    if (error == null) return;
    setState(() {
      _notice = switch (error.code) {
        'cancelled' => QuickTranslateNotice.captureCancelled,
        'permission_denied' ||
        'accessibility_denied' ||
        'screen_recording_denied' => QuickTranslateNotice.permissionDenied,
        _ => QuickTranslateNotice.none,
      };
    });
    _scheduleWindowResize();
  }

  void _onPermissionChanged() {
    final snapshot = permissionController.snapshot;
    final denied =
        snapshot.accessibility == PermissionState.denied ||
        snapshot.screenRecording == PermissionState.denied;
    if (!denied && _notice == QuickTranslateNotice.permissionDenied) {
      setState(() => _notice = QuickTranslateNotice.none);
    }
  }

  void _scheduleWindowResize() {
    if (!(kIsLinux || kIsMacOS || kIsWindows)) return;
    if (_isWindowResizeScheduled) return;
    _isWindowResizeScheduled = true;
    WidgetsBinding.instance.endOfFrame.then((_) {
      _isWindowResizeScheduled = false;
      if (!mounted) return;
      _resizeWindow();
      _resizeSettledTimer?.cancel();
      _resizeSettledTimer = Timer(const Duration(milliseconds: 120), () {
        if (mounted) _resizeWindow();
      });
    });
  }

  void _resizeWindow() {
    try {
      final toolbar = _renderHeight(_toolbarKey);
      final content = _renderHeight(_contentKey);
      final height = (toolbar + content + 24.0).clamp(180.0, 800.0);
      final size = _window.size;
      if ((size.height - height).abs() < 1) return;
      _window.setSize(size.width, height, animate: true);
    } catch (_) {}
  }

  double _renderHeight(GlobalKey key) {
    final box = key.currentContext?.findRenderObject() as RenderBox?;
    return box?.size.height ?? 0;
  }

  Future<void> _copy(String value) async {
    await Clipboard.setData(ClipboardData(text: value));
    if (!mounted) return;
    setState(() => _copied = true);
    _copiedTimer?.cancel();
    _copiedTimer = Timer(const Duration(milliseconds: 1400), () {
      if (mounted) setState(() => _copied = false);
    });
  }

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(quickTranslateViewModelProvider);
    ref.listen(quickTranslateViewModelProvider, (previous, next) {
      _scheduleWindowResize();
    });

    return QuickTranslateView(
      labels: quickTranslateLabels(),
      toolbarKey: _toolbarKey,
      contentKey: _contentKey,
      languages: state.languages,
      services: state.services,
      sourceText: state.sourceText,
      sourceLanguage: state.sourceLanguage,
      targetLanguage: state.targetLanguage,
      selectedServiceId: state.selectedServiceId,
      selectedResult: state.selectedResult,
      results: state.run?.results ?? const [],
      detectedLanguage: state.run?.detectedLanguage,
      submitting: state.submitting,
      copied: _copied,
      pinned: _pinned,
      notice: _notice,
      onSourceTextChanged: (value) {
        ref.read(quickTranslateViewModelProvider.notifier).setSourceText(value);
        _scheduleWindowResize();
      },
      onSourceLanguageChanged: ref
          .read(quickTranslateViewModelProvider.notifier)
          .setSourceLanguage,
      onTargetLanguageChanged: ref
          .read(quickTranslateViewModelProvider.notifier)
          .setTargetLanguage,
      onServiceSelected: ref
          .read(quickTranslateViewModelProvider.notifier)
          .selectService,
      onSwapLanguages: ref
          .read(quickTranslateViewModelProvider.notifier)
          .swapLanguages,
      onTranslate: () => unawaited(
        ref.read(quickTranslateViewModelProvider.notifier).submit(),
      ),
      onClear: () {
        ref.read(quickTranslateViewModelProvider.notifier).clearSourceText();
        _scheduleWindowResize();
      },
      onCopy: (value) => unawaited(_copy(value)),
      onTogglePin: () {
        setState(() => _pinned = !_pinned);
        _window.isAlwaysOnTop = _pinned;
      },
      onCapture: () => unawaited(
        triggerController.trigger(TriggerAction.captureAndTranslate),
      ),
      onClipboard: () =>
          unawaited(triggerController.trigger(TriggerAction.translateInput)),
      onOpenWorkbench: () => showWorkbenchWindow(text: state.sourceText),
      onOpenSettings: showSettingsWindow,
      onConfigureServices: showSettingsWindow,
      onRecheckPermissions: () => unawaited(permissionController.refresh()),
    );
  }
}
