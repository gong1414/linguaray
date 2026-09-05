import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_runtime/linguaray_runtime.dart';

import '../../../i18n/i18n.dart';
import '../../../platform/credentials/secret_fields.dart';
import 'provider_util.dart';

/// Builds the settings picker from the Rust catalog. Widgets never read JSON.
List<ProviderTypeOption> providerTypeOptionsFromCatalog(
  List<CatalogProviderPreset> presets,
) {
  return [for (final preset in presets) _optionFromPreset(preset)];
}

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

String fieldLabel(String key, String? labelKey) {
  final fields = t.settings.providers.fields;
  return switch (key) {
    'apiKey' => fields.api_key,
    'authKey' => fields.auth_key,
    'appId' => fields.app_id,
    'appKey' => fields.app_key,
    'appSecret' => fields.app_secret,
    'secretId' => fields.secret_id,
    'secretKey' => fields.secret_key,
    'token' => fields.token,
    'requestId' => fields.request_id,
    'username' => fields.username,
    'defaultModel' => fields.default_model,
    'baseUrl' => fields.base_url,
    'modelsUrl' => fields.models_url,
    _ => labelKey ?? key,
  };
}

String catalogDescription(ProviderTypeOption option) {
  final catalog = t.settings.providers.catalog;
  return switch (option.id) {
    'system' => catalog.system,
    'ecdict' => catalog.ecdict,
    'google-web' => catalog.google_web,
    'bing-web' => catalog.bing_web,
    'tencent-transmart-web' => catalog.tencent_transmart_web,
    'deepl-free' => catalog.deepl_free,
    'deepl-pro' => catalog.deepl_pro,
    'google-cloud-translation' => catalog.google_cloud,
    'baidu-translate' => catalog.baidu,
    'tencent-cloud-tmt' => catalog.tencent_cloud,
    'youdao-zhiyun' => catalog.youdao,
    'caiyun' => catalog.caiyun,
    'openai' => catalog.openai,
    'openai-compatible' => catalog.custom,
    'minimax' => catalog.minimax,
    'stepfun' => catalog.stepfun,
    'mistral' => catalog.mistral,
    'together' => catalog.together,
    'fireworks' => catalog.fireworks,

    'anthropic' => catalog.anthropic,
    'gemini' => catalog.gemini,
    'deepseek' => catalog.deepseek,
    'bailian-qwen' => catalog.qwen,
    'zhipu-bigmodel' => catalog.zhipu,
    'moonshot-kimi' => catalog.moonshot,
    'doubao-ark' => catalog.doubao,
    'xai' => catalog.xai,
    'groq' => catalog.groq,
    'openrouter' => catalog.openrouter,
    'siliconflow-cn' => catalog.siliconflow_cn,
    'siliconflow-global' => catalog.siliconflow_global,
    'modelscope' => catalog.modelscope,
    'ollama' => catalog.ollama,
    'lm-studio' => catalog.lm_studio,
    'localai' => catalog.localai,
    'vllm' => catalog.vllm,
    'llama-cpp' => catalog.llama_cpp,
    'litellm' => catalog.litellm,
    'libretranslate' => catalog.libretranslate,
    'mtranserver' => catalog.mtranserver,
    _ => t.settings.providers.description.fallback,
  };
}

String categoryLabel(String category) {
  final section = t.settings.providers.section;
  return switch (category) {
    'builtIn' => section.built_in,
    'traditionalApi' => section.traditional_api,
    'llmOfficial' => section.llm_official,
    'aggregator' => section.aggregator,
    'localOrSelfHosted' => section.local_or_self_hosted,
    _ => category,
  };
}

String stabilityLabel(String stability) {
  final labels = t.settings.providers.stability;
  return switch (stability) {
    'experimental' => labels.experimental,
    _ => labels.stable,
  };
}

const List<String> kCatalogCategoryOrder = [
  'builtIn',
  'traditionalApi',
  'llmOfficial',
  'aggregator',
  'localOrSelfHosted',
];

ProviderType parseProviderType(String id) {
  for (final type in ProviderType.values) {
    if (providerTypeValue(type) == id) return type;
  }
  throw ArgumentError.value(id, 'typeId', 'Unknown provider type');
}
