import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../app/dependencies.dart';

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
      final repository = ref.read(preferencesRepositoryProvider);
      final preferences = await repository.loadGeneral();
      final languages = await repository.listAppLanguages();
      final translationLanguages = await ref
          .read(translationPreferencesRepositoryProvider)
          .listTranslationLanguages();
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
    () => ref.read(preferencesRepositoryProvider).setLaunchAtLogin(value),
  );

  Future<void> setShowInMenuBar(bool value) => _run(
    () => ref.read(preferencesRepositoryProvider).setShowInMenuBar(value),
  );

  Future<void> setLanguage(String language) =>
      _run(() => ref.read(preferencesRepositoryProvider).setLanguage(language));

  Future<void> setThemeMode(ThemePreference mode) =>
      _run(() => ref.read(preferencesRepositoryProvider).setThemeMode(mode));

  Future<void> setCommonLanguages(List<String> codes) => _run(
    () => ref
        .read(translationPreferencesRepositoryProvider)
        .setCommonLanguages(codes),
  );

  Future<void> setTranslationTargets(List<TranslationTargetRule> targets) =>
      _run(
        () => ref
            .read(translationPreferencesRepositoryProvider)
            .setTranslationTargets(targets),
      );

  Future<void> setInputSubmitMode(InputSubmitMode mode) => _run(
    () => ref
        .read(translationPreferencesRepositoryProvider)
        .setInputSubmitMode(mode),
  );

  Future<void> setAutoCopyDetectedText(bool value) => _run(
    () => ref
        .read(translationPreferencesRepositoryProvider)
        .setAutoCopyDetectedText(value),
  );

  Future<void> setDoubleClickCopyResult(bool value) => _run(
    () => ref
        .read(translationPreferencesRepositoryProvider)
        .setDoubleClickCopyResult(value),
  );
}

const Object _unset = Object();
