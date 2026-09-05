import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../app/dependencies.dart';

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
          .read(serviceSettingsRepositoryProvider)
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
          .read(serviceSettingsRepositoryProvider)
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
    final repository = ref.read(serviceSettingsRepositoryProvider);
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
          .read(serviceSettingsRepositoryProvider)
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
      await ref.read(serviceSettingsRepositoryProvider).saveService(draft);
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
      await ref.read(serviceSettingsRepositoryProvider).deleteService(id);
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
