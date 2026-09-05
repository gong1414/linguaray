import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../app/dependencies.dart';
import '../../platform/network/system_proxy.dart';

final advancedSettingsViewModelProvider =
    NotifierProvider<AdvancedSettingsViewModel, AdvancedSettingsViewState>(
      AdvancedSettingsViewModel.new,
    );

final class AdvancedSettingsViewState {
  const AdvancedSettingsViewState({
    this.api,
    this.network,
    this.apiError,
    this.networkError,
    this.loading = true,
  });

  final ApiServerStatus? api;
  final NetworkSettings? network;
  final String? apiError;
  final String? networkError;
  final bool loading;

  AdvancedSettingsViewState copyWith({
    ApiServerStatus? api,
    NetworkSettings? network,
    Object? apiError = _unset,
    Object? networkError = _unset,
    bool? loading,
  }) {
    return AdvancedSettingsViewState(
      api: api ?? this.api,
      network: network ?? this.network,
      apiError: identical(apiError, _unset)
          ? this.apiError
          : apiError as String?,
      networkError: identical(networkError, _unset)
          ? this.networkError
          : networkError as String?,
      loading: loading ?? this.loading,
    );
  }
}

const _unset = Object();

final class AdvancedSettingsViewModel
    extends Notifier<AdvancedSettingsViewState> {
  @override
  AdvancedSettingsViewState build() {
    scheduleMicrotask(reload);
    return const AdvancedSettingsViewState();
  }

  IntegrationSettingsRepository get _repository =>
      ref.read(integrationSettingsRepositoryProvider);

  Future<void> reload() async {
    state = state.copyWith(loading: true);
    try {
      final api = await _repository.loadApiServer();
      state = state.copyWith(api: api, apiError: api.bindErrorCode);
    } catch (_) {
      state = state.copyWith(
        apiError: AppErrorCode.apiServerBindFailed.wireName,
      );
    }
    try {
      final network = await _repository.loadNetworkSettings();
      state = state.copyWith(
        network: network,
        networkError: null,
        loading: false,
      );
    } catch (_) {
      state = state.copyWith(
        networkError: AppErrorCode.networkFailure.wireName,
        loading: false,
      );
    }
  }

  Future<void> setApiServerEnabled(bool enabled) async {
    try {
      final api = await _repository.setApiServerEnabled(enabled);
      state = state.copyWith(api: api, apiError: api.bindErrorCode);
    } catch (_) {
      state = state.copyWith(
        apiError: AppErrorCode.apiServerBindFailed.wireName,
      );
    }
  }

  Future<void> setApiServerPort(int port) async {
    final api = await _repository.setApiServerPort(port);
    state = state.copyWith(api: api, apiError: api.bindErrorCode);
  }

  void setInvalidPort() {
    state = state.copyWith(apiError: AppErrorCode.invalidPort.wireName);
  }

  void setProxyMode(NetworkProxyMode mode) {
    final network = state.network;
    if (network == null) return;
    state = state.copyWith(
      network: NetworkSettings(
        proxyMode: mode,
        proxyUrl: network.proxyUrl,
        proxyBypass: network.proxyBypass,
        checkUpdatesOnLaunch: network.checkUpdatesOnLaunch,
      ),
    );
  }

  void setCheckUpdatesOnLaunch(bool value) {
    final network = state.network;
    if (network == null) return;
    state = state.copyWith(
      network: NetworkSettings(
        proxyMode: network.proxyMode,
        proxyUrl: network.proxyUrl,
        proxyBypass: network.proxyBypass,
        checkUpdatesOnLaunch: value,
      ),
    );
  }

  Future<void> saveNetwork(
    NetworkSettings settings, {
    Future<void> Function()? applyNetwork,
  }) async {
    if (settings.proxyMode == NetworkProxyMode.custom) {
      final uri = Uri.tryParse(settings.proxyUrl.trim());
      if (uri == null ||
          (uri.scheme != 'http' && uri.scheme != 'https') ||
          uri.host.isEmpty ||
          uri.userInfo.isNotEmpty) {
        state = state.copyWith(
          networkError: AppErrorCode.proxyConfigurationInvalid.wireName,
        );
        return;
      }
    }
    try {
      final saved = await _repository.saveNetworkSettings(settings);
      await (applyNetwork ?? initializeSystemProxy)();
      state = state.copyWith(network: saved, networkError: null);
    } catch (_) {
      state = state.copyWith(
        networkError: AppErrorCode.networkFailure.wireName,
      );
    }
  }
}
