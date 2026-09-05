import 'dart:convert';

import 'package:cryptography/cryptography.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../config/update_signing_key.dart';

const updateRepositoryName = 'gong1414/linguaray';
const updateManifestUrl =
    'https://github.com/$updateRepositoryName/releases/latest/download/update.json';
const maximumUpdateBytes = 1024 * 1024 * 1024;

/// The signature covers the original payload bytes, including version, platform,
/// file name, length and digest. No URL supplied by the feed is ever followed.
final class SignedUpdateFeed {
  const SignedUpdateFeed({
    required this.platform,
    this.publicKey = updateSigningPublicKey,
    this.releaseBaseUrl =
        'https://github.com/$updateRepositoryName/releases/download',
  });

  final String platform;
  final String publicKey;
  final String releaseBaseUrl;

  Future<UpdateManifest> decode(List<int> envelopeBytes) async {
    try {
      final envelope =
          jsonDecode(utf8.decode(envelopeBytes)) as Map<String, dynamic>;
      final payload = envelope['payload'] as String;
      final signature = envelope['signature'] as String;
      final bytes = base64Decode(payload);
      final valid = await Ed25519().verify(
        bytes,
        signature: Signature(
          base64Decode(signature),
          publicKey: SimplePublicKey(
            base64Decode(publicKey),
            type: KeyPairType.ed25519,
          ),
        ),
      );
      if (!valid) throw const AppFailure(AppErrorCode.updateSignatureInvalid);
      final data = jsonDecode(utf8.decode(bytes)) as Map<String, dynamic>;
      final version = data['version'] as String;
      if (data['schemaVersion'] != 1 ||
          data['repository'] != updateRepositoryName ||
          !RegExp(r'^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$')
              .hasMatch(version)) {
        throw const AppFailure(AppErrorCode.updateCheckFailed);
      }
      final artifacts = data['artifacts'] as Map<String, dynamic>;
      final artifact = artifacts[platform] as Map<String, dynamic>;
      final wanted = switch (platform) {
        'macos' => 'LinguaRay-macos.dmg',
        'windows' => 'LinguaRay-windows-x64.exe',
        _ => throw const AppFailure(AppErrorCode.updateCheckFailed),
      };
      final digest = artifact['sha256'] as String;
      final size = artifact['size'] as int;
      if (artifact['name'] != wanted ||
          !RegExp(r'^[a-f0-9]{64}$').hasMatch(digest) ||
          size <= 0 ||
          size > maximumUpdateBytes ||
          artifact['platformSigned'] is! bool) {
        throw const AppFailure(AppErrorCode.updateCheckFailed);
      }
      return UpdateManifest(
        version: version,
        notes: data['notes'] as String,
        assetName: wanted,
        assetUrl: '$releaseBaseUrl/v$version/$wanted',
        checksumSha256: digest,
        byteLength: size,
        platformSigned: artifact['platformSigned'] as bool,
        signedPayload: payload,
        signatureBase64: signature,
        publishedAt: data['publishedAt'] as String,
      );
    } on AppFailure {
      rethrow;
    } catch (_) {
      throw const AppFailure(AppErrorCode.updateSignatureInvalid);
    }
  }

  Future<void> verify(UpdateManifest manifest) async {
    final verified = await decode(
      utf8.encode(
        jsonEncode({
          'payload': manifest.signedPayload,
          'signature': manifest.signatureBase64,
        }),
      ),
    );
    if (verified.version != manifest.version ||
        verified.assetName != manifest.assetName ||
        verified.assetUrl != manifest.assetUrl ||
        verified.checksumSha256 != manifest.checksumSha256 ||
        verified.byteLength != manifest.byteLength ||
        verified.platformSigned != manifest.platformSigned) {
      throw const AppFailure(AppErrorCode.updateSignatureInvalid);
    }
  }
}
