import 'dart:io';

import 'package:crypto/crypto.dart' as crypto;
import 'package:linguaray_application/linguaray_application.dart';
import 'package:package_info_plus/package_info_plus.dart';
import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';

import 'signed_update_feed.dart';

const String kLinguaRayReleasesUrl = updateManifestUrl;

final class GitHubUpdateRepository implements UpdateRepository {
  GitHubUpdateRepository({
    HttpClient? client,
    this.releasesUrl = kLinguaRayReleasesUrl,
    SignedUpdateFeed? feed,
    this.downloadDirectory,
    this.platformVerifier,
  }) : _client = client ?? HttpClient(),
       _feed = feed ?? SignedUpdateFeed(platform: Platform.operatingSystem) {
    _client.connectionTimeout = const Duration(seconds: 15);
  }

  final HttpClient _client;
  final SignedUpdateFeed _feed;
  final String releasesUrl;
  final Directory? downloadDirectory;
  final Future<void> Function(String path, UpdateManifest manifest)?
  platformVerifier;

  void close() => _client.close(force: true);

  @override
  Future<UpdateManifest?> checkLatest() async {
    final request = await _client.getUrl(Uri.parse(releasesUrl));
    request.headers.set('User-Agent', 'LinguaRay');
    request.headers.set('Cache-Control', 'no-cache');
    final response = await request.close().timeout(const Duration(seconds: 15));
    if (response.statusCode != HttpStatus.ok) {
      await response.timeout(const Duration(seconds: 15)).drain<void>();
      throw const AppFailure(AppErrorCode.updateCheckFailed);
    }
    final bytes = <int>[];
    await for (final chunk in response.timeout(const Duration(seconds: 15))) {
      bytes.addAll(chunk);
      if (bytes.length > 256 * 1024) {
        throw const AppFailure(AppErrorCode.updateCheckFailed);
      }
    }
    return _feed.decode(bytes);
  }

  @override
  Future<void> verifyManifest(UpdateManifest manifest) =>
      _feed.verify(manifest);

  @override
  Future<String> currentVersion() async {
    final info = await PackageInfo.fromPlatform();
    return info.version;
  }

  @override
  Future<String> download({
    required UpdateManifest manifest,
    void Function(double progress)? onProgress,
  }) async {
    await verifyManifest(manifest);
    final request = await _client.getUrl(Uri.parse(manifest.assetUrl));
    final response = await request.close().timeout(const Duration(seconds: 15));
    if (response.statusCode != HttpStatus.ok) {
      await response.timeout(const Duration(seconds: 15)).drain<void>();
      throw const AppFailure(AppErrorCode.updateDownloadFailed);
    }
    final expected = manifest.byteLength!;
    if (response.contentLength >= 0 && response.contentLength != expected) {
      await response.timeout(const Duration(seconds: 15)).drain<void>();
      throw const AppFailure(AppErrorCode.updateDownloadFailed);
    }
    final directory = downloadDirectory ?? await getTemporaryDirectory();
    final root = Directory(p.join(directory.path, 'LinguaRay', 'updates'));
    await root.create(recursive: true);
    final attempt = await root.createTemp('v${manifest.version}-');
    final partial = File(p.join(attempt.path, '${manifest.assetName}.part'));
    final sink = partial.openWrite();
    var received = 0;
    try {
      await for (final chunk in response.timeout(const Duration(seconds: 30))) {
        received += chunk.length;
        if (received > expected || received > maximumUpdateBytes) {
          throw const AppFailure(AppErrorCode.updateDownloadFailed);
        }
        sink.add(chunk);
        onProgress?.call(received / expected);
      }
      await sink.close();
      if (received != expected) {
        throw const AppFailure(AppErrorCode.updateDownloadFailed);
      }
      return (await partial.rename(p.join(attempt.path, manifest.assetName)))
          .path;
    } catch (_) {
      await sink.close();
      if (await attempt.exists()) await attempt.delete(recursive: true);
      rethrow;
    }
  }

  @override
  Future<void> verifyPlatformSignature({
    required String filePath,
    required UpdateManifest manifest,
  }) async {
    await verifyManifest(manifest);
    if (platformVerifier != null) return platformVerifier!(filePath, manifest);
    if (Platform.isMacOS) {
      final currentTeam = await _macTeamIdentifier(Platform.resolvedExecutable);
      // A cryptographically authenticated update can bootstrap an unsigned
      // installation. Once platform signing is in use it may not be downgraded.
      if (!manifest.platformSigned && currentTeam == null) return;
      final updateVerification = await Process.run('/usr/bin/codesign', [
        '--verify',
        '--deep',
        '--strict',
        filePath,
      ]);
      if (updateVerification.exitCode != 0) {
        throw const AppFailure(AppErrorCode.updateSignatureInvalid);
      }
      final updateTeam = await _macTeamIdentifier(filePath);
      if (updateTeam == null ||
          (currentTeam != null && updateTeam != currentTeam)) {
        throw const AppFailure(AppErrorCode.updateSignatureInvalid);
      }
      return;
    }
    if (Platform.isWindows) {
      final result = await Process.run('powershell.exe', [
        '-NoProfile',
        '-NonInteractive',
        '-Command',
        r'''& { param([string]$UpdatePath, [string]$CurrentPath, [string]$RequireSignature)
$update = Get-AuthenticodeSignature -LiteralPath $UpdatePath
$current = Get-AuthenticodeSignature -LiteralPath $CurrentPath
if ($RequireSignature -ne 'true' -and $current.Status -ne 'Valid') {
  'Valid'
} elseif ($update.Status -eq 'Valid' -and
    ($current.Status -ne 'Valid' -or
     $update.SignerCertificate.Subject -eq $current.SignerCertificate.Subject)) {
  'Valid'
} else {
  'Invalid'
} }''',
        filePath,
        Platform.resolvedExecutable,
        manifest.platformSigned.toString(),
      ]);
      if (result.exitCode == 0 && result.stdout.toString().trim() != 'Valid') {
        throw const AppFailure(AppErrorCode.updateSignatureInvalid);
      }
      if (result.exitCode != 0) {
        throw const AppFailure(AppErrorCode.updateSignatureInvalid);
      }
      return;
    }
    throw const AppFailure(AppErrorCode.updateSignatureInvalid);
  }

  Future<String?> _macTeamIdentifier(String path) async {
    final result = await Process.run('/usr/bin/codesign', [
      '--display',
      '--verbose=4',
      path,
    ]);
    if (result.exitCode != 0) return null;
    final output = '${result.stdout}\n${result.stderr}';
    final match = RegExp(
      r'^TeamIdentifier=(.+)$',
      multiLine: true,
    ).firstMatch(output);
    final identifier = match?.group(1)?.trim();
    if (identifier == null || identifier.isEmpty || identifier == 'not set') {
      return null;
    }
    return identifier;
  }

  @override
  Future<void> verifyChecksum({
    required String filePath,
    required String sha256,
  }) async {
    final digest = (await crypto.sha256.bind(File(filePath).openRead()).first)
        .toString();
    if (digest.toLowerCase() != sha256.trim().toLowerCase()) {
      throw const AppFailure(AppErrorCode.updateChecksumMismatch);
    }
  }
}

final class DesktopUpdateInstaller implements UpdateInstaller {
  DesktopUpdateInstaller({required this.repository, this.opener});

  final UpdateRepository repository;

  final MethodChannelHandler? opener;

  @override
  Future<void> handOff({
    required String filePath,
    required UpdateManifest manifest,
  }) async {
    if (!manifest.hasChecksum) {
      throw const AppFailure(AppErrorCode.updateChecksumMissing);
    }
    // Revalidate on handoff: the downloaded file may have changed while the
    // update screen was waiting for the user to confirm installation.
    await repository.verifyManifest(manifest);
    await repository.verifyChecksum(
      filePath: filePath,
      sha256: manifest.checksumSha256!,
    );
    await repository.verifyPlatformSignature(
      filePath: filePath,
      manifest: manifest,
    );
    if (opener != null) {
      await opener!(filePath);
      return;
    }
    if (Platform.isMacOS) {
      if (p.extension(filePath).toLowerCase() != '.dmg') {
        throw const AppFailure(AppErrorCode.updateInstallFailed);
      }
      await Process.start('/usr/bin/open', [filePath]);
      return;
    }
    if (Platform.isWindows) {
      if (p.extension(filePath).toLowerCase() != '.exe') {
        throw const AppFailure(AppErrorCode.updateInstallFailed);
      }
      await Process.start(filePath, const [], mode: ProcessStartMode.detached);
      return;
    }
    throw const AppFailure(AppErrorCode.updateInstallFailed);
  }
}

typedef MethodChannelHandler = Future<void> Function(String path);
