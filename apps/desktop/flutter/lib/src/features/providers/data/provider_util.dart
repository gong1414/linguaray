import '../../../app/runtime.dart';
import '../../../i18n/i18n.dart';

final class ProviderCapabilities {
  const ProviderCapabilities({
    required this.isLlm,
    required this.streamingTranslation,
    required this.omitsSourceLanguage,
  });

  final bool isLlm;
  final bool streamingTranslation;
  final bool omitsSourceLanguage;
}

const _llmCapabilities = ProviderCapabilities(
  isLlm: true,
  streamingTranslation: true,
  omitsSourceLanguage: true,
);
const _traditionalCapabilities = ProviderCapabilities(
  isLlm: false,
  streamingTranslation: false,
  omitsSourceLanguage: false,
);

ProviderCapabilities providerCapabilitiesForType(ProviderType? type) {
  return switch (type) {
    ProviderType.anthropic ||
    ProviderType.openAi ||
    ProviderType.ollama ||
    ProviderType.xAi ||
    ProviderType.deepSeek ||
    ProviderType.qwen ||
    ProviderType.zhipu ||
    ProviderType.moonshot ||
    ProviderType.doubao ||
    ProviderType.groq ||
    ProviderType.gemini ||
    ProviderType.openAiCompatible => _llmCapabilities,
    ProviderType.baidu ||
    ProviderType.caiyun ||
    ProviderType.deepL ||
    ProviderType.google ||
    ProviderType.system ||
    ProviderType.ecdict ||
    ProviderType.tencent ||
    ProviderType.youdao ||
    ProviderType.googleWeb ||
    ProviderType.bingWeb ||
    ProviderType.tencentTransmartWeb ||
    ProviderType.libreTranslate ||
    ProviderType.mTranServer ||
    null => _traditionalCapabilities,
  };
}

/// Stable wire value persisted by the Rust runtime for a provider type.
String providerTypeValue(ProviderType type) {
  return switch (type) {
    ProviderType.anthropic => 'anthropic',
    ProviderType.baidu => 'baidu',
    ProviderType.caiyun => 'caiyun',
    ProviderType.deepL => 'deepl',
    ProviderType.google => 'google',
    ProviderType.openAi => 'openai',
    ProviderType.ollama => 'ollama',
    ProviderType.xAi => 'xai',
    ProviderType.deepSeek => 'deepseek',
    ProviderType.qwen => 'qwen',
    ProviderType.zhipu => 'zhipu',
    ProviderType.moonshot => 'moonshot',
    ProviderType.doubao => 'doubao',
    ProviderType.groq => 'groq',
    ProviderType.gemini => 'gemini',
    ProviderType.openAiCompatible => 'openai_compatible',
    ProviderType.system => 'system',
    ProviderType.ecdict => 'ecdict',
    ProviderType.tencent => 'tencent',
    ProviderType.youdao => 'youdao',
    ProviderType.googleWeb => 'google_web',
    ProviderType.bingWeb => 'bing_web',
    ProviderType.tencentTransmartWeb => 'tencent_transmart_web',
    ProviderType.libreTranslate => 'libretranslate',
    ProviderType.mTranServer => 'mtranserver',
  };
}

String providerTypeDisplayName(ProviderType type) {
  return switch (type) {
    ProviderType.anthropic => t.common.provider.anthropic,
    ProviderType.baidu => t.common.provider.baidu,
    ProviderType.caiyun => t.common.provider.caiyun,
    ProviderType.deepL => t.common.provider.deepl,
    ProviderType.google => t.common.provider.google,
    ProviderType.openAi => t.common.provider.openai,
    ProviderType.ollama => t.common.provider.ollama,
    ProviderType.xAi => t.common.provider.xai,
    ProviderType.deepSeek => 'DeepSeek',
    ProviderType.qwen => 'Qwen',
    ProviderType.zhipu => 'Zhipu GLM',
    ProviderType.moonshot => 'Moonshot Kimi',
    ProviderType.doubao => 'Doubao',
    ProviderType.groq => 'Groq',
    ProviderType.gemini => 'Gemini',
    ProviderType.openAiCompatible => 'OpenAI Compatible',
    ProviderType.system => t.common.provider.system,
    ProviderType.ecdict => 'ECDICT',
    ProviderType.tencent => t.common.provider.tencent,
    ProviderType.youdao => t.common.provider.youdao,
    ProviderType.googleWeb => 'Google Web',
    ProviderType.bingWeb => 'Bing Web',
    ProviderType.tencentTransmartWeb => 'Tencent Transmart',
    ProviderType.libreTranslate => 'LibreTranslate',
    ProviderType.mTranServer => 'MTranServer',
  };
}

/// Missing means enabled so settings created before this flag keep working.
bool isServiceEnabled(ServiceConfigEntry service) =>
    service.fields[kServiceEnabledField] != 'false';

const String kServiceEnabledField = 'enabled';
