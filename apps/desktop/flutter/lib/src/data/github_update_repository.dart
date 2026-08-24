import 'dart:convert';
import 'dart:io';

import 'package:crypto/crypto.dart' as crypto;
import 'package:linguaray_application/linguaray_application.dart';
import 'package:package_info_plus/package_info_plus.dart';
import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';

const String kLinguaRayReleasesUrl =
    'https://api.github.com/repos/gong1414/linguaray/releases/latest';
const int _maximumUpdateBytes = 1024 * 1024 * 1024;

final class GitHubUpdateRepository implements UpdateRepository {
  GitHubUpdateRepository({
    HttpClient? client,
    this.releasesUrl = kLinguaRayReleasesUrl,
  }) : _client = client ?? HttpClient();

  final HttpClient _client;
  final String releasesUrl;

  void close() => _client.close(force: true);

  @override
  Future<UpdateManifest?> checkLatest() async {
    final request = await _client.getUrl(Uri.parse(releasesUrl));
    request.headers.set('Accept', 'application/vnd.github+json');
    request.headers.set('User-Agent', 'LinguaRay');
    final response = await request.close();
    if (response.statusCode >= 400) {
      throw const AppFailure(AppErrorCode.updateCheckFailed);
    }
    final body = await response.transform(utf8.decoder).join();
    final json = jsonDecode(body);
    if (json is! Map<String, dynamic>) return null;
    final tag = json['tag_name'] as String? ?? '';
    final notes = json['body'] as String? ?? '';
    final assets = json['assets'];
    if (assets is! List) return null;
    final wanted = Platform.isWindows
        ? 'LinguaRay-windows-x64.exe'
        : 'LinguaRay-macos.dmg';
    Map<String, dynamic>? asset;
    Map<String, dynamic>? checksums;
    for (final item in assets) {
      if (item is! Map) continue;
      final name = item['name'] as String? ?? '';
      if (name == wanted) {
        asset = Map<String, dynamic>.from(item);
      }
      if (name == 'SHA256SUMS.txt') {
        checksums = Map<String, dynamic>.from(item);
      }
    }
    if (asset == null) return null;
    String? checksum;
    if (checksums != null) {
      checksum = await _downloadChecksum(
        checksums['browser_download_url'] as String? ?? '',
        asset['name'] as String? ?? '',
      );
    }
    return UpdateManifest(
      version: tag,
      notes: notes,
      assetName: asset['name'] as String? ?? '',
      assetUrl: asset['browser_download_url'] as String? ?? '',
      checksumSha256: checksum,
      publishedAt: json['published_at'] as String?,
    );
  }

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
    final request = await _client.getUrl(Uri.parse(manifest.assetUrl));
    final response = await request.close();
    if (response.statusCode >= 400) {
      throw const AppFailure(AppErrorCode.updateDownloadFailed);
    }
    final directory = await getTemporaryDirectory();
    final safeName = p.basename(manifest.assetName);
    if (safeName != manifest.assetName || safeName.isEmpty) {
      throw const AppFailure(AppErrorCode.updateDownloadFailed);
    }
    final updateDirectory = Directory(
      p.join(directory.path, 'LinguaRay', 'updates', manifest.version),
    );
    await updateDirectory.create(recursive: true);
    final file = File(p.join(updateDirectory.path, safeName));
    if (await file.exists()) await file.delete();
    final sink = file.openWrite();
    final total = response.contentLength;
    if (total > _maximumUpdateBytes) {
      await sink.close();
      throw const AppFailure(AppErrorCode.updateDownloadFailed);
    }
    var received = 0;
    try {
      await for (final chunk in response) {
        received += chunk.length;
        if (received > _maximumUpdateBytes) {
          throw const AppFailure(AppErrorCode.updateDownloadFailed);
        }
        sink.add(chunk);
        if (total > 0) onProgress?.call(received / total);
      }
    } catch (_) {
      await sink.close();
      if (await file.exists()) await file.delete();
      rethrow;
    }
    await sink.close();
    return file.path;
  }

  @override
  Future<void> verifyPlatformSignature({required String filePath}) async {
    if (Platform.isMacOS) {
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
      final currentTeam = await _macTeamIdentifier(Platform.resolvedExecutable);
      if (updateTeam == null || currentTeam == null || updateTeam != currentTeam) {
        throw const AppFailure(AppErrorCode.updateSignatureInvalid);
      }
      return;
    }
    if (Platform.isWindows) {
      final result = await Process.run('powershell.exe', [
        '-NoProfile',
        '-NonInteractive',
        '-Command',
        r'''& { param([string]$UpdatePath, [string]$CurrentPath)
$update = Get-AuthenticodeSignature -LiteralPath $UpdatePath
$current = Get-AuthenticodeSignature -LiteralPath $CurrentPath
if ($update.Status -eq 'Valid' -and $current.Status -eq 'Valid' -and
    $update.SignerCertificate.Subject -eq $current.SignerCertificate.Subject) {
  'Valid'
} else {
  'Invalid'
} }''',
        filePath,
        Platform.resolvedExecutable,
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
    final match = RegExp(r'^TeamIdentifier=(.+)$', multiLine: true)
        .firstMatch(output);
    final identifier = match?.group(1)?.trim();
    if (identifier == null || identifier.isEmpty || identifier == 'not set') {
      throw const AppFailure(AppErrorCode.updateSignatureInvalid);
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

  Future<String?> _downloadChecksum(String url, String assetName) async {
    if (url.isEmpty) return null;
    try {
      final request = await _client.getUrl(Uri.parse(url));
      final response = await request.close();
      if (response.statusCode >= 400) return null;
      final body = await response.transform(utf8.decoder).join();
      for (final line in body.split(RegExp(r'\r?\n'))) {
        final trimmed = line.trim();
        if (trimmed.isEmpty) continue;
        final match = RegExp(r'^([a-fA-F0-9]{64})\s+\*?(.+)$')
            .firstMatch(trimmed);
        if (match?.group(2) == assetName) return match!.group(1);
      }
      return null;
    } catch (_) {
      return null;
    }
  }
}

final class DesktopUpdateInstaller implements UpdateInstaller {
  DesktopUpdateInstaller({this.opener});

  final MethodChannelHandler? opener;

  @override
  Future<void> handOff({
    required String filePath,
    required UpdateManifest manifest,
  }) async {
    if (!manifest.hasChecksum) {
      throw const AppFailure(AppErrorCode.updateChecksumMissing);
    }
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
