import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../i18n/i18n.dart';
import '../../platform/external_url.dart';
import '../../shared/i18n_labels.dart';
import 'advanced_settings_view.dart';
import 'advanced_settings_view_model.dart';

class AdvancedSettingsScreen extends ConsumerStatefulWidget {
  const AdvancedSettingsScreen({super.key});

  @override
  ConsumerState<AdvancedSettingsScreen> createState() =>
      _AdvancedSettingsScreenState();
}

class _AdvancedSettingsScreenState
    extends ConsumerState<AdvancedSettingsScreen> {
  final TextEditingController _port = TextEditingController();
  final TextEditingController _proxyUrl = TextEditingController();
  final TextEditingController _proxyBypass = TextEditingController();

  @override
  void initState() {
    super.initState();
    ref.listenManual(advancedSettingsViewModelProvider, (previous, next) {
      _syncControllers(previous, next);
    }, fireImmediately: true);
  }

  @override
  void dispose() {
    _port.dispose();
    _proxyUrl.dispose();
    _proxyBypass.dispose();
    super.dispose();
  }

  void _syncControllers(
    AdvancedSettingsViewState? previous,
    AdvancedSettingsViewState next,
  ) {
    if (previous?.api?.port != next.api?.port && next.api != null) {
      final text = '${next.api!.port}';
      if (_port.text != text) _port.text = text;
    }
    if (previous?.network?.proxyUrl != next.network?.proxyUrl) {
      _proxyUrl.text = next.network?.proxyUrl ?? '';
    }
    if (previous?.network?.proxyBypass != next.network?.proxyBypass) {
      _proxyBypass.text = next.network?.proxyBypass ?? '';
    }
  }

  NetworkSettings? _draftNetwork(AdvancedSettingsViewState state) {
    final network = state.network;
    if (network == null) return null;
    return NetworkSettings(
      proxyMode: network.proxyMode,
      proxyUrl: _proxyUrl.text,
      proxyBypass: _proxyBypass.text,
      checkUpdatesOnLaunch: network.checkUpdatesOnLaunch,
    );
  }

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(advancedSettingsViewModelProvider);
    final advanced = t.settings.advanced;
    return AdvancedSettingsView(
      labels: AdvancedSettingsViewLabels(
        title: advanced.title,
        apiServerDescription: advanced.api_server_description,
        enable: advanced.enable,
        port: advanced.port,
        runningAt: advanced.running_at,
        copied: t.common.ui.feedback.copied,
        network: advanced.network,
        proxyMode: advanced.proxy_mode,
        proxySystem: advanced.proxy_system,
        proxyDirect: advanced.proxy_direct,
        proxyCustom: advanced.proxy_custom,
        proxyUrl: advanced.proxy_url,
        proxyUrlHint: advanced.proxy_url_hint,
        proxyBypass: advanced.proxy_bypass,
        proxyBypassHint: advanced.proxy_bypass_hint,
        checkUpdatesOnLaunch: advanced.check_updates_on_launch,
        saveNetwork: advanced.save_network,
        errorMessage: appErrorMessage,
      ),
      state: state,
      portController: _port,
      proxyUrlController: _proxyUrl,
      proxyBypassController: _proxyBypass,
      onApiEnabledChanged: (value) => unawaited(
        ref
            .read(advancedSettingsViewModelProvider.notifier)
            .setApiServerEnabled(value),
      ),
      onPortSubmitted: (value) {
        final port = int.tryParse(value);
        final notifier = ref.read(advancedSettingsViewModelProvider.notifier);
        if (port == null) {
          notifier.setInvalidPort();
          return;
        }
        unawaited(notifier.setApiServerPort(port));
      },
      onOpenUrl: (url) => unawaited(openExternalUrl(url)),
      onProxyModeChanged: (mode) => ref
          .read(advancedSettingsViewModelProvider.notifier)
          .setProxyMode(mode),
      onCheckUpdatesChanged: (value) => ref
          .read(advancedSettingsViewModelProvider.notifier)
          .setCheckUpdatesOnLaunch(value),
      onSaveNetwork: () {
        final draft = _draftNetwork(state);
        if (draft != null) {
          unawaited(
            ref
                .read(advancedSettingsViewModelProvider.notifier)
                .saveNetwork(draft),
          );
        }
      },
    );
  }
}
