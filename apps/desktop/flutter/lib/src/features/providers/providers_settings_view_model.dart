import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../app/dependencies.dart';

final providersSettingsViewModelProvider =
    NotifierProvider<ProvidersSettingsViewModel, ProvidersSettingsViewState>(
      ProvidersSettingsViewModel.new,
    );

final class ProvidersSettingsViewState {
  const ProvidersSettingsViewState({
    this.providers = const [],
    this.types = const [],
    this.loading = true,
    this.saving = false,
    this.testing = false,
    this.testResult,
    this.operationErrorCode,
  });

  final List<ProviderRecord> providers;
  final List<ProviderTypeOption> types;
  final bool loading;
  final bool saving;
  final bool testing;
  final ProviderTestResult? testResult;
  final String? operationErrorCode;

  ProvidersSettingsViewState copyWith({
    List<ProviderRecord>? providers,
    List<ProviderTypeOption>? types,
    bool? loading,
    bool? saving,
    bool? testing,
    Object? testResult = _unset,
    Object? operationErrorCode = _unset,
  }) {
    return ProvidersSettingsViewState(
      providers: providers ?? this.providers,
      types: types ?? this.types,
      loading: loading ?? this.loading,
      saving: saving ?? this.saving,
      testing: testing ?? this.testing,
      testResult: identical(testResult, _unset)
          ? this.testResult
          : testResult as ProviderTestResult?,
      operationErrorCode: identical(operationErrorCode, _unset)
          ? this.operationErrorCode
          : operationErrorCode as String?,
    );
  }
}

const Object _unset = Object();

final class ProvidersSettingsViewModel
    extends Notifier<ProvidersSettingsViewState> {
  int _testGeneration = 0;

  @override
  ProvidersSettingsViewState build() {
    scheduleMicrotask(reload);
    return const ProvidersSettingsViewState();
  }

  Future<void> reload() async {
    final repository = ref.read(providerSettingsRepositoryProvider);
    final providers = await repository.listProviders();
    final types = await repository.listProviderTypes();
    state = state.copyWith(providers: providers, types: types, loading: false);
  }

  Future<bool> save(ProviderDraft draft) async {
    final validationError = _validationError(draft);
    if (validationError != null) {
      state = state.copyWith(operationErrorCode: validationError);
      return false;
    }

    state = state.copyWith(saving: true, operationErrorCode: null);
    try {
      await ref.read(providerSettingsRepositoryProvider).saveProvider(draft);
      await reload();
      return true;
    } catch (_) {
      state = state.copyWith(operationErrorCode: 'save_failed');
      return false;
    } finally {
      state = state.copyWith(saving: false);
    }
  }

  Future<void> delete(String id) async {
    await ref.read(providerSettingsRepositoryProvider).deleteProvider(id);
    await reload();
  }

  Future<void> test(ProviderDraft draft) async {
    final generation = ++_testGeneration;
    final validationError = _validationError(draft);
    if (validationError != null) {
      state = state.copyWith(
        testing: false,
        testResult: ProviderTestResult(
          status: ProviderTestStatus.failed,
          errorCode: validationError,
        ),
      );
      return;
    }

    state = state.copyWith(
      testing: true,
      testResult: null,
      operationErrorCode: null,
    );
    try {
      final result = await ref
          .read(providerSettingsRepositoryProvider)
          .testProvider(draft);
      if (generation != _testGeneration || !ref.mounted) return;
      state = state.copyWith(testResult: result);
    } catch (_) {
      if (generation != _testGeneration || !ref.mounted) return;
      state = state.copyWith(
        testResult: const ProviderTestResult(
          status: ProviderTestStatus.failed,
          errorCode: 'network_error',
        ),
      );
    } finally {
      if (generation == _testGeneration && ref.mounted) {
        state = state.copyWith(testing: false);
      }
    }
  }

  void clearFeedback() {
    _testGeneration++;
    state = state.copyWith(
      testing: false,
      testResult: null,
      operationErrorCode: null,
    );
  }

  String? _validationError(ProviderDraft draft) {
    final type =
        state.types.where((item) => item.id == draft.presetId).firstOrNull ??
        state.types
            .where(
              (item) =>
                  item.engineTypeId == draft.typeId || item.id == draft.typeId,
            )
            .firstOrNull;
    if (type == null) return 'validation_missing';
    final existing = state.providers
        .where((item) => item.id == draft.id)
        .firstOrNull;
    return validateProviderDraft(
      draft: draft,
      type: type,
      storedSecretKeys: existing?.storedSecretKeys ?? const {},
    ).errorCode;
  }
}
