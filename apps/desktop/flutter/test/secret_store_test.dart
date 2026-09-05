import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_desktop/src/platform/credentials/secret_store.dart';
import 'package:linguaray_desktop/src/platform/platform_types.dart';

class _MemorySecretStore implements SecretStore {
  final values = <String, String>{};

  String _key(String providerId, String field) => '$providerId::$field';

  @override
  Future<void> delete({
    required String providerId,
    required String field,
  }) async {
    values.remove(_key(providerId, field));
  }

  @override
  Future<String?> read({
    required String providerId,
    required String field,
  }) async => values[_key(providerId, field)];

  @override
  Future<void> write({
    required String providerId,
    required String field,
    required String value,
  }) async {
    values[_key(providerId, field)] = value;
  }
}

void main() {
  test('replaces provider secrets with opaque references', () async {
    final store = _MemorySecretStore();
    final controller = ProviderCredentialsController(store: store);

    final fields = await controller.protectFields(
      providerId: 'openai',
      fields: const {
        'apiKey': 'sk-plaintext',
        'baseUrl': 'https://example.test',
      },
    );

    expect(store.values['openai::apiKey'], 'sk-plaintext');
    expect(fields['apiKey'], startsWith('linguaray-secret://'));
    expect(fields.values, isNot(contains('sk-plaintext')));
    expect(fields['baseUrl'], 'https://example.test');
  });

  test('blank edit preserves the existing secret reference', () async {
    final controller = ProviderCredentialsController(
      store: _MemorySecretStore(),
    );
    const reference = 'linguaray-secret://openai/apiKey';

    final fields = await controller.protectFields(
      providerId: 'openai',
      fields: const {'apiKey': ''},
      existingFields: const {'apiKey': reference},
    );

    expect(fields['apiKey'], reference);
  });

  test('untouched edit preserves a secret omitted from the draft', () async {
    final controller = ProviderCredentialsController(
      store: _MemorySecretStore(),
    );
    const reference = 'linguaray-secret://openai/apiKey';

    final fields = await controller.protectFields(
      providerId: 'openai',
      fields: const {'baseUrl': 'https://example.test'},
      existingFields: const {'apiKey': reference},
    );

    expect(fields['apiKey'], reference);
    expect(fields['baseUrl'], 'https://example.test');
  });

  test('provider deletion removes only its own secrets', () async {
    final store = _MemorySecretStore()
      ..values.addAll({'openai::apiKey': 'one', 'deepl::apiKey': 'two'});
    final controller = ProviderCredentialsController(store: store);

    await controller.deleteProvider('openai', fields: ['apiKey', 'baseUrl']);

    expect(store.values, {'deepl::apiKey': 'two'});
  });

  test(
    'materializes a draft for testing without writing secure storage',
    () async {
      final store = _MemorySecretStore()
        ..values['openai::apiKey'] = 'stored-secret';
      final controller = ProviderCredentialsController(store: store);
      final before = Map<String, String>.of(store.values);

      final existing = await controller.materializeFields(
        providerId: 'openai',
        fields: const {'baseUrl': 'https://example.test'},
        existingFields: const {'apiKey': 'linguaray-secret://openai/apiKey'},
      );
      final unsaved = await controller.materializeFields(
        providerId: 'draft',
        fields: const {'apiKey': 'temporary-secret'},
      );

      expect(existing['apiKey'], 'stored-secret');
      expect(existing['baseUrl'], 'https://example.test');
      expect(unsaved['apiKey'], 'temporary-secret');
      expect(store.values, before);
    },
  );
}
