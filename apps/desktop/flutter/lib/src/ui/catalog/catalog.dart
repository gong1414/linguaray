import 'package:flutter/material.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../chrome/workbench_shell_view.dart';
import '../first_run/first_run_view.dart';
import '../glossary/glossary_view.dart';
import '../history/history_view.dart';
import '../quick_translate/widgets/quick_translate_view.dart';
import '../settings/settings_labels.dart';
import '../settings/settings_shell_view.dart';
import '../settings/views/about_settings_view.dart';
import '../settings/views/general_settings_view.dart';
import '../settings/views/permissions_settings_view.dart';
import '../settings/views/providers_settings_view.dart';
import '../settings/views/services_settings_view.dart';
import '../settings/views/shortcuts_settings_view.dart';
import '../translation/widgets/translation_workspace_view.dart';
import '../updates/updates_view.dart';
import '../vocabulary/vocabulary_view.dart';

enum CatalogTranslationScenario {
  empty('Empty · 中文'),
  emptyEn('Empty · English'),
  typing('Typing · 中文'),
  loading('Loading · 中文'),
  translating('Translating · 中文'),
  streaming('Streaming · 中文'),
  success('Success · 中文'),
  multipleProviders('Multiple providers · 中文'),
  partialFailure('Partial failure · 中文'),
  allFailure('All failure · 中文'),
  error('Error · 中文'),
  languagePackMissing('macOS language pack missing · 中文'),
  noServices('No services · 中文'),
  longText('Long text · 中文');

  const CatalogTranslationScenario(this.label);
  final String label;
}

enum CatalogQuickScenario {
  empty,
  prefilled,
  ocr,
  translating,
  success,
  multiple,
  permissionDenied,
  captureCancelled,
  serviceError,
  pinned,
  longResult,
}

enum CatalogFirstRunScenario {
  checking,
  granted,
  denied,
  notRequired,
  shortcutConflict,
  noProvider,
  ready,
}

Map<String, Widget> buildCatalogGoldenStates({
  TargetPlatform platform = TargetPlatform.macOS,
}) {
  final chrome = platform == TargetPlatform.windows
      ? WindowChromeKind.windows
      : WindowChromeKind.macos;
  return {
    'translation_empty': const TranslationCatalogPreview(
      scenario: CatalogTranslationScenario.empty,
    ),
    'translation_success': const TranslationCatalogPreview(
      scenario: CatalogTranslationScenario.success,
    ),
    'translation_multiple': const TranslationCatalogPreview(
      scenario: CatalogTranslationScenario.multipleProviders,
    ),
    'translation_error': const TranslationCatalogPreview(
      scenario: CatalogTranslationScenario.error,
    ),
    'translation_language_pack_missing': const TranslationCatalogPreview(
      scenario: CatalogTranslationScenario.languagePackMissing,
    ),
    'translation_no_services': const TranslationCatalogPreview(
      scenario: CatalogTranslationScenario.noServices,
    ),
    'shell_translate': CatalogShellPreview(
      chrome: chrome,
      destination: WorkbenchDestinationId.translate,
      child: const TranslationCatalogPreview(
        scenario: CatalogTranslationScenario.success,
        showHeader: false,
      ),
    ),
    'first_run_ready': FirstRunCatalogPreview(
      scenario: platform == TargetPlatform.windows
          ? CatalogFirstRunScenario.notRequired
          : CatalogFirstRunScenario.ready,
    ),
    'quick_empty': const QuickTranslateCatalogPreview(
      scenario: CatalogQuickScenario.empty,
    ),
    'settings_general': const SettingsCatalogPreview(
      section: SettingsSection.general,
    ),
  };
}

class CatalogShellPreview extends StatelessWidget {
  const CatalogShellPreview({
    required this.chrome,
    required this.destination,
    required this.child,
    super.key,
    this.english = false,
  });

  final WindowChromeKind chrome;
  final WorkbenchDestinationId destination;
  final Widget child;
  final bool english;

  @override
  Widget build(BuildContext context) {
    return WorkbenchShellView(
      labels: english ? _shellEn : _shellZh,
      chrome: chrome,
      destination: destination,
      onDestinationSelected: (_) {},
      onMinimize: () {},
      onToggleMaximize: () {},
      onClose: () {},
      child: child,
    );
  }
}

class TranslationCatalogPreview extends StatefulWidget {
  const TranslationCatalogPreview({
    required this.scenario,
    super.key,
    this.showHeader = true,
  });

  final CatalogTranslationScenario scenario;
  final bool showHeader;

  @override
  State<TranslationCatalogPreview> createState() =>
      _TranslationCatalogPreviewState();
}

class _TranslationCatalogPreviewState extends State<TranslationCatalogPreview> {
  late String _sourceText = _initialSource;
  String _sourceLanguage = autoLanguageCode;
  String _targetLanguage = 'zh-Hans';
  late String _selectedService =
      widget.scenario == CatalogTranslationScenario.languagePackMissing
      ? 'system'
      : 'deepl';
  bool _copied = false;

  String get _initialSource => switch (widget.scenario) {
    CatalogTranslationScenario.empty ||
    CatalogTranslationScenario.emptyEn ||
    CatalogTranslationScenario.loading ||
    CatalogTranslationScenario.noServices => '',
    CatalogTranslationScenario.typing => 'A quiet instrument for translation.',
    CatalogTranslationScenario.longText => _longSource,
    _ => 'A stable interface should make every state inspectable.',
  };

  List<TranslationServiceOption> get _services => switch (widget.scenario) {
    CatalogTranslationScenario.noServices => const [],
    CatalogTranslationScenario.languagePackMissing => const [_system],
    CatalogTranslationScenario.multipleProviders ||
    CatalogTranslationScenario.partialFailure ||
    CatalogTranslationScenario.allFailure ||
    CatalogTranslationScenario.streaming => _allServices,
    _ => const [_deepL],
  };

  List<ServiceTranslationResult> get _results => switch (widget.scenario) {
    CatalogTranslationScenario.success => [
      const ServiceTranslationResult(
        service: _deepL,
        text: '稳定的界面应该让每一种状态都可检查。',
        status: TranslationResultStatus.completed,
      ),
    ],
    CatalogTranslationScenario.multipleProviders => [
      const ServiceTranslationResult(
        service: _deepL,
        text: '稳定的界面应该让每一种状态都可检查。',
        status: TranslationResultStatus.completed,
      ),
      const ServiceTranslationResult(
        service: TranslationServiceOption(
          id: 'openai',
          name: 'OpenAI',
          isStreaming: true,
        ),
        text: '一个稳定的界面，应当让每一种状态都能被检查。',
        status: TranslationResultStatus.completed,
      ),
      const ServiceTranslationResult(
        service: TranslationServiceOption(
          id: 'google',
          name: 'Google',
          isStreaming: false,
        ),
        text: '稳定的界面应让每种状态都可检查。',
        status: TranslationResultStatus.completed,
      ),
    ],
    CatalogTranslationScenario.partialFailure => [
      const ServiceTranslationResult(
        service: _deepL,
        text: '稳定的界面应该让每一种状态都可检查。',
        status: TranslationResultStatus.completed,
      ),
      const ServiceTranslationResult(
        service: TranslationServiceOption(
          id: 'openai',
          name: 'OpenAI',
          isStreaming: true,
        ),
        status: TranslationResultStatus.failed,
        errorCode: 'network_error',
      ),
    ],
    CatalogTranslationScenario.allFailure => [
      const ServiceTranslationResult(
        service: _deepL,
        status: TranslationResultStatus.failed,
        errorCode: 'network_error',
      ),
      const ServiceTranslationResult(
        service: TranslationServiceOption(
          id: 'openai',
          name: 'OpenAI',
          isStreaming: true,
        ),
        status: TranslationResultStatus.failed,
        errorCode: 'service_unavailable',
      ),
    ],
    CatalogTranslationScenario.error => const [
      ServiceTranslationResult(
        service: _deepL,
        status: TranslationResultStatus.failed,
        errorCode: 'service_unavailable',
      ),
    ],
    CatalogTranslationScenario.languagePackMissing => const [
      ServiceTranslationResult(
        service: _system,
        status: TranslationResultStatus.failed,
        errorCode: 'language_pair_not_installed',
      ),
    ],
    CatalogTranslationScenario.translating => const [
      ServiceTranslationResult(
        service: _deepL,
        status: TranslationResultStatus.translating,
      ),
    ],
    CatalogTranslationScenario.streaming => const [
      ServiceTranslationResult(
        service: _deepL,
        text: '稳定的界面',
        status: TranslationResultStatus.translating,
      ),
      ServiceTranslationResult(
        service: TranslationServiceOption(
          id: 'openai',
          name: 'OpenAI',
          isStreaming: true,
        ),
        status: TranslationResultStatus.waiting,
      ),
    ],
    CatalogTranslationScenario.longText => [
      const ServiceTranslationResult(
        service: _deepL,
        text: _longResult,
        status: TranslationResultStatus.completed,
      ),
    ],
    _ => const [],
  };

  ServiceTranslationResult? get _selectedResult {
    if (_results.isEmpty) return null;
    for (final result in _results) {
      if (result.service.id == _selectedService) return result;
    }
    return _results.first;
  }

  @override
  Widget build(BuildContext context) {
    final english = widget.scenario == CatalogTranslationScenario.emptyEn;
    return TranslationWorkspaceView(
      labels: english ? _translationEn : _translationZh,
      showHeader: widget.showHeader,
      languages: _languages,
      services: _services,
      sourceText: _sourceText,
      sourceLanguage: _sourceLanguage,
      targetLanguage: _targetLanguage,
      selectedServiceId: _selectedService,
      selectedResult: _selectedResult,
      results: _results,
      detectedLanguage: _sourceText.isEmpty ? null : 'en',
      loadingCatalog: widget.scenario == CatalogTranslationScenario.loading,
      submitting:
          widget.scenario == CatalogTranslationScenario.translating ||
          widget.scenario == CatalogTranslationScenario.streaming,
      catalogFailed: widget.scenario == CatalogTranslationScenario.error,
      submissionFailed: widget.scenario == CatalogTranslationScenario.error,
      copied: _copied,
      onSourceTextChanged: (value) => setState(() => _sourceText = value),
      onSourceLanguageChanged: (value) =>
          setState(() => _sourceLanguage = value),
      onTargetLanguageChanged: (value) =>
          setState(() => _targetLanguage = value),
      onServiceSelected: (value) => setState(() => _selectedService = value),
      onSwapLanguages: () => setState(() {
        final source = _sourceLanguage;
        _sourceLanguage = _targetLanguage;
        _targetLanguage = source == autoLanguageCode ? 'en' : source;
      }),
      onTranslate: () {},
      onClear: () => setState(() => _sourceText = ''),
      onCopy: (_) => setState(() => _copied = true),
      onConfigureServices: () {},
    );
  }
}

class QuickTranslateCatalogPreview extends StatelessWidget {
  const QuickTranslateCatalogPreview({required this.scenario, super.key});

  final CatalogQuickScenario scenario;

  @override
  Widget build(BuildContext context) {
    final source = switch (scenario) {
      CatalogQuickScenario.empty => '',
      CatalogQuickScenario.longResult => _longSource,
      CatalogQuickScenario.prefilled ||
      CatalogQuickScenario.ocr ||
      CatalogQuickScenario.translating ||
      CatalogQuickScenario.success ||
      CatalogQuickScenario.multiple ||
      CatalogQuickScenario.permissionDenied ||
      CatalogQuickScenario.captureCancelled ||
      CatalogQuickScenario.serviceError ||
      CatalogQuickScenario.pinned => 'Selected text from another app.',
    };
    final result = switch (scenario) {
      CatalogQuickScenario.success ||
      CatalogQuickScenario.pinned ||
      CatalogQuickScenario.multiple => const ServiceTranslationResult(
        service: _deepL,
        text: '来自另一个应用的选中文本。',
        status: TranslationResultStatus.completed,
      ),
      CatalogQuickScenario.longResult => const ServiceTranslationResult(
        service: _deepL,
        text: _longResult,
        status: TranslationResultStatus.completed,
      ),
      CatalogQuickScenario.serviceError => const ServiceTranslationResult(
        service: _deepL,
        status: TranslationResultStatus.failed,
        errorCode: 'network_error',
      ),
      CatalogQuickScenario.translating => const ServiceTranslationResult(
        service: _deepL,
        status: TranslationResultStatus.translating,
      ),
      _ => null,
    };

    return SizedBox(
      width: 396,
      child: QuickTranslateView(
        labels: _quickZh,
        languages: _languages,
        services: scenario == CatalogQuickScenario.multiple
            ? _allServices
            : const [_deepL],
        sourceText: source,
        sourceLanguage: autoLanguageCode,
        targetLanguage: automaticTargetCode,
        selectedServiceId: 'deepl',
        selectedResult: result,
        results: scenario == CatalogQuickScenario.multiple
            ? [
                result!,
                const ServiceTranslationResult(
                  service: TranslationServiceOption(
                    id: 'openai',
                    name: 'OpenAI',
                    isStreaming: true,
                  ),
                  text: '从其他应用选中的文字。',
                  status: TranslationResultStatus.completed,
                ),
              ]
            : [if (result != null) result],
        submitting: scenario == CatalogQuickScenario.translating,
        pinned: scenario == CatalogQuickScenario.pinned,
        notice: switch (scenario) {
          CatalogQuickScenario.permissionDenied =>
            QuickTranslateNotice.permissionDenied,
          CatalogQuickScenario.captureCancelled =>
            QuickTranslateNotice.captureCancelled,
          _ => QuickTranslateNotice.none,
        },
        onSourceTextChanged: (_) {},
        onSourceLanguageChanged: (_) {},
        onTargetLanguageChanged: (_) {},
        onServiceSelected: (_) {},
        onSwapLanguages: () {},
        onTranslate: () {},
        onClear: () {},
        onCopy: (_) {},
        onTogglePin: () {},
        onCapture: () {},
        onClipboard: () {},
        onOpenWorkbench: () {},
        onOpenSettings: () {},
        onConfigureServices: () {},
        onRecheckPermissions: () {},
      ),
    );
  }
}

class FirstRunCatalogPreview extends StatelessWidget {
  const FirstRunCatalogPreview({required this.scenario, super.key});

  final CatalogFirstRunScenario scenario;

  @override
  Widget build(BuildContext context) {
    final permissions = switch (scenario) {
      CatalogFirstRunScenario.checking => const AccessSnapshot(
        accessibility: AccessState.checking,
        screenRecording: AccessState.checking,
      ),
      CatalogFirstRunScenario.granted ||
      CatalogFirstRunScenario.ready ||
      CatalogFirstRunScenario.shortcutConflict ||
      CatalogFirstRunScenario.noProvider => const AccessSnapshot(
        accessibility: AccessState.granted,
        screenRecording: AccessState.granted,
      ),
      CatalogFirstRunScenario.denied => const AccessSnapshot(
        accessibility: AccessState.denied,
        screenRecording: AccessState.denied,
      ),
      CatalogFirstRunScenario.notRequired => const AccessSnapshot.notRequired(),
    };

    return FirstRunView(
      labels: _firstRunZh,
      permissions: permissions,
      shortcutsReady: scenario != CatalogFirstRunScenario.shortcutConflict,
      shortcutConflict: scenario == CatalogFirstRunScenario.shortcutConflict,
      hasServices: scenario != CatalogFirstRunScenario.noProvider,
      checkingPermissions: scenario == CatalogFirstRunScenario.checking,
      onGrantAccessibility: () {},
      onGrantScreenRecording: () {},
      onRecheck: () {},
      onConfigureServices: () {},
      onStart: () {},
      onSkip: () {},
    );
  }
}

class SettingsCatalogPreview extends StatelessWidget {
  const SettingsCatalogPreview({
    required this.section,
    super.key,
    this.providersEmpty = false,
    this.servicesEmpty = false,
    this.shortcutConflict = false,
    this.english = false,
  });

  final SettingsSection section;
  final bool providersEmpty;
  final bool servicesEmpty;
  final bool shortcutConflict;
  final bool english;

  @override
  Widget build(BuildContext context) {
    return SettingsShellView(
      labels: english ? _settingsShellEn : _settingsShellZh,
      section: section,
      onSectionSelected: (_) {},
      child: switch (section) {
        SettingsSection.general => GeneralSettingsView(
          labels: english ? _generalEn : _generalZh,
          preferences: const GeneralPreferences(
            launchAtLogin: false,
            showInMenuBar: true,
            language: 'zh-Hans',
            themeMode: ThemePreference.system,
          ),
          languages: const [
            LanguageChoice(code: 'en', name: 'English'),
            LanguageChoice(code: 'zh-Hans', name: '简体中文'),
          ],
          onLaunchAtLoginChanged: (_) {},
          onShowInMenuBarChanged: (_) {},
          onLanguageChanged: (_) {},
          onThemeModeChanged: (_) {},
        ),
        SettingsSection.services => ServicesSettingsView(
          labels: _servicesZh,
          loading: false,
          services: servicesEmpty
              ? const []
              : const [
                  ServiceRecord(
                    id: 'deepl+translation',
                    name: 'DeepL',
                    providerId: 'deepl',
                    providerName: 'DeepL',
                    kind: 'translation',
                    enabled: true,
                    isDefault: true,
                  ),
                  ServiceRecord(
                    id: 'system+ocr',
                    name: '系统 OCR',
                    providerId: 'system',
                    providerName: '系统',
                    kind: 'ocr',
                    enabled: true,
                    isDefault: true,
                  ),
                ],
          onEnabledChanged: (_, _) {},
          onMakeDefault: (_) {},
          onConfigureProviders: () {},
        ),
        SettingsSection.providers => ProvidersSettingsView(
          labels: _providersZh,
          loading: false,
          providers: providersEmpty
              ? const []
              : const [
                  ProviderRecord(
                    id: 'deepl',
                    typeId: 'deepl',
                    displayName: 'DeepL',
                    publicFields: {},
                    storedSecretKeys: {'authKey'},
                  ),
                ],
          onAdd: () {},
          onEdit: (_) {},
          onDelete: (_) {},
        ),
        SettingsSection.shortcuts => ShortcutsSettingsView(
          labels: _shortcutsZh,
          recordingActionId: null,
          shortcuts: [
            ShortcutRecord(
              actionId: 'toggleQuickWindow',
              labelKey: '显示/隐藏快捷翻译',
              accelerator: '⌥1',
              status: shortcutConflict
                  ? ShortcutStatus.localDuplicate
                  : ShortcutStatus.registered,
              conflictReason: shortcutConflict ? '显示/隐藏快捷翻译' : null,
            ),
            const ShortcutRecord(
              actionId: 'translateSelection',
              labelKey: '划词翻译',
              accelerator: '⌥Q',
              status: ShortcutStatus.registered,
            ),
            const ShortcutRecord(
              actionId: 'captureAndTranslate',
              labelKey: '截图 OCR',
              accelerator: '⌥W',
              status: ShortcutStatus.registered,
            ),
            const ShortcutRecord(
              actionId: 'translateInput',
              labelKey: '剪贴板翻译',
              accelerator: '⌥E',
              status: ShortcutStatus.registered,
            ),
          ],
          onStartRecording: (_) {},
          onCancelRecording: () {},
          onClear: (_) {},
          onReset: () {},
        ),
        SettingsSection.permissions => PermissionsSettingsView(
          labels: _permissionsZh,
          snapshot: const AccessSnapshot(
            accessibility: AccessState.granted,
            screenRecording: AccessState.denied,
          ),
          onGrantAccessibility: () {},
          onGrantScreenRecording: () {},
          onRecheck: () {},
        ),
        SettingsSection.advanced || SettingsSection.updates => Padding(
          padding: const EdgeInsets.all(24),
          child: Text(section.name),
        ),
        SettingsSection.about => AboutSettingsView(
          labels: _aboutZh,
          info: const AboutInfo(
            appName: 'LinguaRay',
            version: '0.5.0',
            buildNumber: '18',
            platformLabel: 'macOS',
            license: 'MIT',
          ),
          copied: false,
          onCopyVersion: () {},
          onOpenWebsite: () {},
          onOpenChangelog: () {},
          onOpenIssues: () {},
          onOpenLicense: () {},
        ),
      },
    );
  }
}

class ProviderEditorCatalogPreview extends StatelessWidget {
  const ProviderEditorCatalogPreview({
    super.key,
    this.secretStored = false,
    this.testing = false,
    this.failed = false,
  });

  final bool secretStored;
  final bool testing;
  final bool failed;

  @override
  Widget build(BuildContext context) {
    return ProviderEditorView(
      labels: _providersZh,
      types: const [
        ProviderTypeOption(
          id: 'deepl',
          label: 'DeepL',
          isLlm: false,
          fields: [
            ProviderFieldSpec(
              key: 'authKey',
              label: 'Auth Key',
              secret: true,
              requiredField: true,
            ),
          ],
        ),
      ],
      draftId: 'deepl',
      typeId: 'deepl',
      fields: const {},
      storedSecretKeys: secretStored ? const {'authKey'} : const {},
      testing: testing,
      testResult: failed
          ? const ProviderTestResult(
              status: ProviderTestStatus.failed,
              errorCode: 'network_error',
              message: '无法连接到翻译服务。',
            )
          : testing
          ? const ProviderTestResult(status: ProviderTestStatus.testing)
          : secretStored
          ? const ProviderTestResult(status: ProviderTestStatus.passed)
          : null,
      saving: false,
      operationError: null,
      onIdChanged: (_) {},
      onTypeChanged: (_) {},
      onFieldChanged: (_, _) {},
      onTest: () {},
      onSave: () {},
      onCancel: () {},
    );
  }
}

const _shellZh = WorkbenchShellLabels(
  appName: 'LinguaRay',
  translate: '翻译',
  history: '历史',
  glossary: '术语库',
  vocabulary: '生词本',
  settings: '设置',
  minimize: '最小化',
  maximize: '最大化',
  close: '关闭',
);

const _shellEn = WorkbenchShellLabels(
  appName: 'LinguaRay',
  translate: 'Translate',
  history: 'History',
  glossary: 'Glossary',
  vocabulary: 'Vocabulary',
  settings: 'Settings',
  minimize: 'Minimize',
  maximize: 'Maximize',
  close: 'Close',
);

const _translationZh = TranslationWorkspaceLabels(
  title: '输入翻译',
  subtitle: '在工作台里处理长文本，并比较多个翻译服务',
  source: '原文',
  target: '译文',
  autoDetect: '自动检测',
  autoMatch: '自动匹配',
  inputHint: '输入或粘贴需要翻译的文本',
  translate: '翻译',
  clear: '清空',
  swapLanguages: '交换语言',
  loadingServices: '正在读取翻译服务…',
  noServices: '请先配置一个翻译服务',
  translating: '正在翻译…',
  failed: '翻译失败，请检查服务配置后重试',
  empty: '译文将在这里显示',
  services: '翻译服务',
  copy: '复制译文',
  copied: '已复制',
  configureServices: '配置服务',
  retry: '重试',
  characterCount: _characterCountZh,
  failureMessage: _failureMessageZh,
  partialFailure: _partialFailureZh,
  streaming: '正在生成译文…',
);

const _translationEn = TranslationWorkspaceLabels(
  title: 'Input translation',
  subtitle: 'Translate longer text and compare services in the workbench',
  source: 'Source',
  target: 'Translation',
  autoDetect: 'Auto detect',
  autoMatch: 'Auto match',
  inputHint: 'Type or paste text to translate',
  translate: 'Translate',
  clear: 'Clear',
  swapLanguages: 'Swap languages',
  loadingServices: 'Loading translation services…',
  noServices: 'Configure a translation service first',
  translating: 'Translating…',
  failed: 'Translation failed. Check the service, then try again.',
  empty: 'The translation will appear here',
  services: 'Translation services',
  copy: 'Copy translation',
  copied: 'Copied',
  configureServices: 'Configure services',
  retry: 'Try again',
  characterCount: _characterCountEn,
  failureMessage: _failureMessageEn,
  partialFailure: _partialFailureEn,
  streaming: 'Writing translation…',
);

String _characterCountZh(int count) => '$count 个字符';
String _characterCountEn(int count) => '$count characters';
String _partialFailureZh(int count) => '$count 个服务失败，可切换查看原因';
String _partialFailureEn(int count) =>
    '$count services failed. Switch to inspect the reason.';

String _failureMessageZh(String? code) => switch (code) {
  'language_pair_not_installed' => '请先在 macOS 语言与地区中安装这个语言组合，然后重试。',
  'unsupported_language_pair' => '当前服务不支持这对语言。',
  'network_error' => '无法连接到翻译服务。请检查网络后重试。',
  _ => '翻译失败，请检查服务配置后重试',
};

String _failureMessageEn(String? code) => switch (code) {
  'language_pair_not_installed' =>
    'Install this language pair in macOS Language & Region, then try again.',
  'unsupported_language_pair' =>
    'The selected service does not support this language pair.',
  'network_error' => 'The translation service could not be reached. Check your connection and try again.',
  _ => 'Translation failed. Check the service configuration and try again.',
};

const _quickZh = QuickTranslateLabels(
  title: '快捷翻译',
  inputHint: '输入、粘贴，或由划词和截图填入',
  translate: '翻译',
  clear: '清空',
  copy: '复制',
  copied: '已复制',
  pin: '置顶',
  unpin: '取消置顶',
  capture: '截图 OCR',
  clipboard: '读取剪贴板',
  openWorkbench: '打开工作台',
  openSettings: '设置',
  autoDetect: '自动检测',
  autoMatch: '自动匹配',
  swapLanguages: '交换语言',
  translating: '正在翻译…',
  empty: '译文将显示在这里',
  retry: '重试',
  configureServices: '配置服务',
  permissionDenied: '没有所需的系统权限',
  permissionNext: '打开系统设置授予辅助功能或屏幕录制权限，然后返回重新检查。',
  captureCancelled: '已取消截图。原文未改动。',
  serviceError: '翻译服务未能返回结果',
  noServices: '尚未配置翻译服务',
  failureMessage: _failureMessageZh,
);

const _firstRunZh = FirstRunLabels(
  title: '开始使用 LinguaRay',
  subtitle: '完成这几步后，就可以从任何应用唤起翻译。',
  permissionsTitle: '系统权限',
  permissionsBody: '划词和截图 OCR 需要辅助功能与屏幕录制权限。',
  accessibility: '辅助功能',
  screenRecording: '屏幕录制',
  shortcutsTitle: '全局快捷键',
  shortcutsBody: '四个首版动作已准备好。若有冲突，可稍后在设置中修改。',
  servicesTitle: '翻译服务',
  servicesBody: '至少启用一个翻译服务。',
  granted: '已授权',
  denied: '未授权',
  notRequired: '当前系统无需授权',
  unknown: '状态未知',
  checking: '正在检查…',
  conflict: '有快捷键冲突。可先跳过，之后在设置中修复。',
  noProvider: '还没有可用的翻译服务。',
  ready: '已有可用服务。',
  grant: '授权',
  recheck: '重新检查',
  configureServices: '配置服务',
  start: '开始使用',
  skip: '稍后再说',
);

const _settingsShellZh = SettingsShellLabels(
  title: '设置',
  general: '常规',
  services: '服务',
  providers: '提供商',
  shortcuts: '快捷键',
  permissions: '权限',
  about: '关于',
);

const _settingsShellEn = SettingsShellLabels(
  title: 'Settings',
  general: 'General',
  services: 'Services',
  providers: 'Providers',
  shortcuts: 'Shortcuts',
  permissions: 'Permissions',
  about: 'About',
);

const _generalZh = GeneralSettingsLabels(
  startup: '启动与菜单栏',
  launchAtLogin: '登录时启动',
  showInMenuBar: '在菜单栏显示',
  appearance: '外观',
  language: '界面语言',
  theme: '主题',
  light: '浅色',
  dark: '深色',
  system: '跟随系统',
);

const _generalEn = GeneralSettingsLabels(
  startup: 'Startup & menu bar',
  launchAtLogin: 'Launch at login',
  showInMenuBar: 'Show in menu bar',
  appearance: 'Appearance',
  language: 'Interface language',
  theme: 'Theme',
  light: 'Light',
  dark: 'Dark',
  system: 'System',
);

const _servicesZh = ServicesSettingsLabels(
  title: '服务',
  empty: '还没有翻译或 OCR 服务。请先添加提供商。',
  loading: '正在读取服务…',
  translation: '翻译',
  ocr: 'OCR',
  enabled: '启用',
  makeDefault: '设为默认',
  isDefault: '默认',
  configureProviders: '配置提供商',
  commonLanguages: '常用语言',
  defaultService: '默认服务',
);

const _providersZh = ProvidersSettingsLabels(
  title: '提供商',
  empty: '还没有提供商。添加后即可使用翻译服务。',
  loading: '正在读取提供商…',
  add: '添加提供商',
  edit: '编辑',
  delete: '删除',
  deleteConfirmTitle: '删除这个提供商？',
  deleteConfirmBody: '服务会一并移除。系统钥匙串中的密钥也会删除。',
  secretStored: '密钥已保存在系统钥匙串',
  secretPlaceholder: '留空则保留已保存的密钥',
  save: '保存',
  cancel: '取消',
  test: '测试连接',
  testing: '正在测试…',
  testPassed: '连接成功',
  testFailed: '连接失败。请检查密钥或网络后重试。',
  idLabel: '提供商 ID',
  typeLabel: '类型',
  validationMissing: '请填写必填项。',
  saveFailed: '无法保存提供商，请检查配置后重试。',
);

const _shortcutsZh = ShortcutsSettingsLabels(
  title: '快捷键',
  record: '录制',
  recording: '请按下按键…',
  clear: '清除',
  reset: '恢复默认',
  resetConfirmTitle: '恢复默认快捷键？',
  resetConfirmBody: '四个首版动作会回到默认组合。',
  registered: '已注册',
  unregistered: '未注册',
  invalid: '组合无法使用',
  conflict: _shortcutConflictZh,
  cancel: '取消',
  confirm: '恢复',
);

String _shortcutConflictZh(String label) => '与「$label」冲突';

const _permissionsZh = PermissionsSettingsLabels(
  title: '权限',
  accessibility: '辅助功能',
  accessibilityHint: '划词翻译需要读取其他应用中的选中文本。',
  screenRecording: '屏幕录制',
  screenRecordingHint: '截图 OCR 需要读取屏幕内容。',
  granted: '已授权',
  denied: '未授权',
  notRequired: '无需授权',
  unknown: '未知',
  grant: '授权',
  recheck: '重新检查',
  windowsNote: 'Windows 上无需额外授权即可使用划词和截图。',
);

const _aboutZh = AboutSettingsLabels(
  title: '关于',
  copyVersion: '复制版本信息',
  copied: '已复制',
  license: '开源协议',
  website: '网站',
  changelog: '更新说明',
  issues: '报告问题',
  copyright: '© LinguaRay contributors. MIT License.',
);

const _languages = [
  LanguageOption(code: 'en', name: '英语'),
  LanguageOption(code: 'zh-Hans', name: '简体中文'),
  LanguageOption(code: 'zh-Hant', name: '繁体中文'),
  LanguageOption(code: 'ja', name: '日语'),
  LanguageOption(code: 'ko', name: '韩语'),
];

const _deepL = TranslationServiceOption(
  id: 'deepl',
  name: 'DeepL',
  isStreaming: false,
);

const _system = TranslationServiceOption(
  id: 'system',
  name: '系统翻译',
  isStreaming: false,
);

const _allServices = [
  _deepL,
  TranslationServiceOption(id: 'openai', name: 'OpenAI', isStreaming: true),
  TranslationServiceOption(id: 'google', name: 'Google', isStreaming: false),
];

const _longSource =
    'LinguaRay is a privacy-first desktop translator. It stays out of the way, '
    'keeps provider keys in the system keychain, and treats every empty, loading, '
    'streaming and failure state as something a person should be able to inspect.';

const _longResult =
    'LinguaRay 是一款隐私优先的桌面翻译工具。它不打扰当前工作，把提供商密钥留在系统钥匙串中，'
    '并让空白、加载、流式输出和失败都成为可以检查的状态。';

class HistoryCatalogPreview extends StatelessWidget {
  const HistoryCatalogPreview({required this.empty, super.key});

  final bool empty;

  @override
  Widget build(BuildContext context) {
    return HistoryView(
      labels: const HistoryViewLabels(
        title: '历史',
        all: '全部',
        favorites: '收藏',
        search: '搜索',
        emptyTitle: '还没有翻译历史',
        emptyDescription: '成功的翻译会保存在这里。',
        noResults: '没有匹配的历史',
        loading: '正在读取…',
        retry: '重试',
        delete: '删除',
        clear: '清空',
        clearConfirm: '确定清空？',
        select: '选择',
        open: '打开',
        favorite: '收藏',
        unfavorite: '取消收藏',
      ),
      snapshot: HistorySnapshot(
        entries: empty
            ? const []
            : const [
                HistoryRecord(
                  id: '1',
                  source: 'hello',
                  translation: '你好',
                  sourceLanguage: 'en',
                  targetLanguage: 'zh-Hans',
                  serviceId: 'deepl',
                  serviceName: 'DeepL',
                  favorite: true,
                  edited: false,
                  createdAt: 0,
                  updatedAt: 0,
                ),
              ],
        counts: HistoryCounts(
          all: empty ? 0 : 1,
          favorites: empty ? 0 : 1,
          edited: 0,
        ),
        filter: HistoryFilter.all,
        query: '',
      ),
      selectedIds: const {},
      onQueryChanged: (_) {},
      onFilterChanged: (_) {},
      onOpen: (_) {},
      onFavorite: (_, _) {},
      onDelete: (_) {},
      onClear: () {},
      onRetry: () {},
      onToggleSelected: (_) {},
    );
  }
}

class GlossaryCatalogPreview extends StatelessWidget {
  const GlossaryCatalogPreview({required this.empty, super.key});

  final bool empty;

  @override
  Widget build(BuildContext context) {
    return GlossaryView(
      labels: const GlossaryViewLabels(
        title: '术语库',
        newBook: '新建',
        rename: '重命名',
        enable: '启用',
        disable: '停用',
        delete: '删除',
        addEntry: '新术语',
        term: '原文',
        translation: '译文',
        forbidden: '禁用译法',
        search: '搜索',
        emptyTitle: '这本术语库是空的',
        emptyDescription: '添加术语后，翻译会优先使用它们。',
        noBooksTitle: '还没有术语库',
        noBooksDescription: '先创建一本，再添加术语。',
        loading: '正在读取…',
        retry: '重试',
        save: '保存',
        cancel: '取消',
        caseSensitive: 'Aa',
        wholeWord: '[]',
        corrupt: '有一本术语库无法读取。',
      ),
      books: empty
          ? const []
          : const [
              GlossaryBookRecord(
                id: 'ml',
                name: '机器学习',
                enabled: true,
                entryCount: 1,
              ),
            ],
      entries: empty
          ? const []
          : const [
              GlossaryEntryRecord(
                id: '1',
                term: 'teacher forcing',
                translation: '强制教学',
                forbidden: ['强迫教学'],
                caseSensitive: false,
                wholeWord: true,
              ),
            ],
      selectedBookId: empty ? null : 'ml',
      loading: false,
      query: '',
      onSelectBook: (_) {},
      onQueryChanged: (_) {},
      onCreateBook: () {},
      onRenameBook: () {},
      onToggleBook: () {},
      onDeleteBook: () {},
      onAddEntry: () {},
      onEditEntry: (_) {},
      onDeleteEntry: (_) {},
      onRetry: () {},
    );
  }
}

class VocabularyCatalogPreview extends StatelessWidget {
  const VocabularyCatalogPreview({required this.empty, super.key});

  final bool empty;

  @override
  Widget build(BuildContext context) {
    return VocabularyView(
      labels: const VocabularyViewLabels(
        title: '生词本',
        search: '搜索生词',
        all: '全部',
        favorites: '收藏',
        emptyTitle: '还没有生词',
        emptyDescription: '可以从词典或译文加入。',
        noResults: '没有匹配的生词',
        note: '笔记',
        delete: '删除',
        favorite: '收藏',
        unfavorite: '取消收藏',
        retry: '重试',
      ),
      snapshot: VocabularySnapshot(
        entries: empty
            ? const []
            : const [
                VocabularyRecord(
                  id: '1',
                  word: 'ray',
                  translation: '光线',
                  sourceLanguage: 'en',
                  targetLanguage: 'zh-Hans',
                  source: 'dictionary',
                  favorite: false,
                  createdAt: 0,
                  updatedAt: 0,
                ),
              ],
        filter: VocabularyFilter.all,
        query: '',
      ),
      onQueryChanged: (_) {},
      onFilterChanged: (_) {},
      onFavorite: (_, _) {},
      onDelete: (_) {},
      onEditNote: (_) {},
      onRetry: () {},
    );
  }
}

class UpdatesCatalogPreview extends StatelessWidget {
  const UpdatesCatalogPreview({super.key});

  @override
  Widget build(BuildContext context) {
    return UpdatesView(
      labels: UpdatesViewLabels(
        title: '更新',
        current: '当前版本',
        check: '检查更新',
        checking: '正在检查…',
        upToDate: '已是最新版本',
        available: (version) => '发现新版本 $version',
        download: '下载',
        downloading: '正在下载…',
        ready: '已校验，可以安装',
        install: '安装',
        unsigned: '没有校验和，不会安装',
        notes: '发行说明',
        retry: '重试',
      ),
      state: const UpdateState(
        status: UpdateStatus.available,
        currentVersion: '0.5.0',
        manifest: UpdateManifest(
          version: '0.6.0',
          notes: 'New interface.',
          assetName: 'LinguaRay-macos.zip',
          assetUrl: 'https://example.invalid/app.zip',
          checksumSha256: 'abc',
        ),
      ),
      onCheck: () {},
      onDownload: () {},
      onInstall: () {},
    );
  }
}
