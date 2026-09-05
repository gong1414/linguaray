import 'package:flutter/material.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../platform/ocr_controller.dart';
import '../../platform/platform_types.dart';
import '../history/history_view.dart';
import '../i18n_labels.dart';
import '../ocr/ocr_view.dart';
import '../quick_translate/widgets/quick_translate_view.dart';
import '../settings/data_transfer_settings_screen.dart';
import '../settings/settings_labels.dart';
import '../settings/settings_shell_view.dart';
import '../settings/views/about_settings_view.dart';
import '../settings/views/general_settings_view.dart';
import '../settings/views/permissions_settings_view.dart';
import '../settings/views/providers_settings_view.dart';
import '../settings/views/services_settings_view.dart';
import '../settings/views/shortcuts_settings_view.dart';
import '../updates/updates_view.dart';

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

enum CatalogOcrScenario { empty, recognizing, success, continuous, error }

Map<String, Widget> buildCatalogGoldenStates({
  TargetPlatform platform = TargetPlatform.macOS,
}) {
  return {
    'provider_models_live': const ProviderModelsCatalogPreview(),
    'provider_models_auth_error': const ProviderModelsCatalogPreview(
      failed: true,
    ),
    'quick_empty': const QuickTranslateCatalogPreview(
      scenario: CatalogQuickScenario.empty,
    ),
    'quick_success': const QuickTranslateCatalogPreview(
      scenario: CatalogQuickScenario.success,
    ),
    'quick_service_error': const QuickTranslateCatalogPreview(
      scenario: CatalogQuickScenario.serviceError,
    ),
    'quick_long_result': const QuickTranslateCatalogPreview(
      scenario: CatalogQuickScenario.longResult,
    ),
    'ocr_empty': const OcrCatalogPreview(scenario: CatalogOcrScenario.empty),
    'ocr_success': const OcrCatalogPreview(
      scenario: CatalogOcrScenario.success,
    ),
    'ocr_continuous': const OcrCatalogPreview(
      scenario: CatalogOcrScenario.continuous,
    ),
    'settings_translation': const SettingsCatalogPreview(
      section: SettingsSection.translation,
    ),
    'settings_translation_services': const SettingsCatalogPreview(
      section: SettingsSection.translationServices,
    ),
    'settings_ocr': const SettingsCatalogPreview(section: SettingsSection.ocr),
    'settings_permissions': const SettingsCatalogPreview(
      section: SettingsSection.permissions,
    ),
    'settings_general': const SettingsCatalogPreview(
      section: SettingsSection.general,
    ),
  };
}

class OcrCatalogPreview extends StatelessWidget {
  const OcrCatalogPreview({required this.scenario, super.key});

  final CatalogOcrScenario scenario;

  @override
  Widget build(BuildContext context) {
    const result = OcrRecognitionResult(
      text: 'LinguaRay keeps OCR independent from translation.',
      source: OcrInputSource.screenRegion,
    );
    final continuous = scenario == CatalogOcrScenario.continuous;
    final populated = scenario == CatalogOcrScenario.success || continuous;
    return SizedBox(
      width: 600,
      height: 520,
      child: OcrView(
        labels: const OcrViewLabels(
          title: 'OCR',
          emptyTitle: '还没有识别结果',
          emptyDescription: '可以框选屏幕区域、选择图片文件，或识别剪贴板中的图片。',
          capture: '截图',
          file: '图片文件',
          clipboard: '剪贴板图片',
          continuous: '连续识别',
          copy: '复制',
          clear: '清空',
          close: '关闭',
          resultCount: _ocrResultCount,
          errorMessage: _catalogError,
        ),
        state: OcrViewState(
          results: populated ? [result, if (continuous) result] : const [],
          text: continuous
              ? '${result.text}\n\n第二次识别结果会继续追加。'
              : populated
              ? result.text
              : '',
          busy: scenario == CatalogOcrScenario.recognizing,
          continuous: continuous,
          errorCode: scenario == CatalogOcrScenario.error ? 'ocr_empty' : null,
        ),
        onTextChanged: (_) {},
        onCapture: _noop,
        onFile: _noop,
        onClipboard: _noop,
        onContinuousChanged: (_) {},
        onCopy: _noop,
        onClear: _noop,
        onClose: _noop,
      ),
    );
  }
}

String _ocrResultCount(int count) => '$count 次结果';

String _catalogError(String? code) => code ?? '识别失败';

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
        onOpenSettings: () {},
        onConfigureServices: () {},
        onRecheckPermissions: () {},
      ),
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
        SettingsSection.translationServices ||
        SettingsSection.ocrServices => ServicesSettingsView(
          labels: _servicesZh,
          pageTitle: '服务',
          loading: false,
          serviceKind: section == SettingsSection.ocrServices
              ? 'ocr'
              : 'translation',
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
                  ServiceRecord(
                    id: 'ecdict+dictionary',
                    name: 'ECDICT',
                    providerId: 'ecdict',
                    providerName: 'ECDICT',
                    kind: 'dictionary',
                    enabled: true,
                    isDefault: true,
                  ),
                ],
          onEnabledChanged: (_, _) {},
          onMakeDefault: (_) {},
          onConfigureProviders: () {},
        ),
        SettingsSection.translation ||
        SettingsSection.ocr => ShortcutsSettingsView(
          labels: _shortcutsZh,
          title: section == SettingsSection.ocr ? 'OCR 设置' : '翻译设置',
          recordingActionId: null,
          shortcuts: section == SettingsSection.ocr
              ? const [
                  ShortcutRecord(
                    actionId: 'captureOcr',
                    labelKey: '截图 OCR',
                    accelerator: '⌥⇧W',
                    status: ShortcutStatus.registered,
                  ),
                  ShortcutRecord(
                    actionId: 'silentCaptureOcr',
                    labelKey: '静默截图 OCR',
                    accelerator: '',
                    status: ShortcutStatus.unregistered,
                  ),
                  ShortcutRecord(
                    actionId: 'fileOcr',
                    labelKey: '选图 OCR',
                    accelerator: '',
                    status: ShortcutStatus.unregistered,
                  ),
                  ShortcutRecord(
                    actionId: 'clipboardOcr',
                    labelKey: '剪贴板 OCR',
                    accelerator: '',
                    status: ShortcutStatus.unregistered,
                  ),
                  ShortcutRecord(
                    actionId: 'showOcrWindow',
                    labelKey: '显示 OCR 窗口',
                    accelerator: '',
                    status: ShortcutStatus.unregistered,
                  ),
                ]
              : [
                  const ShortcutRecord(
                    actionId: 'translateSelection',
                    labelKey: '划词翻译',
                    accelerator: '⌥Q',
                    status: ShortcutStatus.registered,
                  ),
                  const ShortcutRecord(
                    actionId: 'captureAndTranslate',
                    labelKey: '截图翻译',
                    accelerator: '⌥W',
                    status: ShortcutStatus.registered,
                  ),
                  const ShortcutRecord(
                    actionId: 'openInputWindow',
                    labelKey: '输入翻译',
                    accelerator: '⌥Z',
                    status: ShortcutStatus.registered,
                  ),
                  const ShortcutRecord(
                    actionId: 'translateInput',
                    labelKey: '剪贴板翻译',
                    accelerator: '⌥E',
                    status: ShortcutStatus.registered,
                  ),
                  ShortcutRecord(
                    actionId: 'toggleQuickWindow',
                    labelKey: '显示翻译窗口',
                    accelerator: '⌥1',
                    status: shortcutConflict
                        ? ShortcutStatus.localDuplicate
                        : ShortcutStatus.registered,
                    conflictReason: shortcutConflict ? '显示翻译窗口' : null,
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
        SettingsSection.dataTransfer => const DataTransferSettingsScreen(),
        SettingsSection.favorites ||
        SettingsSection.history ||
        SettingsSection.glossary ||
        SettingsSection.vocabulary ||
        SettingsSection.integration ||
        SettingsSection.updates => Padding(
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

class ProviderModelsCatalogPreview extends StatelessWidget {
  const ProviderModelsCatalogPreview({super.key, this.failed = false});
  final bool failed;

  @override
  Widget build(BuildContext context) => ProviderEditorView(
    labels: providersSettingsLabels(),
    types: const [
      ProviderTypeOption(
        id: 'openrouter',
        label: 'OpenRouter',
        isLlm: true,
        engineTypeId: 'openai_compatible',
        fields: [
          ProviderFieldSpec(
            key: 'apiKey',
            label: 'API Key',
            secret: true,
            requiredField: true,
          ),
          ProviderFieldSpec(
            key: 'defaultModel',
            label: 'Model',
            secret: false,
            requiredField: true,
          ),
        ],
      ),
    ],
    draftId: 'openrouter',
    typeId: 'openrouter',
    idReadOnly: true,
    fields: const {'defaultModel': 'anthropic/claude-sonnet-4-6'},
    storedSecretKeys: const {'apiKey'},
    testing: false,
    testResult: null,
    saving: false,
    operationError: null,
    discovery: ProviderModelDiscovery(
      liveModels: failed
          ? const []
          : const [
              'anthropic/claude-sonnet-4-6',
              'deepseek/deepseek-chat',
              'vendor/custom-model',
            ],
      referenceModels: const ['offline/reference-model'],
      queriedAt: DateTime(2026, 9, 5, 12, 30),
      errorCode: failed ? 'auth_error' : null,
    ),
    onFetchModels: () {},
    onIdChanged: (_) {},
    onTypeChanged: (_) {},
    onFieldChanged: (_, _) {},
    onTest: () {},
    onSave: () {},
    onCancel: () {},
  );
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

String _failureMessageZh(String? code) => switch (code) {
  'language_pair_not_installed' => '请先安装这个语言组合，然后重试。',
  'unsupported_language_pair' => '当前服务不支持这对语言。',
  'network_error' => '无法连接到翻译服务。请检查网络后重试。',
  _ => '翻译失败，请检查服务配置后重试',
};

const _settingsShellZh = SettingsShellLabels(
  translationGroup: '翻译',
  translationSettings: '翻译设置',
  translationServices: '服务',
  favorites: '收藏夹',
  history: '历史记录',
  ocrGroup: 'OCR',
  ocrSettings: 'OCR 设置',
  ocrServices: '服务',
  generalGroup: '通用',
  general: '常规',
  permissions: '权限',
  about: '关于',
);

const _settingsShellEn = SettingsShellLabels(
  translationGroup: 'Translation',
  translationSettings: 'Translation Settings',
  translationServices: 'Services',
  favorites: 'Favorites',
  history: 'History',
  ocrGroup: 'OCR',
  ocrSettings: 'OCR Settings',
  ocrServices: 'Services',
  generalGroup: 'General',
  general: 'General',
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
  dictionary: '词典',
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

const _allServices = [
  _deepL,
  TranslationServiceOption(id: 'openai', name: 'OpenAI', isStreaming: true),
];

const _longSource =
    'LinguaRay is a privacy-first desktop translator. It stays out of the way, '
    'keeps provider keys in the system keychain, and treats every empty, loading, '
    'streaming and failure state as something a person should be able to inspect.';

const _longResult = 'LinguaRay 是一款隐私优先的桌面翻译工具。它不打扰当前工作，把提供商密钥留在系统钥匙串中。';

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
        edited: '我改过的',
        search: '搜索',
        emptyTitle: '还没有翻译历史',
        emptyDescription: '成功的翻译会保存在这里。',
        noResults: '没有匹配的历史',
        loading: '正在读取…',
        retry: '重试',
        delete: '删除',
        clear: '清空',
        exitSelection: '退出多选',
        clearConfirm: '确定清空？',
        select: '选择',
        open: '打开',
        favorite: '收藏',
        unfavorite: '取消收藏',
        edit: '编辑',
        selectedCount: _historySelectedCount,
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
      onEdit: (_) {},
      onDelete: (_) {},
      onClear: () {},
      onRetry: () {},
      onToggleSelected: (_) {},
      onExitSelection: _noop,
    );
  }
}

String _historySelectedCount(int count) => '已选 $count 条';

void _noop() {}

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
