import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_desktop/src/app/runtime.dart';
import 'package:linguaray_desktop/src/features/providers/data/provider_util.dart';

void main() {
  test(
    'provider capabilities are exhaustive and LLM translation is streamed',
    () {
      const llmTypes = {
        ProviderType.anthropic,
        ProviderType.openAi,
        ProviderType.ollama,
        ProviderType.xAi,
        ProviderType.deepSeek,
        ProviderType.qwen,
        ProviderType.zhipu,
        ProviderType.moonshot,
        ProviderType.doubao,
        ProviderType.groq,
        ProviderType.gemini,
        ProviderType.openAiCompatible,
      };

      for (final type in ProviderType.values) {
        final capabilities = providerCapabilitiesForType(type);
        expect(capabilities.isLlm, llmTypes.contains(type), reason: '$type');
        expect(
          capabilities.streamingTranslation,
          capabilities.isLlm,
          reason: '$type',
        );
        expect(
          capabilities.omitsSourceLanguage,
          capabilities.streamingTranslation,
          reason: '$type',
        );
      }
    },
  );

  test('draft validation shares required-field and stored-secret rules', () {
    const type = ProviderTypeOption(
      id: 'openai',
      label: 'OpenAI',
      isLlm: true,
      fields: [
        ProviderFieldSpec(
          key: 'apiKey',
          label: 'API key',
          secret: true,
          requiredField: true,
        ),
        ProviderFieldSpec(
          key: 'defaultModel',
          label: 'Default model',
          secret: false,
          requiredField: true,
        ),
      ],
    );
    const draft = ProviderDraft(
      id: 'provider',
      typeId: 'openai',
      fields: {'apiKey': '', 'defaultModel': 'gpt'},
    );

    expect(validateProviderDraft(draft: draft, type: type).fieldKey, 'apiKey');
    expect(
      validateProviderDraft(
        draft: draft,
        type: type,
        storedSecretKeys: const {'apiKey'},
      ).isValid,
      isTrue,
    );
    expect(
      validateProviderDraft(
        draft: const ProviderDraft(
          id: 'provider',
          typeId: 'openai',
          fields: {'apiKey': 'secret'},
        ),
        type: type,
        ignoredRequiredFields: const {'defaultModel'},
      ).isValid,
      isTrue,
    );
  });

  test('missing ids and unknown types use the stable validation error', () {
    const draft = ProviderDraft(id: '', typeId: 'unknown', fields: {});
    expect(
      validateProviderDraft(draft: draft, type: null).issue,
      ProviderDraftValidationIssue.missingId,
    );
    expect(
      validateProviderDraft(draft: draft, type: null).errorCode,
      'validation_missing',
    );
  });
}
