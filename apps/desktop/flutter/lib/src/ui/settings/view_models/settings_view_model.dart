import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../../config/dependencies.dart';

final generalSettingsViewModelProvider =
    NotifierProvider<GeneralSettingsViewModel, GeneralSettingsViewState>(
      GeneralSettingsViewModel.new,
    );

final class GeneralSettingsViewState {
  const GeneralSettingsViewState({
    this.preferences,
    this.languages = const [],
    this.translationLanguages = const [],
    this.loading = true,
    this.errorCode,
  });

  final GeneralPreferences? preferences;
  final List<LanguageChoice> languages;
  final List<LanguageChoice> translationLanguages;
  final bool loading;
  final String? errorCode;

  GeneralSettingsViewState copyWith({
    GeneralPreferences? preferences,
    List<LanguageChoice>? languages,
    List<LanguageChoice>? translationLanguages,
    bool? loading,
    Object? errorCode = _unset,
  }) {
    return GeneralSettingsViewState(
      preferences: preferences ?? this.preferences,
      languages: languages ?? this.languages,
      translationLanguages: translationLanguages ?? this.translationLanguages,
      loading: loading ?? this.loading,
      errorCode: identical(errorCode, _unset)
          ? this.errorCode
          : errorCode as String?,
    );
  }
}

final class GeneralSettingsViewModel
    extends Notifier<GeneralSettingsViewState> {
  @override
  GeneralSettingsViewState build() {
    scheduleMicrotask(reload);
    return const GeneralSettingsViewState();
  }

  Future<void> reload() async {
    try {
      final repository = ref.read(workspaceSettingsRepositoryProvider);
      final preferences = await repository.loadGeneral();
      final languages = await repository.listAppLanguages();
      final translationLanguages = await repository.listTranslationLanguages();
      state = GeneralSettingsViewState(
        preferences: preferences,
        languages: languages,
        translationLanguages: translationLanguages,
        loading: false,
      );
    } catch (_) {
      state = state.copyWith(
        loading: false,
        errorCode: AppErrorCode.unknown.wireName,
      );
    }
  }

  Future<void> _run(Future<void> Function() action) async {
    try {
      await action();
      await reload();
    } catch (_) {
      state = state.copyWith(errorCode: AppErrorCode.unknown.wireName);
    }
  }

  Future<void> setLaunchAtLogin(bool value) => _run(
    () => ref.read(workspaceSettingsRepositoryProvider).setLaunchAtLogin(value),
  );

  Future<void> setShowInMenuBar(bool value) => _run(
    () => ref.read(workspaceSettingsRepositoryProvider).setShowInMenuBar(value),
  );

  Future<void> setLanguage(String language) => _run(
    () => ref.read(workspaceSettingsRepositoryProvider).setLanguage(language),
  );

  Future<void> setThemeMode(ThemePreference mode) => _run(
    () => ref.read(workspaceSettingsRepositoryProvider).setThemeMode(mode),
  );

  Future<void> setCommonLanguages(List<String> codes) => _run(
    () =>
        ref.read(workspaceSettingsRepositoryProvider).setCommonLanguages(codes),
  );

  Future<void> setTranslationTargets(List<TranslationTargetRule> targets) =>
      _run(
        () => ref
            .read(workspaceSettingsRepositoryProvider)
            .setTranslationTargets(targets),
      );

  Future<void> setInputSubmitMode(InputSubmitMode mode) => _run(
    () =>
        ref.read(workspaceSettingsRepositoryProvider).setInputSubmitMode(mode),
  );

  Future<void> setAutoCopyDetectedText(bool value) => _run(
    () => ref
        .read(workspaceSettingsRepositoryProvider)
        .setAutoCopyDetectedText(value),
  );

  Future<void> setDoubleClickCopyResult(bool value) => _run(
    () => ref
        .read(workspaceSettingsRepositoryProvider)
        .setDoubleClickCopyResult(value),
  );
}

final servicesSettingsViewModelProvider =
    NotifierProvider<ServicesSettingsViewModel, ServicesSettingsViewState>(
      ServicesSettingsViewModel.new,
    );

final class ServicesSettingsViewState {
  const ServicesSettingsViewState({
    this.services = const [],
    this.loading = true,
    this.operationErrorCode,
  });

  final List<ServiceRecord> services;
  final bool loading;
  final String? operationErrorCode;
}

final class ServicesSettingsViewModel
    extends Notifier<ServicesSettingsViewState> {
  @override
  ServicesSettingsViewState build() {
    scheduleMicrotask(reload);
    return const ServicesSettingsViewState();
  }

  Future<void> reload() async {
    try {
      final services = await ref
          .read(workspaceSettingsRepositoryProvider)
          .listServices();
      state = ServicesSettingsViewState(services: services, loading: false);
    } catch (_) {
      state = ServicesSettingsViewState(
        services: state.services,
        loading: false,
        operationErrorCode: AppErrorCode.unknown.wireName,
      );
    }
  }

  Future<void> setEnabled(String id, bool enabled) async {
    try {
      await ref
          .read(workspaceSettingsRepositoryProvider)
          .setServiceEnabled(serviceId: id, enabled: enabled);
      await reload();
    } catch (_) {
      state = ServicesSettingsViewState(
        services: state.services,
        loading: false,
        operationErrorCode: AppErrorCode.unknown.wireName,
      );
    }
  }

  Future<void> makeDefault(String id) async {
    final service = state.services.where((item) => item.id == id).firstOrNull;
    if (service == null) return;
    final repository = ref.read(workspaceSettingsRepositoryProvider);
    try {
      if (service.kind == 'ocr') {
        await repository.setDefaultOcrService(id);
      } else if (service.kind == 'dictionary') {
        await repository.setDefaultDictionaryService(id);
      } else {
        await repository.setDefaultTranslationService(id);
      }
      await reload();
    } catch (_) {
      state = ServicesSettingsViewState(
        services: state.services,
        loading: false,
        operationErrorCode: AppErrorCode.unknown.wireName,
      );
    }
  }

  Future<void> reorderTranslation(int oldIndex, int newIndex) async {
    final translation = state.services
        .where((service) => service.kind == 'translation')
        .toList();
    if (oldIndex < 0 || oldIndex >= translation.length) return;
    final item = translation.removeAt(oldIndex);
    translation.insert(newIndex.clamp(0, translation.length), item);
    try {
      await ref
          .read(workspaceSettingsRepositoryProvider)
          .reorderTranslationServices([
            for (final service in translation) service.id,
          ]);
      await reload();
    } catch (_) {
      state = ServicesSettingsViewState(
        services: state.services,
        loading: false,
        operationErrorCode: AppErrorCode.unknown.wireName,
      );
    }
  }

  Future<void> addService(ServiceDraft draft) async {
    try {
      await ref.read(workspaceSettingsRepositoryProvider).saveService(draft);
      await reload();
    } catch (_) {
      state = ServicesSettingsViewState(
        services: state.services,
        loading: false,
        operationErrorCode: AppErrorCode.unknown.wireName,
      );
    }
  }

  Future<void> deleteService(String id) async {
    try {
      await ref.read(workspaceSettingsRepositoryProvider).deleteService(id);
      await reload();
    } catch (_) {
      state = ServicesSettingsViewState(
        services: state.services,
        loading: false,
        operationErrorCode: AppErrorCode.unknown.wireName,
      );
    }
  }
}

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
  @override
  ProvidersSettingsViewState build() {
    scheduleMicrotask(reload);
    return const ProvidersSettingsViewState();
  }

  Future<void> reload() async {
    final repository = ref.read(workspaceSettingsRepositoryProvider);
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
      await ref.read(workspaceSettingsRepositoryProvider).saveProvider(draft);
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
    await ref.read(workspaceSettingsRepositoryProvider).deleteProvider(id);
    await reload();
  }

  Future<void> test(ProviderDraft draft) async {
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
          .read(workspaceSettingsRepositoryProvider)
          .testProvider(draft);
      state = state.copyWith(testResult: result);
    } catch (_) {
      state = state.copyWith(
        testResult: const ProviderTestResult(
          status: ProviderTestStatus.failed,
          errorCode: 'network_error',
        ),
      );
    } finally {
      state = state.copyWith(testing: false);
    }
  }

  void clearFeedback() {
    state = state.copyWith(testResult: null, operationErrorCode: null);
  }

  String? _validationError(ProviderDraft draft) {
    if (draft.id.trim().isEmpty) return 'validation_missing';
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
    for (final field in type.fields.where((item) => item.requiredField)) {
      final value = draft.fields[field.key]?.trim() ?? '';
      final keepsStoredSecret =
          field.secret &&
          existing?.storedSecretKeys.contains(field.key) == true;
      if (value.isEmpty && !keepsStoredSecret) return 'validation_missing';
    }
    return null;
  }
}

final aboutViewModelProvider = NotifierProvider<AboutViewModel, AboutInfo?>(
  AboutViewModel.new,
);

final class AboutViewModel extends Notifier<AboutInfo?> {
  @override
  AboutInfo? build() {
    scheduleMicrotask(() async {
      state = await ref.read(workspaceSettingsRepositoryProvider).loadAbout();
    });
    return null;
  }
}
