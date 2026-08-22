import 'dart:convert';
import 'dart:io';

import 'package:crypto/crypto.dart' as crypto;
import 'package:linguaray_application/linguaray_application.dart';
import 'package:package_info_plus/package_info_plus.dart';
import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';

const String kLinguaRayReleasesUrl =
    'https://api.github.com/repos/gong1414/linguaray/releases/latest';

final class GitHubUpdateRepository implements UpdateRepository {
  GitHubUpdateRepository({
    HttpClient? client,
    this.releasesUrl = kLinguaRayReleasesUrl,
  }) : _client = client ?? HttpClient();

  final HttpClient _client;
  final String releasesUrl;

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
    final wanted = Platform.isWindows ? 'LinguaRay-windows' : 'LinguaRay-macos';
    Map<String, dynamic>? asset;
    Map<String, dynamic>? checksums;
    for (final item in assets) {
      if (item is! Map) continue;
      final name = item['name'] as String? ?? '';
      if (name.contains(wanted) && name.endsWith('.zip')) {
        asset = Map<String, dynamic>.from(item);
      }
      if (name.toUpperCase().contains('SHA256')) {
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
      throw const AppFailure(AppErrorCode.updateCheckFailed);
    }
    final directory = await getTemporaryDirectory();
    final file = File(p.join(directory.path, manifest.assetName));
    final sink = file.openWrite();
    final total = response.contentLength;
    var received = 0;
    await for (final chunk in response) {
      sink.add(chunk);
      received += chunk.length;
      if (total > 0) onProgress?.call(received / total);
    }
    await sink.close();
    return file.path;
  }

  @override
  Future<void> verifyChecksum({
    required String filePath,
    required String sha256,
  }) async {
    final bytes = await File(filePath).readAsBytes();
    final digest = crypto.sha256.convert(bytes).toString();
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
        if (trimmed.toLowerCase().contains(assetName.toLowerCase())) {
          return trimmed.split(RegExp(r'\s+')).first;
        }
      }
      return body.trim().split(RegExp(r'\s+')).first;
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
      await Process.start('open', [filePath]);
      return;
    }
    if (Platform.isWindows) {
      await Process.start('explorer.exe', [filePath], runInShell: true);
    }
  }
}

typedef MethodChannelHandler = Future<void> Function(String path);
