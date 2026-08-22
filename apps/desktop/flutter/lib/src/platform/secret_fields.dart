bool isSecretField(String key) {
  final lower = key.toLowerCase();
  return lower.contains('key') ||
      lower.contains('secret') ||
      lower.contains('token') ||
      lower.contains('password');
}
