import 'dart:io';

import 'package:fluentui_system_icons/fluentui_system_icons.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../../i18n/i18n.dart';
import '../../utils/env.dart';
import '../../widgets/settings_page.dart';
import '../../widgets/ui.dart'
    show
        Button,
        ButtonVariant,
        DesignThemeContext,
        DesignTypographyStyles,
        PreferenceSection,
        Pressable;

/// 关于 — the one settings page you read rather than change, which is why it
/// sits in its own run of the rail. Three blocks: what this build is, whether
/// a newer one exists, and where to go for everything else.
class AboutSettingsPage extends StatefulWidget {
  const AboutSettingsPage({super.key});

  @override
  State<AboutSettingsPage> createState() => _AboutSettingsPageState();
}

class _AboutSettingsPageState extends State<AboutSettingsPage> {
  bool _copied = false;

  /// `settings.version` is a literal `v{} (Build {})` — slang leaves it alone
  /// because `{}` is not its placeholder syntax — so the two slots are filled
  /// here, in order.
  String get _versionLabel {
    final parts = t.settings.version.split('{}');
    if (parts.length < 3) {
      return 'v${Env.instance.appVersion} (Build ${Env.instance.appBuildNumber})';
    }
    return parts[0] +
        Env.instance.appVersion +
        parts[1] +
        Env.instance.appBuildNumber.toString() +
        parts[2];
  }

  Future<void> _copyVersion() async {
    await Clipboard.setData(ClipboardData(text: _versionLabel));
    if (!mounted) return;
    setState(() => _copied = true);
    await Future<void>.delayed(const Duration(seconds: 2));
    if (mounted) setState(() => _copied = false);
  }

  @override
  Widget build(BuildContext context) {
    final tokens = context.tokens;
    final colors = tokens.colors;
    final about = t.settings.about;

    return SettingsPage(
      children: [
        // The identity block carries no heading: it is the page announcing
        // itself, and a label over a centred banner would name what the type
        // already says.
        Padding(
          padding: const EdgeInsets.symmetric(vertical: 4),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(
                'LinguaRay',
                style: tokens.typography.displayStyle(
                  fontSize: 15,
                  fontWeight: FontWeight.w700,
                  height: 1,
                  color: colors.fg,
                ),
              ),
              const SizedBox(height: 6),
              Text(
                _versionLabel,
                style: tokens.typography.monoStyle(
                  fontSize: 11,
                  height: 1,
                  color: colors.fgSubtle,
                ),
              ),
              const SizedBox(height: 8),
              Button(
                variant: ButtonVariant.quiet,
                onPressed: _copyVersion,
                child: Text(
                  _copied
                      ? t.common.ui.feedback.copied
                      : about.copy_version_info,
                ),
              ),
            ],
          ),
        ),
        const SettingsSectionDivider(),
        PreferenceSection(
          label: Text(about.links),
          children: [
            _ExternalRow(
              title: about.open_changelog,
              url: 'https://github.com/gong1414/linguaray/releases',
            ),
            _ExternalRow(
              title: about.website,
              url: 'https://github.com/gong1414/linguaray',
            ),
            _ExternalRow(
              title: about.github,
              url: 'https://github.com/gong1414/linguaray',
            ),
            _ExternalRow(
              title: about.report_issue,
              url: 'https://github.com/gong1414/linguaray/issues',
            ),
            _ExternalRow(
              title: about.license,
              url: 'https://github.com/gong1414/linguaray/blob/main/LICENSE',
            ),
          ],
        ),
      ],
    );
  }
}

/// A row that leaves the app — the glyph is the whole affordance.
class _ExternalRow extends StatelessWidget {
  const _ExternalRow({required this.title, required this.url});

  final String title;
  final String url;

  @override
  Widget build(BuildContext context) {
    final tokens = context.tokens;
    final colors = tokens.colors;

    return Pressable(
      onPressed: () => openExternalUrl(url),
      semanticsLabel: title,
      builder: (context, state) => ConstrainedBox(
        constraints: const BoxConstraints(minHeight: 28),
        child: Row(
          children: [
            Expanded(
              child: Text(
                title,
                style: tokens.typography.sansStyle(
                  fontSize: 12,
                  fontWeight: FontWeight.w600,
                  height: 1,
                  color: state.hovered ? colors.accentText : colors.fg,
                ),
              ),
            ),
            const SizedBox(width: 16),
            Icon(FluentIcons.open_20_regular, size: 13, color: colors.fgFaint),
          ],
        ),
      ),
    );
  }
}

/// Hands a URL to the platform's own opener.
Future<void> openExternalUrl(String url) async {
  if (Platform.isMacOS) {
    await Process.start('open', [url]);
  } else if (Platform.isWindows) {
    await Process.start('rundll32', ['url.dll,FileProtocolHandler', url]);
  } else {
    await Process.start('xdg-open', [url]);
  }
}
