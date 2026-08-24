import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_desktop/src/services/system_proxy.dart';

void main() {
  test('selects scheme-specific system proxies', () {
    const proxy = SystemProxySnapshot(
      http: '127.0.0.1:8080',
      https: '127.0.0.1:8443',
    );

    expect(
      proxy.resolve(Uri.parse('http://example.com')),
      'PROXY 127.0.0.1:8080; DIRECT',
    );
    expect(
      proxy.resolve(Uri.parse('https://example.com')),
      'PROXY 127.0.0.1:8443; DIRECT',
    );
  });

  test('honours local and domain bypass rules', () {
    const proxy = SystemProxySnapshot(
      http: '127.0.0.1:8080',
      bypass: ['<local>', '*.example.test'],
    );

    expect(proxy.resolve(Uri.parse('http://printer')), 'DIRECT');
    expect(proxy.resolve(Uri.parse('http://api.example.test')), 'DIRECT');
    expect(
      proxy.resolve(Uri.parse('http://example.com')),
      'PROXY 127.0.0.1:8080; DIRECT',
    );
  });
}
