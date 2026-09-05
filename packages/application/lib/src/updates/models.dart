enum UpdateStatus {
  idle,
  checking,
  upToDate,
  available,
  downloading,
  readyToInstall,
  failed,
}

final class UpdateManifest {
  const UpdateManifest({
    required this.version,
    required this.notes,
    required this.assetName,
    required this.assetUrl,
    this.checksumSha256,
    this.publishedAt,
    this.byteLength,
    this.platformSigned = true,
    this.signedPayload,
    this.signatureBase64,
  });

  final String version;
  final String notes;
  final String assetName;
  final String assetUrl;
  final String? checksumSha256;
  final String? publishedAt;
  final int? byteLength;
  final bool platformSigned;
  final String? signedPayload;
  final String? signatureBase64;

  bool get hasChecksum =>
      checksumSha256 != null && checksumSha256!.trim().isNotEmpty;
}

final class UpdateState {
  const UpdateState({
    required this.status,
    required this.currentVersion,
    this.manifest,
    this.downloadedPath,
    this.progress,
    this.errorCode,
  });

  const UpdateState.idle(this.currentVersion)
    : status = UpdateStatus.idle,
      manifest = null,
      downloadedPath = null,
      progress = null,
      errorCode = null;

  final UpdateStatus status;
  final String currentVersion;
  final UpdateManifest? manifest;
  final String? downloadedPath;
  final double? progress;
  final String? errorCode;

  bool get canInstall =>
      status == UpdateStatus.readyToInstall &&
      downloadedPath != null &&
      manifest != null &&
      manifest!.hasChecksum;
}
