///
/// Generated file. Do not edit.
///
// coverage:ignore-file
// ignore_for_file: type=lint, unused_import

part of 'strings.g.dart';

// Path: <root>
typedef TranslationsEn = Translations; // ignore: unused_element

class Translations with BaseTranslations<AppLocale, Translations> {
  /// Returns the current translations of the given [context].
  ///
  /// Usage:
  /// final t = Translations.of(context);
  static Translations of(BuildContext context) =>
      InheritedLocaleData.of<AppLocale, Translations>(context).translations;

  /// You can call this constructor and build your own translation instance of this locale.
  /// Constructing via the enum [AppLocale.build] is preferred.
  Translations({
    Map<String, Node>? overrides,
    PluralResolver? cardinalResolver,
    PluralResolver? ordinalResolver,
    TranslationMetadata<AppLocale, Translations>? meta,
  }) : assert(
         overrides == null,
         'Set "translation_overrides: true" in order to enable this feature.',
       ),
       $meta =
           meta ??
           TranslationMetadata(
             locale: AppLocale.en,
             overrides: overrides ?? {},
             cardinalResolver: cardinalResolver,
             ordinalResolver: ordinalResolver,
           ) {
    $meta.setFlatMapFunction(_flatMapFunction);
  }

  /// Metadata for the translations of <en>.
  @override
  final TranslationMetadata<AppLocale, Translations> $meta;

  /// Access flat map
  dynamic operator [](String key) => $meta.getTranslation(key);

  late final Translations _root = this; // ignore: unused_field

  Translations $copyWith({
    TranslationMetadata<AppLocale, Translations>? meta,
  }) => Translations(meta: meta ?? this.$meta);

  // Translations
  late final TranslationsCommonEn common = TranslationsCommonEn.internal(_root);
  late final TranslationsAppEn app = TranslationsAppEn.internal(_root);
  late final TranslationsMiniTranslatorEn mini_translator =
      TranslationsMiniTranslatorEn.internal(_root);
  late final TranslationsWorkbenchEn workbench =
      TranslationsWorkbenchEn.internal(_root);
  late final TranslationsSettingsEn settings = TranslationsSettingsEn.internal(
    _root,
  );
  late final TranslationsUiEn ui = TranslationsUiEn.internal(_root);
}

// Path: common
class TranslationsCommonEn {
  TranslationsCommonEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations
  late final TranslationsCommonUiEn ui = TranslationsCommonUiEn.internal(_root);
  late final TranslationsCommonLanguageEn language =
      TranslationsCommonLanguageEn.internal(_root);
  late final TranslationsCommonThemeModeEn theme_mode =
      TranslationsCommonThemeModeEn.internal(_root);
  late final TranslationsCommonThemeStyleEn theme_style =
      TranslationsCommonThemeStyleEn.internal(_root);
  late final TranslationsCommonProviderEn provider =
      TranslationsCommonProviderEn.internal(_root);
}

// Path: app
class TranslationsAppEn {
  TranslationsAppEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations
  late final TranslationsAppTrayEn tray = TranslationsAppTrayEn.internal(_root);
}

// Path: mini_translator
class TranslationsMiniTranslatorEn {
  TranslationsMiniTranslatorEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations
  late final TranslationsMiniTranslatorLimitedBannerEn limited_banner =
      TranslationsMiniTranslatorLimitedBannerEn.internal(_root);
  late final TranslationsMiniTranslatorInputEn input =
      TranslationsMiniTranslatorInputEn.internal(_root);
  late final TranslationsMiniTranslatorToolbarEn toolbar =
      TranslationsMiniTranslatorToolbarEn.internal(_root);
  late final TranslationsMiniTranslatorButtonEn button =
      TranslationsMiniTranslatorButtonEn.internal(_root);
  late final TranslationsMiniTranslatorLanguageEn language =
      TranslationsMiniTranslatorLanguageEn.internal(_root);
  late final TranslationsMiniTranslatorMessageEn message =
      TranslationsMiniTranslatorMessageEn.internal(_root);
  late final TranslationsMiniTranslatorResultEn result =
      TranslationsMiniTranslatorResultEn.internal(_root);
}

// Path: workbench
class TranslationsWorkbenchEn {
  TranslationsWorkbenchEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Workspace'
  String get workspace => 'Workspace';

  /// en: 'Translate'
  String get translate => 'Translate';

  /// en: 'History'
  String get history => 'History';

  late final TranslationsWorkbenchHistoryPageEn history_page =
      TranslationsWorkbenchHistoryPageEn.internal(_root);

  /// en: 'Glossary'
  String get glossary => 'Glossary';

  /// en: 'Recent Languages'
  String get recent_languages => 'Recent Languages';

  /// en: 'Not configured'
  String get not_configured => 'Not configured';

  late final TranslationsWorkbenchSubtitleEn subtitle =
      TranslationsWorkbenchSubtitleEn.internal(_root);
  late final TranslationsWorkbenchPlaceholderEn placeholder =
      TranslationsWorkbenchPlaceholderEn.internal(_root);
  late final TranslationsWorkbenchGlossaryPageEn glossary_page =
      TranslationsWorkbenchGlossaryPageEn.internal(_root);
  late final TranslationsWorkbenchTranslationEn translation =
      TranslationsWorkbenchTranslationEn.internal(_root);
  late final TranslationsWorkbenchStatusEn status =
      TranslationsWorkbenchStatusEn.internal(_root);

  /// en: 'Up to date'
  String get version_latest => 'Up to date';

  /// en: 'Checking…'
  String get version_checking => 'Checking…';

  /// en: 'Check for updates'
  String get check_updates => 'Check for updates';
}

// Path: settings
class TranslationsSettingsEn {
  TranslationsSettingsEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'v{} (Build {})'
  String get version => 'v{} (Build {})';

  late final TranslationsSettingsNavigationEn navigation =
      TranslationsSettingsNavigationEn.internal(_root);
  late final TranslationsSettingsGeneralEn general =
      TranslationsSettingsGeneralEn.internal(_root);
  late final TranslationsSettingsAppearanceEn appearance =
      TranslationsSettingsAppearanceEn.internal(_root);
  late final TranslationsSettingsShortcutsEn shortcuts =
      TranslationsSettingsShortcutsEn.internal(_root);
  late final TranslationsSettingsAdvancedEn advanced =
      TranslationsSettingsAdvancedEn.internal(_root);
  late final TranslationsSettingsServicesEn services =
      TranslationsSettingsServicesEn.internal(_root);
  late final TranslationsSettingsProvidersEn providers =
      TranslationsSettingsProvidersEn.internal(_root);
  late final TranslationsSettingsLayoutEn layout =
      TranslationsSettingsLayoutEn.internal(_root);
  late final TranslationsSettingsAboutEn about =
      TranslationsSettingsAboutEn.internal(_root);
  late final TranslationsSettingsPermissionsEn permissions =
      TranslationsSettingsPermissionsEn.internal(_root);
}

// Path: ui
class TranslationsUiEn {
  TranslationsUiEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations
  late final TranslationsUiShellEn shell = TranslationsUiShellEn.internal(
    _root,
  );
  late final TranslationsUiFirstRunEn first_run =
      TranslationsUiFirstRunEn.internal(_root);
  late final TranslationsUiQuickEn quick = TranslationsUiQuickEn.internal(
    _root,
  );
  late final TranslationsUiErrorsEn errors = TranslationsUiErrorsEn.internal(
    _root,
  );
  late final TranslationsUiRecoveryEn recovery =
      TranslationsUiRecoveryEn.internal(_root);
  late final TranslationsUiVocabularyEn vocabulary =
      TranslationsUiVocabularyEn.internal(_root);
  late final TranslationsUiDictionaryEn dictionary =
      TranslationsUiDictionaryEn.internal(_root);
  late final TranslationsUiUpdatesEn updates = TranslationsUiUpdatesEn.internal(
    _root,
  );
  late final TranslationsUiSpeechEn speech = TranslationsUiSpeechEn.internal(
    _root,
  );
  late final TranslationsUiProvidersEn providers =
      TranslationsUiProvidersEn.internal(_root);
}

// Path: common.ui
class TranslationsCommonUiEn {
  TranslationsCommonUiEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations
  late final TranslationsCommonUiButtonEn button =
      TranslationsCommonUiButtonEn.internal(_root);
  late final TranslationsCommonUiFeedbackEn feedback =
      TranslationsCommonUiFeedbackEn.internal(_root);
}

// Path: common.language
class TranslationsCommonLanguageEn {
  TranslationsCommonLanguageEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Arabic'
  String get ar => 'Arabic';

  /// en: 'Bengali'
  String get bn => 'Bengali';

  /// en: 'German'
  String get de => 'German';

  /// en: 'English'
  String get en => 'English';

  /// en: 'Spanish'
  String get es => 'Spanish';

  /// en: 'Persian'
  String get fa => 'Persian';

  /// en: 'French'
  String get fr => 'French';

  /// en: 'Gujarati'
  String get gu => 'Gujarati';

  /// en: 'Hausa'
  String get ha => 'Hausa';

  /// en: 'Hindi'
  String get hi => 'Hindi';

  /// en: 'Indonesian'
  String get id => 'Indonesian';

  /// en: 'Italian'
  String get it => 'Italian';

  /// en: 'Japanese'
  String get ja => 'Japanese';

  /// en: 'Javanese'
  String get jv => 'Javanese';

  /// en: 'Korean'
  String get ko => 'Korean';

  /// en: 'Malayalam'
  String get ml => 'Malayalam';

  /// en: 'Marathi'
  String get mr => 'Marathi';

  /// en: 'Malay'
  String get ms => 'Malay';

  /// en: 'Dutch'
  String get nl => 'Dutch';

  /// en: 'Punjabi'
  String get pa => 'Punjabi';

  /// en: 'Polish'
  String get pl => 'Polish';

  /// en: 'Portuguese'
  String get pt => 'Portuguese';

  /// en: 'Romanian'
  String get ro => 'Romanian';

  /// en: 'Russian'
  String get ru => 'Russian';

  /// en: 'Swahili'
  String get sw => 'Swahili';

  /// en: 'Tamil'
  String get ta => 'Tamil';

  /// en: 'Telugu'
  String get te => 'Telugu';

  /// en: 'Thai'
  String get th => 'Thai';

  /// en: 'Turkish'
  String get tr => 'Turkish';

  /// en: 'Ukrainian'
  String get uk => 'Ukrainian';

  /// en: 'Urdu'
  String get ur => 'Urdu';

  /// en: 'Vietnamese'
  String get vi => 'Vietnamese';

  /// en: 'Yoruba'
  String get yo => 'Yoruba';

  /// en: 'Chinese (Simplified)'
  String get zh_hans => 'Chinese (Simplified)';

  /// en: 'Chinese (Traditional)'
  String get zh_hant => 'Chinese (Traditional)';
}

// Path: common.theme_mode
class TranslationsCommonThemeModeEn {
  TranslationsCommonThemeModeEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Light'
  String get light => 'Light';

  /// en: 'Dark'
  String get dark => 'Dark';

  /// en: 'System'
  String get system => 'System';
}

// Path: common.theme_style
class TranslationsCommonThemeStyleEn {
  TranslationsCommonThemeStyleEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Studio'
  String get studio => 'Studio';

  /// en: 'Bright'
  String get bright => 'Bright';
}

// Path: common.provider
class TranslationsCommonProviderEn {
  TranslationsCommonProviderEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Anthropic'
  String get anthropic => 'Anthropic';

  /// en: 'Baidu'
  String get baidu => 'Baidu';

  /// en: 'Caiyun'
  String get caiyun => 'Caiyun';

  /// en: 'DeepL'
  String get deepl => 'DeepL';

  /// en: 'Google'
  String get google => 'Google';

  /// en: 'Ollama'
  String get ollama => 'Ollama';

  /// en: 'OpenAI'
  String get openai => 'OpenAI';

  /// en: 'Sogou'
  String get sogou => 'Sogou';

  /// en: 'xAI'
  String get xai => 'xAI';

  /// en: 'System'
  String get system => 'System';

  /// en: 'Tencent'
  String get tencent => 'Tencent';

  /// en: 'Youda'
  String get youdao => 'Youda';
}

// Path: app.tray
class TranslationsAppTrayEn {
  TranslationsAppTrayEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations
  late final TranslationsAppTrayContextMenuEn context_menu =
      TranslationsAppTrayContextMenuEn.internal(_root);
}

// Path: mini_translator.limited_banner
class TranslationsMiniTranslatorLimitedBannerEn {
  TranslationsMiniTranslatorLimitedBannerEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations
  late final TranslationsMiniTranslatorLimitedBannerPermissionEn permission =
      TranslationsMiniTranslatorLimitedBannerPermissionEn.internal(_root);
  late final TranslationsMiniTranslatorLimitedBannerInstructionEn instruction =
      TranslationsMiniTranslatorLimitedBannerInstructionEn.internal(_root);
  late final TranslationsMiniTranslatorLimitedBannerActionEn action =
      TranslationsMiniTranslatorLimitedBannerActionEn.internal(_root);
  late final TranslationsMiniTranslatorLimitedBannerFeedbackEn feedback =
      TranslationsMiniTranslatorLimitedBannerFeedbackEn.internal(_root);
  late final TranslationsMiniTranslatorLimitedBannerTooltipEn tooltip =
      TranslationsMiniTranslatorLimitedBannerTooltipEn.internal(_root);
}

// Path: mini_translator.input
class TranslationsMiniTranslatorInputEn {
  TranslationsMiniTranslatorInputEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Enter the word or text here'
  String get hint => 'Enter the word or text here';

  /// en: 'Extracting text...'
  String get extracting_text => 'Extracting text...';

  /// en: 'Type a word or text to translate into $language'
  String hint_translate_to({required Object language}) =>
      'Type a word or text to translate into ${language}';
}

// Path: mini_translator.toolbar
class TranslationsMiniTranslatorToolbarEn {
  TranslationsMiniTranslatorToolbarEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations
  late final TranslationsMiniTranslatorToolbarTooltipEn tooltip =
      TranslationsMiniTranslatorToolbarTooltipEn.internal(_root);
  late final TranslationsMiniTranslatorToolbarMenuEn menu =
      TranslationsMiniTranslatorToolbarMenuEn.internal(_root);
}

// Path: mini_translator.button
class TranslationsMiniTranslatorButtonEn {
  TranslationsMiniTranslatorButtonEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Clear'
  String get clear => 'Clear';

  /// en: 'Translate'
  String get translate => 'Translate';

  /// en: 'Copy'
  String get copy => 'Copy';

  /// en: 'Copied'
  String get copied => 'Copied';

  /// en: 'Save'
  String get bookmark => 'Save';

  /// en: 'Saved'
  String get bookmarked => 'Saved';
}

// Path: mini_translator.language
class TranslationsMiniTranslatorLanguageEn {
  TranslationsMiniTranslatorLanguageEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Auto Detect'
  String get auto_detect => 'Auto Detect';

  /// en: 'Auto Match'
  String get auto_match => 'Auto Match';

  /// en: 'Switch Target'
  String get switch_config => 'Switch Target';

  /// en: 'More languages...'
  String get more_languages => 'More languages...';

  /// en: 'Manage Common Languages...'
  String get manage_common_languages => 'Manage Common Languages...';

  /// en: 'Manage Translation Targets...'
  String get manage_targets => 'Manage Translation Targets...';

  /// en: 'Add Translation Target...'
  String get add_target => 'Add Translation Target...';
}

// Path: mini_translator.message
class TranslationsMiniTranslatorMessageEn {
  TranslationsMiniTranslatorMessageEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'No text entered or text not extracted'
  String get please_enter_word_or_text =>
      'No text entered or text not extracted';

  /// en: 'Capture screen area has been canceled'
  String get capture_screen_area_canceled =>
      'Capture screen area has been canceled';

  /// en: 'No default text recognition service configured. Please set one in Settings.'
  String get ocr_service_not_configured =>
      'No default text recognition service configured. Please set one in Settings.';

  /// en: 'Text recognition failed'
  String get ocr_recognition_failed => 'Text recognition failed';
}

// Path: mini_translator.result
class TranslationsMiniTranslatorResultEn {
  TranslationsMiniTranslatorResultEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Translating…'
  String get translating => 'Translating…';

  /// en: 'Source changed · $key retranslate'
  String stale_requery({required Object key}) =>
      'Source changed · ${key} retranslate';

  /// en: 'Compare $count services'
  String compare_services({required Object count}) =>
      'Compare ${count} services';

  /// en: 'Collapse'
  String get collapse_compare => 'Collapse';

  /// en: 'Set as preferred'
  String get set_preferred => 'Set as preferred';

  /// en: 'Retry'
  String get retry => 'Retry';

  /// en: 'No service returned a result — check your network or try another service.'
  String get no_result =>
      'No service returned a result — check your network or try another service.';

  /// en: 'Your text is kept; retrying will not duplicate history.'
  String get no_result_note =>
      'Your text is kept; retrying will not duplicate history.';

  /// en: '$count services returned nothing'
  String no_result_meta({required Object count}) =>
      '${count} services returned nothing';

  /// en: 'No translation came back. Check your network and press ⏎ to retry, or expand to see each service's reason.'
  String get no_result_body =>
      'No translation came back. Check your network and press ⏎ to retry, or expand to see each service\'s reason.';

  /// en: 'Check services'
  String get check_services => 'Check services';

  /// en: 'Why $count services failed'
  String show_reasons({required Object count}) =>
      'Why ${count} services failed';

  /// en: 'Hide reasons'
  String get collapse_reasons => 'Hide reasons';

  /// en: 'The service gave no reason.'
  String get unknown_error => 'The service gave no reason.';

  /// en: 'no result'
  String get no_result_tag => 'no result';
}

// Path: workbench.history_page
class TranslationsWorkbenchHistoryPageEn {
  TranslationsWorkbenchHistoryPageEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'All'
  String get all => 'All';

  /// en: 'Favorites'
  String get favorites => 'Favorites';

  /// en: 'Edited by me'
  String get edited => 'Edited by me';

  /// en: 'Search'
  String get search => 'Search';

  /// en: 'Search source, translation, or service'
  String get search_placeholder => 'Search source, translation, or service';

  /// en: 'Search history'
  String get search_label => 'Search history';

  /// en: '$label · $count entries'
  String entry_count({required Object label, required Object count}) =>
      '${label} · ${count} entries';

  /// en: 'By time'
  String get by_time => 'By time';

  /// en: 'Loading history…'
  String get loading => 'Loading history…';

  /// en: 'Failed to load history'
  String get load_failed => 'Failed to load history';

  /// en: 'Retry'
  String get retry => 'Retry';

  /// en: 'No translation history yet'
  String get empty_title => 'No translation history yet';

  /// en: 'Your preferred result is saved here after a successful translation.'
  String get empty_description =>
      'Your preferred result is saved here after a successful translation.';

  /// en: 'No history matches “$query”'
  String no_results({required Object query}) => 'No history matches “${query}”';

  /// en: 'Clear search'
  String get clear_search => 'Clear search';

  /// en: 'Select'
  String get select => 'Select';

  /// en: '$count selected'
  String selected_count({required Object count}) => '${count} selected';

  /// en: 'Exit selection'
  String get exit_select => 'Exit selection';

  /// en: 'Add to glossary'
  String get add_to_glossary => 'Add to glossary';

  /// en: 'Favorite'
  String get favorite => 'Favorite';

  /// en: 'Unfavorite'
  String get unfavorite => 'Unfavorite';

  /// en: 'Delete the selected $count history entries? This cannot be undone.'
  String delete_confirm({required Object count}) =>
      'Delete the selected ${count} history entries? This cannot be undone.';

  /// en: 'Create a glossary first'
  String get no_glossary => 'Create a glossary first';

  /// en: 'Added $count entries to the glossary'
  String added_to_glossary({required Object count}) =>
      'Added ${count} entries to the glossary';

  /// en: 'Favorite'
  String get favorite_flag => 'Favorite';

  /// en: 'Edited'
  String get edited_flag => 'Edited';

  /// en: 'The edited translation will be saved to history'
  String get edit_history_hint =>
      'The edited translation will be saved to history';

  /// en: 'Copy translation'
  String get copy_translation => 'Copy translation';

  /// en: 'More'
  String get more_actions => 'More';

  /// en: 'Delete this record'
  String get delete_title_one => 'Delete this record';

  /// en: 'Delete $count records'
  String delete_title_many({required Object count}) =>
      'Delete ${count} records';

  /// en: 'This cannot be undone. Favourites and your own wording go with it; the glossary is untouched.'
  String get delete_message =>
      'This cannot be undone. Favourites and your own wording go with it; the glossary is untouched.';
}

// Path: workbench.subtitle
class TranslationsWorkbenchSubtitleEn {
  TranslationsWorkbenchSubtitleEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Workbench · Service comparison'
  String get translate => 'Workbench · Service comparison';

  /// en: 'Settings'
  String get settings => 'Settings';
}

// Path: workbench.placeholder
class TranslationsWorkbenchPlaceholderEn {
  TranslationsWorkbenchPlaceholderEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Favorites and history will be available in a future release'
  String get history =>
      'Favorites and history will be available in a future release';

  /// en: 'Glossary management is being built'
  String get glossary => 'Glossary management is being built';
}

// Path: workbench.glossary_page
class TranslationsWorkbenchGlossaryPageEn {
  TranslationsWorkbenchGlossaryPageEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'New entry'
  String get add_entry => 'New entry';

  /// en: 'Source'
  String get term => 'Source';

  /// en: 'Required translation'
  String get translation => 'Required translation';

  /// en: 'Forbidden'
  String get forbidden => 'Forbidden';

  /// en: 'Hits'
  String get hits => 'Hits';

  /// en: 'teacher forcing'
  String get term_placeholder => 'teacher forcing';

  /// en: 'forced teaching'
  String get translation_placeholder => 'forced teaching';

  /// en: 'coerced teaching'
  String get forbidden_placeholder => 'coerced teaching';

  /// en: 'Search'
  String get search => 'Search';

  /// en: 'Search terms or required translations'
  String get search_placeholder => 'Search terms or required translations';

  /// en: 'Search the glossary'
  String get search_label => 'Search the glossary';

  /// en: '$name · $count terms'
  String entry_count({required Object name, required Object count}) =>
      '${name} · ${count} terms';

  /// en: 'Glossary terms outrank any service output'
  String get priority_note => 'Glossary terms outrank any service output';

  /// en: 'New glossary'
  String get new_book => 'New glossary';

  /// en: 'Glossary name'
  String get new_book_placeholder => 'Glossary name';

  /// en: 'Rename'
  String get rename_book => 'Rename';

  /// en: 'Delete “$name” and all $count of its terms?'
  String delete_book_confirm({required Object name, required Object count}) =>
      'Delete “${name}” and all ${count} of its terms?';

  /// en: 'Off'
  String get disabled => 'Off';

  /// en: 'Enable'
  String get enable => 'Enable';

  /// en: 'Disable'
  String get disable => 'Disable';

  /// en: 'This glossary is empty'
  String get empty_title => 'This glossary is empty';

  /// en: 'Glossary terms outrank any service output. Add them one at a time, or drop in a CSV to merge.'
  String get empty_description =>
      'Glossary terms outrank any service output. Add them one at a time, or drop in a CSV to merge.';

  /// en: 'No terms match “$query”'
  String no_results_title({required Object query}) =>
      'No terms match “${query}”';

  /// en: 'Try another keyword, or add the term.'
  String get no_results_description => 'Try another keyword, or add the term.';

  /// en: 'No glossaries yet'
  String get no_books_title => 'No glossaries yet';

  /// en: 'A glossary keeps your chosen wording consistent across every service. Create one, then start adding terms.'
  String get no_books_description =>
      'A glossary keeps your chosen wording consistent across every service. Create one, then start adding terms.';

  /// en: 'Loading…'
  String get loading => 'Loading…';

  /// en: 'One book per domain; all of them apply while translating'
  String get new_book_subtitle =>
      'One book per domain; all of them apply while translating';

  /// en: 'Name'
  String get name => 'Name';

  /// en: 'Name · already exists'
  String get name_taken => 'Name · already exists';

  /// en: 'There is already a glossary called “$name”.'
  String name_taken_hint({required Object name}) =>
      'There is already a glossary called “${name}”.';

  /// en: 'Machine learning'
  String get name_placeholder => 'Machine learning';

  /// en: 'Source language'
  String get source_language => 'Source language';

  /// en: 'Target language'
  String get target_language => 'Target language';

  /// en: 'The source and target have to be two different languages.'
  String get same_language =>
      'The source and target have to be two different languages.';

  /// en: 'Initial content'
  String get seed => 'Initial content';

  /// en: 'Blank'
  String get seed_blank => 'Blank';

  /// en: 'Add entries one at a time later'
  String get seed_blank_hint => 'Add entries one at a time later';

  /// en: 'Two columns: term / translation'
  String get seed_csv_hint => 'Two columns: term / translation';

  /// en: 'Industry term exchange format'
  String get seed_tbx_hint => 'Industry term exchange format';

  /// en: 'Once it exists you can add entries one by one, or drop a CSV / TBX onto the list to merge.'
  String get seed_blank_note =>
      'Once it exists you can add entries one by one, or drop a CSV / TBX onto the list to merge.';

  /// en: 'Imports the $format file right after creating · duplicates take the file’s translation'
  String seed_file_note({required Object format}) =>
      'Imports the ${format} file right after creating · duplicates take the file’s translation';

  /// en: 'Choose file…'
  String get choose_file => 'Choose file…';

  /// en: 'Create'
  String get create => 'Create';

  /// en: 'A term outranks anything a service returns'
  String get add_entry_subtitle => 'A term outranks anything a service returns';

  /// en: 'Glossary'
  String get book => 'Glossary';

  /// en: 'Forbidden translations'
  String get forbidden_label => 'Forbidden translations';

  /// en: 'A service offering these is flagged as a conflict; separate several with /, leave empty to forbid none.'
  String get forbidden_hint =>
      'A service offering these is flagged as a conflict; separate several with /, leave empty to forbid none.';

  /// en: 'forced teaching / forcing teacher'
  String get forbidden_placeholder_full => 'forced teaching / forcing teacher';

  /// en: '“$term” is already in $book; saving overwrites the existing translation.'
  String duplicate({required Object term, required Object book}) =>
      '“${term}” is already in ${book}; saving overwrites the existing translation.';

  /// en: 'this glossary'
  String get duplicate_book_fallback => 'this glossary';

  /// en: 'Keep adding after saving'
  String get keep_adding => 'Keep adding after saving';

  /// en: 'Added $count so far'
  String added_count({required Object count}) => 'Added ${count} so far';

  /// en: 'Overwrite'
  String get overwrite => 'Overwrite';

  /// en: 'Done'
  String get done => 'Done';
}

// Path: workbench.translation
class TranslationsWorkbenchTranslationEn {
  TranslationsWorkbenchTranslationEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Source'
  String get source => 'Source';

  /// en: 'Translation'
  String get target => 'Translation';

  /// en: 'Enter or paste text to translate'
  String get input_hint => 'Enter or paste text to translate';

  /// en: 'Translate'
  String get button => 'Translate';

  /// en: 'Auto detected'
  String get auto_detected => 'Auto detected';

  /// en: 'Loading translation services…'
  String get loading_services => 'Loading translation services…';

  /// en: 'Configure a translation service in Settings first'
  String get no_services => 'Configure a translation service in Settings first';

  /// en: 'Translating…'
  String get translating => 'Translating…';

  /// en: 'Translation failed. Check the service configuration and try again.'
  String get failed =>
      'Translation failed. Check the service configuration and try again.';

  /// en: 'The translation will appear here'
  String get empty => 'The translation will appear here';

  /// en: 'Service comparison'
  String get service_compare => 'Service comparison';

  /// en: 'Primary'
  String get main_translation => 'Primary';

  /// en: 'Service unavailable'
  String get service_unavailable => 'Service unavailable';

  /// en: 'Waiting to translate'
  String get waiting => 'Waiting to translate';

  /// en: 'Copy'
  String get copy => 'Copy';

  /// en: 'Favorites will be available in a future release'
  String get favorite_unavailable =>
      'Favorites will be available in a future release';

  /// en: 'Preferred translation'
  String get preferred => 'Preferred translation';

  /// en: 'Other services'
  String get other_services => 'Other services';

  /// en: 'Copy translation'
  String get copy_result => 'Copy translation';

  /// en: 'Copied'
  String get copied => 'Copied';

  /// en: 'Save'
  String get favorite => 'Save';

  /// en: 'Glossary hits'
  String get terms => 'Glossary hits';

  /// en: 'Terms are matched once you type.'
  String get terms_hint => 'Terms are matched once you type.';

  /// en: 'Quality signals'
  String get quality => 'Quality signals';

  /// en: 'Computed once a translation lands.'
  String get quality_hint => 'Computed once a translation lands.';

  /// en: 'Shortcuts'
  String get shortcuts => 'Shortcuts';

  /// en: 'Other services are disabled'
  String get other_services_disabled => 'Other services are disabled';

  /// en: 'Type or paste text to translate into $language'
  String input_hint_translate_to({required Object language}) =>
      'Type or paste text to translate into ${language}';

  /// en: '$key for a new line'
  String newline_hint({required Object key}) => '${key} for a new line';

  /// en: 'This passage did not get a translation. Check your connection and retry, or expand to work through each service's reason.'
  String get failed_body =>
      'This passage did not get a translation. Check your connection and retry, or expand to work through each service\'s reason.';

  /// en: 'Clear'
  String get clear => 'Clear';

  /// en: 'Swap languages'
  String get swap_languages => 'Swap languages';

  /// en: 'Translation services'
  String get services => 'Translation services';

  /// en: 'Configure services'
  String get configure_services => 'Configure services';

  /// en: 'Try again'
  String get retry => 'Try again';

  /// en: '$count characters'
  String character_count({required Object count}) => '${count} characters';

  /// en: 'Install this language pair in macOS Language & Region, then try again.'
  String get language_pair_not_installed =>
      'Install this language pair in macOS Language & Region, then try again.';

  /// en: 'The selected service does not support this language pair.'
  String get unsupported_language_pair =>
      'The selected service does not support this language pair.';

  /// en: 'The source language could not be detected. Choose it manually and try again.'
  String get source_language_detection_failed =>
      'The source language could not be detected. Choose it manually and try again.';

  /// en: 'The translation service could not be reached. Check your connection and try again.'
  String get network_error =>
      'The translation service could not be reached. Check your connection and try again.';

  /// en: '$count services failed. Switch to inspect the reason.'
  String partial_failure({required Object count}) =>
      '${count} services failed. Switch to inspect the reason.';

  /// en: 'Writing translation…'
  String get streaming => 'Writing translation…';
}

// Path: workbench.status
class TranslationsWorkbenchStatusEn {
  TranslationsWorkbenchStatusEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Translation runtime ready'
  String get runtime_ready => 'Translation runtime ready';

  /// en: 'Settings synced'
  String get settings_synced => 'Settings synced';

  /// en: '⌥Space Quick window · ⌥⇧2 Capture'
  String get shortcuts => '⌥Space Quick window · ⌥⇧2 Capture';
}

// Path: settings.navigation
class TranslationsSettingsNavigationEn {
  TranslationsSettingsNavigationEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Translation'
  String get translation_group => 'Translation';

  /// en: 'Translation Settings'
  String get translation_settings => 'Translation Settings';

  /// en: 'Services'
  String get translation_services => 'Services';

  /// en: 'Favorites'
  String get favorites => 'Favorites';

  /// en: 'History'
  String get history => 'History';

  /// en: 'OCR'
  String get ocr_group => 'OCR';

  /// en: 'OCR Settings'
  String get ocr_settings => 'OCR Settings';

  /// en: 'Services'
  String get ocr_services => 'Services';

  /// en: 'General'
  String get general_group => 'General';

  /// en: 'General Settings'
  String get general_settings => 'General Settings';
}

// Path: settings.general
class TranslationsSettingsGeneralEn {
  TranslationsSettingsGeneralEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'General'
  String get title => 'General';

  late final TranslationsSettingsGeneralSectionEn section =
      TranslationsSettingsGeneralSectionEn.internal(_root);
  late final TranslationsSettingsGeneralRowEn row =
      TranslationsSettingsGeneralRowEn.internal(_root);
  late final TranslationsSettingsGeneralButtonEn button =
      TranslationsSettingsGeneralButtonEn.internal(_root);
  late final TranslationsSettingsGeneralOptionEn option =
      TranslationsSettingsGeneralOptionEn.internal(_root);
  late final TranslationsSettingsGeneralEditorEn editor =
      TranslationsSettingsGeneralEditorEn.internal(_root);
}

// Path: settings.appearance
class TranslationsSettingsAppearanceEn {
  TranslationsSettingsAppearanceEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Appearance'
  String get title => 'Appearance';

  late final TranslationsSettingsAppearanceSectionEn section =
      TranslationsSettingsAppearanceSectionEn.internal(_root);

  /// en: 'Changes apply to the whole window immediately.'
  String get footer => 'Changes apply to the whole window immediately.';
}

// Path: settings.shortcuts
class TranslationsSettingsShortcutsEn {
  TranslationsSettingsShortcutsEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Shortcuts'
  String get title => 'Shortcuts';

  late final TranslationsSettingsShortcutsSectionEn section =
      TranslationsSettingsShortcutsSectionEn.internal(_root);
  late final TranslationsSettingsShortcutsRowEn row =
      TranslationsSettingsShortcutsRowEn.internal(_root);
  late final TranslationsSettingsShortcutsDescriptionEn description =
      TranslationsSettingsShortcutsDescriptionEn.internal(_root);
  late final TranslationsSettingsShortcutsResetDialogEn reset_dialog =
      TranslationsSettingsShortcutsResetDialogEn.internal(_root);
  late final TranslationsSettingsShortcutsGroupEn group =
      TranslationsSettingsShortcutsGroupEn.internal(_root);

  /// en: 'Restore Defaults...'
  String get reset => 'Restore Defaults...';

  /// en: 'Record shortcut'
  String get record_placeholder => 'Record shortcut';

  /// en: 'Press keys…'
  String get recording => 'Press keys…';

  /// en: 'Clear'
  String get clear => 'Clear';

  /// en: 'Conflicts with “$label”'
  String conflict({required Object label}) => 'Conflicts with “${label}”';
}

// Path: settings.advanced
class TranslationsSettingsAdvancedEn {
  TranslationsSettingsAdvancedEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Advanced'
  String get title => 'Advanced';

  /// en: 'Local API server'
  String get api_server => 'Local API server';

  /// en: 'Expose the translation API on 127.0.0.1 for local integrations.'
  String get api_server_description =>
      'Expose the translation API on 127.0.0.1 for local integrations.';

  /// en: 'Enable'
  String get enable => 'Enable';

  /// en: 'Port'
  String get port => 'Port';

  /// en: 'Running at {url}'
  String get running_at => 'Running at {url}';

  /// en: 'Disabled'
  String get disabled => 'Disabled';
}

// Path: settings.services
class TranslationsSettingsServicesEn {
  TranslationsSettingsServicesEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Services'
  String get title => 'Services';

  late final TranslationsSettingsServicesButtonEn button =
      TranslationsSettingsServicesButtonEn.internal(_root);
  late final TranslationsSettingsServicesSectionEn section =
      TranslationsSettingsServicesSectionEn.internal(_root);
  late final TranslationsSettingsServicesEditorEn editor =
      TranslationsSettingsServicesEditorEn.internal(_root);
  late final TranslationsSettingsServicesDetailEn detail =
      TranslationsSettingsServicesDetailEn.internal(_root);

  /// en: 'Make Default'
  String get make_default => 'Make Default';

  late final TranslationsSettingsServicesItemEn item =
      TranslationsSettingsServicesItemEn.internal(_root);
}

// Path: settings.providers
class TranslationsSettingsProvidersEn {
  TranslationsSettingsProvidersEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Providers'
  String get title => 'Providers';

  late final TranslationsSettingsProvidersSectionEn section =
      TranslationsSettingsProvidersSectionEn.internal(_root);

  /// en: 'Search providers'
  String get search => 'Search providers';

  late final TranslationsSettingsProvidersNetworkEn network =
      TranslationsSettingsProvidersNetworkEn.internal(_root);
  late final TranslationsSettingsProvidersStabilityEn stability =
      TranslationsSettingsProvidersStabilityEn.internal(_root);

  /// en: 'This uses an unofficial web interface. Text is sent over the network when you translate. It is not an official free API.'
  String get unofficial_notice =>
      'This uses an unofficial web interface. Text is sent over the network when you translate. It is not an official free API.';

  /// en: 'Get API key'
  String get get_api_key => 'Get API key';

  /// en: 'Fetch models'
  String get fetch_models => 'Fetch models';

  /// en: 'Advanced settings'
  String get advanced => 'Advanced settings';

  /// en: 'Verify connection'
  String get verify => 'Verify connection';

  /// en: 'Save'
  String get save_without_verify => 'Save';

  /// en: 'Enter a model id'
  String get model_manual => 'Enter a model id';

  /// en: 'No models yet. Fetch from the provider or enter one manually.'
  String get model_empty =>
      'No models yet. Fetch from the provider or enter one manually.';

  /// en: 'Could not fetch models. You can still enter a model id and save.'
  String get model_failed =>
      'Could not fetch models. You can still enter a model id and save.';

  /// en: 'Add'
  String get add_directly => 'Add';

  /// en: 'Switch to Google Web'
  String get switch_to_google_web => 'Switch to Google Web';

  late final TranslationsSettingsProvidersFieldsEn fields =
      TranslationsSettingsProvidersFieldsEn.internal(_root);
  late final TranslationsSettingsProvidersCatalogEn catalog =
      TranslationsSettingsProvidersCatalogEn.internal(_root);
  late final TranslationsSettingsProvidersItemEn item =
      TranslationsSettingsProvidersItemEn.internal(_root);
  late final TranslationsSettingsProvidersButtonEn button =
      TranslationsSettingsProvidersButtonEn.internal(_root);
  late final TranslationsSettingsProvidersAlertEn alert =
      TranslationsSettingsProvidersAlertEn.internal(_root);
  late final TranslationsSettingsProvidersIntroEn intro =
      TranslationsSettingsProvidersIntroEn.internal(_root);
  late final TranslationsSettingsProvidersEditorEn editor =
      TranslationsSettingsProvidersEditorEn.internal(_root);
  late final TranslationsSettingsProvidersDetailEn detail =
      TranslationsSettingsProvidersDetailEn.internal(_root);
  late final TranslationsSettingsProvidersCapabilityEn capability =
      TranslationsSettingsProvidersCapabilityEn.internal(_root);
  late final TranslationsSettingsProvidersDescriptionEn description =
      TranslationsSettingsProvidersDescriptionEn.internal(_root);
  late final TranslationsSettingsProvidersDeleteDialogEn delete_dialog =
      TranslationsSettingsProvidersDeleteDialogEn.internal(_root);
}

// Path: settings.layout
class TranslationsSettingsLayoutEn {
  TranslationsSettingsLayoutEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Settings'
  String get title => 'Settings';

  late final TranslationsSettingsLayoutEmptyEn empty =
      TranslationsSettingsLayoutEmptyEn.internal(_root);

  /// en: 'Groups'
  String get groups => 'Groups';

  /// en: 'Changes apply immediately'
  String get effect_hint => 'Changes apply immediately';

  /// en: 'Translations and keys stay on this Mac'
  String get footer_note => 'Translations and keys stay on this Mac';

  /// en: 'Support'
  String get support => 'Support';
}

// Path: settings.about
class TranslationsSettingsAboutEn {
  TranslationsSettingsAboutEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'About'
  String get title => 'About';

  /// en: 'Copy Version Info'
  String get copy_version_info => 'Copy Version Info';

  /// en: 'You're up to date.'
  String get up_to_date => 'You\'re up to date.';

  /// en: 'Check Again'
  String get check_again => 'Check Again';

  /// en: 'Links'
  String get links => 'Links';

  /// en: 'Website'
  String get website => 'Website';

  /// en: 'GitHub'
  String get github => 'GitHub';

  /// en: 'Report an Issue'
  String get report_issue => 'Report an Issue';

  /// en: 'License'
  String get license => 'License';

  /// en: 'Open Changelog'
  String get open_changelog => 'Open Changelog';

  /// en: 'Update'
  String get update => 'Update';
}

// Path: settings.permissions
class TranslationsSettingsPermissionsEn {
  TranslationsSettingsPermissionsEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Permissions'
  String get title => 'Permissions';

  /// en: 'Windows does not require extra permissions for selection or capture.'
  String get windows_note =>
      'Windows does not require extra permissions for selection or capture.';

  /// en: 'Recheck'
  String get recheck => 'Recheck';
}

// Path: ui.shell
class TranslationsUiShellEn {
  TranslationsUiShellEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'LinguaRay'
  String get app_name => 'LinguaRay';

  /// en: 'Translate'
  String get translate => 'Translate';

  /// en: 'History'
  String get history => 'History';

  /// en: 'Glossary'
  String get glossary => 'Glossary';

  /// en: 'Vocabulary'
  String get vocabulary => 'Vocabulary';

  /// en: 'Settings'
  String get settings => 'Settings';

  /// en: 'Minimize'
  String get minimize => 'Minimize';

  /// en: 'Maximize'
  String get maximize => 'Maximize';

  /// en: 'Close'
  String get close => 'Close';
}

// Path: ui.first_run
class TranslationsUiFirstRunEn {
  TranslationsUiFirstRunEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Set up LinguaRay'
  String get title => 'Set up LinguaRay';

  /// en: 'A few checks, then you can translate from any app.'
  String get subtitle => 'A few checks, then you can translate from any app.';

  /// en: 'Permissions'
  String get permissions_title => 'Permissions';

  /// en: 'Selection translate and capture OCR need Accessibility and Screen Recording.'
  String get permissions_body =>
      'Selection translate and capture OCR need Accessibility and Screen Recording.';

  /// en: 'Accessibility'
  String get accessibility => 'Accessibility';

  /// en: 'Screen Recording'
  String get screen_recording => 'Screen Recording';

  /// en: 'Global shortcuts'
  String get shortcuts_title => 'Global shortcuts';

  /// en: 'The four first-release actions are ready. Conflicts can be fixed later in Settings.'
  String get shortcuts_body =>
      'The four first-release actions are ready. Conflicts can be fixed later in Settings.';

  /// en: 'Translation services'
  String get services_title => 'Translation services';

  /// en: 'Enable at least one translation service.'
  String get services_body => 'Enable at least one translation service.';

  /// en: 'Granted'
  String get granted => 'Granted';

  /// en: 'Not granted'
  String get denied => 'Not granted';

  /// en: 'Not required on this system'
  String get not_required => 'Not required on this system';

  /// en: 'Unknown'
  String get unknown => 'Unknown';

  /// en: 'Checking…'
  String get checking => 'Checking…';

  /// en: 'A shortcut is in conflict. You can skip this and fix it in Settings.'
  String get conflict =>
      'A shortcut is in conflict. You can skip this and fix it in Settings.';

  /// en: 'No translation service is available yet.'
  String get no_provider => 'No translation service is available yet.';

  /// en: 'A translation service is ready.'
  String get ready => 'A translation service is ready.';

  /// en: 'Grant access'
  String get grant => 'Grant access';

  /// en: 'Recheck'
  String get recheck => 'Recheck';

  /// en: 'Configure services'
  String get configure_services => 'Configure services';

  /// en: 'Get started'
  String get start => 'Get started';

  /// en: 'Skip for now'
  String get skip => 'Skip for now';
}

// Path: ui.quick
class TranslationsUiQuickEn {
  TranslationsUiQuickEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Quick translate'
  String get title => 'Quick translate';

  /// en: 'Type, paste, or fill from selection or capture'
  String get input_hint => 'Type, paste, or fill from selection or capture';

  /// en: 'Pin window'
  String get pin => 'Pin window';

  /// en: 'Unpin window'
  String get unpin => 'Unpin window';

  /// en: 'Required system permissions are missing'
  String get permission_denied => 'Required system permissions are missing';

  /// en: 'Grant Accessibility or Screen Recording in System Settings, then return and recheck.'
  String get permission_next =>
      'Grant Accessibility or Screen Recording in System Settings, then return and recheck.';

  /// en: 'Capture was cancelled. Your text was not changed.'
  String get capture_cancelled =>
      'Capture was cancelled. Your text was not changed.';

  /// en: 'Screen capture failed. Try again.'
  String get capture_failed => 'Screen capture failed. Try again.';

  /// en: 'No OCR service is configured.'
  String get ocr_not_configured => 'No OCR service is configured.';

  /// en: 'No text was found in the selected region.'
  String get ocr_empty => 'No text was found in the selected region.';

  /// en: 'No text is selected.'
  String get empty_selection => 'No text is selected.';

  /// en: 'The clipboard could not be read.'
  String get clipboard_unavailable => 'The clipboard could not be read.';

  /// en: 'The previous clipboard contents could not be restored.'
  String get clipboard_restore_failed =>
      'The previous clipboard contents could not be restored.';

  /// en: 'Recheck'
  String get recheck => 'Recheck';
}

// Path: ui.errors
class TranslationsUiErrorsEn {
  TranslationsUiErrorsEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Accessibility permission is required to read selected text.'
  String get accessibility_denied =>
      'Accessibility permission is required to read selected text.';

  /// en: 'Screen Recording permission is required to capture the screen.'
  String get screen_recording_denied =>
      'Screen Recording permission is required to capture the screen.';

  /// en: 'Screen capture failed.'
  String get capture_failed => 'Screen capture failed.';

  /// en: 'Capture was cancelled.'
  String get capture_cancelled => 'Capture was cancelled.';

  /// en: 'Configure an OCR service before capturing text.'
  String get ocr_not_configured =>
      'Configure an OCR service before capturing text.';

  /// en: 'No text was found in the selected region.'
  String get ocr_empty => 'No text was found in the selected region.';

  /// en: 'Nothing is selected.'
  String get empty_selection => 'Nothing is selected.';

  /// en: 'The clipboard is unavailable.'
  String get clipboard_unavailable => 'The clipboard is unavailable.';

  /// en: 'Clipboard restoration failed. Your previous contents may still be in the clipboard.'
  String get clipboard_restore_failed =>
      'Clipboard restoration failed. Your previous contents may still be in the clipboard.';

  /// en: 'The source language could not be detected. Choose it manually.'
  String get source_language_detection_failed =>
      'The source language could not be detected. Choose it manually.';

  /// en: 'This service does not support the selected language pair.'
  String get unsupported_language_pair =>
      'This service does not support the selected language pair.';

  /// en: 'Install this language pair in macOS Language & Region, then try again.'
  String get language_pack_missing =>
      'Install this language pair in macOS Language & Region, then try again.';

  /// en: 'The service could not be reached. Check your connection.'
  String get network_failure =>
      'The service could not be reached. Check your connection.';

  /// en: 'Authentication failed. Check the provider key in Settings.'
  String get provider_auth_failed =>
      'Authentication failed. Check the provider key in Settings.';

  /// en: 'Translation failed. Check the service and try again.'
  String get translation_failed =>
      'Translation failed. Check the service and try again.';

  /// en: 'The service returned no translation.'
  String get empty_result => 'The service returned no translation.';

  /// en: 'Translation services could not be loaded.'
  String get catalog_unavailable => 'Translation services could not be loaded.';

  /// en: 'Enable a usable translation provider before translating.'
  String get no_translation_service =>
      'Enable a usable translation provider before translating.';

  /// en: 'One glossary file could not be read. Other books still apply.'
  String get glossary_corrupt =>
      'One glossary file could not be read. Other books still apply.';

  /// en: 'History could not be loaded.'
  String get history_unavailable => 'History could not be loaded.';

  /// en: 'No dictionary service is available on this system.'
  String get dictionary_unavailable =>
      'No dictionary service is available on this system.';

  /// en: 'The vocabulary book could not be loaded.'
  String get vocabulary_unavailable =>
      'The vocabulary book could not be loaded.';

  /// en: 'Speech is not available on this system.'
  String get speech_unavailable => 'Speech is not available on this system.';

  /// en: 'Speech was interrupted.'
  String get speech_interrupted => 'Speech was interrupted.';

  /// en: 'Speech playback failed.'
  String get speech_failed => 'Speech playback failed.';

  /// en: 'The update check failed.'
  String get update_check_failed => 'The update check failed.';

  /// en: 'This update has no checksum and will not be installed.'
  String get update_checksum_missing =>
      'This update has no checksum and will not be installed.';

  /// en: 'The downloaded file did not match its checksum.'
  String get update_checksum_mismatch =>
      'The downloaded file did not match its checksum.';

  /// en: 'This LinguaRay link is not valid.'
  String get protocol_invalid => 'This LinguaRay link is not valid.';

  /// en: 'The linked text is too large to open.'
  String get protocol_too_large => 'The linked text is too large to open.';

  /// en: 'The local API server could not bind to that port.'
  String get api_server_bind_failed =>
      'The local API server could not bind to that port.';

  /// en: 'Enter 0 for an automatic port, or a port from 1 to 65535.'
  String get invalid_port =>
      'Enter 0 for an automatic port, or a port from 1 to 65535.';

  /// en: 'Something went wrong. Try again.'
  String get unknown => 'Something went wrong. Try again.';
}

// Path: ui.recovery
class TranslationsUiRecoveryEn {
  TranslationsUiRecoveryEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Try again'
  String get retry => 'Try again';

  /// en: 'Recheck permission'
  String get recheck_permission => 'Recheck permission';

  /// en: 'Open permission settings'
  String get open_permission_settings => 'Open permission settings';

  /// en: 'Configure OCR'
  String get configure_ocr => 'Configure OCR';

  /// en: 'Configure a translation provider'
  String get configure_translation_provider =>
      'Configure a translation provider';

  /// en: 'Edit input'
  String get edit_input => 'Edit input';

  /// en: 'Choose language'
  String get choose_language => 'Choose language';

  /// en: 'Switch to Google Web'
  String get switch_to_google_web => 'Switch to Google Web';
}

// Path: ui.vocabulary
class TranslationsUiVocabularyEn {
  TranslationsUiVocabularyEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Vocabulary'
  String get title => 'Vocabulary';

  /// en: 'No saved words yet'
  String get empty_title => 'No saved words yet';

  /// en: 'Save a word from a dictionary look-up or a translation result.'
  String get empty_description =>
      'Save a word from a dictionary look-up or a translation result.';

  /// en: 'Search saved words'
  String get search => 'Search saved words';

  /// en: 'All'
  String get all => 'All';

  /// en: 'Favorites'
  String get favorites => 'Favorites';

  /// en: 'Note'
  String get note => 'Note';

  /// en: 'Save word'
  String get add => 'Save word';

  /// en: 'Saved'
  String get saved => 'Saved';

  /// en: 'Delete'
  String get delete => 'Delete';

  /// en: 'Favorite'
  String get favorite => 'Favorite';

  /// en: 'Unfavorite'
  String get unfavorite => 'Unfavorite';

  /// en: 'Dictionary'
  String get source_dictionary => 'Dictionary';

  /// en: 'Translation'
  String get source_translation => 'Translation';

  /// en: 'No saved words match this search.'
  String get no_results => 'No saved words match this search.';
}

// Path: ui.dictionary
class TranslationsUiDictionaryEn {
  TranslationsUiDictionaryEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Dictionary'
  String get title => 'Dictionary';

  /// en: 'Look up a word from the translation result.'
  String get empty => 'Look up a word from the translation result.';

  /// en: 'No dictionary service is available.'
  String get unavailable => 'No dictionary service is available.';

  /// en: 'Look up'
  String get lookup => 'Look up';

  /// en: 'Pronunciation'
  String get pronunciation => 'Pronunciation';

  /// en: 'Definitions'
  String get definitions => 'Definitions';

  /// en: 'Save to vocabulary'
  String get save => 'Save to vocabulary';
}

// Path: ui.updates
class TranslationsUiUpdatesEn {
  TranslationsUiUpdatesEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Updates'
  String get title => 'Updates';

  /// en: 'Current version'
  String get current => 'Current version';

  /// en: 'Check for updates'
  String get check => 'Check for updates';

  /// en: 'Checking…'
  String get checking => 'Checking…';

  /// en: 'LinguaRay is up to date.'
  String get up_to_date => 'LinguaRay is up to date.';

  /// en: 'Version $version is available.'
  String available({required Object version}) =>
      'Version ${version} is available.';

  /// en: 'Download'
  String get download => 'Download';

  /// en: 'Downloading…'
  String get downloading => 'Downloading…';

  /// en: 'Verified. Install when you are ready.'
  String get ready => 'Verified. Install when you are ready.';

  /// en: 'Install'
  String get install => 'Install';

  /// en: 'This update has no checksum and will not be installed.'
  String get unsigned =>
      'This update has no checksum and will not be installed.';

  /// en: 'Release notes'
  String get notes => 'Release notes';
}

// Path: ui.speech
class TranslationsUiSpeechEn {
  TranslationsUiSpeechEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Speak source'
  String get speak_source => 'Speak source';

  /// en: 'Speak translation'
  String get speak_result => 'Speak translation';

  /// en: 'Stop speaking'
  String get stop => 'Stop speaking';
}

// Path: ui.providers
class TranslationsUiProvidersEn {
  TranslationsUiProvidersEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Key saved in the system keychain'
  String get secret_stored => 'Key saved in the system keychain';

  /// en: 'Leave blank to keep the saved key'
  String get secret_placeholder => 'Leave blank to keep the saved key';

  /// en: 'Fill in the required fields.'
  String get validation_missing => 'Fill in the required fields.';

  /// en: 'The provider could not be saved. Check its settings and try again.'
  String get save_failed =>
      'The provider could not be saved. Check its settings and try again.';
}

// Path: common.ui.button
class TranslationsCommonUiButtonEn {
  TranslationsCommonUiButtonEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'OK'
  String get ok => 'OK';

  /// en: 'Cancel'
  String get cancel => 'Cancel';

  /// en: 'Add'
  String get add => 'Add';

  /// en: 'Delete'
  String get delete => 'Delete';

  /// en: 'Edit'
  String get edit => 'Edit';

  /// en: 'Save'
  String get save => 'Save';

  /// en: 'Manage'
  String get manage => 'Manage';

  /// en: 'Continue'
  String get kContinue => 'Continue';
}

// Path: common.ui.feedback
class TranslationsCommonUiFeedbackEn {
  TranslationsCommonUiFeedbackEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Copied'
  String get copied => 'Copied';
}

// Path: app.tray.context_menu
class TranslationsAppTrayContextMenuEn {
  TranslationsAppTrayContextMenuEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Show Window'
  String get show_window => 'Show Window';

  /// en: 'Selection Translation'
  String get selection_translation => 'Selection Translation';

  /// en: 'Screenshot Translation'
  String get capture_translation => 'Screenshot Translation';

  /// en: 'Input Translation'
  String get input_translation => 'Input Translation';

  /// en: 'Clipboard Translation'
  String get clipboard_translation => 'Clipboard Translation';

  /// en: 'Show Translation Window'
  String get show_translation_window => 'Show Translation Window';

  /// en: 'Screenshot OCR'
  String get capture_ocr => 'Screenshot OCR';

  /// en: 'Preferences...'
  String get preferences => 'Preferences...';

  /// en: 'About LinguaRay'
  String get about => 'About LinguaRay';

  late final TranslationsAppTrayContextMenuDevToolsEn dev_tools =
      TranslationsAppTrayContextMenuDevToolsEn.internal(_root);

  /// en: 'Check for Updates'
  String get check_for_updates => 'Check for Updates';

  /// en: 'Settings'
  String get settings => 'Settings';

  /// en: 'Quit'
  String get quit => 'Quit';
}

// Path: mini_translator.limited_banner.permission
class TranslationsMiniTranslatorLimitedBannerPermissionEn {
  TranslationsMiniTranslatorLimitedBannerPermissionEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Grant Screen Recording and Accessibility permissions to enable all features.'
  String get missing_both =>
      'Grant Screen Recording and Accessibility permissions to enable all features.';

  /// en: 'Grant Screen Recording permission to enable all features.'
  String get missing_screen_capture =>
      'Grant Screen Recording permission to enable all features.';

  /// en: 'Grant Accessibility permission to enable all features.'
  String get missing_accessibility =>
      'Grant Accessibility permission to enable all features.';
}

// Path: mini_translator.limited_banner.instruction
class TranslationsMiniTranslatorLimitedBannerInstructionEn {
  TranslationsMiniTranslatorLimitedBannerInstructionEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Go to '
  String get app_settings_prefix => 'Go to ';

  /// en: ', follow the guide, then click '
  String get follow_guide_prefix => ', follow the guide, then click ';

  /// en: '.'
  String get suffix => '.';
}

// Path: mini_translator.limited_banner.action
class TranslationsMiniTranslatorLimitedBannerActionEn {
  TranslationsMiniTranslatorLimitedBannerActionEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'App Settings'
  String get app_settings => 'App Settings';

  /// en: 'Recheck'
  String get recheck => 'Recheck';
}

// Path: mini_translator.limited_banner.feedback
class TranslationsMiniTranslatorLimitedBannerFeedbackEn {
  TranslationsMiniTranslatorLimitedBannerFeedbackEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Screen text extraction is enabled.'
  String get enabled => 'Screen text extraction is enabled.';

  /// en: 'Required permissions are still missing. Please check your settings and try again.'
  String get still_missing =>
      'Required permissions are still missing.\nPlease check your settings and try again.';
}

// Path: mini_translator.limited_banner.tooltip
class TranslationsMiniTranslatorLimitedBannerTooltipEn {
  TranslationsMiniTranslatorLimitedBannerTooltipEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'View help'
  String get help => 'View help';
}

// Path: mini_translator.toolbar.tooltip
class TranslationsMiniTranslatorToolbarTooltipEn {
  TranslationsMiniTranslatorToolbarTooltipEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Capture screen area and recognize text'
  String get extract_text_from_screen_capture =>
      'Capture screen area and recognize text';

  /// en: 'Read clipboard content'
  String get extract_text_from_clipboard => 'Read clipboard content';

  /// en: 'Pin window'
  String get pin => 'Pin window';

  /// en: 'More actions'
  String get more_actions => 'More actions';
}

// Path: mini_translator.toolbar.menu
class TranslationsMiniTranslatorToolbarMenuEn {
  TranslationsMiniTranslatorToolbarMenuEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Capture from screen'
  String get extract_from_screen_capture => 'Capture from screen';

  /// en: 'From clipboard'
  String get extract_from_clipboard => 'From clipboard';

  /// en: 'Open main window'
  String get open_main_window => 'Open main window';

  /// en: 'Settings…'
  String get open_settings => 'Settings…';
}

// Path: settings.general.section
class TranslationsSettingsGeneralSectionEn {
  TranslationsSettingsGeneralSectionEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'System Permissions'
  String get permissions => 'System Permissions';

  /// en: 'Text Recognition'
  String get ocr => 'Text Recognition';

  /// en: 'Directory'
  String get directory => 'Directory';

  /// en: 'Translation'
  String get translation => 'Translation';

  /// en: 'Translation Target'
  String get translation_target => 'Translation Target';

  /// en: 'Languages'
  String get languages => 'Languages';

  /// en: 'Input Settings'
  String get input => 'Input Settings';

  /// en: 'Startup & Integration'
  String get startup => 'Startup & Integration';

  /// en: 'Capture Behaviour'
  String get ocr_behaviour => 'Capture Behaviour';

  /// en: 'Translation Behaviour'
  String get translation_behaviour => 'Translation Behaviour';
}

// Path: settings.general.row
class TranslationsSettingsGeneralRowEn {
  TranslationsSettingsGeneralRowEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Launch when you log in'
  String get launch_at_login => 'Launch when you log in';

  /// en: 'Show in menu bar'
  String get show_in_menu_bar => 'Show in menu bar';

  /// en: 'Grant screen recording access'
  String get screen_capture_access => 'Grant screen recording access';

  /// en: 'Grant accessibility access'
  String get screen_selection_access => 'Grant accessibility access';

  /// en: 'Default text recognition service'
  String get default_ocr_service => 'Default text recognition service';

  /// en: 'Auto copy detected text'
  String get auto_copy_detected_text => 'Auto copy detected text';

  /// en: 'Default directory service'
  String get default_directory_service => 'Default directory service';

  /// en: 'Default translation service'
  String get default_translation_service => 'Default translation service';

  /// en: 'Configure language pairs used by the translator.'
  String get translation_target_hint =>
      'Configure language pairs used by the translator.';

  /// en: 'Common Languages'
  String get common_languages => 'Common Languages';

  /// en: 'Shown at the top of language pickers.'
  String get common_languages_description =>
      'Shown at the top of language pickers.';

  /// en: 'Select your common languages:'
  String get common_languages_hint => 'Select your common languages:';

  /// en: 'Sort by Code'
  String get common_languages_sort => 'Sort by Code';

  /// en: 'Double click to copy translation result'
  String get double_click_copy_result =>
      'Double click to copy translation result';

  /// en: 'Reset to Defaults'
  String get common_languages_reset => 'Reset to Defaults';

  /// en: 'Reset to the default set of common languages'
  String get common_languages_reset_help =>
      'Reset to the default set of common languages';

  /// en: 'Search languages...'
  String get common_languages_search => 'Search languages...';

  /// en: 'All Languages'
  String get common_languages_all => 'All Languages';

  /// en: 'Submit with Enter'
  String get submit_with_enter => 'Submit with Enter';

  /// en: 'Submit with ⌘ + Enter'
  String get submit_with_meta_enter_mac => 'Submit with ⌘ + Enter';

  /// en: 'Capturing text from the screen needs to read screen contents.'
  String get screen_capture_access_hint =>
      'Capturing text from the screen needs to read screen contents.';

  /// en: 'Capturing selected text needs to read selections in other apps.'
  String get screen_selection_access_hint =>
      'Capturing selected text needs to read selections in other apps.';

  /// en: 'No translation targets yet — add one to set the default language to translate into.'
  String get no_translation_targets =>
      'No translation targets yet — add one to set the default language to translate into.';
}

// Path: settings.general.button
class TranslationsSettingsGeneralButtonEn {
  TranslationsSettingsGeneralButtonEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Add...'
  String get add_provider => 'Add...';

  /// en: 'Add Target...'
  String get add_target => 'Add Target...';

  /// en: 'Manage Translation Targets...'
  String get manage_targets => 'Manage Translation Targets...';

  /// en: 'Manage Common Languages...'
  String get manage_languages => 'Manage Common Languages...';

  /// en: 'Grant'
  String get grant => 'Grant';
}

// Path: settings.general.option
class TranslationsSettingsGeneralOptionEn {
  TranslationsSettingsGeneralOptionEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'None'
  String get none => 'None';

  /// en: 'No services available'
  String get no_services_available => 'No services available';

  /// en: 'Granted'
  String get granted => 'Granted';

  /// en: 'Built-in OCR'
  String get built_in_ocr => 'Built-in OCR';

  /// en: 'Tesseract'
  String get tesseract => 'Tesseract';

  /// en: 'Youdao OCR'
  String get youdao_ocr => 'Youdao OCR';
}

// Path: settings.general.editor
class TranslationsSettingsGeneralEditorEn {
  TranslationsSettingsGeneralEditorEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Add Translation Target'
  String get add_target_title => 'Add Translation Target';

  /// en: 'Edit Translation Target'
  String get edit_target_title => 'Edit Translation Target';

  late final TranslationsSettingsGeneralEditorRowEn row =
      TranslationsSettingsGeneralEditorRowEn.internal(_root);

  /// en: 'Edit Translation Target'
  String get title_edit => 'Edit Translation Target';

  /// en: 'Decide which language a source language translates into by default'
  String get subtitle =>
      'Decide which language a source language translates into by default';

  /// en: 'Source and target are the same language — pick another target.'
  String get same_language =>
      'Source and target are the same language — pick another target.';

  /// en: 'There is already a target with this pair.'
  String get duplicate => 'There is already a target with this pair.';

  /// en: 'Translate into {} when no other rule matches.'
  String get hint_auto => 'Translate into {} when no other rule matches.';

  /// en: 'Translate into {} when {} is detected.'
  String get hint_source => 'Translate into {} when {} is detected.';
}

// Path: settings.appearance.section
class TranslationsSettingsAppearanceSectionEn {
  TranslationsSettingsAppearanceSectionEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Display Language'
  String get app_language => 'Display Language';

  /// en: 'Theme Mode'
  String get theme_mode => 'Theme Mode';

  /// en: 'Theme style'
  String get theme_style => 'Theme style';
}

// Path: settings.shortcuts.section
class TranslationsSettingsShortcutsSectionEn {
  TranslationsSettingsShortcutsSectionEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Text Extraction'
  String get text_extraction => 'Text Extraction';

  /// en: 'Input Assist Function'
  String get input_assist => 'Input Assist Function';

  /// en: 'Submit With'
  String get submit_mode => 'Submit With';
}

// Path: settings.shortcuts.row
class TranslationsSettingsShortcutsRowEn {
  TranslationsSettingsShortcutsRowEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Show Translation Window'
  String get toggle_mini_translator => 'Show Translation Window';

  /// en: 'Selection Translation'
  String get extract_text_from_screen_selection => 'Selection Translation';

  /// en: 'Screenshot Translation'
  String get extract_text_from_screen_capture => 'Screenshot Translation';

  /// en: 'Screenshot OCR'
  String get capture_ocr => 'Screenshot OCR';

  /// en: 'Clipboard Translation'
  String get extract_text_from_clipboard => 'Clipboard Translation';

  /// en: 'Input Translation'
  String get translate_input => 'Input Translation';
}

// Path: settings.shortcuts.description
class TranslationsSettingsShortcutsDescriptionEn {
  TranslationsSettingsShortcutsDescriptionEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Show the translation window without clearing its previous content.'
  String get toggle_mini_translator =>
      'Show the translation window without clearing its previous content.';

  /// en: 'Translate selected text from any app.'
  String get selection => 'Translate selected text from any app.';

  /// en: 'Clear the translation window and open it for manual input.'
  String get input =>
      'Clear the translation window and open it for manual input.';

  /// en: 'Select a screen region, recognize its text, and translate it.'
  String get capture =>
      'Select a screen region, recognize its text, and translate it.';

  /// en: 'Select a screen region and show only the recognized text.'
  String get capture_ocr =>
      'Select a screen region and show only the recognized text.';

  /// en: 'Translate text currently stored in the clipboard.'
  String get clipboard => 'Translate text currently stored in the clipboard.';
}

// Path: settings.shortcuts.reset_dialog
class TranslationsSettingsShortcutsResetDialogEn {
  TranslationsSettingsShortcutsResetDialogEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Reset Shortcuts'
  String get title => 'Reset Shortcuts';

  /// en: 'Are you sure you want to reset all shortcuts to their default values?'
  String get message =>
      'Are you sure you want to reset all shortcuts to their default values?';

  /// en: 'Reset'
  String get confirm => 'Reset';

  /// en: 'Cancel'
  String get cancel => 'Cancel';
}

// Path: settings.shortcuts.group
class TranslationsSettingsShortcutsGroupEn {
  TranslationsSettingsShortcutsGroupEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations
  late final TranslationsSettingsShortcutsGroupGlobalEn global =
      TranslationsSettingsShortcutsGroupGlobalEn.internal(_root);
  late final TranslationsSettingsShortcutsGroupInAppEn in_app =
      TranslationsSettingsShortcutsGroupInAppEn.internal(_root);
}

// Path: settings.services.button
class TranslationsSettingsServicesButtonEn {
  TranslationsSettingsServicesButtonEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Add Service...'
  String get add_service => 'Add Service...';

  /// en: 'Manage Providers...'
  String get manage_providers => 'Manage Providers...';
}

// Path: settings.services.section
class TranslationsSettingsServicesSectionEn {
  TranslationsSettingsServicesSectionEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Available Services'
  String get available_services => 'Available Services';
}

// Path: settings.services.editor
class TranslationsSettingsServicesEditorEn {
  TranslationsSettingsServicesEditorEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Add Service'
  String get title => 'Add Service';

  /// en: 'Add one more service to a configured provider'
  String get subtitle => 'Add one more service to a configured provider';

  late final TranslationsSettingsServicesEditorRowEn row =
      TranslationsSettingsServicesEditorRowEn.internal(_root);

  /// en: 'Leave blank to use the default prompt for this kind'
  String get prompt_placeholder =>
      'Leave blank to use the default prompt for this kind';

  /// en: '{} already has a {} service; this one is added alongside it as a second configuration.'
  String get variant_hint =>
      '{} already has a {} service; this one is added alongside it as a second configuration.';

  /// en: '{} is a traditional endpoint — no model or prompt to tune. Its parameters live on the provider's detail page.'
  String get traditional_note =>
      '{} is a traditional endpoint — no model or prompt to tune. Its parameters live on the provider\'s detail page.';
}

// Path: settings.services.detail
class TranslationsSettingsServicesDetailEn {
  TranslationsSettingsServicesDetailEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations
  late final TranslationsSettingsServicesDetailRowEn row =
      TranslationsSettingsServicesDetailRowEn.internal(_root);
  late final TranslationsSettingsServicesDetailDeleteDialogEn delete_dialog =
      TranslationsSettingsServicesDetailDeleteDialogEn.internal(_root);

  /// en: 'Available variables: {{sourceLanguage}}, {{targetLanguage}}, {{text}}'
  String get prompt_variables =>
      'Available variables: {{sourceLanguage}}, {{targetLanguage}}, {{text}}';
}

// Path: settings.services.item
class TranslationsSettingsServicesItemEn {
  TranslationsSettingsServicesItemEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'No {} service available yet.'
  String get none_of_kind => 'No {} service available yet.';
}

// Path: settings.providers.section
class TranslationsSettingsProvidersSectionEn {
  TranslationsSettingsProvidersSectionEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Available Services'
  String get services => 'Available Services';

  /// en: 'View available services from configured providers and switch between service types.'
  String get services_description =>
      'View available services from configured providers and switch between service types.';

  /// en: 'Built-in services'
  String get built_in => 'Built-in services';

  /// en: 'Traditional translation APIs'
  String get traditional_api => 'Traditional translation APIs';

  /// en: 'Official large language models'
  String get llm_official => 'Official large language models';

  /// en: 'Aggregators'
  String get aggregator => 'Aggregators';

  /// en: 'Local and self-hosted'
  String get local_or_self_hosted => 'Local and self-hosted';
}

// Path: settings.providers.network
class TranslationsSettingsProvidersNetworkEn {
  TranslationsSettingsProvidersNetworkEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'On this device'
  String get local_only => 'On this device';

  /// en: 'Official API'
  String get official_api => 'Official API';

  /// en: 'Unofficial web interface'
  String get unofficial_web => 'Unofficial web interface';

  /// en: 'Self-hosted'
  String get self_hosted => 'Self-hosted';
}

// Path: settings.providers.stability
class TranslationsSettingsProvidersStabilityEn {
  TranslationsSettingsProvidersStabilityEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Stable'
  String get stable => 'Stable';

  /// en: 'Experimental'
  String get experimental => 'Experimental';
}

// Path: settings.providers.fields
class TranslationsSettingsProvidersFieldsEn {
  TranslationsSettingsProvidersFieldsEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'API key'
  String get api_key => 'API key';

  /// en: 'Auth key'
  String get auth_key => 'Auth key';

  /// en: 'App ID'
  String get app_id => 'App ID';

  /// en: 'App key'
  String get app_key => 'App key';

  /// en: 'App secret'
  String get app_secret => 'App secret';

  /// en: 'Secret ID'
  String get secret_id => 'Secret ID';

  /// en: 'Secret key'
  String get secret_key => 'Secret key';

  /// en: 'Token'
  String get token => 'Token';

  /// en: 'Request ID'
  String get request_id => 'Request ID';

  /// en: 'Username'
  String get username => 'Username';

  /// en: 'Model'
  String get default_model => 'Model';

  /// en: 'Base URL'
  String get base_url => 'Base URL';

  /// en: 'Models URL'
  String get models_url => 'Models URL';
}

// Path: settings.providers.catalog
class TranslationsSettingsProvidersCatalogEn {
  TranslationsSettingsProvidersCatalogEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Uses the operating system translation, dictionary, and OCR on this Mac.'
  String get system =>
      'Uses the operating system translation, dictionary, and OCR on this Mac.';

  /// en: 'Translates through Google Translate’s unofficial web interface. No API key. Experimental.'
  String get google_web =>
      'Translates through Google Translate’s unofficial web interface. No API key. Experimental.';

  /// en: 'Translates through Microsoft’s unofficial Edge translator interface. No API key. Experimental.'
  String get bing_web =>
      'Translates through Microsoft’s unofficial Edge translator interface. No API key. Experimental.';

  /// en: 'Translates through Tencent Transmart’s unofficial web interface. Optional login. Experimental.'
  String get tencent_transmart_web =>
      'Translates through Tencent Transmart’s unofficial web interface. Optional login. Experimental.';

  /// en: 'DeepL API Free.'
  String get deepl_free => 'DeepL API Free.';

  /// en: 'DeepL API Pro.'
  String get deepl_pro => 'DeepL API Pro.';

  /// en: 'Google Cloud Translation API.'
  String get google_cloud => 'Google Cloud Translation API.';

  /// en: 'Baidu Translate API.'
  String get baidu => 'Baidu Translate API.';

  /// en: 'Tencent Cloud Machine Translation.'
  String get tencent_cloud => 'Tencent Cloud Machine Translation.';

  /// en: 'Youdao Zhiyun translation, dictionary, and OCR.'
  String get youdao => 'Youdao Zhiyun translation, dictionary, and OCR.';

  /// en: 'Caiyun Xiaoyi translation API.'
  String get caiyun => 'Caiyun Xiaoyi translation API.';

  /// en: 'OpenAI official API.'
  String get openai => 'OpenAI official API.';

  /// en: 'Anthropic official API.'
  String get anthropic => 'Anthropic official API.';

  /// en: 'Google Gemini via the OpenAI-compatible endpoint.'
  String get gemini => 'Google Gemini via the OpenAI-compatible endpoint.';

  /// en: 'DeepSeek official API.'
  String get deepseek => 'DeepSeek official API.';

  /// en: 'Alibaba Bailian Qwen official API.'
  String get qwen => 'Alibaba Bailian Qwen official API.';

  /// en: 'Zhipu BigModel official API.'
  String get zhipu => 'Zhipu BigModel official API.';

  /// en: 'Moonshot Kimi official API.'
  String get moonshot => 'Moonshot Kimi official API.';

  /// en: 'Volcano Engine Doubao Ark official API.'
  String get doubao => 'Volcano Engine Doubao Ark official API.';

  /// en: 'xAI official API.'
  String get xai => 'xAI official API.';

  /// en: 'Groq official API.'
  String get groq => 'Groq official API.';

  /// en: 'OpenRouter aggregator.'
  String get openrouter => 'OpenRouter aggregator.';

  /// en: 'SiliconFlow China endpoint.'
  String get siliconflow_cn => 'SiliconFlow China endpoint.';

  /// en: 'SiliconFlow global endpoint.'
  String get siliconflow_global => 'SiliconFlow global endpoint.';

  /// en: 'ModelScope inference API.'
  String get modelscope => 'ModelScope inference API.';

  /// en: 'Local Ollama server.'
  String get ollama => 'Local Ollama server.';

  /// en: 'Local LM Studio server.'
  String get lm_studio => 'Local LM Studio server.';

  /// en: 'LocalAI server.'
  String get localai => 'LocalAI server.';

  /// en: 'Local vLLM server.'
  String get vllm => 'Local vLLM server.';

  /// en: 'Local llama.cpp server.'
  String get llama_cpp => 'Local llama.cpp server.';

  /// en: 'Local LiteLLM proxy.'
  String get litellm => 'Local LiteLLM proxy.';

  /// en: 'Self-hosted LibreTranslate. Default http://127.0.0.1:5000. Not a public instance.'
  String get libretranslate =>
      'Self-hosted LibreTranslate. Default http://127.0.0.1:5000. Not a public instance.';

  /// en: 'Self-hosted MTranServer. Default http://127.0.0.1:8989.'
  String get mtranserver =>
      'Self-hosted MTranServer. Default http://127.0.0.1:8989.';
}

// Path: settings.providers.item
class TranslationsSettingsProvidersItemEn {
  TranslationsSettingsProvidersItemEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'No providers configured. Add one to enable translation services.'
  String get empty =>
      'No providers configured. Add one to enable translation services.';

  /// en: 'Loading providers...'
  String get loading => 'Loading providers...';

  /// en: 'No services available.'
  String get no_services => 'No services available.';
}

// Path: settings.providers.button
class TranslationsSettingsProvidersButtonEn {
  TranslationsSettingsProvidersButtonEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Add a Provider...'
  String get add => 'Add a Provider...';
}

// Path: settings.providers.alert
class TranslationsSettingsProvidersAlertEn {
  TranslationsSettingsProvidersAlertEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Error'
  String get error => 'Error';
}

// Path: settings.providers.intro
class TranslationsSettingsProvidersIntroEn {
  TranslationsSettingsProvidersIntroEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Manage the service providers used by the app.'
  String get body => 'Manage the service providers used by the app.';

  /// en: 'Connected providers may process the text or images you send. Only enable services you trust.'
  String get warning =>
      'Connected providers may process the text or images you send. Only enable services you trust.';
}

// Path: settings.providers.editor
class TranslationsSettingsProvidersEditorEn {
  TranslationsSettingsProvidersEditorEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations
  late final TranslationsSettingsProvidersEditorRowEn row =
      TranslationsSettingsProvidersEditorRowEn.internal(_root);
  late final TranslationsSettingsProvidersEditorPlaceholderEn placeholder =
      TranslationsSettingsProvidersEditorPlaceholderEn.internal(_root);
  late final TranslationsSettingsProvidersEditorTypePickerEn type_picker =
      TranslationsSettingsProvidersEditorTypePickerEn.internal(_root);
  late final TranslationsSettingsProvidersEditorTooltipEn tooltip =
      TranslationsSettingsProvidersEditorTooltipEn.internal(_root);
  late final TranslationsSettingsProvidersEditorStepEn step =
      TranslationsSettingsProvidersEditorStepEn.internal(_root);

  /// en: 'Add {}'
  String get add_title => 'Add {}';

  late final TranslationsSettingsProvidersEditorCapabilityNoteEn
  capability_note =
      TranslationsSettingsProvidersEditorCapabilityNoteEn.internal(_root);
  late final TranslationsSettingsProvidersEditorTestEn test =
      TranslationsSettingsProvidersEditorTestEn.internal(_root);
}

// Path: settings.providers.detail
class TranslationsSettingsProvidersDetailEn {
  TranslationsSettingsProvidersDetailEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations
  late final TranslationsSettingsProvidersDetailTooltipEn tooltip =
      TranslationsSettingsProvidersDetailTooltipEn.internal(_root);
  late final TranslationsSettingsProvidersDetailRowEn row =
      TranslationsSettingsProvidersDetailRowEn.internal(_root);
  late final TranslationsSettingsProvidersDetailSectionEn section =
      TranslationsSettingsProvidersDetailSectionEn.internal(_root);
  late final TranslationsSettingsProvidersDetailModelsEn models =
      TranslationsSettingsProvidersDetailModelsEn.internal(_root);
}

// Path: settings.providers.capability
class TranslationsSettingsProvidersCapabilityEn {
  TranslationsSettingsProvidersCapabilityEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Translation'
  String get translation => 'Translation';

  /// en: 'Dictionary'
  String get dictionary => 'Dictionary';

  /// en: 'OCR'
  String get ocr => 'OCR';

  /// en: 'AI'
  String get llm => 'AI';
}

// Path: settings.providers.description
class TranslationsSettingsProvidersDescriptionEn {
  TranslationsSettingsProvidersDescriptionEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Provides dictionary lookup and text translation'
  String get all => 'Provides dictionary lookup and text translation';

  /// en: 'Provides dictionary lookup and word definitions'
  String get dictionary => 'Provides dictionary lookup and word definitions';

  /// en: 'Provides text translation between languages'
  String get translation => 'Provides text translation between languages';

  /// en: 'Provides translation services'
  String get fallback => 'Provides translation services';
}

// Path: settings.providers.delete_dialog
class TranslationsSettingsProvidersDeleteDialogEn {
  TranslationsSettingsProvidersDeleteDialogEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Delete "{}"?'
  String get title => 'Delete "{}"?';

  /// en: 'This action cannot be undone.'
  String get message => 'This action cannot be undone.';
}

// Path: settings.layout.empty
class TranslationsSettingsLayoutEmptyEn {
  TranslationsSettingsLayoutEmptyEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Select a Category'
  String get title => 'Select a Category';

  /// en: 'Choose a settings section from the sidebar.'
  String get message => 'Choose a settings section from the sidebar.';
}

// Path: app.tray.context_menu.dev_tools
class TranslationsAppTrayContextMenuDevToolsEn {
  TranslationsAppTrayContextMenuDevToolsEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Dev Tools'
  String get title => 'Dev Tools';

  /// en: 'Open Data Directory'
  String get open_data_directory => 'Open Data Directory';
}

// Path: settings.general.editor.row
class TranslationsSettingsGeneralEditorRowEn {
  TranslationsSettingsGeneralEditorRowEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Source Language'
  String get source_language => 'Source Language';

  /// en: 'Target Language'
  String get target_language => 'Target Language';
}

// Path: settings.shortcuts.group.global
class TranslationsSettingsShortcutsGroupGlobalEn {
  TranslationsSettingsShortcutsGroupGlobalEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Global Shortcuts'
  String get title => 'Global Shortcuts';

  /// en: 'Work in any app.'
  String get description => 'Work in any app.';
}

// Path: settings.shortcuts.group.in_app
class TranslationsSettingsShortcutsGroupInAppEn {
  TranslationsSettingsShortcutsGroupInAppEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'In-App Keys'
  String get title => 'In-App Keys';

  /// en: 'Only apply inside this app’s own text fields.'
  String get description => 'Only apply inside this app’s own text fields.';
}

// Path: settings.services.editor.row
class TranslationsSettingsServicesEditorRowEn {
  TranslationsSettingsServicesEditorRowEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Model'
  String get model => 'Model';

  /// en: 'System Prompt'
  String get system_prompt => 'System Prompt';
}

// Path: settings.services.detail.row
class TranslationsSettingsServicesDetailRowEn {
  TranslationsSettingsServicesDetailRowEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Service ID'
  String get id => 'Service ID';

  /// en: 'Name'
  String get name => 'Name';

  /// en: 'Provider'
  String get provider => 'Provider';

  /// en: 'Type'
  String get type => 'Type';
}

// Path: settings.services.detail.delete_dialog
class TranslationsSettingsServicesDetailDeleteDialogEn {
  TranslationsSettingsServicesDetailDeleteDialogEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Delete "{}"?'
  String get title => 'Delete "{}"?';

  /// en: 'This service will be removed from the provider.'
  String get message => 'This service will be removed from the provider.';
}

// Path: settings.providers.editor.row
class TranslationsSettingsProvidersEditorRowEn {
  TranslationsSettingsProvidersEditorRowEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Provider ID'
  String get id => 'Provider ID';

  /// en: 'Provider Type'
  String get type => 'Provider Type';

  /// en: 'Default Model'
  String get default_model => 'Default Model';
}

// Path: settings.providers.editor.placeholder
class TranslationsSettingsProvidersEditorPlaceholderEn {
  TranslationsSettingsProvidersEditorPlaceholderEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'e.g. deepl-main'
  String get id => 'e.g. deepl-main';
}

// Path: settings.providers.editor.type_picker
class TranslationsSettingsProvidersEditorTypePickerEn {
  TranslationsSettingsProvidersEditorTypePickerEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Select the type of provider you would like to add:'
  String get prompt => 'Select the type of provider you would like to add:';

  /// en: 'LLM'
  String get section_llm => 'LLM';

  /// en: 'Traditional'
  String get section_traditional => 'Traditional';
}

// Path: settings.providers.editor.tooltip
class TranslationsSettingsProvidersEditorTooltipEn {
  TranslationsSettingsProvidersEditorTooltipEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Help'
  String get help => 'Help';
}

// Path: settings.providers.editor.step
class TranslationsSettingsProvidersEditorStepEn {
  TranslationsSettingsProvidersEditorStepEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Continue'
  String get next => 'Continue';

  /// en: 'Back'
  String get back => 'Back';
}

// Path: settings.providers.editor.capability_note
class TranslationsSettingsProvidersEditorCapabilityNoteEn {
  TranslationsSettingsProvidersEditorCapabilityNoteEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Joins the translation candidates'
  String get translation => 'Joins the translation candidates';

  /// en: 'Provides dictionary definitions'
  String get dictionary => 'Provides dictionary definitions';

  /// en: 'Recognises text in images'
  String get ocr => 'Recognises text in images';
}

// Path: settings.providers.editor.test
class TranslationsSettingsProvidersEditorTestEn {
  TranslationsSettingsProvidersEditorTestEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Test Connection'
  String get run => 'Test Connection';

  /// en: 'Testing connection · {}s elapsed'
  String get running => 'Testing connection · {}s elapsed';

  /// en: 'Connection OK · {} models available'
  String get passed_models => 'Connection OK · {} models available';

  /// en: 'Connection OK · service available'
  String get passed_service => 'Connection OK · service available';

  /// en: 'Connection test passed'
  String get passed_footer => 'Connection test passed';

  /// en: 'Test Again'
  String get retest => 'Test Again';

  /// en: 'Things to try'
  String get tips_title => 'Things to try';

  /// en: '· Check the key matches the provider type you picked · Check whether the Base URL needs /v1 · Confirm the model is enabled in the provider console'
  String get tips_llm =>
      '· Check the key matches the provider type you picked\n· Check whether the Base URL needs /v1\n· Confirm the model is enabled in the provider console';

  /// en: '· Check the credentials match the provider type you picked · Confirm the service is enabled in the provider console'
  String get tips_traditional =>
      '· Check the credentials match the provider type you picked\n· Confirm the service is enabled in the provider console';

  /// en: 'verification failed'
  String get failed_suffix => 'verification failed';

  /// en: 'verified'
  String get passed_suffix => 'verified';
}

// Path: settings.providers.detail.tooltip
class TranslationsSettingsProvidersDetailTooltipEn {
  TranslationsSettingsProvidersDetailTooltipEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Edit provider'
  String get edit => 'Edit provider';
}

// Path: settings.providers.detail.row
class TranslationsSettingsProvidersDetailRowEn {
  TranslationsSettingsProvidersDetailRowEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Cannot be changed after creation'
  String get id_hint => 'Cannot be changed after creation';
}

// Path: settings.providers.detail.section
class TranslationsSettingsProvidersDetailSectionEn {
  TranslationsSettingsProvidersDetailSectionEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Configuration'
  String get configuration => 'Configuration';

  /// en: 'Models'
  String get models => 'Models';
}

// Path: settings.providers.detail.models
class TranslationsSettingsProvidersDetailModelsEn {
  TranslationsSettingsProvidersDetailModelsEn.internal(this._root);

  final Translations _root; // ignore: unused_field

  // Translations

  /// en: 'Loading models...'
  String get loading => 'Loading models...';

  /// en: 'No models found.'
  String get empty => 'No models found.';

  /// en: 'Retry'
  String get retry => 'Retry';

  /// en: 'Refresh List'
  String get refresh => 'Refresh List';

  /// en: 'Default'
  String get default_badge => 'Default';

  /// en: 'Set as Default'
  String get set_default => 'Set as Default';

  /// en: 'Could not fetch models from provider API.'
  String get fetch_error => 'Could not fetch models from provider API.';
}

/// The flat map containing all translations for locale <en>.
/// Only for edge cases! For simple maps, use the map function of this library.
///
/// The Dart AOT compiler has issues with very large switch statements,
/// so the map is split into smaller functions (512 entries each).
extension on Translations {
  dynamic _flatMapFunction(String path) {
    return switch (path) {
          'common.ui.button.ok' => 'OK',
          'common.ui.button.cancel' => 'Cancel',
          'common.ui.button.add' => 'Add',
          'common.ui.button.delete' => 'Delete',
          'common.ui.button.edit' => 'Edit',
          'common.ui.button.save' => 'Save',
          'common.ui.button.manage' => 'Manage',
          'common.ui.button.kContinue' => 'Continue',
          'common.ui.feedback.copied' => 'Copied',
          'common.language.ar' => 'Arabic',
          'common.language.bn' => 'Bengali',
          'common.language.de' => 'German',
          'common.language.en' => 'English',
          'common.language.es' => 'Spanish',
          'common.language.fa' => 'Persian',
          'common.language.fr' => 'French',
          'common.language.gu' => 'Gujarati',
          'common.language.ha' => 'Hausa',
          'common.language.hi' => 'Hindi',
          'common.language.id' => 'Indonesian',
          'common.language.it' => 'Italian',
          'common.language.ja' => 'Japanese',
          'common.language.jv' => 'Javanese',
          'common.language.ko' => 'Korean',
          'common.language.ml' => 'Malayalam',
          'common.language.mr' => 'Marathi',
          'common.language.ms' => 'Malay',
          'common.language.nl' => 'Dutch',
          'common.language.pa' => 'Punjabi',
          'common.language.pl' => 'Polish',
          'common.language.pt' => 'Portuguese',
          'common.language.ro' => 'Romanian',
          'common.language.ru' => 'Russian',
          'common.language.sw' => 'Swahili',
          'common.language.ta' => 'Tamil',
          'common.language.te' => 'Telugu',
          'common.language.th' => 'Thai',
          'common.language.tr' => 'Turkish',
          'common.language.uk' => 'Ukrainian',
          'common.language.ur' => 'Urdu',
          'common.language.vi' => 'Vietnamese',
          'common.language.yo' => 'Yoruba',
          'common.language.zh_hans' => 'Chinese (Simplified)',
          'common.language.zh_hant' => 'Chinese (Traditional)',
          'common.theme_mode.light' => 'Light',
          'common.theme_mode.dark' => 'Dark',
          'common.theme_mode.system' => 'System',
          'common.theme_style.studio' => 'Studio',
          'common.theme_style.bright' => 'Bright',
          'common.provider.anthropic' => 'Anthropic',
          'common.provider.baidu' => 'Baidu',
          'common.provider.caiyun' => 'Caiyun',
          'common.provider.deepl' => 'DeepL',
          'common.provider.google' => 'Google',
          'common.provider.ollama' => 'Ollama',
          'common.provider.openai' => 'OpenAI',
          'common.provider.sogou' => 'Sogou',
          'common.provider.xai' => 'xAI',
          'common.provider.system' => 'System',
          'common.provider.tencent' => 'Tencent',
          'common.provider.youdao' => 'Youda',
          'app.tray.context_menu.show_window' => 'Show Window',
          'app.tray.context_menu.selection_translation' =>
            'Selection Translation',
          'app.tray.context_menu.capture_translation' =>
            'Screenshot Translation',
          'app.tray.context_menu.input_translation' => 'Input Translation',
          'app.tray.context_menu.clipboard_translation' =>
            'Clipboard Translation',
          'app.tray.context_menu.show_translation_window' =>
            'Show Translation Window',
          'app.tray.context_menu.capture_ocr' => 'Screenshot OCR',
          'app.tray.context_menu.preferences' => 'Preferences...',
          'app.tray.context_menu.about' => 'About LinguaRay',
          'app.tray.context_menu.dev_tools.title' => 'Dev Tools',
          'app.tray.context_menu.dev_tools.open_data_directory' =>
            'Open Data Directory',
          'app.tray.context_menu.check_for_updates' => 'Check for Updates',
          'app.tray.context_menu.settings' => 'Settings',
          'app.tray.context_menu.quit' => 'Quit',
          'mini_translator.limited_banner.permission.missing_both' => 'Grant Screen Recording and Accessibility permissions to enable all features.',
          'mini_translator.limited_banner.permission.missing_screen_capture' =>
            'Grant Screen Recording permission to enable all features.',
          'mini_translator.limited_banner.permission.missing_accessibility' =>
            'Grant Accessibility permission to enable all features.',
          'mini_translator.limited_banner.instruction.app_settings_prefix' =>
            'Go to ',
          'mini_translator.limited_banner.instruction.follow_guide_prefix' =>
            ', follow the guide, then click ',
          'mini_translator.limited_banner.instruction.suffix' => '.',
          'mini_translator.limited_banner.action.app_settings' =>
            'App Settings',
          'mini_translator.limited_banner.action.recheck' => 'Recheck',
          'mini_translator.limited_banner.feedback.enabled' =>
            'Screen text extraction is enabled.',
          'mini_translator.limited_banner.feedback.still_missing' => 'Required permissions are still missing.\nPlease check your settings and try again.',
          'mini_translator.limited_banner.tooltip.help' => 'View help',
          'mini_translator.input.hint' => 'Enter the word or text here',
          'mini_translator.input.extracting_text' => 'Extracting text...',
          'mini_translator.input.hint_translate_to' => ({
            required Object language,
          }) => 'Type a word or text to translate into ${language}',
          'mini_translator.toolbar.tooltip.extract_text_from_screen_capture' =>
            'Capture screen area and recognize text',
          'mini_translator.toolbar.tooltip.extract_text_from_clipboard' =>
            'Read clipboard content',
          'mini_translator.toolbar.tooltip.pin' => 'Pin window',
          'mini_translator.toolbar.tooltip.more_actions' => 'More actions',
          'mini_translator.toolbar.menu.extract_from_screen_capture' =>
            'Capture from screen',
          'mini_translator.toolbar.menu.extract_from_clipboard' =>
            'From clipboard',
          'mini_translator.toolbar.menu.open_main_window' => 'Open main window',
          'mini_translator.toolbar.menu.open_settings' => 'Settings…',
          'mini_translator.button.clear' => 'Clear',
          'mini_translator.button.translate' => 'Translate',
          'mini_translator.button.copy' => 'Copy',
          'mini_translator.button.copied' => 'Copied',
          'mini_translator.button.bookmark' => 'Save',
          'mini_translator.button.bookmarked' => 'Saved',
          'mini_translator.language.auto_detect' => 'Auto Detect',
          'mini_translator.language.auto_match' => 'Auto Match',
          'mini_translator.language.switch_config' => 'Switch Target',
          'mini_translator.language.more_languages' => 'More languages...',
          'mini_translator.language.manage_common_languages' =>
            'Manage Common Languages...',
          'mini_translator.language.manage_targets' =>
            'Manage Translation Targets...',
          'mini_translator.language.add_target' => 'Add Translation Target...',
          'mini_translator.message.please_enter_word_or_text' =>
            'No text entered or text not extracted',
          'mini_translator.message.capture_screen_area_canceled' =>
            'Capture screen area has been canceled',
          'mini_translator.message.ocr_service_not_configured' => 'No default text recognition service configured. Please set one in Settings.',
          'mini_translator.message.ocr_recognition_failed' =>
            'Text recognition failed',
          'mini_translator.result.translating' => 'Translating…',
          'mini_translator.result.stale_requery' => ({
            required Object key,
          }) => 'Source changed · ${key} retranslate',
          'mini_translator.result.compare_services' => ({
            required Object count,
          }) => 'Compare ${count} services',
          'mini_translator.result.collapse_compare' => 'Collapse',
          'mini_translator.result.set_preferred' => 'Set as preferred',
          'mini_translator.result.retry' => 'Retry',
          'mini_translator.result.no_result' => 'No service returned a result — check your network or try another service.',
          'mini_translator.result.no_result_note' =>
            'Your text is kept; retrying will not duplicate history.',
          'mini_translator.result.no_result_meta' => ({
            required Object count,
          }) => '${count} services returned nothing',
          'mini_translator.result.no_result_body' => 'No translation came back. Check your network and press ⏎ to retry, or expand to see each service\'s reason.',
          'mini_translator.result.check_services' => 'Check services',
          'mini_translator.result.show_reasons' => ({
            required Object count,
          }) => 'Why ${count} services failed',
          'mini_translator.result.collapse_reasons' => 'Hide reasons',
          'mini_translator.result.unknown_error' =>
            'The service gave no reason.',
          'mini_translator.result.no_result_tag' => 'no result',
          'workbench.workspace' => 'Workspace',
          'workbench.translate' => 'Translate',
          'workbench.history' => 'History',
          'workbench.history_page.all' => 'All',
          'workbench.history_page.favorites' => 'Favorites',
          'workbench.history_page.edited' => 'Edited by me',
          'workbench.history_page.search' => 'Search',
          'workbench.history_page.search_placeholder' =>
            'Search source, translation, or service',
          'workbench.history_page.search_label' => 'Search history',
          'workbench.history_page.entry_count' => ({
            required Object label,
            required Object count,
          }) => '${label} · ${count} entries',
          'workbench.history_page.by_time' => 'By time',
          'workbench.history_page.loading' => 'Loading history…',
          'workbench.history_page.load_failed' => 'Failed to load history',
          'workbench.history_page.retry' => 'Retry',
          'workbench.history_page.empty_title' => 'No translation history yet',
          'workbench.history_page.empty_description' => 'Your preferred result is saved here after a successful translation.',
          'workbench.history_page.no_results' => ({
            required Object query,
          }) => 'No history matches “${query}”',
          'workbench.history_page.clear_search' => 'Clear search',
          'workbench.history_page.select' => 'Select',
          'workbench.history_page.selected_count' => ({
            required Object count,
          }) => '${count} selected',
          'workbench.history_page.exit_select' => 'Exit selection',
          'workbench.history_page.add_to_glossary' => 'Add to glossary',
          'workbench.history_page.favorite' => 'Favorite',
          'workbench.history_page.unfavorite' => 'Unfavorite',
          'workbench.history_page.delete_confirm' => ({
            required Object count,
          }) => 'Delete the selected ${count} history entries? This cannot be undone.',
          'workbench.history_page.no_glossary' => 'Create a glossary first',
          'workbench.history_page.added_to_glossary' => ({
            required Object count,
          }) => 'Added ${count} entries to the glossary',
          'workbench.history_page.favorite_flag' => 'Favorite',
          'workbench.history_page.edited_flag' => 'Edited',
          'workbench.history_page.edit_history_hint' =>
            'The edited translation will be saved to history',
          'workbench.history_page.copy_translation' => 'Copy translation',
          'workbench.history_page.more_actions' => 'More',
          'workbench.history_page.delete_title_one' => 'Delete this record',
          'workbench.history_page.delete_title_many' => ({
            required Object count,
          }) => 'Delete ${count} records',
          'workbench.history_page.delete_message' => 'This cannot be undone. Favourites and your own wording go with it; the glossary is untouched.',
          'workbench.glossary' => 'Glossary',
          'workbench.recent_languages' => 'Recent Languages',
          'workbench.not_configured' => 'Not configured',
          'workbench.subtitle.translate' => 'Workbench · Service comparison',
          'workbench.subtitle.settings' => 'Settings',
          'workbench.placeholder.history' =>
            'Favorites and history will be available in a future release',
          'workbench.placeholder.glossary' =>
            'Glossary management is being built',
          'workbench.glossary_page.add_entry' => 'New entry',
          'workbench.glossary_page.term' => 'Source',
          'workbench.glossary_page.translation' => 'Required translation',
          'workbench.glossary_page.forbidden' => 'Forbidden',
          'workbench.glossary_page.hits' => 'Hits',
          'workbench.glossary_page.term_placeholder' => 'teacher forcing',
          'workbench.glossary_page.translation_placeholder' =>
            'forced teaching',
          'workbench.glossary_page.forbidden_placeholder' => 'coerced teaching',
          'workbench.glossary_page.search' => 'Search',
          'workbench.glossary_page.search_placeholder' =>
            'Search terms or required translations',
          'workbench.glossary_page.search_label' => 'Search the glossary',
          'workbench.glossary_page.entry_count' => ({
            required Object name,
            required Object count,
          }) => '${name} · ${count} terms',
          'workbench.glossary_page.priority_note' =>
            'Glossary terms outrank any service output',
          'workbench.glossary_page.new_book' => 'New glossary',
          'workbench.glossary_page.new_book_placeholder' => 'Glossary name',
          'workbench.glossary_page.rename_book' => 'Rename',
          'workbench.glossary_page.delete_book_confirm' => ({
            required Object name,
            required Object count,
          }) => 'Delete “${name}” and all ${count} of its terms?',
          'workbench.glossary_page.disabled' => 'Off',
          'workbench.glossary_page.enable' => 'Enable',
          'workbench.glossary_page.disable' => 'Disable',
          'workbench.glossary_page.empty_title' => 'This glossary is empty',
          'workbench.glossary_page.empty_description' => 'Glossary terms outrank any service output. Add them one at a time, or drop in a CSV to merge.',
          'workbench.glossary_page.no_results_title' => ({
            required Object query,
          }) => 'No terms match “${query}”',
          'workbench.glossary_page.no_results_description' =>
            'Try another keyword, or add the term.',
          'workbench.glossary_page.no_books_title' => 'No glossaries yet',
          'workbench.glossary_page.no_books_description' => 'A glossary keeps your chosen wording consistent across every service. Create one, then start adding terms.',
          'workbench.glossary_page.loading' => 'Loading…',
          'workbench.glossary_page.new_book_subtitle' =>
            'One book per domain; all of them apply while translating',
          'workbench.glossary_page.name' => 'Name',
          'workbench.glossary_page.name_taken' => 'Name · already exists',
          'workbench.glossary_page.name_taken_hint' => ({
            required Object name,
          }) => 'There is already a glossary called “${name}”.',
          'workbench.glossary_page.name_placeholder' => 'Machine learning',
          'workbench.glossary_page.source_language' => 'Source language',
          'workbench.glossary_page.target_language' => 'Target language',
          'workbench.glossary_page.same_language' =>
            'The source and target have to be two different languages.',
          'workbench.glossary_page.seed' => 'Initial content',
          'workbench.glossary_page.seed_blank' => 'Blank',
          'workbench.glossary_page.seed_blank_hint' =>
            'Add entries one at a time later',
          'workbench.glossary_page.seed_csv_hint' =>
            'Two columns: term / translation',
          'workbench.glossary_page.seed_tbx_hint' =>
            'Industry term exchange format',
          'workbench.glossary_page.seed_blank_note' => 'Once it exists you can add entries one by one, or drop a CSV / TBX onto the list to merge.',
          'workbench.glossary_page.seed_file_note' => ({
            required Object format,
          }) => 'Imports the ${format} file right after creating · duplicates take the file’s translation',
          'workbench.glossary_page.choose_file' => 'Choose file…',
          'workbench.glossary_page.create' => 'Create',
          'workbench.glossary_page.add_entry_subtitle' =>
            'A term outranks anything a service returns',
          'workbench.glossary_page.book' => 'Glossary',
          'workbench.glossary_page.forbidden_label' => 'Forbidden translations',
          'workbench.glossary_page.forbidden_hint' => 'A service offering these is flagged as a conflict; separate several with /, leave empty to forbid none.',
          'workbench.glossary_page.forbidden_placeholder_full' =>
            'forced teaching / forcing teacher',
          'workbench.glossary_page.duplicate' => ({
            required Object term,
            required Object book,
          }) => '“${term}” is already in ${book}; saving overwrites the existing translation.',
          'workbench.glossary_page.duplicate_book_fallback' => 'this glossary',
          'workbench.glossary_page.keep_adding' => 'Keep adding after saving',
          'workbench.glossary_page.added_count' => ({
            required Object count,
          }) => 'Added ${count} so far',
          'workbench.glossary_page.overwrite' => 'Overwrite',
          'workbench.glossary_page.done' => 'Done',
          'workbench.translation.source' => 'Source',
          'workbench.translation.target' => 'Translation',
          'workbench.translation.input_hint' =>
            'Enter or paste text to translate',
          'workbench.translation.button' => 'Translate',
          'workbench.translation.auto_detected' => 'Auto detected',
          'workbench.translation.loading_services' =>
            'Loading translation services…',
          'workbench.translation.no_services' =>
            'Configure a translation service in Settings first',
          'workbench.translation.translating' => 'Translating…',
          'workbench.translation.failed' => 'Translation failed. Check the service configuration and try again.',
          'workbench.translation.empty' => 'The translation will appear here',
          'workbench.translation.service_compare' => 'Service comparison',
          'workbench.translation.main_translation' => 'Primary',
          'workbench.translation.service_unavailable' => 'Service unavailable',
          'workbench.translation.waiting' => 'Waiting to translate',
          'workbench.translation.copy' => 'Copy',
          'workbench.translation.favorite_unavailable' =>
            'Favorites will be available in a future release',
          'workbench.translation.preferred' => 'Preferred translation',
          'workbench.translation.other_services' => 'Other services',
          'workbench.translation.copy_result' => 'Copy translation',
          'workbench.translation.copied' => 'Copied',
          'workbench.translation.favorite' => 'Save',
          'workbench.translation.terms' => 'Glossary hits',
          'workbench.translation.terms_hint' =>
            'Terms are matched once you type.',
          'workbench.translation.quality' => 'Quality signals',
          'workbench.translation.quality_hint' =>
            'Computed once a translation lands.',
          'workbench.translation.shortcuts' => 'Shortcuts',
          'workbench.translation.other_services_disabled' =>
            'Other services are disabled',
          'workbench.translation.input_hint_translate_to' => ({
            required Object language,
          }) => 'Type or paste text to translate into ${language}',
          'workbench.translation.newline_hint' => ({
            required Object key,
          }) => '${key} for a new line',
          'workbench.translation.failed_body' => 'This passage did not get a translation. Check your connection and retry, or expand to work through each service\'s reason.',
          'workbench.translation.clear' => 'Clear',
          'workbench.translation.swap_languages' => 'Swap languages',
          'workbench.translation.services' => 'Translation services',
          'workbench.translation.configure_services' => 'Configure services',
          'workbench.translation.retry' => 'Try again',
          'workbench.translation.character_count' => ({
            required Object count,
          }) => '${count} characters',
          'workbench.translation.language_pair_not_installed' => 'Install this language pair in macOS Language & Region, then try again.',
          'workbench.translation.unsupported_language_pair' =>
            'The selected service does not support this language pair.',
          'workbench.translation.source_language_detection_failed' => 'The source language could not be detected. Choose it manually and try again.',
          'workbench.translation.network_error' => 'The translation service could not be reached. Check your connection and try again.',
          'workbench.translation.partial_failure' => ({
            required Object count,
          }) => '${count} services failed. Switch to inspect the reason.',
          'workbench.translation.streaming' => 'Writing translation…',
          'workbench.status.runtime_ready' => 'Translation runtime ready',
          'workbench.status.settings_synced' => 'Settings synced',
          'workbench.status.shortcuts' => '⌥Space Quick window · ⌥⇧2 Capture',
          'workbench.version_latest' => 'Up to date',
          'workbench.version_checking' => 'Checking…',
          'workbench.check_updates' => 'Check for updates',
          'settings.version' => 'v{} (Build {})',
          'settings.navigation.translation_group' => 'Translation',
          'settings.navigation.translation_settings' => 'Translation Settings',
          'settings.navigation.translation_services' => 'Services',
          'settings.navigation.favorites' => 'Favorites',
          'settings.navigation.history' => 'History',
          'settings.navigation.ocr_group' => 'OCR',
          'settings.navigation.ocr_settings' => 'OCR Settings',
          'settings.navigation.ocr_services' => 'Services',
          'settings.navigation.general_group' => 'General',
          'settings.navigation.general_settings' => 'General Settings',
          'settings.general.title' => 'General',
          'settings.general.section.permissions' => 'System Permissions',
          'settings.general.section.ocr' => 'Text Recognition',
          'settings.general.section.directory' => 'Directory',
          'settings.general.section.translation' => 'Translation',
          'settings.general.section.translation_target' => 'Translation Target',
          'settings.general.section.languages' => 'Languages',
          'settings.general.section.input' => 'Input Settings',
          'settings.general.section.startup' => 'Startup & Integration',
          'settings.general.section.ocr_behaviour' => 'Capture Behaviour',
          'settings.general.section.translation_behaviour' =>
            'Translation Behaviour',
          'settings.general.row.launch_at_login' => 'Launch when you log in',
          'settings.general.row.show_in_menu_bar' => 'Show in menu bar',
          'settings.general.row.screen_capture_access' =>
            'Grant screen recording access',
          'settings.general.row.screen_selection_access' =>
            'Grant accessibility access',
          'settings.general.row.default_ocr_service' =>
            'Default text recognition service',
          'settings.general.row.auto_copy_detected_text' =>
            'Auto copy detected text',
          'settings.general.row.default_directory_service' =>
            'Default directory service',
          'settings.general.row.default_translation_service' =>
            'Default translation service',
          'settings.general.row.translation_target_hint' =>
            'Configure language pairs used by the translator.',
          'settings.general.row.common_languages' => 'Common Languages',
          'settings.general.row.common_languages_description' =>
            'Shown at the top of language pickers.',
          'settings.general.row.common_languages_hint' =>
            'Select your common languages:',
          'settings.general.row.common_languages_sort' => 'Sort by Code',
          'settings.general.row.double_click_copy_result' =>
            'Double click to copy translation result',
          'settings.general.row.common_languages_reset' => 'Reset to Defaults',
          'settings.general.row.common_languages_reset_help' =>
            'Reset to the default set of common languages',
          'settings.general.row.common_languages_search' =>
            'Search languages...',
          'settings.general.row.common_languages_all' => 'All Languages',
          'settings.general.row.submit_with_enter' => 'Submit with Enter',
          'settings.general.row.submit_with_meta_enter_mac' =>
            'Submit with ⌘ + Enter',
          'settings.general.row.screen_capture_access_hint' =>
            'Capturing text from the screen needs to read screen contents.',
          'settings.general.row.screen_selection_access_hint' =>
            'Capturing selected text needs to read selections in other apps.',
          'settings.general.row.no_translation_targets' => 'No translation targets yet — add one to set the default language to translate into.',
          'settings.general.button.add_provider' => 'Add...',
          'settings.general.button.add_target' => 'Add Target...',
          'settings.general.button.manage_targets' =>
            'Manage Translation Targets...',
          'settings.general.button.manage_languages' =>
            'Manage Common Languages...',
          'settings.general.button.grant' => 'Grant',
          'settings.general.option.none' => 'None',
          'settings.general.option.no_services_available' =>
            'No services available',
          'settings.general.option.granted' => 'Granted',
          'settings.general.option.built_in_ocr' => 'Built-in OCR',
          'settings.general.option.tesseract' => 'Tesseract',
          'settings.general.option.youdao_ocr' => 'Youdao OCR',
          'settings.general.editor.add_target_title' =>
            'Add Translation Target',
          'settings.general.editor.edit_target_title' =>
            'Edit Translation Target',
          'settings.general.editor.row.source_language' => 'Source Language',
          'settings.general.editor.row.target_language' => 'Target Language',
          'settings.general.editor.title_edit' => 'Edit Translation Target',
          'settings.general.editor.subtitle' => 'Decide which language a source language translates into by default',
          'settings.general.editor.same_language' =>
            'Source and target are the same language — pick another target.',
          'settings.general.editor.duplicate' =>
            'There is already a target with this pair.',
          'settings.general.editor.hint_auto' =>
            'Translate into {} when no other rule matches.',
          'settings.general.editor.hint_source' =>
            'Translate into {} when {} is detected.',
          'settings.appearance.title' => 'Appearance',
          'settings.appearance.section.app_language' => 'Display Language',
          'settings.appearance.section.theme_mode' => 'Theme Mode',
          'settings.appearance.section.theme_style' => 'Theme style',
          'settings.appearance.footer' =>
            'Changes apply to the whole window immediately.',
          'settings.shortcuts.title' => 'Shortcuts',
          'settings.shortcuts.section.text_extraction' => 'Text Extraction',
          'settings.shortcuts.section.input_assist' => 'Input Assist Function',
          'settings.shortcuts.section.submit_mode' => 'Submit With',
          'settings.shortcuts.row.toggle_mini_translator' =>
            'Show Translation Window',
          'settings.shortcuts.row.extract_text_from_screen_selection' =>
            'Selection Translation',
          'settings.shortcuts.row.extract_text_from_screen_capture' =>
            'Screenshot Translation',
          'settings.shortcuts.row.capture_ocr' => 'Screenshot OCR',
          'settings.shortcuts.row.extract_text_from_clipboard' =>
            'Clipboard Translation',
          'settings.shortcuts.row.translate_input' => 'Input Translation',
          'settings.shortcuts.description.toggle_mini_translator' => 'Show the translation window without clearing its previous content.',
          'settings.shortcuts.description.selection' =>
            'Translate selected text from any app.',
          'settings.shortcuts.description.input' =>
            'Clear the translation window and open it for manual input.',
          'settings.shortcuts.description.capture' =>
            'Select a screen region, recognize its text, and translate it.',
          'settings.shortcuts.description.capture_ocr' =>
            'Select a screen region and show only the recognized text.',
          'settings.shortcuts.description.clipboard' =>
            'Translate text currently stored in the clipboard.',
          'settings.shortcuts.reset_dialog.title' => 'Reset Shortcuts',
          'settings.shortcuts.reset_dialog.message' => 'Are you sure you want to reset all shortcuts to their default values?',
          'settings.shortcuts.reset_dialog.confirm' => 'Reset',
          'settings.shortcuts.reset_dialog.cancel' => 'Cancel',
          'settings.shortcuts.group.global.title' => 'Global Shortcuts',
          'settings.shortcuts.group.global.description' => 'Work in any app.',
          'settings.shortcuts.group.in_app.title' => 'In-App Keys',
          'settings.shortcuts.group.in_app.description' =>
            'Only apply inside this app’s own text fields.',
          'settings.shortcuts.reset' => 'Restore Defaults...',
          'settings.shortcuts.record_placeholder' => 'Record shortcut',
          'settings.shortcuts.recording' => 'Press keys…',
          'settings.shortcuts.clear' => 'Clear',
          'settings.shortcuts.conflict' => ({
            required Object label,
          }) => 'Conflicts with “${label}”',
          'settings.advanced.title' => 'Advanced',
          'settings.advanced.api_server' => 'Local API server',
          'settings.advanced.api_server_description' =>
            'Expose the translation API on 127.0.0.1 for local integrations.',
          'settings.advanced.enable' => 'Enable',
          'settings.advanced.port' => 'Port',
          'settings.advanced.running_at' => 'Running at {url}',
          'settings.advanced.disabled' => 'Disabled',
          'settings.services.title' => 'Services',
          'settings.services.button.add_service' => 'Add Service...',
          'settings.services.button.manage_providers' => 'Manage Providers...',
          'settings.services.section.available_services' =>
            'Available Services',
          'settings.services.editor.title' => 'Add Service',
          'settings.services.editor.subtitle' =>
            'Add one more service to a configured provider',
          'settings.services.editor.row.model' => 'Model',
          'settings.services.editor.row.system_prompt' => 'System Prompt',
          'settings.services.editor.prompt_placeholder' =>
            'Leave blank to use the default prompt for this kind',
          'settings.services.editor.variant_hint' => '{} already has a {} service; this one is added alongside it as a second configuration.',
          'settings.services.editor.traditional_note' => '{} is a traditional endpoint — no model or prompt to tune. Its parameters live on the provider\'s detail page.',
          'settings.services.detail.row.id' => 'Service ID',
          'settings.services.detail.row.name' => 'Name',
          'settings.services.detail.row.provider' => 'Provider',
          'settings.services.detail.row.type' => 'Type',
          'settings.services.detail.delete_dialog.title' => 'Delete "{}"?',
          'settings.services.detail.delete_dialog.message' =>
            'This service will be removed from the provider.',
          'settings.services.detail.prompt_variables' => 'Available variables: {{sourceLanguage}}, {{targetLanguage}}, {{text}}',
          'settings.services.make_default' => 'Make Default',
          'settings.services.item.none_of_kind' =>
            'No {} service available yet.',
          'settings.providers.title' => 'Providers',
          'settings.providers.section.services' => 'Available Services',
          'settings.providers.section.services_description' => 'View available services from configured providers and switch between service types.',
          'settings.providers.section.built_in' => 'Built-in services',
          'settings.providers.section.traditional_api' =>
            'Traditional translation APIs',
          'settings.providers.section.llm_official' =>
            'Official large language models',
          'settings.providers.section.aggregator' => 'Aggregators',
          'settings.providers.section.local_or_self_hosted' =>
            'Local and self-hosted',
          'settings.providers.search' => 'Search providers',
          'settings.providers.network.local_only' => 'On this device',
          'settings.providers.network.official_api' => 'Official API',
          'settings.providers.network.unofficial_web' =>
            'Unofficial web interface',
          'settings.providers.network.self_hosted' => 'Self-hosted',
          'settings.providers.stability.stable' => 'Stable',
          'settings.providers.stability.experimental' => 'Experimental',
          'settings.providers.unofficial_notice' => 'This uses an unofficial web interface. Text is sent over the network when you translate. It is not an official free API.',
          'settings.providers.get_api_key' => 'Get API key',
          'settings.providers.fetch_models' => 'Fetch models',
          'settings.providers.advanced' => 'Advanced settings',
          'settings.providers.verify' => 'Verify connection',
          'settings.providers.save_without_verify' => 'Save',
          'settings.providers.model_manual' => 'Enter a model id',
          'settings.providers.model_empty' =>
            'No models yet. Fetch from the provider or enter one manually.',
          'settings.providers.model_failed' =>
            'Could not fetch models. You can still enter a model id and save.',
          'settings.providers.add_directly' => 'Add',
          'settings.providers.switch_to_google_web' => 'Switch to Google Web',
          'settings.providers.fields.api_key' => 'API key',
          'settings.providers.fields.auth_key' => 'Auth key',
          'settings.providers.fields.app_id' => 'App ID',
          'settings.providers.fields.app_key' => 'App key',
          'settings.providers.fields.app_secret' => 'App secret',
          'settings.providers.fields.secret_id' => 'Secret ID',
          'settings.providers.fields.secret_key' => 'Secret key',
          'settings.providers.fields.token' => 'Token',
          'settings.providers.fields.request_id' => 'Request ID',
          'settings.providers.fields.username' => 'Username',
          'settings.providers.fields.default_model' => 'Model',
          'settings.providers.fields.base_url' => 'Base URL',
          'settings.providers.fields.models_url' => 'Models URL',
          'settings.providers.catalog.system' => 'Uses the operating system translation, dictionary, and OCR on this Mac.',
          'settings.providers.catalog.google_web' => 'Translates through Google Translate’s unofficial web interface. No API key. Experimental.',
          'settings.providers.catalog.bing_web' => 'Translates through Microsoft’s unofficial Edge translator interface. No API key. Experimental.',
          'settings.providers.catalog.tencent_transmart_web' => 'Translates through Tencent Transmart’s unofficial web interface. Optional login. Experimental.',
          'settings.providers.catalog.deepl_free' => 'DeepL API Free.',
          'settings.providers.catalog.deepl_pro' => 'DeepL API Pro.',
          'settings.providers.catalog.google_cloud' =>
            'Google Cloud Translation API.',
          'settings.providers.catalog.baidu' => 'Baidu Translate API.',
          'settings.providers.catalog.tencent_cloud' =>
            'Tencent Cloud Machine Translation.',
          'settings.providers.catalog.youdao' =>
            'Youdao Zhiyun translation, dictionary, and OCR.',
          'settings.providers.catalog.caiyun' =>
            'Caiyun Xiaoyi translation API.',
          'settings.providers.catalog.openai' => 'OpenAI official API.',
          'settings.providers.catalog.anthropic' => 'Anthropic official API.',
          'settings.providers.catalog.gemini' =>
            'Google Gemini via the OpenAI-compatible endpoint.',
          'settings.providers.catalog.deepseek' => 'DeepSeek official API.',
          'settings.providers.catalog.qwen' =>
            'Alibaba Bailian Qwen official API.',
          'settings.providers.catalog.zhipu' => 'Zhipu BigModel official API.',
          'settings.providers.catalog.moonshot' =>
            'Moonshot Kimi official API.',
          'settings.providers.catalog.doubao' =>
            'Volcano Engine Doubao Ark official API.',
          'settings.providers.catalog.xai' => 'xAI official API.',
          'settings.providers.catalog.groq' => 'Groq official API.',
          'settings.providers.catalog.openrouter' => 'OpenRouter aggregator.',
          'settings.providers.catalog.siliconflow_cn' =>
            'SiliconFlow China endpoint.',
          'settings.providers.catalog.siliconflow_global' =>
            'SiliconFlow global endpoint.',
          'settings.providers.catalog.modelscope' =>
            'ModelScope inference API.',
          'settings.providers.catalog.ollama' => 'Local Ollama server.',
          'settings.providers.catalog.lm_studio' => 'Local LM Studio server.',
          'settings.providers.catalog.localai' => 'LocalAI server.',
          'settings.providers.catalog.vllm' => 'Local vLLM server.',
          'settings.providers.catalog.llama_cpp' => 'Local llama.cpp server.',
          'settings.providers.catalog.litellm' => 'Local LiteLLM proxy.',
          'settings.providers.catalog.libretranslate' => 'Self-hosted LibreTranslate. Default http://127.0.0.1:5000. Not a public instance.',
          'settings.providers.catalog.mtranserver' =>
            'Self-hosted MTranServer. Default http://127.0.0.1:8989.',
          'settings.providers.item.empty' =>
            'No providers configured. Add one to enable translation services.',
          'settings.providers.item.loading' => 'Loading providers...',
          'settings.providers.item.no_services' => 'No services available.',
          'settings.providers.button.add' => 'Add a Provider...',
          'settings.providers.alert.error' => 'Error',
          'settings.providers.intro.body' =>
            'Manage the service providers used by the app.',
          'settings.providers.intro.warning' => 'Connected providers may process the text or images you send. Only enable services you trust.',
          'settings.providers.editor.row.id' => 'Provider ID',
          'settings.providers.editor.row.type' => 'Provider Type',
          'settings.providers.editor.row.default_model' => 'Default Model',
          'settings.providers.editor.placeholder.id' => 'e.g. deepl-main',
          'settings.providers.editor.type_picker.prompt' =>
            'Select the type of provider you would like to add:',
          'settings.providers.editor.type_picker.section_llm' => 'LLM',
          'settings.providers.editor.type_picker.section_traditional' =>
            'Traditional',
          'settings.providers.editor.tooltip.help' => 'Help',
          'settings.providers.editor.step.next' => 'Continue',
          'settings.providers.editor.step.back' => 'Back',
          'settings.providers.editor.add_title' => 'Add {}',
          'settings.providers.editor.capability_note.translation' =>
            'Joins the translation candidates',
          'settings.providers.editor.capability_note.dictionary' =>
            'Provides dictionary definitions',
          'settings.providers.editor.capability_note.ocr' =>
            'Recognises text in images',
          'settings.providers.editor.test.run' => 'Test Connection',
          'settings.providers.editor.test.running' =>
            'Testing connection · {}s elapsed',
          'settings.providers.editor.test.passed_models' =>
            'Connection OK · {} models available',
          'settings.providers.editor.test.passed_service' =>
            'Connection OK · service available',
          'settings.providers.editor.test.passed_footer' =>
            'Connection test passed',
          'settings.providers.editor.test.retest' => 'Test Again',
          'settings.providers.editor.test.tips_title' => 'Things to try',
          'settings.providers.editor.test.tips_llm' => '· Check the key matches the provider type you picked\n· Check whether the Base URL needs /v1\n· Confirm the model is enabled in the provider console',
          'settings.providers.editor.test.tips_traditional' => '· Check the credentials match the provider type you picked\n· Confirm the service is enabled in the provider console',
          'settings.providers.editor.test.failed_suffix' =>
            'verification failed',
          'settings.providers.editor.test.passed_suffix' => 'verified',
          'settings.providers.detail.tooltip.edit' => 'Edit provider',
          'settings.providers.detail.row.id_hint' =>
            'Cannot be changed after creation',
          'settings.providers.detail.section.configuration' => 'Configuration',
          'settings.providers.detail.section.models' => 'Models',
          'settings.providers.detail.models.loading' => 'Loading models...',
          'settings.providers.detail.models.empty' => 'No models found.',
          'settings.providers.detail.models.retry' => 'Retry',
          _ => null,
        } ??
        switch (path) {
          'settings.providers.detail.models.refresh' => 'Refresh List',
          'settings.providers.detail.models.default_badge' => 'Default',
          'settings.providers.detail.models.set_default' => 'Set as Default',
          'settings.providers.detail.models.fetch_error' =>
            'Could not fetch models from provider API.',
          'settings.providers.capability.translation' => 'Translation',
          'settings.providers.capability.dictionary' => 'Dictionary',
          'settings.providers.capability.ocr' => 'OCR',
          'settings.providers.capability.llm' => 'AI',
          'settings.providers.description.all' =>
            'Provides dictionary lookup and text translation',
          'settings.providers.description.dictionary' =>
            'Provides dictionary lookup and word definitions',
          'settings.providers.description.translation' =>
            'Provides text translation between languages',
          'settings.providers.description.fallback' =>
            'Provides translation services',
          'settings.providers.delete_dialog.title' => 'Delete "{}"?',
          'settings.providers.delete_dialog.message' =>
            'This action cannot be undone.',
          'settings.layout.title' => 'Settings',
          'settings.layout.empty.title' => 'Select a Category',
          'settings.layout.empty.message' =>
            'Choose a settings section from the sidebar.',
          'settings.layout.groups' => 'Groups',
          'settings.layout.effect_hint' => 'Changes apply immediately',
          'settings.layout.footer_note' =>
            'Translations and keys stay on this Mac',
          'settings.layout.support' => 'Support',
          'settings.about.title' => 'About',
          'settings.about.copy_version_info' => 'Copy Version Info',
          'settings.about.up_to_date' => 'You\'re up to date.',
          'settings.about.check_again' => 'Check Again',
          'settings.about.links' => 'Links',
          'settings.about.website' => 'Website',
          'settings.about.github' => 'GitHub',
          'settings.about.report_issue' => 'Report an Issue',
          'settings.about.license' => 'License',
          'settings.about.open_changelog' => 'Open Changelog',
          'settings.about.update' => 'Update',
          'settings.permissions.title' => 'Permissions',
          'settings.permissions.windows_note' => 'Windows does not require extra permissions for selection or capture.',
          'settings.permissions.recheck' => 'Recheck',
          'ui.shell.app_name' => 'LinguaRay',
          'ui.shell.translate' => 'Translate',
          'ui.shell.history' => 'History',
          'ui.shell.glossary' => 'Glossary',
          'ui.shell.vocabulary' => 'Vocabulary',
          'ui.shell.settings' => 'Settings',
          'ui.shell.minimize' => 'Minimize',
          'ui.shell.maximize' => 'Maximize',
          'ui.shell.close' => 'Close',
          'ui.first_run.title' => 'Set up LinguaRay',
          'ui.first_run.subtitle' =>
            'A few checks, then you can translate from any app.',
          'ui.first_run.permissions_title' => 'Permissions',
          'ui.first_run.permissions_body' => 'Selection translate and capture OCR need Accessibility and Screen Recording.',
          'ui.first_run.accessibility' => 'Accessibility',
          'ui.first_run.screen_recording' => 'Screen Recording',
          'ui.first_run.shortcuts_title' => 'Global shortcuts',
          'ui.first_run.shortcuts_body' => 'The four first-release actions are ready. Conflicts can be fixed later in Settings.',
          'ui.first_run.services_title' => 'Translation services',
          'ui.first_run.services_body' =>
            'Enable at least one translation service.',
          'ui.first_run.granted' => 'Granted',
          'ui.first_run.denied' => 'Not granted',
          'ui.first_run.not_required' => 'Not required on this system',
          'ui.first_run.unknown' => 'Unknown',
          'ui.first_run.checking' => 'Checking…',
          'ui.first_run.conflict' => 'A shortcut is in conflict. You can skip this and fix it in Settings.',
          'ui.first_run.no_provider' =>
            'No translation service is available yet.',
          'ui.first_run.ready' => 'A translation service is ready.',
          'ui.first_run.grant' => 'Grant access',
          'ui.first_run.recheck' => 'Recheck',
          'ui.first_run.configure_services' => 'Configure services',
          'ui.first_run.start' => 'Get started',
          'ui.first_run.skip' => 'Skip for now',
          'ui.quick.title' => 'Quick translate',
          'ui.quick.input_hint' =>
            'Type, paste, or fill from selection or capture',
          'ui.quick.pin' => 'Pin window',
          'ui.quick.unpin' => 'Unpin window',
          'ui.quick.permission_denied' =>
            'Required system permissions are missing',
          'ui.quick.permission_next' => 'Grant Accessibility or Screen Recording in System Settings, then return and recheck.',
          'ui.quick.capture_cancelled' =>
            'Capture was cancelled. Your text was not changed.',
          'ui.quick.capture_failed' => 'Screen capture failed. Try again.',
          'ui.quick.ocr_not_configured' => 'No OCR service is configured.',
          'ui.quick.ocr_empty' => 'No text was found in the selected region.',
          'ui.quick.empty_selection' => 'No text is selected.',
          'ui.quick.clipboard_unavailable' =>
            'The clipboard could not be read.',
          'ui.quick.clipboard_restore_failed' =>
            'The previous clipboard contents could not be restored.',
          'ui.quick.recheck' => 'Recheck',
          'ui.errors.accessibility_denied' =>
            'Accessibility permission is required to read selected text.',
          'ui.errors.screen_recording_denied' =>
            'Screen Recording permission is required to capture the screen.',
          'ui.errors.capture_failed' => 'Screen capture failed.',
          'ui.errors.capture_cancelled' => 'Capture was cancelled.',
          'ui.errors.ocr_not_configured' =>
            'Configure an OCR service before capturing text.',
          'ui.errors.ocr_empty' => 'No text was found in the selected region.',
          'ui.errors.empty_selection' => 'Nothing is selected.',
          'ui.errors.clipboard_unavailable' => 'The clipboard is unavailable.',
          'ui.errors.clipboard_restore_failed' => 'Clipboard restoration failed. Your previous contents may still be in the clipboard.',
          'ui.errors.source_language_detection_failed' =>
            'The source language could not be detected. Choose it manually.',
          'ui.errors.unsupported_language_pair' =>
            'This service does not support the selected language pair.',
          'ui.errors.language_pack_missing' => 'Install this language pair in macOS Language & Region, then try again.',
          'ui.errors.network_failure' =>
            'The service could not be reached. Check your connection.',
          'ui.errors.provider_auth_failed' =>
            'Authentication failed. Check the provider key in Settings.',
          'ui.errors.translation_failed' =>
            'Translation failed. Check the service and try again.',
          'ui.errors.empty_result' => 'The service returned no translation.',
          'ui.errors.catalog_unavailable' =>
            'Translation services could not be loaded.',
          'ui.errors.no_translation_service' =>
            'Enable a usable translation provider before translating.',
          'ui.errors.glossary_corrupt' =>
            'One glossary file could not be read. Other books still apply.',
          'ui.errors.history_unavailable' => 'History could not be loaded.',
          'ui.errors.dictionary_unavailable' =>
            'No dictionary service is available on this system.',
          'ui.errors.vocabulary_unavailable' =>
            'The vocabulary book could not be loaded.',
          'ui.errors.speech_unavailable' =>
            'Speech is not available on this system.',
          'ui.errors.speech_interrupted' => 'Speech was interrupted.',
          'ui.errors.speech_failed' => 'Speech playback failed.',
          'ui.errors.update_check_failed' => 'The update check failed.',
          'ui.errors.update_checksum_missing' =>
            'This update has no checksum and will not be installed.',
          'ui.errors.update_checksum_mismatch' =>
            'The downloaded file did not match its checksum.',
          'ui.errors.protocol_invalid' => 'This LinguaRay link is not valid.',
          'ui.errors.protocol_too_large' =>
            'The linked text is too large to open.',
          'ui.errors.api_server_bind_failed' =>
            'The local API server could not bind to that port.',
          'ui.errors.invalid_port' =>
            'Enter 0 for an automatic port, or a port from 1 to 65535.',
          'ui.errors.unknown' => 'Something went wrong. Try again.',
          'ui.recovery.retry' => 'Try again',
          'ui.recovery.recheck_permission' => 'Recheck permission',
          'ui.recovery.open_permission_settings' => 'Open permission settings',
          'ui.recovery.configure_ocr' => 'Configure OCR',
          'ui.recovery.configure_translation_provider' =>
            'Configure a translation provider',
          'ui.recovery.edit_input' => 'Edit input',
          'ui.recovery.choose_language' => 'Choose language',
          'ui.recovery.switch_to_google_web' => 'Switch to Google Web',
          'ui.vocabulary.title' => 'Vocabulary',
          'ui.vocabulary.empty_title' => 'No saved words yet',
          'ui.vocabulary.empty_description' =>
            'Save a word from a dictionary look-up or a translation result.',
          'ui.vocabulary.search' => 'Search saved words',
          'ui.vocabulary.all' => 'All',
          'ui.vocabulary.favorites' => 'Favorites',
          'ui.vocabulary.note' => 'Note',
          'ui.vocabulary.add' => 'Save word',
          'ui.vocabulary.saved' => 'Saved',
          'ui.vocabulary.delete' => 'Delete',
          'ui.vocabulary.favorite' => 'Favorite',
          'ui.vocabulary.unfavorite' => 'Unfavorite',
          'ui.vocabulary.source_dictionary' => 'Dictionary',
          'ui.vocabulary.source_translation' => 'Translation',
          'ui.vocabulary.no_results' => 'No saved words match this search.',
          'ui.dictionary.title' => 'Dictionary',
          'ui.dictionary.empty' =>
            'Look up a word from the translation result.',
          'ui.dictionary.unavailable' => 'No dictionary service is available.',
          'ui.dictionary.lookup' => 'Look up',
          'ui.dictionary.pronunciation' => 'Pronunciation',
          'ui.dictionary.definitions' => 'Definitions',
          'ui.dictionary.save' => 'Save to vocabulary',
          'ui.updates.title' => 'Updates',
          'ui.updates.current' => 'Current version',
          'ui.updates.check' => 'Check for updates',
          'ui.updates.checking' => 'Checking…',
          'ui.updates.up_to_date' => 'LinguaRay is up to date.',
          'ui.updates.available' => ({
            required Object version,
          }) => 'Version ${version} is available.',
          'ui.updates.download' => 'Download',
          'ui.updates.downloading' => 'Downloading…',
          'ui.updates.ready' => 'Verified. Install when you are ready.',
          'ui.updates.install' => 'Install',
          'ui.updates.unsigned' =>
            'This update has no checksum and will not be installed.',
          'ui.updates.notes' => 'Release notes',
          'ui.speech.speak_source' => 'Speak source',
          'ui.speech.speak_result' => 'Speak translation',
          'ui.speech.stop' => 'Stop speaking',
          'ui.providers.secret_stored' => 'Key saved in the system keychain',
          'ui.providers.secret_placeholder' =>
            'Leave blank to keep the saved key',
          'ui.providers.validation_missing' => 'Fill in the required fields.',
          'ui.providers.save_failed' => 'The provider could not be saved. Check its settings and try again.',
          _ => null,
        };
  }
}
