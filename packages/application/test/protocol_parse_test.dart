import 'package:linguaray_application/linguaray_application.dart';
import 'package:test/test.dart';

void main() {
  const parse = ParseProtocolLink();

  test('parses translate and settings links', () {
    final translate = parse('linguaray://translate?text=hello%20world');
    expect(translate.action, ProtocolAction.translate);
    expect(translate.text, 'hello world');

    final settings = parse('linguaray://settings');
    expect(settings.action, ProtocolAction.settings);
  });

  test('ignores unknown actions without throwing', () {
    final ignored = parse('linguaray://replace-selection?text=no');
    expect(ignored.action, ProtocolAction.ignored);
    expect(ignored.errorCode, isNull);
  });

  test('rejects oversized payloads', () {
    final huge = 'a' * (kProtocolMaxTextBytes + 1);
    final result = parse('linguaray://translate?text=$huge');
    expect(result.action, ProtocolAction.ignored);
    expect(result.errorCode, AppErrorCode.protocolTooLarge.wireName);
  });

  test('applies the protocol limit to UTF-8 bytes', () {
    final multibyte = '界' * (kProtocolMaxTextBytes ~/ 3 + 1);
    final result = parse('linguaray://translate?text=$multibyte');
    expect(result.action, ProtocolAction.ignored);
    expect(result.errorCode, AppErrorCode.protocolTooLarge.wireName);
  });

  test('rejects non-linguaray schemes', () {
    final result = parse('https://example.com/translate?text=hi');
    expect(result.action, ProtocolAction.ignored);
    expect(result.errorCode, AppErrorCode.protocolInvalid.wireName);
  });
}
