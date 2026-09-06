import 'package:flutter/foundation.dart';
import 'package:linguaray_runtime/linguaray_runtime.dart';

enum SettingsSection {
  general,
  appearance,
  shortcuts,
  providers,
  services,
  advanced,
}

/// Per-section [Listenable]s so a shortcuts or theme listener is not rebuilt
/// by unrelated settings writes.
final class SettingsSectionListenables {
  SettingsSectionListenables()
    : _ticks = {
        for (final section in SettingsSection.values)
          section: ValueNotifier<int>(0),
      };

  final Map<SettingsSection, ValueNotifier<int>> _ticks;

  Listenable of(SettingsSection section) => _ticks[section]!;

  Listenable merge(Iterable<SettingsSection> sections) {
    final listenables = [for (final section in sections) of(section)];
    if (listenables.length == 1) return listenables.single;
    return Listenable.merge(listenables);
  }

  void notify(SettingsSection section) => _ticks[section]!.value++;

  void dispose() {
    for (final notifier in _ticks.values) {
      notifier.dispose();
    }
  }
}

/// Runtime snapshot used by the lifecycle coordinator. The cache does not
/// apply login items, native appearance, or the local API server.
abstract interface class SettingsSnapshotSource {
  GeneralSettings get general;
  AppearanceSettings get appearance;
  AdvancedSettings get advanced;
  Listenable listenableFor(SettingsSection section);
  Future<void> updateGeneral(GeneralSettingsPatch patch);
}
