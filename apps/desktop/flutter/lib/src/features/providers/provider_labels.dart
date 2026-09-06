import 'package:linguaray_application/linguaray_application.dart';

import '../../i18n/i18n.dart';

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
