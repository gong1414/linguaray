import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../shared/settings_page.dart';
import '../../shared/status_message.dart';
import 'advanced_settings_view_model.dart';

final class AdvancedSettingsViewLabels {
  const AdvancedSettingsViewLabels({
    required this.title,
    required this.apiServerDescription,
    required this.enable,
    required this.port,
    required this.runningAt,
    required this.copied,
    required this.network,
    required this.proxyMode,
    required this.proxySystem,
    required this.proxyDirect,
    required this.proxyCustom,
    required this.proxyUrl,
    required this.proxyUrlHint,
    required this.proxyBypass,
    required this.proxyBypassHint,
    required this.checkUpdatesOnLaunch,
    required this.saveNetwork,
    required this.errorMessage,
  });

  final String title;
  final String apiServerDescription;
  final String enable;
  final String port;
  final String runningAt;
  final String copied;
  final String network;
  final String proxyMode;
  final String proxySystem;
  final String proxyDirect;
  final String proxyCustom;
  final String proxyUrl;
  final String proxyUrlHint;
  final String proxyBypass;
  final String proxyBypassHint;
  final String checkUpdatesOnLaunch;
  final String saveNetwork;
  final String Function(String? code) errorMessage;
}

class AdvancedSettingsView extends StatelessWidget {
  const AdvancedSettingsView({
    required this.labels,
    required this.state,
    required this.portController,
    required this.proxyUrlController,
    required this.proxyBypassController,
    required this.onApiEnabledChanged,
    required this.onPortSubmitted,
    required this.onOpenUrl,
    required this.onProxyModeChanged,
    required this.onCheckUpdatesChanged,
    required this.onSaveNetwork,
    super.key,
  });

  final AdvancedSettingsViewLabels labels;
  final AdvancedSettingsViewState state;
  final TextEditingController portController;
  final TextEditingController proxyUrlController;
  final TextEditingController proxyBypassController;
  final ValueChanged<bool> onApiEnabledChanged;
  final ValueChanged<String> onPortSubmitted;
  final ValueChanged<String> onOpenUrl;
  final ValueChanged<NetworkProxyMode> onProxyModeChanged;
  final ValueChanged<bool> onCheckUpdatesChanged;
  final VoidCallback onSaveNetwork;

  @override
  Widget build(BuildContext context) {
    final status = state.api;
    final network = state.network;
    return SettingsPage(
      title: labels.title,
      children: [
        Text(labels.apiServerDescription),
        if (status != null)
          SwitchListTile(
            contentPadding: EdgeInsets.zero,
            title: Text(labels.enable),
            value: status.enabled,
            onChanged: onApiEnabledChanged,
          ),
        ListTile(
          contentPadding: EdgeInsets.zero,
          title: Text(labels.port),
          trailing: SizedBox(
            width: 96,
            child: TextField(
              controller: portController,
              keyboardType: TextInputType.number,
              onSubmitted: onPortSubmitted,
            ),
          ),
        ),
        if (status?.baseUrl != null) ...[
          ListTile(
            contentPadding: EdgeInsets.zero,
            title: Text('${labels.runningAt} ${status!.baseUrl}'),
            trailing: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                IconButton(
                  tooltip: labels.copied,
                  onPressed: () =>
                      Clipboard.setData(ClipboardData(text: status.baseUrl!)),
                  icon: const Icon(Icons.copy_rounded),
                ),
                IconButton(
                  onPressed: () => onOpenUrl(status.baseUrl!),
                  icon: const Icon(Icons.open_in_new_rounded),
                ),
              ],
            ),
          ),
        ],
        if (state.apiError != null)
          StatusMessage(
            kind: StatusKind.error,
            title: labels.errorMessage(state.apiError),
          ),
        const SizedBox(height: 24),
        const Divider(),
        const SizedBox(height: 16),
        Text(labels.network, style: Theme.of(context).textTheme.titleMedium),
        if (network != null) ...[
          const SizedBox(height: 8),
          DropdownButtonFormField<NetworkProxyMode>(
            initialValue: network.proxyMode,
            decoration: InputDecoration(labelText: labels.proxyMode),
            items: [
              DropdownMenuItem(
                value: NetworkProxyMode.system,
                child: Text(labels.proxySystem),
              ),
              DropdownMenuItem(
                value: NetworkProxyMode.direct,
                child: Text(labels.proxyDirect),
              ),
              DropdownMenuItem(
                value: NetworkProxyMode.custom,
                child: Text(labels.proxyCustom),
              ),
            ],
            onChanged: (mode) {
              if (mode != null) onProxyModeChanged(mode);
            },
          ),
          if (network.proxyMode == NetworkProxyMode.custom) ...[
            const SizedBox(height: 12),
            TextField(
              controller: proxyUrlController,
              decoration: InputDecoration(
                labelText: labels.proxyUrl,
                hintText: labels.proxyUrlHint,
              ),
            ),
            const SizedBox(height: 12),
            TextField(
              controller: proxyBypassController,
              decoration: InputDecoration(
                labelText: labels.proxyBypass,
                hintText: labels.proxyBypassHint,
              ),
            ),
          ],
          SwitchListTile(
            contentPadding: EdgeInsets.zero,
            title: Text(labels.checkUpdatesOnLaunch),
            value: network.checkUpdatesOnLaunch,
            onChanged: onCheckUpdatesChanged,
          ),
          Align(
            alignment: Alignment.centerRight,
            child: FilledButton(
              onPressed: onSaveNetwork,
              child: Text(labels.saveNetwork),
            ),
          ),
          if (state.networkError != null) ...[
            const SizedBox(height: 12),
            StatusMessage(
              kind: StatusKind.error,
              title: labels.errorMessage(state.networkError),
            ),
          ],
        ],
      ],
    );
  }
}
