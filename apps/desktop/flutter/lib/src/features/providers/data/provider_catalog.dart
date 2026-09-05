import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_runtime/linguaray_runtime.dart';

import '../../../platform/credentials/secret_fields.dart';
import '../provider_labels.dart';
import 'provider_util.dart';

/// Builds the settings picker from the Rust catalog. Widgets never read JSON.
List<ProviderTypeOption> providerTypeOptionsFromCatalog(
  List<CatalogProviderPreset> presets,
) {
  return [for (final preset in presets) _optionFromPreset(preset)];
}

ProviderTypeOption _optionFromPreset(CatalogProviderPreset preset) {
  return ProviderTypeOption(
    id: preset.id,
    engineTypeId: preset.engineType,
    label: preset.name,
    isLlm: preset.llm,
    category: _categoryId(preset.category),
    networkPolicy: _networkId(preset.networkPolicy),
    stability: _stabilityId(preset.stability),
    homepageUrl: preset.homepageUrl,
    apiKeyUrl: preset.apiKeyUrl,
    descriptionKey: preset.descriptionKey,
    baseUrl: preset.baseUrl,
    modelsUrl: preset.modelsUrl,
    supportsTranslation: preset.translation,
    supportsOcr: preset.ocr,
    supportsDictionary: preset.dictionary,
    fields: [
      for (final field in preset.fields)
        ProviderFieldSpec(
          key: field.key,
          label: fieldLabel(field.key, field.labelKey),
          secret: field.secret || isSecretField(field.key),
          requiredField: field.required_,
          placeholder: field.placeholder ?? field.defaultValue,
          advanced: field.advanced,
          defaultValue: field.defaultValue,
          labelKey: field.labelKey,
        ),
    ],
  );
}

String _categoryId(CatalogCategory category) {
  return switch (category) {
    CatalogCategory.builtIn => 'builtIn',
    CatalogCategory.traditionalApi => 'traditionalApi',
    CatalogCategory.llmOfficial => 'llmOfficial',
    CatalogCategory.aggregator => 'aggregator',
    CatalogCategory.localOrSelfHosted => 'localOrSelfHosted',
  };
}

String _networkId(CatalogNetworkPolicy policy) {
  return switch (policy) {
    CatalogNetworkPolicy.localOnly => 'localOnly',
    CatalogNetworkPolicy.officialApi => 'officialApi',
    CatalogNetworkPolicy.unofficialWeb => 'unofficialWeb',
    CatalogNetworkPolicy.selfHosted => 'selfHosted',
  };
}

String _stabilityId(CatalogStability stability) {
  return switch (stability) {
    CatalogStability.stable => 'stable',
    CatalogStability.experimental => 'experimental',
  };
}

ProviderType parseProviderType(String id) {
  for (final type in ProviderType.values) {
    if (providerTypeValue(type) == id) return type;
  }
  throw ArgumentError.value(id, 'typeId', 'Unknown provider type');
}
