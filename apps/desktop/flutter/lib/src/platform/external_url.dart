import 'package:nativeapi/nativeapi.dart';

Future<void> openExternalUrl(String url) async {
  final uri = Uri.tryParse(url);
  if (uri == null || (uri.scheme != 'https' && uri.scheme != 'http')) return;
  UrlOpener.instance.open(uri.toString());
}
