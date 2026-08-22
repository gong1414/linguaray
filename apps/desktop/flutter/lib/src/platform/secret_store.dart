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
    final keys = {
      ...fields.keys,
      for (final key in existingFields.keys)
        if (isSecretField(key)) key,
    };
    for (final key in keys) {
      final fieldValue = fields[key] ?? '';
      if (!isSecretField(key)) {
        protected[key] = fieldValue;
        continue;
      }

      final value = fieldValue.trim();
      if (value.isNotEmpty && !isReference(value)) {
        _store.write(providerId: providerId, field: key, value: value);
        protected[key] = _reference(providerId, key);
      } else if (isReference(value)) {
        protected[key] = value;
      } else {
        final existing = existingFields[key];
        if (existing != null && isReference(existing)) {
          protected[key] = existing;
        }
      }
    }
    return protected;
  }

  /// Resolves a provider draft into an in-memory configuration suitable for
  /// a connection test. Unlike [protectFields], this method never writes to
  /// secure storage and never returns opaque secret references.
  Map<String, String> materializeFields({
    required String providerId,
    required Map<String, String> fields,
    Map<String, String> existingFields = const {},
  }) {
    final materialized = <String, String>{};
    final keys = {
      ...fields.keys,
      for (final key in existingFields.keys)
        if (isSecretField(key)) key,
    };
    for (final key in keys) {
      final fieldValue = fields[key] ?? '';
      if (!isSecretField(key)) {
        materialized[key] = fieldValue;
        continue;
      }

      final value = fieldValue.trim();
      if (value.isNotEmpty && !isReference(value)) {
        materialized[key] = value;
        continue;
      }

      final reference = isReference(value) ? value : existingFields[key];
      if (reference == null || !isReference(reference)) continue;
      final stored = _store.read(providerId: providerId, field: key);
      if (stored != null) materialized[key] = stored;
    }
    return materialized;
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
