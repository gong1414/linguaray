import 'package:flutter/widgets.dart';

import '../../i18n/i18n.dart';
import '../../platform/permission_controller.dart';
import '../../platform/platform_types.dart';
import '../../services/runtime.dart'
    show AppearanceSettingsPatch, GeneralSettings, GeneralSettingsPatch;
import '../../services/settings_store.dart';
import '../../utils/language_util.dart';
import '../../utils/platform_util.dart';
import '../../widgets/native_select.dart';
import '../../widgets/settings_page.dart';
import '../../widgets/ui.dart'
    show Button, ButtonVariant, PreferenceRow, PreferenceSection, Switch;

/// Mirrors macOS `GeneralView.swift`.
class GeneralSettingsPage extends StatefulWidget {
  const GeneralSettingsPage({super.key});

  @override
  State<GeneralSettingsPage> createState() => _GeneralSettingsPageState();
}

class _GeneralSettingsPageState extends State<GeneralSettingsPage> {
  @override
  void initState() {
    super.initState();
    settingsStore.addListener(_handleChanged);
    // Refresh when entering the page.
    settingsStore.reloadGeneral();
    settingsStore.reloadProviders();
    // 外观 folded into this page — a display language and a theme mode are
    // preferences like any other, and a rail entry each was more navigation
    // than they earn.
    settingsStore.reloadAppearance();
  }

  @override
  void dispose() {
    settingsStore.removeListener(_handleChanged);
    super.dispose();
  }

  void _handleChanged() {
    if (mounted) setState(() {});
  }

  GeneralSettings get _general => settingsStore.general;

  @override
  Widget build(BuildContext context) {
    final general = t.settings.general;
    final appearance = t.settings.appearance;

    return SettingsPage(
      children: [
        PreferenceSection(
          label: Text(general.section.startup),
          children: [
            PreferenceRow(
              title: Text(general.row.launch_at_login),
              trailing: [
                Switch(
                  checked: _general.launchAtLogin,
                  semanticsLabel: general.row.launch_at_login,
                  onChanged: (v) => settingsStore.updateGeneral(
                    GeneralSettingsPatch(launchAtLogin: v),
                  ),
                ),
              ],
            ),
            PreferenceRow(
              title: Text(general.row.show_in_menu_bar),
              trailing: [
                Switch(
                  checked: _general.showInMenuBar,
                  semanticsLabel: general.row.show_in_menu_bar,
                  onChanged: (v) => settingsStore.updateGeneral(
                    GeneralSettingsPatch(showInMenuBar: v),
                  ),
                ),
              ],
            ),
          ],
        ),
        const SettingsSectionDivider(),
        // 外观 was its own page until it held two rows; a display language and
        // a theme mode are preferences like any other, and a rail entry each
        // was more navigation than they earn. The language is a menu rather
        // than a radio stack — the list grows with every locale we ship.
        PreferenceSection(
          label: Text(appearance.title),
          footer: Text(appearance.footer),
          children: [
            PreferenceRow(
              title: Text(appearance.section.app_language),
              trailing: [
                _AppearanceSelect(
                  value: settingsStore.appearance.language,
                  items: [
                    for (final code in appLanguages)
                      NativeSelectItem(
                          value: code, label: getLanguageName(code)),
                  ],
                  onChanged: (v) => settingsStore.updateAppearance(
                    AppearanceSettingsPatch(language: v),
                  ),
                ),
              ],
            ),
            PreferenceRow(
              title: Text(appearance.section.theme_mode),
              trailing: [
                _AppearanceSelect(
                  value: settingsStore.appearance.themeMode,
                  items: [
                    NativeSelectItem(
                      value: 'light',
                      label: t.common.theme_mode.light,
                    ),
                    NativeSelectItem(
                        value: 'dark', label: t.common.theme_mode.dark),
                    NativeSelectItem(
                      value: 'system',
                      label: t.common.theme_mode.system,
                    ),
                  ],
                  onChanged: (v) => settingsStore.updateAppearance(
                    AppearanceSettingsPatch(themeMode: v),
                  ),
                ),
              ],
            ),
          ],
        ),
        if (kIsMacOS) ...[
          const SettingsSectionDivider(),
          // 系统权限 — these are what the OS lets the app do, not an advanced
          // option, and every shortcut that reads the screen stops working
          // without them. They sit last because they are granted once and then
          // never touched again.
          PreferenceSection(
            label: Text(general.section.permissions),
            children: [
              _PermissionAccessRow(
                title: general.row.screen_capture_access,
                subtitle: general.row.screen_capture_access_hint,
                accessibility: false,
              ),
              _PermissionAccessRow(
                title: general.row.screen_selection_access,
                subtitle: general.row.screen_selection_access_hint,
                accessibility: true,
              ),
            ],
          ),
        ],
      ],
    );
  }
}

class _PermissionAccessRow extends StatefulWidget {
  const _PermissionAccessRow({
    required this.title,
    required this.subtitle,
    required this.accessibility,
  });

  final String title;

  /// What the grant actually buys — every shortcut that reads the screen
  /// stops working without it, and the row is the only place that says so.
  final String subtitle;
  final bool accessibility;

  @override
  State<_PermissionAccessRow> createState() => _PermissionAccessRowState();
}

class _PermissionAccessRowState extends State<_PermissionAccessRow> {
  @override
  void initState() {
    super.initState();
    permissionController.addListener(_handleChanged);
    permissionController.refresh();
  }

  @override
  void dispose() {
    permissionController.removeListener(_handleChanged);
    super.dispose();
  }

  void _handleChanged() {
    if (mounted) setState(() {});
  }

  Future<void> _request() async {
    if (widget.accessibility) {
      await permissionController.requestAccessibility();
    } else {
      await permissionController.requestScreenRecording();
    }
  }

  @override
  Widget build(BuildContext context) {
    final state = widget.accessibility
        ? permissionController.snapshot.accessibility
        : permissionController.snapshot.screenRecording;
    final granted = state == PermissionState.granted ||
        state == PermissionState.notRequired;
    return PreferenceRow(
      title: Text(widget.title),
      subtitle: Text(widget.subtitle),
      trailing: [
        Button(
          variant: ButtonVariant.secondary,
          onPressed: granted ? permissionController.refresh : _request,
          child: Text(
            granted
                ? t.settings.general.option.granted
                : t.settings.general.button.grant,
          ),
        ),
      ],
    );
  }
}

/// The right-hand control of an 外观 row: a fixed-width menu, the way the deck
/// draws a preference whose value comes from a list.
class _AppearanceSelect extends StatelessWidget {
  const _AppearanceSelect({
    required this.value,
    required this.items,
    required this.onChanged,
  });

  final String value;
  final List<NativeSelectItem<String>> items;
  final ValueChanged<String> onChanged;

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: 160,
      child: NativeSelect<String>(
        value: value,
        items: items,
        onChanged: onChanged,
      ),
    );
  }
}
