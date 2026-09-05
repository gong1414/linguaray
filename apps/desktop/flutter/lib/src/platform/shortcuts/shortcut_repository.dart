import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_runtime/linguaray_runtime.dart'
    show ShortcutSettingsPatch;

import '../../app/settings/settings_section.dart';
import '../../app/settings/settings_store.dart';
import '../platform_types.dart';
import 'shortcut_service.dart';

const kShortcutToggle = 'toggleQuickWindow';
const kShortcutSelection = 'translateSelection';
const kShortcutInput = 'openInputWindow';
const kShortcutCapture = 'captureAndTranslate';
const kShortcutCaptureOcr = 'captureOcr';
const kShortcutSilentCaptureOcr = 'silentCaptureOcr';
const kShortcutFileOcr = 'fileOcr';
const kShortcutClipboardOcr = 'clipboardOcr';
const kShortcutShowOcrWindow = 'showOcrWindow';
const kShortcutClipboard = 'translateInput';

final class RuntimeShortcutRepository implements ShortcutRepository {
  RuntimeShortcutRepository({SettingsStore? store, ShortcutService? service})
    : _store = store ?? settingsStore,
      _service = service ?? shortcutService;

  final SettingsStore _store;
  final ShortcutService _service;

  @override
  Future<void> beginRecording() => _service.suspendRegistration();

  @override
  Future<void> endRecording() => _service.resumeRegistration();

  @override
  Future<List<ShortcutRecord>> load() async {
    await _store.reloadShortcuts();
    _store.throwIfErrored(SettingsSection.shortcuts);
    final byAction = {
      for (final binding in _service.bindings) binding.action: binding,
    };
    return [
      _record(
        kShortcutSelection,
        _store.shortcuts.extractTextFromScreenSelection,
        byAction[TriggerAction.translateSelection],
      ),
      _record(
        kShortcutCapture,
        _store.shortcuts.extractTextFromScreenCapture,
        byAction[TriggerAction.captureAndTranslate],
      ),
      _record(
        kShortcutInput,
        _store.shortcuts.translateInputContent,
        byAction[TriggerAction.openInputWindow],
      ),
      _record(
        kShortcutToggle,
        _store.shortcuts.toggleMiniTranslator,
        byAction[TriggerAction.toggleQuickWindow],
      ),
      _record(
        kShortcutClipboard,
        _store.shortcuts.extractTextFromClipboard,
        byAction[TriggerAction.translateInput],
      ),
      _record(
        kShortcutCaptureOcr,
        _store.shortcuts.captureOcr,
        byAction[TriggerAction.captureOcr],
      ),
      _record(
        kShortcutSilentCaptureOcr,
        _store.shortcuts.silentCaptureOcr,
        byAction[TriggerAction.silentCaptureOcr],
      ),
      _record(
        kShortcutFileOcr,
        _store.shortcuts.fileOcr,
        byAction[TriggerAction.fileOcr],
      ),
      _record(
        kShortcutClipboardOcr,
        _store.shortcuts.clipboardOcr,
        byAction[TriggerAction.clipboardOcr],
      ),
      _record(
        kShortcutShowOcrWindow,
        _store.shortcuts.showOcrWindow,
        byAction[TriggerAction.showOcrWindow],
      ),
    ];
  }

  @override
  Future<void> setAccelerator({
    required String actionId,
    required String accelerator,
  }) {
    return _store.updateShortcuts(_patch(actionId, accelerator));
  }

  @override
  Future<void> clear(String actionId) {
    return setAccelerator(actionId: actionId, accelerator: '');
  }

  @override
  Future<void> resetDefaults() => _store.resetShortcuts();

  ShortcutRecord _record(
    String actionId,
    String accelerator,
    ShortcutBinding? binding,
  ) {
    return ShortcutRecord(
      actionId: actionId,
      labelKey: actionId,
      accelerator: accelerator,
      status: switch (binding?.state) {
        ShortcutRegistrationState.registered => ShortcutStatus.registered,
        ShortcutRegistrationState.conflict => ShortcutStatus.osConflict,
        ShortcutRegistrationState.invalid => ShortcutStatus.invalid,
        ShortcutRegistrationState.unregistered || null =>
          accelerator.isEmpty
              ? ShortcutStatus.unregistered
              : ShortcutStatus.registered,
      },
      conflictReason: binding?.conflictReason,
    );
  }

  ShortcutSettingsPatch _patch(String actionId, String accelerator) {
    return switch (actionId) {
      kShortcutToggle => ShortcutSettingsPatch(
        toggleMiniTranslator: accelerator,
      ),
      kShortcutSelection => ShortcutSettingsPatch(
        extractTextFromScreenSelection: accelerator,
      ),
      kShortcutCapture => ShortcutSettingsPatch(
        extractTextFromScreenCapture: accelerator,
      ),
      kShortcutCaptureOcr => ShortcutSettingsPatch(captureOcr: accelerator),
      kShortcutSilentCaptureOcr => ShortcutSettingsPatch(
        silentCaptureOcr: accelerator,
      ),
      kShortcutFileOcr => ShortcutSettingsPatch(fileOcr: accelerator),
      kShortcutClipboardOcr => ShortcutSettingsPatch(clipboardOcr: accelerator),
      kShortcutShowOcrWindow => ShortcutSettingsPatch(
        showOcrWindow: accelerator,
      ),
      kShortcutClipboard => ShortcutSettingsPatch(
        extractTextFromClipboard: accelerator,
      ),
      kShortcutInput => ShortcutSettingsPatch(
        translateInputContent: accelerator,
      ),
      _ => throw ArgumentError.value(actionId, 'actionId'),
    };
  }
}
