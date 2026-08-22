import 'package:nativeapi/nativeapi.dart';

import '../services/runtime.dart';
import 'platform_types.dart';
import 'secret_fields.dart';

const _kSecretScope = 'io.github.gong1414.linguaray.v2.providers';
const _kSecretScheme = 'linguaray-secret';

class NativeSecretStore implements SecretStore {
  NativeSecretStore() : _storage = SecureStorage.withScope(_kSecretScope);

  final SecureStorage _storage;

  String _key(String providerId, String field) => '$providerId::$field';

  @override
  String? read({required String providerId, required String field}) {
    final value = _storage.get(_key(providerId, field));
    return value.isEmpty ? null : value;
  }

  @override
  void write({
    required String providerId,
    required String field,
    required String value,
  }) {
    if (!_storage.set(_key(providerId, field), value)) {
      throw StateError('Secure storage rejected a provider credential.');
    }
  }

  @override
  void delete({required String providerId, required String field}) {
    _storage.remove(_key(providerId, field));
  }

  @override
  void deleteProvider(String providerId) {
    final prefix = '$providerId::';
    for (final key in _storage.keys.where((key) => key.startsWith(prefix))) {
      _storage.remove(key);
    }
  }
}

class ProviderCredentialsController {
  ProviderCredentialsController({SecretStore? store})
      : _store = store ?? NativeSecretStore();

  final SecretStore _store;

  String _reference(String providerId, String field) => Uri(
        scheme: _kSecretScheme,
        host: providerId,
        pathSegments: [field],
      ).toString();

  bool isReference(String value) =>
      Uri.tryParse(value)?.scheme == _kSecretScheme;

  /// Stores secrets in the OS vault and returns fields safe for settings.json.
  Map<String, String> protectFields({
    required String providerId,
    required Map<String, String> fields,
    Map<String, String> existingFields = const {},
  }) {
    final protected = <String, String>{};
    for (final entry in fields.entries) {
      if (!isSecretField(entry.key)) {
        protected[entry.key] = entry.value;
        continue;
      }

      final value = entry.value.trim();
      if (value.isNotEmpty && !isReference(value)) {
        _store.write(
          providerId: providerId,
          field: entry.key,
          value: value,
        );
        protected[entry.key] = _reference(providerId, entry.key);
      } else if (isReference(value)) {
        protected[entry.key] = value;
      } else {
        final existing = existingFields[entry.key];
        if (existing != null && isReference(existing)) {
          protected[entry.key] = existing;
        }
      }
    }
    return protected;
  }

  Future<void> hydrateProvider(ProviderConfigEntry provider) async {
    final secrets = <String, String>{};
    for (final entry in provider.fields.entries) {
      if (!isSecretField(entry.key) || !isReference(entry.value)) continue;
      final value = _store.read(providerId: provider.id, field: entry.key);
      if (value != null) secrets[entry.key] = value;
    }
    await runtime.settings().setProviderSecrets(
          providerId: provider.id,
          secrets: secrets,
        );
  }

  Future<void> hydrateAll() async {
    final providers = await runtime.settings().listProviders();
    for (final provider in providers) {
      await hydrateProvider(provider);
    }
  }

  void deleteProvider(String providerId) => _store.deleteProvider(providerId);
}

late final ProviderCredentialsController providerCredentialsController;

void initProviderCredentialsController() {
  providerCredentialsController = ProviderCredentialsController();
}
