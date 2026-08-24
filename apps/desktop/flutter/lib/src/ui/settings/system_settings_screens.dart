import 'dart:async';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../config/dependencies.dart';
import '../../i18n/i18n.dart';
import '../../services/system_proxy.dart';
import '../../utils/external_url.dart';
import '../i18n_labels.dart';
import '../shared/status_message.dart';
import 'view_models/permissions_view_model.dart';
import 'view_models/settings_view_model.dart';
import 'views/about_settings_view.dart';
import 'views/permissions_settings_view.dart';

class PermissionsSettingsScreen extends ConsumerWidget {
  const PermissionsSettingsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return PermissionsSettingsView(
      labels: permissionsSettingsLabels(),
      snapshot: ref.watch(permissionsViewModelProvider),
      onGrantAccessibility: () => unawaited(
        ref.read(permissionsViewModelProvider.notifier).requestAccessibility(),
      ),
      onGrantScreenRecording: () => unawaited(
        ref
            .read(permissionsViewModelProvider.notifier)
            .requestScreenRecording(),
      ),
      onRecheck: () =>
          unawaited(ref.read(permissionsViewModelProvider.notifier).refresh()),
    );
  }
}

class AboutSettingsScreen extends ConsumerStatefulWidget {
  const AboutSettingsScreen({super.key});

  @override
  ConsumerState<AboutSettingsScreen> createState() =>
      _AboutSettingsScreenState();
}

class _AboutSettingsScreenState extends ConsumerState<AboutSettingsScreen> {
  bool _copied = false;

  @override
  Widget build(BuildContext context) {
    final info = ref.watch(aboutViewModelProvider);
    if (info == null) {
      return const Center(child: CircularProgressIndicator());
    }
    return AboutSettingsView(
      labels: aboutSettingsLabels(),
      info: info,
      copied: _copied,
      onCopyVersion: () async {
        await Clipboard.setData(
          ClipboardData(text: 'v${info.version} (${info.buildNumber})'),
        );
        setState(() => _copied = true);
      },
      onOpenWebsite: () =>
          unawaited(openExternalUrl('https://github.com/gong1414/linguaray')),
      onOpenChangelog: () => unawaited(
        openExternalUrl('https://github.com/gong1414/linguaray/releases'),
      ),
      onOpenIssues: () => unawaited(
        openExternalUrl('https://github.com/gong1414/linguaray/issues'),
      ),
      onOpenLicense: () => unawaited(
        openExternalUrl(
          'https://github.com/gong1414/linguaray/blob/main/LICENSE',
        ),
      ),
    );
  }
}

class AdvancedSettingsScreen extends ConsumerStatefulWidget {
  const AdvancedSettingsScreen({super.key});

  @override
  ConsumerState<AdvancedSettingsScreen> createState() =>
      _AdvancedSettingsScreenState();
}

class _AdvancedSettingsScreenState
    extends ConsumerState<AdvancedSettingsScreen> {
  ApiServerStatus? _status;
  NetworkSettings? _network;
  String? _apiError;
  String? _networkError;
  final TextEditingController _port = TextEditingController();
  final TextEditingController _proxyUrl = TextEditingController();
  final TextEditingController _proxyBypass = TextEditingController();

  @override
  void initState() {
    super.initState();
    unawaited(_reload());
  }

  @override
  void dispose() {
    _port.dispose();
    _proxyUrl.dispose();
    _proxyBypass.dispose();
    super.dispose();
  }

  Future<void> _reload() async {
    final repository = ref.read(workspaceSettingsRepositoryProvider);
    try {
      final status = await repository.loadApiServer();
      if (!mounted) return;
      setState(() {
        _status = status;
        _apiError = status.bindErrorCode;
        _port.text = '${status.port}';
      });
    } catch (_) {
      if (!mounted) return;
      setState(() => _apiError = AppErrorCode.apiServerBindFailed.wireName);
    }
    try {
      final network = await repository.loadNetworkSettings();
      if (!mounted) return;
      setState(() {
        _network = network;
        _networkError = null;
        _proxyUrl.text = network.proxyUrl;
        _proxyBypass.text = network.proxyBypass;
      });
    } catch (_) {
      if (!mounted) return;
      setState(() => _networkError = AppErrorCode.networkFailure.wireName);
    }
  }

  @override
  Widget build(BuildContext context) {
    final status = _status;
    final advanced = t.settings.advanced;
    return ListView(
      padding: const EdgeInsets.fromLTRB(24, 16, 24, 32),
      children: [
        Text(advanced.title, style: Theme.of(context).textTheme.headlineMedium),
        const SizedBox(height: 8),
        Text(advanced.api_server_description),
        if (status != null)
          SwitchListTile(
            contentPadding: EdgeInsets.zero,
            title: Text(advanced.enable),
            value: status.enabled,
            onChanged: (value) async {
              try {
                final next = await ref
                    .read(workspaceSettingsRepositoryProvider)
                    .setApiServerEnabled(value);
                if (!mounted) return;
                setState(() {
                  _status = next;
                  _apiError = next.bindErrorCode;
                });
              } catch (_) {
                if (!mounted) return;
                setState(
                  () => _apiError = AppErrorCode.apiServerBindFailed.wireName,
                );
              }
            },
          ),
        ListTile(
          contentPadding: EdgeInsets.zero,
          title: Text(advanced.port),
          trailing: SizedBox(
            width: 96,
            child: TextField(
              controller: _port,
              keyboardType: TextInputType.number,
              onSubmitted: (value) async {
                final port = int.tryParse(value);
                if (port == null) {
                  setState(() => _apiError = AppErrorCode.invalidPort.wireName);
                  return;
                }
                final next = await ref
                    .read(workspaceSettingsRepositoryProvider)
                    .setApiServerPort(port);
                if (!mounted) return;
                setState(() {
                  _status = next;
                  _apiError = next.bindErrorCode;
                });
              },
            ),
          ),
        ),
        if (status?.baseUrl != null) ...[
          ListTile(
            contentPadding: EdgeInsets.zero,
            title: Text('${advanced.running_at} ${status!.baseUrl}'),
            trailing: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                IconButton(
                  tooltip: t.common.ui.feedback.copied,
                  onPressed: () =>
                      Clipboard.setData(ClipboardData(text: status.baseUrl!)),
                  icon: const Icon(Icons.copy_rounded),
                ),
                IconButton(
                  onPressed: () => unawaited(openExternalUrl(status.baseUrl!)),
                  icon: const Icon(Icons.open_in_new_rounded),
                ),
              ],
            ),
          ),
        ],
        if (_apiError != null)
          StatusMessage(
            kind: StatusKind.error,
            title: appErrorMessage(_apiError),
          ),
        const SizedBox(height: 24),
        const Divider(),
        const SizedBox(height: 16),
        Text(advanced.network, style: Theme.of(context).textTheme.titleMedium),
        if (_network != null) ...[
          const SizedBox(height: 8),
          DropdownButtonFormField<NetworkProxyMode>(
            initialValue: _network!.proxyMode,
            decoration: InputDecoration(labelText: advanced.proxy_mode),
            items: [
              DropdownMenuItem(
                value: NetworkProxyMode.system,
                child: Text(advanced.proxy_system),
              ),
              DropdownMenuItem(
                value: NetworkProxyMode.direct,
                child: Text(advanced.proxy_direct),
              ),
              DropdownMenuItem(
                value: NetworkProxyMode.custom,
                child: Text(advanced.proxy_custom),
              ),
            ],
            onChanged: (mode) {
              if (mode == null) return;
              setState(
                () => _network = NetworkSettings(
                  proxyMode: mode,
                  proxyUrl: _proxyUrl.text,
                  proxyBypass: _proxyBypass.text,
                  checkUpdatesOnLaunch: _network!.checkUpdatesOnLaunch,
                ),
              );
            },
          ),
          if (_network!.proxyMode == NetworkProxyMode.custom) ...[
            const SizedBox(height: 12),
            TextField(
              controller: _proxyUrl,
              decoration: InputDecoration(
                labelText: advanced.proxy_url,
                hintText: advanced.proxy_url_hint,
              ),
            ),
            const SizedBox(height: 12),
            TextField(
              controller: _proxyBypass,
              decoration: InputDecoration(
                labelText: advanced.proxy_bypass,
                hintText: advanced.proxy_bypass_hint,
              ),
            ),
          ],
          SwitchListTile(
            contentPadding: EdgeInsets.zero,
            title: Text(advanced.check_updates_on_launch),
            value: _network!.checkUpdatesOnLaunch,
            onChanged: (value) => setState(
              () => _network = NetworkSettings(
                proxyMode: _network!.proxyMode,
                proxyUrl: _proxyUrl.text,
                proxyBypass: _proxyBypass.text,
                checkUpdatesOnLaunch: value,
              ),
            ),
          ),
          Align(
            alignment: Alignment.centerRight,
            child: FilledButton(
              onPressed: _saveNetwork,
              child: Text(advanced.save_network),
            ),
          ),
          if (_networkError != null) ...[
            const SizedBox(height: 12),
            StatusMessage(
              kind: StatusKind.error,
              title: appErrorMessage(_networkError),
            ),
          ],
        ],
      ],
    );
  }

  Future<void> _saveNetwork() async {
    final network = _network;
    if (network == null) return;
    if (network.proxyMode == NetworkProxyMode.custom) {
      final uri = Uri.tryParse(_proxyUrl.text.trim());
      if (uri == null ||
          (uri.scheme != 'http' && uri.scheme != 'https') ||
          uri.host.isEmpty ||
          uri.userInfo.isNotEmpty) {
        setState(
          () => _networkError = AppErrorCode.proxyConfigurationInvalid.wireName,
        );
        return;
      }
    }
    try {
      final saved = await ref
          .read(workspaceSettingsRepositoryProvider)
          .saveNetworkSettings(
            NetworkSettings(
              proxyMode: network.proxyMode,
              proxyUrl: _proxyUrl.text,
              proxyBypass: _proxyBypass.text,
              checkUpdatesOnLaunch: network.checkUpdatesOnLaunch,
            ),
          );
      await initializeSystemProxy();
      if (!mounted) return;
      setState(() {
        _network = saved;
        _networkError = null;
      });
    } catch (_) {
      if (!mounted) return;
      setState(() => _networkError = AppErrorCode.networkFailure.wireName);
    }
  }
}

bool get isDesktopWindows => Platform.isWindows;
