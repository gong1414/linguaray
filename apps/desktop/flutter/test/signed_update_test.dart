import 'dart:convert';
import 'dart:io';

import 'package:crypto/crypto.dart' as crypto;
import 'package:cryptography/cryptography.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_desktop/src/data/github_update_repository.dart';
import 'package:linguaray_desktop/src/data/signed_update_feed.dart';

void main() {
  late SimpleKeyPair key;
  late String publicKey;
  final content = utf8.encode('Synthetic installer fixture, never executed.');

  Map<String, dynamic> payload() => {
    'schemaVersion': 1,
    'repository': updateRepositoryName,
    'version': '0.6.2',
    'notes': '测试更新',
    'publishedAt': '2026-09-05T00:00:00Z',
    'artifacts': {
      for (final platform in ['macos', 'windows'])
        platform: {
          'name': platform == 'macos'
              ? 'LinguaRay-macos.dmg'
              : 'LinguaRay-windows-x64.exe',
          'size': content.length,
          'sha256': crypto.sha256.convert(content).toString(),
          'platformSigned': false,
        },
    },
  };

  Future<List<int>> signed(Map<String, dynamic> data) async {
    final bytes = utf8.encode(jsonEncode(data));
    final signature = await Ed25519().sign(bytes, keyPair: key);
    return utf8.encode(
      jsonEncode({
        'payload': base64Encode(bytes),
        'signature': base64Encode(signature.bytes),
      }),
    );
  }

  setUp(() async {
    key = await Ed25519().newKeyPair();
    publicKey = base64Encode((await key.extractPublicKey()).bytes);
  });

  test('accepts signed feed and selects the exact platform artifact', () async {
    final envelope = await signed(payload());
    for (final platform in ['macos', 'windows']) {
      final feed = SignedUpdateFeed(platform: platform, publicKey: publicKey);
      final manifest = await feed.decode(envelope);
      expect(manifest.version, '0.6.2');
      expect(
        manifest.assetUrl,
        startsWith(
          'https://github.com/$updateRepositoryName/releases/download/v0.6.2/',
        ),
      );
      expect(manifest.byteLength, content.length);
      expect(manifest.platformSigned, isFalse);
      await feed.verify(manifest);
    }
  });

  test('rejects tampering, unknown key, and unsigned legacy feeds', () async {
    final feed = SignedUpdateFeed(platform: 'macos', publicKey: publicKey);
    final envelope = jsonDecode(
      utf8.decode(await signed(payload())),
    ) as Map<String, dynamic>;
    final changed = payload()..['version'] = '9.9.9';
    envelope['payload'] = base64Encode(utf8.encode(jsonEncode(changed)));
    await expectLater(
      feed.decode(utf8.encode(jsonEncode(envelope))),
      throwsA(isA<AppFailure>()),
    );
    final unknown = await Ed25519().newKeyPair();
    await expectLater(
      SignedUpdateFeed(
        platform: 'macos',
        publicKey: base64Encode((await unknown.extractPublicKey()).bytes),
      ).decode(await signed(payload())),
      throwsA(isA<AppFailure>()),
    );
    await expectLater(
      feed.decode(utf8.encode(jsonEncode(payload()))),
      throwsA(isA<AppFailure>()),
    );
  });

  test('rejects unsafe filenames, versions, sizes and repository changes even when signed', () async {
    final feed = SignedUpdateFeed(platform: 'macos', publicKey: publicKey);
    final cases = <void Function(Map<String, dynamic>)>[
      (data) => data['version'] = '../../main',
      (data) => data['repository'] = 'untrusted/fork',
      (data) => data['schemaVersion'] = 2,
      (data) => data['artifacts']['macos']['name'] = '../other.dmg',
      (data) => data['artifacts']['macos']['size'] = maximumUpdateBytes + 1,
      (data) => data['artifacts']['macos']['size'] = 0,
      (data) => data['artifacts']['macos']['sha256'] = 'invalid',
      (data) => data['artifacts']['macos'].remove('platformSigned'),
    ];
    for (final mutate in cases) {
      final data = payload();
      mutate(data);
      await expectLater(
        feed.decode(await signed(data)),
        throwsA(isA<AppFailure>()),
      );
    }
  });

  test(
    'downloads signed bytes and refuses a modified file at installation',
    () async {
      final server = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
      final directory = await Directory.systemTemp.createTemp(
        'linguaray-update-test-',
      );
      addTearDown(() async {
        await server.close(force: true);
        await directory.delete(recursive: true);
      });
      final base = 'http://${server.address.host}:${server.port}';
      final envelope = await signed(payload());
      final requestedPaths = <String>[];
      List<int> servedContent = content;
      server.listen((request) async {
        requestedPaths.add(request.uri.path);
        request.response.add(
          request.uri.path == '/update.json' ? envelope : servedContent,
        );
        await request.response.close();
      });
      var signatureChecks = 0;
      final repository = GitHubUpdateRepository(
        releasesUrl: '$base/update.json',
        feed: SignedUpdateFeed(
          platform: 'macos',
          publicKey: publicKey,
          releaseBaseUrl: base,
        ),
        downloadDirectory: directory,
        platformVerifier: (_, _) async {
          signatureChecks++;
        },
      );
      addTearDown(repository.close);
      final manifest = (await repository.checkLatest())!;
      final state = await DownloadVerifiedUpdate(repository)(
        currentVersion: '0.6.1',
        manifest: manifest,
      );
      expect(state.canInstall, isTrue);
      expect(requestedPaths, ['/update.json', '/v0.6.2/LinguaRay-macos.dmg']);
      expect(await File(state.downloadedPath!).readAsBytes(), content);
      expect(signatureChecks, 1);
      var installerOpened = false;
      final installer = DesktopUpdateInstaller(
        repository: repository,
        opener: (_) async {
          installerOpened = true;
        },
      );
      await installer.handOff(
        filePath: state.downloadedPath!,
        manifest: manifest,
      );
      expect(installerOpened, isTrue);
      installerOpened = false;
      await File(state.downloadedPath!)
          .writeAsString('tampered after download');
      await expectLater(
        installer.handOff(filePath: state.downloadedPath!, manifest: manifest),
        throwsA(
          isA<AppFailure>().having(
            (e) => e.code,
            'code',
            AppErrorCode.updateChecksumMismatch,
          ),
        ),
      );
      expect(installerOpened, isFalse);

      // The server returns same-size corrupt content: the signed SHA256 catches it.
      servedContent = List.filled(content.length, 0);
      final corrupt = await DownloadVerifiedUpdate(repository)(
        currentVersion: '0.6.1',
        manifest: manifest,
      );
      expect(corrupt.canInstall, isFalse);
      expect(corrupt.errorCode, AppErrorCode.updateChecksumMismatch.wireName);
      // Truncated chunked transfer must leave no partial download.
      servedContent = content.take(3).toList();
      final truncated = await DownloadVerifiedUpdate(repository)(
        currentVersion: '0.6.1',
        manifest: manifest,
      );
      expect(truncated.canInstall, isFalse);
      expect(
        await directory
            .list(recursive: true)
            .where((file) => file.path.endsWith('.part'))
            .isEmpty,
        isTrue,
      );
    },
  );

  test(
    'manifest substitution is rejected before any download request',
    () async {
      final feed = SignedUpdateFeed(platform: 'macos', publicKey: publicKey);
      final original = await feed.decode(await signed(payload()));
      final substituted = UpdateManifest(
        version: original.version,
        notes: original.notes,
        assetName: original.assetName,
        assetUrl: 'https://untrusted.invalid/installer.dmg',
        checksumSha256: original.checksumSha256,
        byteLength: original.byteLength,
        platformSigned: original.platformSigned,
        signedPayload: original.signedPayload,
        signatureBase64: original.signatureBase64,
      );
      final repository = GitHubUpdateRepository(feed: feed);
      addTearDown(repository.close);
      final result = await DownloadVerifiedUpdate(repository)(
        currentVersion: '0.6.1',
        manifest: substituted,
      );
      expect(result.errorCode, AppErrorCode.updateSignatureInvalid.wireName);
      expect(result.canInstall, isFalse);
    },
  );
}
