import 'package:linguaray_application/src/settings/models.dart';

abstract interface class ServiceSettingsRepository {
  Future<String?> loadDefaultTranslationService();

  Future<void> setDefaultTranslationService(String? serviceId);

  Future<String?> loadDefaultOcrService();

  Future<void> setDefaultOcrService(String? serviceId);

  Future<String?> loadDefaultDictionaryService();

  Future<void> setDefaultDictionaryService(String? serviceId);

  Future<List<ServiceRecord>> listServices();

  Future<void> setServiceEnabled({
    required String serviceId,
    required bool enabled,
  });

  Future<void> saveService(ServiceDraft draft);

  Future<void> deleteService(String serviceId);

  Future<void> reorderTranslationServices(List<String> serviceIds);
}
