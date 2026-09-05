import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_desktop/src/ui/quick_translate/quick_translate_screen.dart';
import 'package:linguaray_desktop/src/ui/quick_translate/widgets/quick_translate_view.dart';

void main() {
  testWidgets('source can be collapsed and restored from the quick menu', (
    tester,
  ) async {
    await tester.pumpWidget(_app(_view(sourceText: 'hello')));
    await tester.tap(find.byType(PopupMenuButton<String>));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Hide source'));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('quick-source-input')), findsNothing);
    await tester.tap(find.text('hello'));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('quick-source-input')), findsOneWidget);
  });

  testWidgets('stop button cancels and Escape closes the quick surface', (
    tester,
  ) async {
    var stops = 0;
    var closes = 0;
    await tester.pumpWidget(
      _app(
        _view(
          sourceText: 'hello',
          submitting: true,
          onStop: () => stops++,
          onClose: () => closes++,
        ),
      ),
    );
    await tester.tap(find.text('Stop translation'));
    expect(stops, 1);
    await tester.sendKeyEvent(LogicalKeyboardKey.escape);
    expect(closes, 1);
  });

  const english = LanguageOption(code: 'en', name: 'English');
  const chinese = LanguageOption(code: 'zh-Hans', name: '简体中文');
  const japanese = LanguageOption(code: 'ja', name: '日本語');
  const service = TranslationServiceOption(
    id: 'test',
    name: 'Test',
    isStreaming: false,
  );

  testWidgets('partial output keeps its failure and retry visible', (
    tester,
  ) async {
    await tester.pumpWidget(
      _app(
        _view(
          sourceText: 'hello',
          result: const ServiceTranslationResult(
            service: service,
            status: TranslationResultStatus.failed,
            text: 'Partial text',
            errorCode: 'translation_incomplete',
          ),
        ),
      ),
    );
    expect(find.text('Partial text'), findsOneWidget);
    expect(find.text('失败'), findsOneWidget);
    expect(find.text('重试'), findsOneWidget);
  });

  test('common languages are promoted in the configured order', () {
    final ordered = orderLanguagesByPreference(
      const [english, chinese, japanese],
      const ['ja', 'en'],
    );
    expect(ordered.map((language) => language.code), ['ja', 'en', 'zh-Hans']);
  });

  testWidgets('modifier-submit keeps plain Enter for a new line', (
    tester,
  ) async {
    var submitted = 0;
    await tester.pumpWidget(
      _app(
        _view(
          sourceText: 'hello',
          submitWithModifier: true,
          onTranslate: () => submitted++,
        ),
      ),
    );

    await tester.tap(find.byKey(const ValueKey('quick-source-input')));
    await tester.sendKeyEvent(LogicalKeyboardKey.enter);
    await tester.pump();
    expect(submitted, 0);

    await tester.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
    await tester.sendKeyEvent(LogicalKeyboardKey.enter);
    await tester.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
    await tester.pump();
    expect(submitted, 1);
  });

  testWidgets('result actions and glossary state stay in the compact surface', (
    tester,
  ) async {
    tester.view
      ..devicePixelRatio = 1
      ..physicalSize = const Size(396, 800);
    addTearDown(tester.view.reset);
    var copied = 0;
    var spoken = 0;
    var lookedUp = 0;
    var saved = 0;

    await tester.pumpWidget(
      _app(
        _view(
          sourceText: 'hello',
          result: const ServiceTranslationResult(
            service: service,
            text: '你好',
            status: TranslationResultStatus.completed,
          ),
          copyResultOnDoubleClick: true,
          speechAvailable: true,
          dictionaryAvailable: true,
          glossaryMatches: const [
            GlossaryMatchHit(
              bookId: 'book',
              entryId: 'entry',
              term: 'hello',
              matchedText: 'hello',
              translation: '你好',
              forbidden: [],
              start: 0,
              end: 5,
            ),
          ],
          onCopy: (_) => copied++,
          onSpeakResult: () => spoken++,
          onLookup: (_) => lookedUp++,
          onSaveVocabulary: () => saved++,
        ),
      ),
    );

    await tester.tap(find.byTooltip('朗读译文'));
    await tester.tap(find.byTooltip('查询'));
    await tester.tap(find.byTooltip('加入生词本'));
    await tester.tap(find.byKey(const ValueKey('quick-result')));
    await tester.pump(const Duration(milliseconds: 50));
    await tester.tap(find.byKey(const ValueKey('quick-result')));
    await tester.pump(const Duration(seconds: 1));

    expect(spoken, 1);
    expect(lookedUp, 1);
    expect(saved, 1);
    expect(copied, 1);
    expect(find.text('hello → 你好'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });
}

Widget _app(Widget child) => MaterialApp(home: Scaffold(body: child));

QuickTranslateView _view({
  required String sourceText,
  ServiceTranslationResult? result,
  bool submitWithModifier = false,
  bool submitting = false,
  VoidCallback? onStop,
  VoidCallback? onClose,
  bool copyResultOnDoubleClick = false,
  bool speechAvailable = false,
  bool dictionaryAvailable = false,
  List<GlossaryMatchHit> glossaryMatches = const [],
  VoidCallback? onTranslate,
  ValueChanged<String>? onCopy,
  VoidCallback? onSpeakResult,
  ValueChanged<String>? onLookup,
  VoidCallback? onSaveVocabulary,
}) {
  const service = TranslationServiceOption(
    id: 'test',
    name: 'Test',
    isStreaming: false,
  );
  return QuickTranslateView(
    labels: _labels,
    submitting: submitting,
    onStop: onStop,
    onClose: onClose,
    languages: const [
      LanguageOption(code: 'en', name: 'English'),
      LanguageOption(code: 'zh-Hans', name: '简体中文'),
    ],
    services: const [service],
    sourceText: sourceText,
    sourceLanguage: 'en',
    targetLanguage: 'zh-Hans',
    selectedServiceId: 'test',
    selectedResult: result,
    results: [if (result != null) result],
    submitWithModifier: submitWithModifier,
    copyResultOnDoubleClick: copyResultOnDoubleClick,
    speechAvailable: speechAvailable,
    dictionaryAvailable: dictionaryAvailable,
    glossaryMatches: glossaryMatches,
    onSourceTextChanged: (_) {},
    onSourceLanguageChanged: (_) {},
    onTargetLanguageChanged: (_) {},
    onServiceSelected: (_) {},
    onSwapLanguages: () {},
    onTranslate: onTranslate ?? () {},
    onClear: () {},
    onCopy: onCopy ?? (_) {},
    onTogglePin: () {},
    onCapture: () {},
    onClipboard: () {},
    onOpenSettings: () {},
    onConfigureServices: () {},
    onRecheckPermissions: () {},
    onSpeakResult: onSpeakResult,
    onLookup: onLookup,
    onSaveVocabulary: onSaveVocabulary,
  );
}

const _labels = QuickTranslateLabels(
  title: '快捷翻译',
  inputHint: '输入文本',
  translate: '翻译',
  clear: '清空',
  copy: '复制',
  copied: '已复制',
  pin: '置顶',
  unpin: '取消置顶',
  capture: '截图',
  clipboard: '剪贴板',
  openSettings: '设置',
  autoDetect: '自动检测',
  autoMatch: '自动匹配',
  swapLanguages: '交换语言',
  translating: '翻译中',
  empty: '空',
  retry: '重试',
  configureServices: '配置服务',
  permissionDenied: '权限不足',
  permissionNext: '重新授权',
  captureCancelled: '已取消',
  serviceError: '服务错误',
  noServices: '没有服务',
  failureMessage: _failure,
  speakSource: '朗读原文',
  speakResult: '朗读译文',
  stopSpeaking: '停止朗读',
  lookup: '查询',
  saveVocabulary: '加入生词本',
  vocabularySaved: '已加入生词本',
  glossaryMatches: '命中术语',
  glossaryWarnings: '质量提示',
);

String _failure(String? _) => '失败';
