import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_desktop/src/platform/platform_types.dart';
import 'package:linguaray_desktop/src/platform/secret_store.dart';

class _MemorySecretStore implements SecretStore {
  final values = <String, String>{};

  String _key(String providerId, String field) => '$providerId::$field';

  @override
  void delete({required String providerId, required String field}) {
    values.remove(_key(providerId, field));
  }

  @override
  void deleteProvider(String providerId) {
    values.removeWhere((key, _) => key.startsWith('$providerId::'));
  }

  @override
  String? read({required String providerId, required String field}) =>
      values[_key(providerId, field)];

  @override
  void write({
    required String providerId,
    required String field,
    required String value,
  }) {
    values[_key(providerId, field)] = value;
  }
}

void main() {
  test('replaces provider secrets with opaque references', () {
    final store = _MemorySecretStore();
    final controller = ProviderCredentialsController(store: store);

    final fields = controller.protectFields(
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

  test('blank edit preserves the existing secret reference', () {
    final controller = ProviderCredentialsController(
      store: _MemorySecretStore(),
    );
    const reference = 'linguaray-secret://openai/apiKey';

    final fields = controller.protectFields(
      providerId: 'openai',
      fields: const {'apiKey': ''},
      existingFields: const {'apiKey': reference},
    );

    expect(fields['apiKey'], reference);
  });

  test('untouched edit preserves a secret omitted from the draft', () {
    final controller = ProviderCredentialsController(
      store: _MemorySecretStore(),
    );
    const reference = 'linguaray-secret://openai/apiKey';

    final fields = controller.protectFields(
      providerId: 'openai',
      fields: const {'baseUrl': 'https://example.test'},
      existingFields: const {'apiKey': reference},
    );

    expect(fields['apiKey'], reference);
    expect(fields['baseUrl'], 'https://example.test');
  });

  test('provider deletion removes only its own secrets', () {
    final store = _MemorySecretStore()
      ..values.addAll({'openai::apiKey': 'one', 'deepl::apiKey': 'two'});
    final controller = ProviderCredentialsController(store: store);

    controller.deleteProvider('openai');

    expect(store.values, {'deepl::apiKey': 'two'});
  });

  test('materializes a draft for testing without writing secure storage', () {
    final store = _MemorySecretStore()
      ..values['openai::apiKey'] = 'stored-secret';
    final controller = ProviderCredentialsController(store: store);
    final before = Map<String, String>.of(store.values);

    final existing = controller.materializeFields(
      providerId: 'openai',
      fields: const {'baseUrl': 'https://example.test'},
      existingFields: const {'apiKey': 'linguaray-secret://openai/apiKey'},
    );
    final unsaved = controller.materializeFields(
      providerId: 'draft',
      fields: const {'apiKey': 'temporary-secret'},
    );

    expect(existing['apiKey'], 'stored-secret');
    expect(existing['baseUrl'], 'https://example.test');
    expect(unsaved['apiKey'], 'temporary-secret');
    expect(store.values, before);
  });
}
