import 'package:linguaray_application/src/settings/models.dart';

/// Resolves an exact catalog preset first. Falling back to an engine type is
/// only for settings created before `presetId` was persisted; several catalog
/// entries intentionally share the same OpenAI-compatible engine.
ProviderTypeOption? findProviderCatalogOption(
  List<ProviderTypeOption> options, {
  String? presetId,
  String? engineTypeId,
}) {
  final exactId = presetId?.trim();
  if (exactId != null && exactId.isNotEmpty) {
    for (final option in options) {
      if (option.id == exactId) return option;
    }
  }
  final engineId = engineTypeId?.trim();
  if (engineId != null && engineId.isNotEmpty) {
    if (engineId == 'openai_compatible') {
      final custom = options
          .where((option) => option.id == 'openai-compatible')
          .firstOrNull;
      if (custom != null) return custom;
    }
    for (final option in options) {
      if (option.engineTypeId == engineId || option.id == engineId) {
        return option;
      }
    }
  }
  return null;
}

/// Fields that must accompany a catalog selection even when they are not
/// user-editable. In particular, DeepL Free and Pro share one engine but use
/// different API roots.
Map<String, String> providerPresetInitialFields(ProviderTypeOption option) {
  return <String, String>{
    if (option.baseUrl.trim().isNotEmpty) 'baseUrl': option.baseUrl.trim(),
    for (final field in option.fields)
      if (field.defaultValue?.trim().isNotEmpty == true)
        field.key: field.defaultValue!.trim(),
  };
}
