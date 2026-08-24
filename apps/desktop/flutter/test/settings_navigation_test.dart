import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_desktop/src/models/settings_navigation.dart';
import 'package:linguaray_desktop/src/services/app_windows.dart' as windows;
import 'package:linguaray_desktop/src/ui/settings/settings_labels.dart'
    as labels;

void main() {
  test('every settings destination has one unique section and location', () {
    expect(
      SettingsDestination.values,
      hasLength(SettingsSection.values.length),
    );
    expect(
      SettingsDestination.values.map((value) => value.location).toSet(),
      hasLength(SettingsDestination.values.length),
    );
    expect(
      SettingsDestination.values.map((value) => value.section).toSet(),
      hasLength(SettingsSection.values.length),
    );

    for (final destination in SettingsDestination.values) {
      expect(settingsDestinationForLocation(destination.location), destination);
      expect(destination.section.destination, destination);
    }
  });

  test('legacy import surfaces re-export the canonical enums', () {
    expect(windows.SettingsDestination.values, SettingsDestination.values);
    expect(labels.SettingsSection.values, SettingsSection.values);
  });

  test(
    'nested and unknown locations retain the previous selection behavior',
    () {
      expect(
        settingsDestinationForLocation('/settings/permissions/details'),
        SettingsDestination.settingsPermissions,
      );
      expect(
        settingsDestinationForLocation('/settings/not-a-route'),
        SettingsDestination.settingsGeneral,
      );
    },
  );
}
