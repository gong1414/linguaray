///
/// Generated file. Do not edit.
///
// coverage:ignore-file
// ignore_for_file: type=lint, unused_import

import 'package:flutter/widgets.dart';
import 'package:intl/intl.dart';
import 'package:slang/generated.dart';

import 'strings.g.dart';

// Path: <root>
class TranslationsZhHant extends Translations
    with BaseTranslations<AppLocale, Translations> {
  /// You can call this constructor and build your own translation instance of this locale.
  /// Constructing via the enum [AppLocale.build] is preferred.
  TranslationsZhHant({
    Map<String, Node>? overrides,
    PluralResolver? cardinalResolver,
    PluralResolver? ordinalResolver,
    TranslationMetadata<AppLocale, Translations>? meta,
  }) : assert(
         overrides == null,
         'Set "translation_overrides: true" in order to enable this feature.',
       ),
       _meta =
           meta ??
           TranslationMetadata(
             locale: AppLocale.zhHant,
             overrides: overrides ?? {},
             cardinalResolver: cardinalResolver,
             ordinalResolver: ordinalResolver,
           ),
       super(
         cardinalResolver: cardinalResolver,
         ordinalResolver: ordinalResolver,
       ) {
    _meta.setFlatMapFunction(_flatMapFunction);
  }

  /// Metadata for the translations of <zh-Hant>.
  final TranslationMetadata<AppLocale, Translations> _meta;
  @override
  TranslationMetadata<AppLocale, Translations> get $meta => _meta;

  /// Access flat map
  @override
  dynamic operator [](String key) => _meta.getTranslation(key) ?? super[key];

  late final TranslationsZhHant _root = this; // ignore: unused_field

  @override
  TranslationsZhHant $copyWith({
    TranslationMetadata<AppLocale, Translations>? meta,
  }) => TranslationsZhHant(meta: meta ?? this.$meta);

  // Translations
  @override
  late final Translations$common$zh_Hant common =
      Translations$common$zh_Hant.internal(_root);
  @override
  late final Translations$app$zh_Hant app = Translations$app$zh_Hant.internal(
    _root,
  );
  @override
  late final Translations$mini_translator$zh_Hant mini_translator =
      Translations$mini_translator$zh_Hant.internal(_root);
  @override
  late final Translations$workbench$zh_Hant workbench =
      Translations$workbench$zh_Hant.internal(_root);
  @override
  late final Translations$settings$zh_Hant settings =
      Translations$settings$zh_Hant.internal(_root);
}

// Path: common
class Translations$common$zh_Hant extends Translations$common$en {
  Translations$common$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  late final Translations$common$ui$zh_Hant ui =
      Translations$common$ui$zh_Hant.internal(_root);
  @override
  late final Translations$common$language$zh_Hant language =
      Translations$common$language$zh_Hant.internal(_root);
  @override
  late final Translations$common$theme_mode$zh_Hant theme_mode =
      Translations$common$theme_mode$zh_Hant.internal(_root);
  @override
  late final Translations$common$theme_style$zh_Hant theme_style =
      Translations$common$theme_style$zh_Hant.internal(_root);
  @override
  late final Translations$common$provider$zh_Hant provider =
      Translations$common$provider$zh_Hant.internal(_root);
}

// Path: app
class Translations$app$zh_Hant extends Translations$app$en {
  Translations$app$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  late final Translations$app$tray$zh_Hant tray =
      Translations$app$tray$zh_Hant.internal(_root);
}

// Path: mini_translator
class Translations$mini_translator$zh_Hant
    extends Translations$mini_translator$en {
  Translations$mini_translator$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  late final Translations$mini_translator$limited_banner$zh_Hant
  limited_banner = Translations$mini_translator$limited_banner$zh_Hant.internal(
    _root,
  );
  @override
  late final Translations$mini_translator$input$zh_Hant input =
      Translations$mini_translator$input$zh_Hant.internal(_root);
  @override
  late final Translations$mini_translator$toolbar$zh_Hant toolbar =
      Translations$mini_translator$toolbar$zh_Hant.internal(_root);
  @override
  late final Translations$mini_translator$button$zh_Hant button =
      Translations$mini_translator$button$zh_Hant.internal(_root);
  @override
  late final Translations$mini_translator$language$zh_Hant language =
      Translations$mini_translator$language$zh_Hant.internal(_root);
  @override
  late final Translations$mini_translator$message$zh_Hant message =
      Translations$mini_translator$message$zh_Hant.internal(_root);
  @override
  late final Translations$mini_translator$result$zh_Hant result =
      Translations$mini_translator$result$zh_Hant.internal(_root);
}

// Path: workbench
class Translations$workbench$zh_Hant extends Translations$workbench$en {
  Translations$workbench$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get workspace => '工作區';
  @override
  String get translate => '翻譯';
  @override
  String get history => '歷史';
  @override
  late final Translations$workbench$history_page$zh_Hant history_page =
      Translations$workbench$history_page$zh_Hant.internal(_root);
  @override
  String get glossary => '術語庫';
  @override
  String get recent_languages => '最近語言';
  @override
  String get not_configured => '尚未設定';
  @override
  late final Translations$workbench$subtitle$zh_Hant subtitle =
      Translations$workbench$subtitle$zh_Hant.internal(_root);
  @override
  late final Translations$workbench$placeholder$zh_Hant placeholder =
      Translations$workbench$placeholder$zh_Hant.internal(_root);
  @override
  late final Translations$workbench$glossary_page$zh_Hant glossary_page =
      Translations$workbench$glossary_page$zh_Hant.internal(_root);
  @override
  late final Translations$workbench$translation$zh_Hant translation =
      Translations$workbench$translation$zh_Hant.internal(_root);
  @override
  late final Translations$workbench$status$zh_Hant status =
      Translations$workbench$status$zh_Hant.internal(_root);
  @override
  String get version_latest => '已是最新';
  @override
  String get version_checking => '正在檢查…';
  @override
  String get check_updates => '檢查更新';
}

// Path: settings
class Translations$settings$zh_Hant extends Translations$settings$en {
  Translations$settings$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get version => 'v{} (Build {})';
  @override
  late final Translations$settings$general$zh_Hant general =
      Translations$settings$general$zh_Hant.internal(_root);
  @override
  late final Translations$settings$appearance$zh_Hant appearance =
      Translations$settings$appearance$zh_Hant.internal(_root);
  @override
  late final Translations$settings$shortcuts$zh_Hant shortcuts =
      Translations$settings$shortcuts$zh_Hant.internal(_root);
  @override
  late final Translations$settings$advanced$zh_Hant advanced =
      Translations$settings$advanced$zh_Hant.internal(_root);
  @override
  late final Translations$settings$services$zh_Hant services =
      Translations$settings$services$zh_Hant.internal(_root);
  @override
  late final Translations$settings$providers$zh_Hant providers =
      Translations$settings$providers$zh_Hant.internal(_root);
  @override
  late final Translations$settings$layout$zh_Hant layout =
      Translations$settings$layout$zh_Hant.internal(_root);
  @override
  late final Translations$settings$about$zh_Hant about =
      Translations$settings$about$zh_Hant.internal(_root);
}

// Path: common.ui
class Translations$common$ui$zh_Hant extends Translations$common$ui$en {
  Translations$common$ui$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  late final Translations$common$ui$button$zh_Hant button =
      Translations$common$ui$button$zh_Hant.internal(_root);
  @override
  late final Translations$common$ui$feedback$zh_Hant feedback =
      Translations$common$ui$feedback$zh_Hant.internal(_root);
}

// Path: common.language
class Translations$common$language$zh_Hant
    extends Translations$common$language$en {
  Translations$common$language$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get ar => '阿拉伯語';
  @override
  String get bn => '孟加拉語';
  @override
  String get de => '德語';
  @override
  String get en => '英語';
  @override
  String get es => '西班牙語';
  @override
  String get fa => '波斯語';
  @override
  String get fr => '法語';
  @override
  String get gu => '古吉拉特語';
  @override
  String get ha => '豪薩語';
  @override
  String get hi => '印地語';
  @override
  String get id => '印尼語';
  @override
  String get it => '義大利語';
  @override
  String get ja => '日語';
  @override
  String get jv => '爪哇語';
  @override
  String get ko => '韓語';
  @override
  String get ml => '馬拉雅拉姆語';
  @override
  String get mr => '馬拉地語';
  @override
  String get ms => '馬來語';
  @override
  String get nl => '荷蘭語';
  @override
  String get pa => '旁遮普語';
  @override
  String get pl => '波蘭語';
  @override
  String get pt => '葡萄牙語';
  @override
  String get ro => '羅馬尼亞語';
  @override
  String get ru => '俄語';
  @override
  String get sw => '斯瓦希里語';
  @override
  String get ta => '泰米爾語';
  @override
  String get te => '泰盧固語';
  @override
  String get th => '泰語';
  @override
  String get tr => '土耳其語';
  @override
  String get uk => '烏克蘭語';
  @override
  String get ur => '烏爾都語';
  @override
  String get vi => '越南語';
  @override
  String get yo => '約魯巴語';
  @override
  String get zh_hans => '中文（簡體）';
  @override
  String get zh_hant => '中文（繁體）';
}

// Path: common.theme_mode
class Translations$common$theme_mode$zh_Hant
    extends Translations$common$theme_mode$en {
  Translations$common$theme_mode$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get light => '淺色';
  @override
  String get dark => '深色';
  @override
  String get system => '跟隨系統';
}

// Path: common.theme_style
class Translations$common$theme_style$zh_Hant
    extends Translations$common$theme_style$en {
  Translations$common$theme_style$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get studio => 'Studio';
  @override
  String get bright => 'Bright';
}

// Path: common.provider
class Translations$common$provider$zh_Hant
    extends Translations$common$provider$en {
  Translations$common$provider$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get anthropic => 'Anthropic';
  @override
  String get baidu => '百度';
  @override
  String get caiyun => '彩雲小譯';
  @override
  String get deepl => 'DeepL';
  @override
  String get google => '谷歌';
  @override
  String get ollama => 'Ollama';
  @override
  String get openai => 'OpenAI';
  @override
  String get sogou => '搜狗';
  @override
  String get xai => 'xAI';
  @override
  String get system => '系統';
  @override
  String get tencent => '騰訊';
  @override
  String get youdao => '有道';
}

// Path: app.tray
class Translations$app$tray$zh_Hant extends Translations$app$tray$en {
  Translations$app$tray$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  late final Translations$app$tray$context_menu$zh_Hant context_menu =
      Translations$app$tray$context_menu$zh_Hant.internal(_root);
}

// Path: mini_translator.limited_banner
class Translations$mini_translator$limited_banner$zh_Hant
    extends Translations$mini_translator$limited_banner$en {
  Translations$mini_translator$limited_banner$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  late final Translations$mini_translator$limited_banner$permission$zh_Hant
  permission =
      Translations$mini_translator$limited_banner$permission$zh_Hant.internal(
        _root,
      );
  @override
  late final Translations$mini_translator$limited_banner$instruction$zh_Hant
  instruction =
      Translations$mini_translator$limited_banner$instruction$zh_Hant.internal(
        _root,
      );
  @override
  late final Translations$mini_translator$limited_banner$action$zh_Hant action =
      Translations$mini_translator$limited_banner$action$zh_Hant.internal(
        _root,
      );
  @override
  late final Translations$mini_translator$limited_banner$feedback$zh_Hant
  feedback =
      Translations$mini_translator$limited_banner$feedback$zh_Hant.internal(
        _root,
      );
  @override
  late final Translations$mini_translator$limited_banner$tooltip$zh_Hant
  tooltip =
      Translations$mini_translator$limited_banner$tooltip$zh_Hant.internal(
        _root,
      );
}

// Path: mini_translator.input
class Translations$mini_translator$input$zh_Hant
    extends Translations$mini_translator$input$en {
  Translations$mini_translator$input$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get hint => '在此輸入單字或文字';
  @override
  String get extracting_text => '正在辨識文字…';
  @override
  String hint_translate_to({required Object language}) =>
      '輸入單詞或文字，翻譯為${language}';
}

// Path: mini_translator.toolbar
class Translations$mini_translator$toolbar$zh_Hant
    extends Translations$mini_translator$toolbar$en {
  Translations$mini_translator$toolbar$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  late final Translations$mini_translator$toolbar$tooltip$zh_Hant tooltip =
      Translations$mini_translator$toolbar$tooltip$zh_Hant.internal(_root);
  @override
  late final Translations$mini_translator$toolbar$menu$zh_Hant menu =
      Translations$mini_translator$toolbar$menu$zh_Hant.internal(_root);
}

// Path: mini_translator.button
class Translations$mini_translator$button$zh_Hant
    extends Translations$mini_translator$button$en {
  Translations$mini_translator$button$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get clear => '清除';
  @override
  String get translate => '翻譯';
  @override
  String get copy => '複製';
  @override
  String get copied => '已複製';
  @override
  String get bookmark => '收藏';
  @override
  String get bookmarked => '已收藏';
}

// Path: mini_translator.language
class Translations$mini_translator$language$zh_Hant
    extends Translations$mini_translator$language$en {
  Translations$mini_translator$language$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get auto_detect => '自動偵測';
  @override
  String get auto_match => '自動匹配';
  @override
  String get switch_config => '切換目標';
  @override
  String get more_languages => '更多語言...';
  @override
  String get manage_common_languages => '管理常用語言...';
  @override
  String get manage_targets => '管理翻譯目標...';
  @override
  String get add_target => '添加翻譯目標...';
}

// Path: mini_translator.message
class Translations$mini_translator$message$zh_Hant
    extends Translations$mini_translator$message$en {
  Translations$mini_translator$message$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get please_enter_word_or_text => '未輸入或未擷取到文字';
  @override
  String get capture_screen_area_canceled => '螢幕區域擷取已取消';
  @override
  String get ocr_service_not_configured => '未配置預設文字辨識服務，請在設定中配置。';
  @override
  String get ocr_recognition_failed => '文字辨識失敗';
}

// Path: mini_translator.result
class Translations$mini_translator$result$zh_Hant
    extends Translations$mini_translator$result$en {
  Translations$mini_translator$result$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get translating => '正在翻譯…';
  @override
  String stale_requery({required Object key}) => '原文已修改 · ${key} 重新翻譯';
  @override
  String compare_services({required Object count}) => '對比 ${count} 個服務';
  @override
  String get collapse_compare => '收起對比';
  @override
  String get set_preferred => '設為首選';
  @override
  String get retry => '重試';
  @override
  String get no_result => '所有服務都沒有回傳結果 —— 檢查網路，或換一個服務再試。';
  @override
  String get no_result_note => '原文已保留，重試不會重複計入歷史。';
}

// Path: workbench.history_page
class Translations$workbench$history_page$zh_Hant
    extends Translations$workbench$history_page$en {
  Translations$workbench$history_page$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get all => '全部';
  @override
  String get favorites => '收藏';
  @override
  String get edited => '我改過的';
  @override
  String get search => '搜尋';
  @override
  String get search_placeholder => '搜尋原文、譯文或服務';
  @override
  String get search_label => '搜尋歷史';
  @override
  String entry_count({required Object label, required Object count}) =>
      '${label} · ${count} 筆';
  @override
  String get by_time => '按時間';
  @override
  String get loading => '正在載入歷史…';
  @override
  String get load_failed => '歷史載入失敗';
  @override
  String get retry => '重試';
  @override
  String get empty_title => '尚無翻譯歷史';
  @override
  String get empty_description => '完成翻譯後，首選譯文會自動儲存在這裡。';
  @override
  String no_results({required Object query}) => '找不到符合「${query}」的記錄';
  @override
  String get clear_search => '清除搜尋';
  @override
  String get select => '多選';
  @override
  String selected_count({required Object count}) => '已選 ${count} 筆';
  @override
  String get exit_select => '退出多選';
  @override
  String get add_to_glossary => '加入術語庫';
  @override
  String get favorite => '收藏';
  @override
  String get unfavorite => '取消收藏';
  @override
  String delete_confirm({required Object count}) =>
      '確定刪除選取的 ${count} 筆歷史？此操作無法復原。';
  @override
  String get no_glossary => '請先建立一個術語庫';
  @override
  String added_to_glossary({required Object count}) => '已將 ${count} 筆記錄加入術語庫';
  @override
  String get favorite_flag => '已收藏';
  @override
  String get edited_flag => '我改過';
  @override
  String get edit_history_hint => '修改後的譯文會儲存到歷史';
}

// Path: workbench.subtitle
class Translations$workbench$subtitle$zh_Hant
    extends Translations$workbench$subtitle$en {
  Translations$workbench$subtitle$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get translate => '工作台 · 多服務對照';
  @override
  String get settings => '設定';
}

// Path: workbench.placeholder
class Translations$workbench$placeholder$zh_Hant
    extends Translations$workbench$placeholder$en {
  Translations$workbench$placeholder$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get history => '查看、編輯、收藏並重用以往譯文';
  @override
  String get glossary => '管理指定譯法和禁用譯法';
}

// Path: workbench.glossary_page
class Translations$workbench$glossary_page$zh_Hant
    extends Translations$workbench$glossary_page$en {
  Translations$workbench$glossary_page$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get add_entry => '新增條目';
  @override
  String get term => '原文';
  @override
  String get translation => '指定譯法';
  @override
  String get forbidden => '禁用';
  @override
  String get hits => '命中';
  @override
  String get term_placeholder => 'teacher forcing';
  @override
  String get translation_placeholder => '教師強制';
  @override
  String get forbidden_placeholder => '強制教學';
  @override
  String get search => '搜尋';
  @override
  String get search_placeholder => '搜尋術語或指定譯法';
  @override
  String get search_label => '搜尋術語庫';
  @override
  String entry_count({required Object name, required Object count}) =>
      '${name} · ${count} 條';
  @override
  String get priority_note => '術語優先級高於任何服務輸出';
  @override
  String get new_book => '新增術語庫';
  @override
  String get new_book_placeholder => '術語庫名稱';
  @override
  String get rename_book => '重新命名';
  @override
  String delete_book_confirm({required Object name, required Object count}) =>
      '刪除「${name}」及其中的 ${count} 條術語？';
  @override
  String get disabled => '已停用';
  @override
  String get enable => '啟用';
  @override
  String get disable => '停用';
  @override
  String get empty_title => '這個術語庫還是空的';
  @override
  String get empty_description => '術語優先級高於任何服務輸出。可以逐條新增，也可以把 CSV 拖進來合併。';
  @override
  String no_results_title({required Object query}) => '沒有符合「${query}」的術語';
  @override
  String get no_results_description => '換個關鍵字，或直接新增一條。';
  @override
  String get no_books_title => '還沒有術語庫';
  @override
  String get no_books_description => '術語庫讓指定譯法在所有服務裡保持一致。先建一個，再往裡加詞。';
  @override
  String get loading => '正在載入…';
}

// Path: workbench.translation
class Translations$workbench$translation$zh_Hant
    extends Translations$workbench$translation$en {
  Translations$workbench$translation$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get source => '原文';
  @override
  String get target => '譯文';
  @override
  String get input_hint => '輸入或貼上需要翻譯的文字';
  @override
  String get button => '翻譯';
  @override
  String get auto_detected => '已自動偵測';
  @override
  String get loading_services => '正在載入翻譯服務…';
  @override
  String get no_services => '請先在設定中配置翻譯服務';
  @override
  String get translating => '正在翻譯…';
  @override
  String get failed => '翻譯失敗，請檢查服務設定後重試';
  @override
  String get empty => '譯文將顯示於此';
  @override
  String get service_compare => '服務比較';
  @override
  String get main_translation => '主譯文';
  @override
  String get service_unavailable => '服務暫不可用';
  @override
  String get waiting => '等待翻譯';
  @override
  String get copy => '複製';
  @override
  String get favorite_unavailable => '無法更新收藏狀態，請重試';
  @override
  String get preferred => '首選譯文';
  @override
  String get other_services => '其他服務';
  @override
  String get copy_result => '複製譯文';
  @override
  String get copied => '已複製';
  @override
  String get favorite => '收藏';
  @override
  String get terms => '命中術語';
  @override
  String get terms_hint => '輸入後自動比對術語庫。';
  @override
  String get quality => '品質信號';
  @override
  String get quality_hint => '譯文產生後計算。';
  @override
  String get shortcuts => '快捷鍵';
  @override
  String get other_services_disabled => '其他服務已停用';
  @override
  String input_hint_translate_to({required Object language}) =>
      '輸入或貼上要翻譯的文字，翻譯為${language}';
  @override
  String get clear => '清除';
  @override
  String get swap_languages => '交換語言';
  @override
  String get services => '翻譯服務';
  @override
  String get configure_services => '設定服務';
  @override
  String get retry => '重試';
  @override
  String character_count({required Object count}) => '${count} 個字元';
}

// Path: workbench.status
class Translations$workbench$status$zh_Hant
    extends Translations$workbench$status$en {
  Translations$workbench$status$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get runtime_ready => '翻譯執行環境已就緒';
  @override
  String get settings_synced => '設定已同步';
  @override
  String get shortcuts => '⌥Space 小窗 · ⌥⇧2 截圖';
}

// Path: settings.general
class Translations$settings$general$zh_Hant
    extends Translations$settings$general$en {
  Translations$settings$general$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get title => '一般';
  @override
  late final Translations$settings$general$section$zh_Hant section =
      Translations$settings$general$section$zh_Hant.internal(_root);
  @override
  late final Translations$settings$general$row$zh_Hant row =
      Translations$settings$general$row$zh_Hant.internal(_root);
  @override
  late final Translations$settings$general$button$zh_Hant button =
      Translations$settings$general$button$zh_Hant.internal(_root);
  @override
  late final Translations$settings$general$option$zh_Hant option =
      Translations$settings$general$option$zh_Hant.internal(_root);
  @override
  late final Translations$settings$general$editor$zh_Hant editor =
      Translations$settings$general$editor$zh_Hant.internal(_root);
}

// Path: settings.appearance
class Translations$settings$appearance$zh_Hant
    extends Translations$settings$appearance$en {
  Translations$settings$appearance$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get title => '外觀';
  @override
  late final Translations$settings$appearance$section$zh_Hant section =
      Translations$settings$appearance$section$zh_Hant.internal(_root);
  @override
  String get footer => '變更立即套用到整個視窗。';
}

// Path: settings.shortcuts
class Translations$settings$shortcuts$zh_Hant
    extends Translations$settings$shortcuts$en {
  Translations$settings$shortcuts$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get title => '快捷鍵';
  @override
  late final Translations$settings$shortcuts$section$zh_Hant section =
      Translations$settings$shortcuts$section$zh_Hant.internal(_root);
  @override
  late final Translations$settings$shortcuts$row$zh_Hant row =
      Translations$settings$shortcuts$row$zh_Hant.internal(_root);
  @override
  late final Translations$settings$shortcuts$reset_dialog$zh_Hant reset_dialog =
      Translations$settings$shortcuts$reset_dialog$zh_Hant.internal(_root);
  @override
  late final Translations$settings$shortcuts$group$zh_Hant group =
      Translations$settings$shortcuts$group$zh_Hant.internal(_root);
  @override
  String get reset => '恢復預設...';
}

// Path: settings.advanced
class Translations$settings$advanced$zh_Hant
    extends Translations$settings$advanced$en {
  Translations$settings$advanced$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get title => '進階';
  @override
  String get api_server => '本機 API 服務';
  @override
  String get api_server_description => '在 127.0.0.1 上開放翻譯 API，供本機整合使用。';
  @override
  String get enable => '啟用';
  @override
  String get port => '埠號';
  @override
  String get running_at => '執行於 {url}';
  @override
  String get disabled => '已關閉';
}

// Path: settings.services
class Translations$settings$services$zh_Hant
    extends Translations$settings$services$en {
  Translations$settings$services$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get title => '服務';
  @override
  late final Translations$settings$services$button$zh_Hant button =
      Translations$settings$services$button$zh_Hant.internal(_root);
  @override
  late final Translations$settings$services$section$zh_Hant section =
      Translations$settings$services$section$zh_Hant.internal(_root);
  @override
  late final Translations$settings$services$editor$zh_Hant editor =
      Translations$settings$services$editor$zh_Hant.internal(_root);
  @override
  late final Translations$settings$services$detail$zh_Hant detail =
      Translations$settings$services$detail$zh_Hant.internal(_root);
  @override
  String get make_default => '設為預設';
  @override
  late final Translations$settings$services$item$zh_Hant item =
      Translations$settings$services$item$zh_Hant.internal(_root);
}

// Path: settings.providers
class Translations$settings$providers$zh_Hant
    extends Translations$settings$providers$en {
  Translations$settings$providers$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get title => '提供者';
  @override
  late final Translations$settings$providers$section$zh_Hant section =
      Translations$settings$providers$section$zh_Hant.internal(_root);
  @override
  late final Translations$settings$providers$item$zh_Hant item =
      Translations$settings$providers$item$zh_Hant.internal(_root);
  @override
  late final Translations$settings$providers$button$zh_Hant button =
      Translations$settings$providers$button$zh_Hant.internal(_root);
  @override
  late final Translations$settings$providers$alert$zh_Hant alert =
      Translations$settings$providers$alert$zh_Hant.internal(_root);
  @override
  late final Translations$settings$providers$intro$zh_Hant intro =
      Translations$settings$providers$intro$zh_Hant.internal(_root);
  @override
  late final Translations$settings$providers$editor$zh_Hant editor =
      Translations$settings$providers$editor$zh_Hant.internal(_root);
  @override
  late final Translations$settings$providers$detail$zh_Hant detail =
      Translations$settings$providers$detail$zh_Hant.internal(_root);
  @override
  late final Translations$settings$providers$capability$zh_Hant capability =
      Translations$settings$providers$capability$zh_Hant.internal(_root);
  @override
  late final Translations$settings$providers$description$zh_Hant description =
      Translations$settings$providers$description$zh_Hant.internal(_root);
  @override
  late final Translations$settings$providers$delete_dialog$zh_Hant
  delete_dialog =
      Translations$settings$providers$delete_dialog$zh_Hant.internal(_root);
}

// Path: settings.layout
class Translations$settings$layout$zh_Hant
    extends Translations$settings$layout$en {
  Translations$settings$layout$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get title => '設定';
  @override
  late final Translations$settings$layout$empty$zh_Hant empty =
      Translations$settings$layout$empty$zh_Hant.internal(_root);
  @override
  String get groups => '設定分組';
  @override
  String get effect_hint => '更改即時生效';
  @override
  String get footer_note => '譯文與金鑰僅保存在本機';
  @override
  String get support => '支援';
}

// Path: settings.about
class Translations$settings$about$zh_Hant
    extends Translations$settings$about$en {
  Translations$settings$about$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get title => '關於';
  @override
  String get copy_version_info => '複製版本資訊';
  @override
  String get up_to_date => '已是最新版本。';
  @override
  String get check_again => '重新檢查';
  @override
  String get links => '連結';
  @override
  String get website => '網站';
  @override
  String get github => 'GitHub';
  @override
  String get report_issue => '回報問題';
  @override
  String get license => '授權條款';
  @override
  String get open_changelog => '查看更新日誌';
  @override
  String get update => '更新';
}

// Path: common.ui.button
class Translations$common$ui$button$zh_Hant
    extends Translations$common$ui$button$en {
  Translations$common$ui$button$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get ok => '確定';
  @override
  String get cancel => '取消';
  @override
  String get add => '新增';
  @override
  String get delete => '刪除';
  @override
  String get edit => '編輯';
  @override
  String get save => '儲存';
  @override
  String get manage => '管理';
  @override
  String get kContinue => '繼續';
}

// Path: common.ui.feedback
class Translations$common$ui$feedback$zh_Hant
    extends Translations$common$ui$feedback$en {
  Translations$common$ui$feedback$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get copied => '已複製';
}

// Path: app.tray.context_menu
class Translations$app$tray$context_menu$zh_Hant
    extends Translations$app$tray$context_menu$en {
  Translations$app$tray$context_menu$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get show_window => '顯示視窗';
  @override
  late final Translations$app$tray$context_menu$dev_tools$zh_Hant dev_tools =
      Translations$app$tray$context_menu$dev_tools$zh_Hant.internal(_root);
  @override
  String get check_for_updates => '檢查更新';
  @override
  String get settings => '設定';
  @override
  String get quit => '結束';
}

// Path: mini_translator.limited_banner.permission
class Translations$mini_translator$limited_banner$permission$zh_Hant
    extends Translations$mini_translator$limited_banner$permission$en {
  Translations$mini_translator$limited_banner$permission$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get missing_both => '請授予螢幕錄製和輔助功能權限以啟用完整功能。';
  @override
  String get missing_screen_capture => '請授予螢幕錄製權限以啟用完整功能。';
  @override
  String get missing_accessibility => '請授予輔助功能權限以啟用完整功能。';
}

// Path: mini_translator.limited_banner.instruction
class Translations$mini_translator$limited_banner$instruction$zh_Hant
    extends Translations$mini_translator$limited_banner$instruction$en {
  Translations$mini_translator$limited_banner$instruction$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get app_settings_prefix => '請前往';
  @override
  String get follow_guide_prefix => '，依指引授權後點選';
  @override
  String get suffix => '。';
}

// Path: mini_translator.limited_banner.action
class Translations$mini_translator$limited_banner$action$zh_Hant
    extends Translations$mini_translator$limited_banner$action$en {
  Translations$mini_translator$limited_banner$action$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get app_settings => '應用程式設定';
  @override
  String get recheck => '重新檢查';
}

// Path: mini_translator.limited_banner.feedback
class Translations$mini_translator$limited_banner$feedback$zh_Hant
    extends Translations$mini_translator$limited_banner$feedback$en {
  Translations$mini_translator$limited_banner$feedback$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get enabled => '螢幕取詞功能已啟用。';
  @override
  String get still_missing => '仍缺少所需權限。\n請檢查您的設定後再試。';
}

// Path: mini_translator.limited_banner.tooltip
class Translations$mini_translator$limited_banner$tooltip$zh_Hant
    extends Translations$mini_translator$limited_banner$tooltip$en {
  Translations$mini_translator$limited_banner$tooltip$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get help => '檢視說明';
}

// Path: mini_translator.toolbar.tooltip
class Translations$mini_translator$toolbar$tooltip$zh_Hant
    extends Translations$mini_translator$toolbar$tooltip$en {
  Translations$mini_translator$toolbar$tooltip$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get extract_text_from_screen_capture => '擷取螢幕區域並辨識文字';
  @override
  String get extract_text_from_clipboard => '讀取剪貼簿內容';
  @override
  String get pin => '固定視窗';
  @override
  String get more_actions => '更多操作';
}

// Path: mini_translator.toolbar.menu
class Translations$mini_translator$toolbar$menu$zh_Hant
    extends Translations$mini_translator$toolbar$menu$en {
  Translations$mini_translator$toolbar$menu$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get extract_from_screen_capture => '螢幕取詞';
  @override
  String get extract_from_clipboard => '剪貼簿取詞';
  @override
  String get open_main_window => '開啟主視窗';
  @override
  String get open_settings => '設定…';
}

// Path: settings.general.section
class Translations$settings$general$section$zh_Hant
    extends Translations$settings$general$section$en {
  Translations$settings$general$section$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get permissions => '系統權限';
  @override
  String get ocr => '文字辨識';
  @override
  String get directory => '辭典';
  @override
  String get translation => '翻譯';
  @override
  String get translation_target => '翻譯目標';
  @override
  String get languages => '語言';
  @override
  String get input => '輸入設定';
  @override
  String get startup => '啟動與整合';
  @override
  String get ocr_behaviour => '取詞行為';
  @override
  String get translation_behaviour => '翻譯行為';
}

// Path: settings.general.row
class Translations$settings$general$row$zh_Hant
    extends Translations$settings$general$row$en {
  Translations$settings$general$row$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get launch_at_login => '登入時啟動';
  @override
  String get show_in_menu_bar => '在選單列中顯示';
  @override
  String get screen_capture_access => '授予螢幕錄製權限';
  @override
  String get screen_selection_access => '授予輔助功能權限';
  @override
  String get default_ocr_service => '預設文字辨識服務';
  @override
  String get auto_copy_detected_text => '自動複製偵測到的文字';
  @override
  String get default_directory_service => '預設辭典服務';
  @override
  String get default_translation_service => '預設翻譯服務';
  @override
  String get translation_target_hint => '設定翻譯器使用的語言目標。';
  @override
  String get common_languages => '常用語言';
  @override
  String get common_languages_description => '顯示在語言選擇列表頂部。';
  @override
  String get common_languages_hint => '選擇你常用的語言：';
  @override
  String get common_languages_sort => '按代碼排序';
  @override
  String get common_languages_reset => '恢復預設';
  @override
  String get common_languages_reset_help => '恢復為預設的常用語言集合';
  @override
  String get common_languages_search => '搜索語言...';
  @override
  String get common_languages_all => '所有語言';
  @override
  String get double_click_copy_result => '雙擊複製翻譯結果';
  @override
  String get submit_with_enter => '按 Enter 提交';
  @override
  String get submit_with_meta_enter_mac => '按 ⌘ + Enter 提交';
  @override
  String get screen_capture_access_hint => '截圖取詞需要讀取螢幕內容。';
  @override
  String get screen_selection_access_hint => '劃詞取詞需要讀取其他應用程式中選取的文字。';
  @override
  String get no_translation_targets => '還沒有翻譯目標，新增一條來決定預設譯入哪種語言。';
}

// Path: settings.general.button
class Translations$settings$general$button$zh_Hant
    extends Translations$settings$general$button$en {
  Translations$settings$general$button$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get add_provider => '新增…';
  @override
  String get add_target => '新增目標...';
  @override
  String get manage_targets => '管理翻譯目標...';
  @override
  String get manage_languages => '管理常用語言...';
  @override
  String get grant => '授權';
}

// Path: settings.general.option
class Translations$settings$general$option$zh_Hant
    extends Translations$settings$general$option$en {
  Translations$settings$general$option$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get none => '無';
  @override
  String get no_services_available => '暫無可用服務';
  @override
  String get granted => '已授權';
  @override
  String get built_in_ocr => '內建 OCR';
  @override
  String get tesseract => 'Tesseract';
  @override
  String get youdao_ocr => '有道 OCR';
}

// Path: settings.general.editor
class Translations$settings$general$editor$zh_Hant
    extends Translations$settings$general$editor$en {
  Translations$settings$general$editor$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get add_target_title => '添加翻譯目標：';
  @override
  String get edit_target_title => '修改翻譯目標：';
  @override
  late final Translations$settings$general$editor$row$zh_Hant row =
      Translations$settings$general$editor$row$zh_Hant.internal(_root);
  @override
  String get title_edit => '編輯翻譯目標';
  @override
  String get subtitle => '決定某種來源語言預設翻譯成哪種語言';
  @override
  String get same_language => '來源語言與目標語言相同，換一個目標語言。';
  @override
  String get duplicate => '已經有一條同樣的翻譯目標了。';
  @override
  String get hint_auto => '未符合其他規則時，一律譯成{}。';
  @override
  String get hint_source => '偵測到{}時，譯成{}。';
}

// Path: settings.appearance.section
class Translations$settings$appearance$section$zh_Hant
    extends Translations$settings$appearance$section$en {
  Translations$settings$appearance$section$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get app_language => '顯示語言';
  @override
  String get theme_mode => '主題模式';
  @override
  String get theme_style => '主題風格';
}

// Path: settings.shortcuts.section
class Translations$settings$shortcuts$section$zh_Hant
    extends Translations$settings$shortcuts$section$en {
  Translations$settings$shortcuts$section$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get text_extraction => '文字擷取';
  @override
  String get input_assist => '輸入輔助功能';
  @override
  String get submit_mode => '提交方式';
}

// Path: settings.shortcuts.row
class Translations$settings$shortcuts$row$zh_Hant
    extends Translations$settings$shortcuts$row$en {
  Translations$settings$shortcuts$row$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get toggle_mini_translator => '顯示/隱藏視窗';
  @override
  String get extract_text_from_screen_selection => '從螢幕選取範圍擷取文字';
  @override
  String get extract_text_from_screen_capture => '從螢幕截圖擷取文字';
  @override
  String get extract_text_from_clipboard => '從剪貼簿擷取文字';
  @override
  String get translate_input => '翻譯輸入內容';
}

// Path: settings.shortcuts.reset_dialog
class Translations$settings$shortcuts$reset_dialog$zh_Hant
    extends Translations$settings$shortcuts$reset_dialog$en {
  Translations$settings$shortcuts$reset_dialog$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get title => '重設快捷鍵';
  @override
  String get message => '確定要重設所有快捷鍵為預設值嗎？';
  @override
  String get confirm => '重設';
  @override
  String get cancel => '取消';
}

// Path: settings.shortcuts.group
class Translations$settings$shortcuts$group$zh_Hant
    extends Translations$settings$shortcuts$group$en {
  Translations$settings$shortcuts$group$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  late final Translations$settings$shortcuts$group$global$zh_Hant global =
      Translations$settings$shortcuts$group$global$zh_Hant.internal(_root);
  @override
  late final Translations$settings$shortcuts$group$in_app$zh_Hant in_app =
      Translations$settings$shortcuts$group$in_app$zh_Hant.internal(_root);
}

// Path: settings.services.button
class Translations$settings$services$button$zh_Hant
    extends Translations$settings$services$button$en {
  Translations$settings$services$button$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get add_service => '新增服務...';
}

// Path: settings.services.section
class Translations$settings$services$section$zh_Hant
    extends Translations$settings$services$section$en {
  Translations$settings$services$section$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get available_services => '可用服務';
}

// Path: settings.services.editor
class Translations$settings$services$editor$zh_Hant
    extends Translations$settings$services$editor$en {
  Translations$settings$services$editor$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get title => '新增服務';
  @override
  String get subtitle => '為已設定的提供者新增一項服務';
  @override
  late final Translations$settings$services$editor$row$zh_Hant row =
      Translations$settings$services$editor$row$zh_Hant.internal(_root);
  @override
  String get prompt_placeholder => '留空則使用該類型的預設提示詞';
  @override
  String get variant_hint => '{} 已有一項{}服務，這條會作為並列的另一份設定。';
  @override
  String get traditional_note => '{} 是傳統介面，沒有模型與提示詞可調；服務參數在提供者詳情頁設定。';
}

// Path: settings.services.detail
class Translations$settings$services$detail$zh_Hant
    extends Translations$settings$services$detail$en {
  Translations$settings$services$detail$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  late final Translations$settings$services$detail$row$zh_Hant row =
      Translations$settings$services$detail$row$zh_Hant.internal(_root);
  @override
  late final Translations$settings$services$detail$delete_dialog$zh_Hant
  delete_dialog =
      Translations$settings$services$detail$delete_dialog$zh_Hant.internal(
        _root,
      );
  @override
  String get prompt_variables =>
      '可用變數：{{sourceLanguage}}、{{targetLanguage}}、{{text}}';
}

// Path: settings.services.item
class Translations$settings$services$item$zh_Hant
    extends Translations$settings$services$item$en {
  Translations$settings$services$item$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get none_of_kind => '還沒有可用的{}服務。';
}

// Path: settings.providers.section
class Translations$settings$providers$section$zh_Hant
    extends Translations$settings$providers$section$en {
  Translations$settings$providers$section$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get services => '可用服務';
  @override
  String get services_description => '查看已設定提供商的可用服務，並依服務類型切換。';
}

// Path: settings.providers.item
class Translations$settings$providers$item$zh_Hant
    extends Translations$settings$providers$item$en {
  Translations$settings$providers$item$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get empty => '尚未設定任何提供者。新增一個提供者以啟用翻譯服務。';
  @override
  String get loading => '正在載入提供者…';
  @override
  String get no_services => '暫無可用服務。';
}

// Path: settings.providers.button
class Translations$settings$providers$button$zh_Hant
    extends Translations$settings$providers$button$en {
  Translations$settings$providers$button$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get add => '新增提供者…';
}

// Path: settings.providers.alert
class Translations$settings$providers$alert$zh_Hant
    extends Translations$settings$providers$alert$en {
  Translations$settings$providers$alert$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get error => '錯誤';
}

// Path: settings.providers.intro
class Translations$settings$providers$intro$zh_Hant
    extends Translations$settings$providers$intro$en {
  Translations$settings$providers$intro$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get body => '管理應用程式使用的服務提供商。';
  @override
  String get warning => '已連線的提供商可能會處理您傳送的文字或圖片，請僅啟用您信任的服務。';
}

// Path: settings.providers.editor
class Translations$settings$providers$editor$zh_Hant
    extends Translations$settings$providers$editor$en {
  Translations$settings$providers$editor$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  late final Translations$settings$providers$editor$row$zh_Hant row =
      Translations$settings$providers$editor$row$zh_Hant.internal(_root);
  @override
  late final Translations$settings$providers$editor$placeholder$zh_Hant
  placeholder =
      Translations$settings$providers$editor$placeholder$zh_Hant.internal(
        _root,
      );
  @override
  late final Translations$settings$providers$editor$type_picker$zh_Hant
  type_picker =
      Translations$settings$providers$editor$type_picker$zh_Hant.internal(
        _root,
      );
  @override
  late final Translations$settings$providers$editor$tooltip$zh_Hant tooltip =
      Translations$settings$providers$editor$tooltip$zh_Hant.internal(_root);
  @override
  late final Translations$settings$providers$editor$step$zh_Hant step =
      Translations$settings$providers$editor$step$zh_Hant.internal(_root);
  @override
  String get add_title => '新增 {}';
  @override
  late final Translations$settings$providers$editor$capability_note$zh_Hant
  capability_note =
      Translations$settings$providers$editor$capability_note$zh_Hant.internal(
        _root,
      );
  @override
  late final Translations$settings$providers$editor$test$zh_Hant test =
      Translations$settings$providers$editor$test$zh_Hant.internal(_root);
}

// Path: settings.providers.detail
class Translations$settings$providers$detail$zh_Hant
    extends Translations$settings$providers$detail$en {
  Translations$settings$providers$detail$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  late final Translations$settings$providers$detail$tooltip$zh_Hant tooltip =
      Translations$settings$providers$detail$tooltip$zh_Hant.internal(_root);
  @override
  late final Translations$settings$providers$detail$row$zh_Hant row =
      Translations$settings$providers$detail$row$zh_Hant.internal(_root);
  @override
  late final Translations$settings$providers$detail$section$zh_Hant section =
      Translations$settings$providers$detail$section$zh_Hant.internal(_root);
  @override
  late final Translations$settings$providers$detail$models$zh_Hant models =
      Translations$settings$providers$detail$models$zh_Hant.internal(_root);
}

// Path: settings.providers.capability
class Translations$settings$providers$capability$zh_Hant
    extends Translations$settings$providers$capability$en {
  Translations$settings$providers$capability$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get translation => '翻譯';
  @override
  String get dictionary => '辭典';
  @override
  String get ocr => 'OCR';
  @override
  String get llm => 'AI';
}

// Path: settings.providers.description
class Translations$settings$providers$description$zh_Hant
    extends Translations$settings$providers$description$en {
  Translations$settings$providers$description$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get all => '提供辭典查詢和文字翻譯';
  @override
  String get translation => '提供語言間文字翻譯';
  @override
  String get dictionary => '提供辭典查詢和單字釋義';
  @override
  String get fallback => '提供翻譯服務';
}

// Path: settings.providers.delete_dialog
class Translations$settings$providers$delete_dialog$zh_Hant
    extends Translations$settings$providers$delete_dialog$en {
  Translations$settings$providers$delete_dialog$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get title => '刪除「{}」？';
  @override
  String get message => '此操作無法復原。';
}

// Path: settings.layout.empty
class Translations$settings$layout$empty$zh_Hant
    extends Translations$settings$layout$empty$en {
  Translations$settings$layout$empty$zh_Hant.internal(TranslationsZhHant root)
    : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get title => '選擇一個分類';
  @override
  String get message => '從側邊欄選擇一個設定分類。';
}

// Path: app.tray.context_menu.dev_tools
class Translations$app$tray$context_menu$dev_tools$zh_Hant
    extends Translations$app$tray$context_menu$dev_tools$en {
  Translations$app$tray$context_menu$dev_tools$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get title => '開發工具';
  @override
  String get open_data_directory => '開啟資料目錄';
}

// Path: settings.general.editor.row
class Translations$settings$general$editor$row$zh_Hant
    extends Translations$settings$general$editor$row$en {
  Translations$settings$general$editor$row$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get source_language => '源語言';
  @override
  String get target_language => '目標語言';
}

// Path: settings.shortcuts.group.global
class Translations$settings$shortcuts$group$global$zh_Hant
    extends Translations$settings$shortcuts$group$global$en {
  Translations$settings$shortcuts$group$global$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get title => '全域快速鍵';
  @override
  String get description => '在任何應用程式裡都能觸發。';
}

// Path: settings.shortcuts.group.in_app
class Translations$settings$shortcuts$group$in_app$zh_Hant
    extends Translations$settings$shortcuts$group$in_app$en {
  Translations$settings$shortcuts$group$in_app$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get title => '應用程式內按鍵';
  @override
  String get description => '僅在應用程式自己的輸入框內生效。';
}

// Path: settings.services.editor.row
class Translations$settings$services$editor$row$zh_Hant
    extends Translations$settings$services$editor$row$en {
  Translations$settings$services$editor$row$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get model => '模型';
  @override
  String get system_prompt => '系統提示詞';
}

// Path: settings.services.detail.row
class Translations$settings$services$detail$row$zh_Hant
    extends Translations$settings$services$detail$row$en {
  Translations$settings$services$detail$row$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get id => '服務 ID';
  @override
  String get name => '名稱';
  @override
  String get provider => '提供者';
  @override
  String get type => '類型';
}

// Path: settings.services.detail.delete_dialog
class Translations$settings$services$detail$delete_dialog$zh_Hant
    extends Translations$settings$services$detail$delete_dialog$en {
  Translations$settings$services$detail$delete_dialog$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get title => '刪除「{}」？';
  @override
  String get message => '此服務將從提供者中移除。';
}

// Path: settings.providers.editor.row
class Translations$settings$providers$editor$row$zh_Hant
    extends Translations$settings$providers$editor$row$en {
  Translations$settings$providers$editor$row$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get id => '提供者 ID';
  @override
  String get type => '提供者類型';
  @override
  String get default_model => '預設模型';
}

// Path: settings.providers.editor.placeholder
class Translations$settings$providers$editor$placeholder$zh_Hant
    extends Translations$settings$providers$editor$placeholder$en {
  Translations$settings$providers$editor$placeholder$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get id => '例如 deepl-main';
}

// Path: settings.providers.editor.type_picker
class Translations$settings$providers$editor$type_picker$zh_Hant
    extends Translations$settings$providers$editor$type_picker$en {
  Translations$settings$providers$editor$type_picker$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get prompt => '請選擇要新增的提供者類型：';
  @override
  String get section_llm => 'LLM';
  @override
  String get section_traditional => '傳統';
}

// Path: settings.providers.editor.tooltip
class Translations$settings$providers$editor$tooltip$zh_Hant
    extends Translations$settings$providers$editor$tooltip$en {
  Translations$settings$providers$editor$tooltip$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get help => '說明';
}

// Path: settings.providers.editor.step
class Translations$settings$providers$editor$step$zh_Hant
    extends Translations$settings$providers$editor$step$en {
  Translations$settings$providers$editor$step$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get next => '繼續';
  @override
  String get back => '上一步';
}

// Path: settings.providers.editor.capability_note
class Translations$settings$providers$editor$capability_note$zh_Hant
    extends Translations$settings$providers$editor$capability_note$en {
  Translations$settings$providers$editor$capability_note$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get translation => '加入翻譯候選';
  @override
  String get dictionary => '提供辭典釋義';
  @override
  String get ocr => '辨識圖片中的文字';
}

// Path: settings.providers.editor.test
class Translations$settings$providers$editor$test$zh_Hant
    extends Translations$settings$providers$editor$test$en {
  Translations$settings$providers$editor$test$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get run => '測試連線';
  @override
  String get running => '正在測試連線 · 已用 {}s';
  @override
  String get passed_models => '連線正常 · {} 個模型可用';
  @override
  String get passed_service => '連線正常 · 服務可用';
  @override
  String get passed_footer => '已通過連線測試';
  @override
  String get retest => '重新測試';
  @override
  String get tips_title => '可以試試';
  @override
  String get tips_llm =>
      '· 確認金鑰與所選提供者類型一致\n· 檢查 Base URL 是否需要包含 /v1\n· 在提供者主控台確認該模型已開通';
  @override
  String get tips_traditional => '· 確認金鑰與所選提供者類型一致\n· 在提供者主控台確認服務已開通';
  @override
  String get failed_suffix => '驗證失敗';
  @override
  String get passed_suffix => '已驗證';
}

// Path: settings.providers.detail.tooltip
class Translations$settings$providers$detail$tooltip$zh_Hant
    extends Translations$settings$providers$detail$tooltip$en {
  Translations$settings$providers$detail$tooltip$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get edit => '編輯提供者';
}

// Path: settings.providers.detail.row
class Translations$settings$providers$detail$row$zh_Hant
    extends Translations$settings$providers$detail$row$en {
  Translations$settings$providers$detail$row$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get id_hint => '建立後不可變更';
}

// Path: settings.providers.detail.section
class Translations$settings$providers$detail$section$zh_Hant
    extends Translations$settings$providers$detail$section$en {
  Translations$settings$providers$detail$section$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get configuration => '配置';
  @override
  String get models => '模型';
}

// Path: settings.providers.detail.models
class Translations$settings$providers$detail$models$zh_Hant
    extends Translations$settings$providers$detail$models$en {
  Translations$settings$providers$detail$models$zh_Hant.internal(
    TranslationsZhHant root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsZhHant _root; // ignore: unused_field

  // Translations
  @override
  String get loading => '正在載入模型...';
  @override
  String get empty => '找不到模型。';
  @override
  String get retry => '重試';
  @override
  String get refresh => '重新整理清單';
  @override
  String get default_badge => '預設';
  @override
  String get set_default => '設為預設';
  @override
  String get fetch_error => '無法從提供者 API 取得模型。';
}

/// The flat map containing all translations for locale <zh-Hant>.
/// Only for edge cases! For simple maps, use the map function of this library.
///
/// The Dart AOT compiler has issues with very large switch statements,
/// so the map is split into smaller functions (512 entries each).
extension on TranslationsZhHant {
  dynamic _flatMapFunction(String path) {
    return switch (path) {
      'common.ui.button.ok' => '確定',
      'common.ui.button.cancel' => '取消',
      'common.ui.button.add' => '新增',
      'common.ui.button.delete' => '刪除',
      'common.ui.button.edit' => '編輯',
      'common.ui.button.save' => '儲存',
      'common.ui.button.manage' => '管理',
      'common.ui.button.kContinue' => '繼續',
      'common.ui.feedback.copied' => '已複製',
      'common.language.ar' => '阿拉伯語',
      'common.language.bn' => '孟加拉語',
      'common.language.de' => '德語',
      'common.language.en' => '英語',
      'common.language.es' => '西班牙語',
      'common.language.fa' => '波斯語',
      'common.language.fr' => '法語',
      'common.language.gu' => '古吉拉特語',
      'common.language.ha' => '豪薩語',
      'common.language.hi' => '印地語',
      'common.language.id' => '印尼語',
      'common.language.it' => '義大利語',
      'common.language.ja' => '日語',
      'common.language.jv' => '爪哇語',
      'common.language.ko' => '韓語',
      'common.language.ml' => '馬拉雅拉姆語',
      'common.language.mr' => '馬拉地語',
      'common.language.ms' => '馬來語',
      'common.language.nl' => '荷蘭語',
      'common.language.pa' => '旁遮普語',
      'common.language.pl' => '波蘭語',
      'common.language.pt' => '葡萄牙語',
      'common.language.ro' => '羅馬尼亞語',
      'common.language.ru' => '俄語',
      'common.language.sw' => '斯瓦希里語',
      'common.language.ta' => '泰米爾語',
      'common.language.te' => '泰盧固語',
      'common.language.th' => '泰語',
      'common.language.tr' => '土耳其語',
      'common.language.uk' => '烏克蘭語',
      'common.language.ur' => '烏爾都語',
      'common.language.vi' => '越南語',
      'common.language.yo' => '約魯巴語',
      'common.language.zh_hans' => '中文（簡體）',
      'common.language.zh_hant' => '中文（繁體）',
      'common.theme_mode.light' => '淺色',
      'common.theme_mode.dark' => '深色',
      'common.theme_mode.system' => '跟隨系統',
      'common.theme_style.studio' => 'Studio',
      'common.theme_style.bright' => 'Bright',
      'common.provider.anthropic' => 'Anthropic',
      'common.provider.baidu' => '百度',
      'common.provider.caiyun' => '彩雲小譯',
      'common.provider.deepl' => 'DeepL',
      'common.provider.google' => '谷歌',
      'common.provider.ollama' => 'Ollama',
      'common.provider.openai' => 'OpenAI',
      'common.provider.sogou' => '搜狗',
      'common.provider.xai' => 'xAI',
      'common.provider.system' => '系統',
      'common.provider.tencent' => '騰訊',
      'common.provider.youdao' => '有道',
      'app.tray.context_menu.show_window' => '顯示視窗',
      'app.tray.context_menu.dev_tools.title' => '開發工具',
      'app.tray.context_menu.dev_tools.open_data_directory' => '開啟資料目錄',
      'app.tray.context_menu.check_for_updates' => '檢查更新',
      'app.tray.context_menu.settings' => '設定',
      'app.tray.context_menu.quit' => '結束',
      'mini_translator.limited_banner.permission.missing_both' =>
        '請授予螢幕錄製和輔助功能權限以啟用完整功能。',
      'mini_translator.limited_banner.permission.missing_screen_capture' =>
        '請授予螢幕錄製權限以啟用完整功能。',
      'mini_translator.limited_banner.permission.missing_accessibility' =>
        '請授予輔助功能權限以啟用完整功能。',
      'mini_translator.limited_banner.instruction.app_settings_prefix' => '請前往',
      'mini_translator.limited_banner.instruction.follow_guide_prefix' =>
        '，依指引授權後點選',
      'mini_translator.limited_banner.instruction.suffix' => '。',
      'mini_translator.limited_banner.action.app_settings' => '應用程式設定',
      'mini_translator.limited_banner.action.recheck' => '重新檢查',
      'mini_translator.limited_banner.feedback.enabled' => '螢幕取詞功能已啟用。',
      'mini_translator.limited_banner.feedback.still_missing' =>
        '仍缺少所需權限。\n請檢查您的設定後再試。',
      'mini_translator.limited_banner.tooltip.help' => '檢視說明',
      'mini_translator.input.hint' => '在此輸入單字或文字',
      'mini_translator.input.extracting_text' => '正在辨識文字…',
      'mini_translator.input.hint_translate_to' => ({
        required Object language,
      }) => '輸入單詞或文字，翻譯為${language}',
      'mini_translator.toolbar.tooltip.extract_text_from_screen_capture' =>
        '擷取螢幕區域並辨識文字',
      'mini_translator.toolbar.tooltip.extract_text_from_clipboard' =>
        '讀取剪貼簿內容',
      'mini_translator.toolbar.tooltip.pin' => '固定視窗',
      'mini_translator.toolbar.tooltip.more_actions' => '更多操作',
      'mini_translator.toolbar.menu.extract_from_screen_capture' => '螢幕取詞',
      'mini_translator.toolbar.menu.extract_from_clipboard' => '剪貼簿取詞',
      'mini_translator.toolbar.menu.open_main_window' => '開啟主視窗',
      'mini_translator.toolbar.menu.open_settings' => '設定…',
      'mini_translator.button.clear' => '清除',
      'mini_translator.button.translate' => '翻譯',
      'mini_translator.button.copy' => '複製',
      'mini_translator.button.copied' => '已複製',
      'mini_translator.button.bookmark' => '收藏',
      'mini_translator.button.bookmarked' => '已收藏',
      'mini_translator.language.auto_detect' => '自動偵測',
      'mini_translator.language.auto_match' => '自動匹配',
      'mini_translator.language.switch_config' => '切換目標',
      'mini_translator.language.more_languages' => '更多語言...',
      'mini_translator.language.manage_common_languages' => '管理常用語言...',
      'mini_translator.language.manage_targets' => '管理翻譯目標...',
      'mini_translator.language.add_target' => '添加翻譯目標...',
      'mini_translator.message.please_enter_word_or_text' => '未輸入或未擷取到文字',
      'mini_translator.message.capture_screen_area_canceled' => '螢幕區域擷取已取消',
      'mini_translator.message.ocr_service_not_configured' =>
        '未配置預設文字辨識服務，請在設定中配置。',
      'mini_translator.message.ocr_recognition_failed' => '文字辨識失敗',
      'mini_translator.result.translating' => '正在翻譯…',
      'mini_translator.result.stale_requery' => ({
        required Object key,
      }) => '原文已修改 · ${key} 重新翻譯',
      'mini_translator.result.compare_services' => ({
        required Object count,
      }) => '對比 ${count} 個服務',
      'mini_translator.result.collapse_compare' => '收起對比',
      'mini_translator.result.set_preferred' => '設為首選',
      'mini_translator.result.retry' => '重試',
      'mini_translator.result.no_result' => '所有服務都沒有回傳結果 —— 檢查網路，或換一個服務再試。',
      'mini_translator.result.no_result_note' => '原文已保留，重試不會重複計入歷史。',
      'workbench.workspace' => '工作區',
      'workbench.translate' => '翻譯',
      'workbench.history' => '歷史',
      'workbench.history_page.all' => '全部',
      'workbench.history_page.favorites' => '收藏',
      'workbench.history_page.edited' => '我改過的',
      'workbench.history_page.search' => '搜尋',
      'workbench.history_page.search_placeholder' => '搜尋原文、譯文或服務',
      'workbench.history_page.search_label' => '搜尋歷史',
      'workbench.history_page.entry_count' => ({
        required Object label,
        required Object count,
      }) => '${label} · ${count} 筆',
      'workbench.history_page.by_time' => '按時間',
      'workbench.history_page.loading' => '正在載入歷史…',
      'workbench.history_page.load_failed' => '歷史載入失敗',
      'workbench.history_page.retry' => '重試',
      'workbench.history_page.empty_title' => '尚無翻譯歷史',
      'workbench.history_page.empty_description' => '完成翻譯後，首選譯文會自動儲存在這裡。',
      'workbench.history_page.no_results' => ({
        required Object query,
      }) => '找不到符合「${query}」的記錄',
      'workbench.history_page.clear_search' => '清除搜尋',
      'workbench.history_page.select' => '多選',
      'workbench.history_page.selected_count' => ({
        required Object count,
      }) => '已選 ${count} 筆',
      'workbench.history_page.exit_select' => '退出多選',
      'workbench.history_page.add_to_glossary' => '加入術語庫',
      'workbench.history_page.favorite' => '收藏',
      'workbench.history_page.unfavorite' => '取消收藏',
      'workbench.history_page.delete_confirm' => ({
        required Object count,
      }) => '確定刪除選取的 ${count} 筆歷史？此操作無法復原。',
      'workbench.history_page.no_glossary' => '請先建立一個術語庫',
      'workbench.history_page.added_to_glossary' => ({
        required Object count,
      }) => '已將 ${count} 筆記錄加入術語庫',
      'workbench.history_page.favorite_flag' => '已收藏',
      'workbench.history_page.edited_flag' => '我改過',
      'workbench.history_page.edit_history_hint' => '修改後的譯文會儲存到歷史',
      'workbench.glossary' => '術語庫',
      'workbench.recent_languages' => '最近語言',
      'workbench.not_configured' => '尚未設定',
      'workbench.subtitle.translate' => '工作台 · 多服務對照',
      'workbench.subtitle.settings' => '設定',
      'workbench.placeholder.history' => '查看、編輯、收藏並重用以往譯文',
      'workbench.placeholder.glossary' => '管理指定譯法和禁用譯法',
      'workbench.glossary_page.add_entry' => '新增條目',
      'workbench.glossary_page.term' => '原文',
      'workbench.glossary_page.translation' => '指定譯法',
      'workbench.glossary_page.forbidden' => '禁用',
      'workbench.glossary_page.hits' => '命中',
      'workbench.glossary_page.term_placeholder' => 'teacher forcing',
      'workbench.glossary_page.translation_placeholder' => '教師強制',
      'workbench.glossary_page.forbidden_placeholder' => '強制教學',
      'workbench.glossary_page.search' => '搜尋',
      'workbench.glossary_page.search_placeholder' => '搜尋術語或指定譯法',
      'workbench.glossary_page.search_label' => '搜尋術語庫',
      'workbench.glossary_page.entry_count' => ({
        required Object name,
        required Object count,
      }) => '${name} · ${count} 條',
      'workbench.glossary_page.priority_note' => '術語優先級高於任何服務輸出',
      'workbench.glossary_page.new_book' => '新增術語庫',
      'workbench.glossary_page.new_book_placeholder' => '術語庫名稱',
      'workbench.glossary_page.rename_book' => '重新命名',
      'workbench.glossary_page.delete_book_confirm' => ({
        required Object name,
        required Object count,
      }) => '刪除「${name}」及其中的 ${count} 條術語？',
      'workbench.glossary_page.disabled' => '已停用',
      'workbench.glossary_page.enable' => '啟用',
      'workbench.glossary_page.disable' => '停用',
      'workbench.glossary_page.empty_title' => '這個術語庫還是空的',
      'workbench.glossary_page.empty_description' =>
        '術語優先級高於任何服務輸出。可以逐條新增，也可以把 CSV 拖進來合併。',
      'workbench.glossary_page.no_results_title' => ({
        required Object query,
      }) => '沒有符合「${query}」的術語',
      'workbench.glossary_page.no_results_description' => '換個關鍵字，或直接新增一條。',
      'workbench.glossary_page.no_books_title' => '還沒有術語庫',
      'workbench.glossary_page.no_books_description' =>
        '術語庫讓指定譯法在所有服務裡保持一致。先建一個，再往裡加詞。',
      'workbench.glossary_page.loading' => '正在載入…',
      'workbench.translation.source' => '原文',
      'workbench.translation.target' => '譯文',
      'workbench.translation.input_hint' => '輸入或貼上需要翻譯的文字',
      'workbench.translation.button' => '翻譯',
      'workbench.translation.auto_detected' => '已自動偵測',
      'workbench.translation.loading_services' => '正在載入翻譯服務…',
      'workbench.translation.no_services' => '請先在設定中配置翻譯服務',
      'workbench.translation.translating' => '正在翻譯…',
      'workbench.translation.failed' => '翻譯失敗，請檢查服務設定後重試',
      'workbench.translation.empty' => '譯文將顯示於此',
      'workbench.translation.service_compare' => '服務比較',
      'workbench.translation.main_translation' => '主譯文',
      'workbench.translation.service_unavailable' => '服務暫不可用',
      'workbench.translation.waiting' => '等待翻譯',
      'workbench.translation.copy' => '複製',
      'workbench.translation.favorite_unavailable' => '無法更新收藏狀態，請重試',
      'workbench.translation.preferred' => '首選譯文',
      'workbench.translation.other_services' => '其他服務',
      'workbench.translation.copy_result' => '複製譯文',
      'workbench.translation.copied' => '已複製',
      'workbench.translation.favorite' => '收藏',
      'workbench.translation.terms' => '命中術語',
      'workbench.translation.terms_hint' => '輸入後自動比對術語庫。',
      'workbench.translation.quality' => '品質信號',
      'workbench.translation.quality_hint' => '譯文產生後計算。',
      'workbench.translation.shortcuts' => '快捷鍵',
      'workbench.translation.other_services_disabled' => '其他服務已停用',
      'workbench.translation.input_hint_translate_to' => ({
        required Object language,
      }) => '輸入或貼上要翻譯的文字，翻譯為${language}',
      'workbench.translation.clear' => '清除',
      'workbench.translation.swap_languages' => '交換語言',
      'workbench.translation.services' => '翻譯服務',
      'workbench.translation.configure_services' => '設定服務',
      'workbench.translation.retry' => '重試',
      'workbench.translation.character_count' => ({
        required Object count,
      }) => '${count} 個字元',
      'workbench.status.runtime_ready' => '翻譯執行環境已就緒',
      'workbench.status.settings_synced' => '設定已同步',
      'workbench.status.shortcuts' => '⌥Space 小窗 · ⌥⇧2 截圖',
      'workbench.version_latest' => '已是最新',
      'workbench.version_checking' => '正在檢查…',
      'workbench.check_updates' => '檢查更新',
      'settings.version' => 'v{} (Build {})',
      'settings.general.title' => '一般',
      'settings.general.section.permissions' => '系統權限',
      'settings.general.section.ocr' => '文字辨識',
      'settings.general.section.directory' => '辭典',
      'settings.general.section.translation' => '翻譯',
      'settings.general.section.translation_target' => '翻譯目標',
      'settings.general.section.languages' => '語言',
      'settings.general.section.input' => '輸入設定',
      'settings.general.section.startup' => '啟動與整合',
      'settings.general.section.ocr_behaviour' => '取詞行為',
      'settings.general.section.translation_behaviour' => '翻譯行為',
      'settings.general.row.launch_at_login' => '登入時啟動',
      'settings.general.row.show_in_menu_bar' => '在選單列中顯示',
      'settings.general.row.screen_capture_access' => '授予螢幕錄製權限',
      'settings.general.row.screen_selection_access' => '授予輔助功能權限',
      'settings.general.row.default_ocr_service' => '預設文字辨識服務',
      'settings.general.row.auto_copy_detected_text' => '自動複製偵測到的文字',
      'settings.general.row.default_directory_service' => '預設辭典服務',
      'settings.general.row.default_translation_service' => '預設翻譯服務',
      'settings.general.row.translation_target_hint' => '設定翻譯器使用的語言目標。',
      'settings.general.row.common_languages' => '常用語言',
      'settings.general.row.common_languages_description' => '顯示在語言選擇列表頂部。',
      'settings.general.row.common_languages_hint' => '選擇你常用的語言：',
      'settings.general.row.common_languages_sort' => '按代碼排序',
      'settings.general.row.common_languages_reset' => '恢復預設',
      'settings.general.row.common_languages_reset_help' => '恢復為預設的常用語言集合',
      'settings.general.row.common_languages_search' => '搜索語言...',
      'settings.general.row.common_languages_all' => '所有語言',
      'settings.general.row.double_click_copy_result' => '雙擊複製翻譯結果',
      'settings.general.row.submit_with_enter' => '按 Enter 提交',
      'settings.general.row.submit_with_meta_enter_mac' => '按 ⌘ + Enter 提交',
      'settings.general.row.screen_capture_access_hint' => '截圖取詞需要讀取螢幕內容。',
      'settings.general.row.screen_selection_access_hint' =>
        '劃詞取詞需要讀取其他應用程式中選取的文字。',
      'settings.general.row.no_translation_targets' =>
        '還沒有翻譯目標，新增一條來決定預設譯入哪種語言。',
      'settings.general.button.add_provider' => '新增…',
      'settings.general.button.add_target' => '新增目標...',
      'settings.general.button.manage_targets' => '管理翻譯目標...',
      'settings.general.button.manage_languages' => '管理常用語言...',
      'settings.general.button.grant' => '授權',
      'settings.general.option.none' => '無',
      'settings.general.option.no_services_available' => '暫無可用服務',
      'settings.general.option.granted' => '已授權',
      'settings.general.option.built_in_ocr' => '內建 OCR',
      'settings.general.option.tesseract' => 'Tesseract',
      'settings.general.option.youdao_ocr' => '有道 OCR',
      'settings.general.editor.add_target_title' => '添加翻譯目標：',
      'settings.general.editor.edit_target_title' => '修改翻譯目標：',
      'settings.general.editor.row.source_language' => '源語言',
      'settings.general.editor.row.target_language' => '目標語言',
      'settings.general.editor.title_edit' => '編輯翻譯目標',
      'settings.general.editor.subtitle' => '決定某種來源語言預設翻譯成哪種語言',
      'settings.general.editor.same_language' => '來源語言與目標語言相同，換一個目標語言。',
      'settings.general.editor.duplicate' => '已經有一條同樣的翻譯目標了。',
      'settings.general.editor.hint_auto' => '未符合其他規則時，一律譯成{}。',
      'settings.general.editor.hint_source' => '偵測到{}時，譯成{}。',
      'settings.appearance.title' => '外觀',
      'settings.appearance.section.app_language' => '顯示語言',
      'settings.appearance.section.theme_mode' => '主題模式',
      'settings.appearance.section.theme_style' => '主題風格',
      'settings.appearance.footer' => '變更立即套用到整個視窗。',
      'settings.shortcuts.title' => '快捷鍵',
      'settings.shortcuts.section.text_extraction' => '文字擷取',
      'settings.shortcuts.section.input_assist' => '輸入輔助功能',
      'settings.shortcuts.section.submit_mode' => '提交方式',
      'settings.shortcuts.row.toggle_mini_translator' => '顯示/隱藏視窗',
      'settings.shortcuts.row.extract_text_from_screen_selection' =>
        '從螢幕選取範圍擷取文字',
      'settings.shortcuts.row.extract_text_from_screen_capture' => '從螢幕截圖擷取文字',
      'settings.shortcuts.row.extract_text_from_clipboard' => '從剪貼簿擷取文字',
      'settings.shortcuts.row.translate_input' => '翻譯輸入內容',
      'settings.shortcuts.reset_dialog.title' => '重設快捷鍵',
      'settings.shortcuts.reset_dialog.message' => '確定要重設所有快捷鍵為預設值嗎？',
      'settings.shortcuts.reset_dialog.confirm' => '重設',
      'settings.shortcuts.reset_dialog.cancel' => '取消',
      'settings.shortcuts.group.global.title' => '全域快速鍵',
      'settings.shortcuts.group.global.description' => '在任何應用程式裡都能觸發。',
      'settings.shortcuts.group.in_app.title' => '應用程式內按鍵',
      'settings.shortcuts.group.in_app.description' => '僅在應用程式自己的輸入框內生效。',
      'settings.shortcuts.reset' => '恢復預設...',
      'settings.advanced.title' => '進階',
      'settings.advanced.api_server' => '本機 API 服務',
      'settings.advanced.api_server_description' =>
        '在 127.0.0.1 上開放翻譯 API，供本機整合使用。',
      'settings.advanced.enable' => '啟用',
      'settings.advanced.port' => '埠號',
      'settings.advanced.running_at' => '執行於 {url}',
      'settings.advanced.disabled' => '已關閉',
      'settings.services.title' => '服務',
      'settings.services.button.add_service' => '新增服務...',
      'settings.services.section.available_services' => '可用服務',
      'settings.services.editor.title' => '新增服務',
      'settings.services.editor.subtitle' => '為已設定的提供者新增一項服務',
      'settings.services.editor.row.model' => '模型',
      'settings.services.editor.row.system_prompt' => '系統提示詞',
      'settings.services.editor.prompt_placeholder' => '留空則使用該類型的預設提示詞',
      'settings.services.editor.variant_hint' => '{} 已有一項{}服務，這條會作為並列的另一份設定。',
      'settings.services.editor.traditional_note' =>
        '{} 是傳統介面，沒有模型與提示詞可調；服務參數在提供者詳情頁設定。',
      'settings.services.detail.row.id' => '服務 ID',
      'settings.services.detail.row.name' => '名稱',
      'settings.services.detail.row.provider' => '提供者',
      'settings.services.detail.row.type' => '類型',
      'settings.services.detail.delete_dialog.title' => '刪除「{}」？',
      'settings.services.detail.delete_dialog.message' => '此服務將從提供者中移除。',
      'settings.services.detail.prompt_variables' =>
        '可用變數：{{sourceLanguage}}、{{targetLanguage}}、{{text}}',
      'settings.services.make_default' => '設為預設',
      'settings.services.item.none_of_kind' => '還沒有可用的{}服務。',
      'settings.providers.title' => '提供者',
      'settings.providers.section.services' => '可用服務',
      'settings.providers.section.services_description' =>
        '查看已設定提供商的可用服務，並依服務類型切換。',
      'settings.providers.item.empty' => '尚未設定任何提供者。新增一個提供者以啟用翻譯服務。',
      'settings.providers.item.loading' => '正在載入提供者…',
      'settings.providers.item.no_services' => '暫無可用服務。',
      'settings.providers.button.add' => '新增提供者…',
      'settings.providers.alert.error' => '錯誤',
      'settings.providers.intro.body' => '管理應用程式使用的服務提供商。',
      'settings.providers.intro.warning' => '已連線的提供商可能會處理您傳送的文字或圖片，請僅啟用您信任的服務。',
      'settings.providers.editor.row.id' => '提供者 ID',
      'settings.providers.editor.row.type' => '提供者類型',
      'settings.providers.editor.row.default_model' => '預設模型',
      'settings.providers.editor.placeholder.id' => '例如 deepl-main',
      'settings.providers.editor.type_picker.prompt' => '請選擇要新增的提供者類型：',
      'settings.providers.editor.type_picker.section_llm' => 'LLM',
      'settings.providers.editor.type_picker.section_traditional' => '傳統',
      'settings.providers.editor.tooltip.help' => '說明',
      'settings.providers.editor.step.next' => '繼續',
      'settings.providers.editor.step.back' => '上一步',
      'settings.providers.editor.add_title' => '新增 {}',
      'settings.providers.editor.capability_note.translation' => '加入翻譯候選',
      'settings.providers.editor.capability_note.dictionary' => '提供辭典釋義',
      'settings.providers.editor.capability_note.ocr' => '辨識圖片中的文字',
      'settings.providers.editor.test.run' => '測試連線',
      'settings.providers.editor.test.running' => '正在測試連線 · 已用 {}s',
      'settings.providers.editor.test.passed_models' => '連線正常 · {} 個模型可用',
      'settings.providers.editor.test.passed_service' => '連線正常 · 服務可用',
      'settings.providers.editor.test.passed_footer' => '已通過連線測試',
      'settings.providers.editor.test.retest' => '重新測試',
      'settings.providers.editor.test.tips_title' => '可以試試',
      'settings.providers.editor.test.tips_llm' =>
        '· 確認金鑰與所選提供者類型一致\n· 檢查 Base URL 是否需要包含 /v1\n· 在提供者主控台確認該模型已開通',
      'settings.providers.editor.test.tips_traditional' =>
        '· 確認金鑰與所選提供者類型一致\n· 在提供者主控台確認服務已開通',
      'settings.providers.editor.test.failed_suffix' => '驗證失敗',
      'settings.providers.editor.test.passed_suffix' => '已驗證',
      'settings.providers.detail.tooltip.edit' => '編輯提供者',
      'settings.providers.detail.row.id_hint' => '建立後不可變更',
      'settings.providers.detail.section.configuration' => '配置',
      'settings.providers.detail.section.models' => '模型',
      'settings.providers.detail.models.loading' => '正在載入模型...',
      'settings.providers.detail.models.empty' => '找不到模型。',
      'settings.providers.detail.models.retry' => '重試',
      'settings.providers.detail.models.refresh' => '重新整理清單',
      'settings.providers.detail.models.default_badge' => '預設',
      'settings.providers.detail.models.set_default' => '設為預設',
      'settings.providers.detail.models.fetch_error' => '無法從提供者 API 取得模型。',
      'settings.providers.capability.translation' => '翻譯',
      'settings.providers.capability.dictionary' => '辭典',
      'settings.providers.capability.ocr' => 'OCR',
      'settings.providers.capability.llm' => 'AI',
      'settings.providers.description.all' => '提供辭典查詢和文字翻譯',
      'settings.providers.description.translation' => '提供語言間文字翻譯',
      'settings.providers.description.dictionary' => '提供辭典查詢和單字釋義',
      'settings.providers.description.fallback' => '提供翻譯服務',
      'settings.providers.delete_dialog.title' => '刪除「{}」？',
      'settings.providers.delete_dialog.message' => '此操作無法復原。',
      'settings.layout.title' => '設定',
      'settings.layout.empty.title' => '選擇一個分類',
      'settings.layout.empty.message' => '從側邊欄選擇一個設定分類。',
      'settings.layout.groups' => '設定分組',
      'settings.layout.effect_hint' => '更改即時生效',
      'settings.layout.footer_note' => '譯文與金鑰僅保存在本機',
      'settings.layout.support' => '支援',
      'settings.about.title' => '關於',
      'settings.about.copy_version_info' => '複製版本資訊',
      'settings.about.up_to_date' => '已是最新版本。',
      'settings.about.check_again' => '重新檢查',
      'settings.about.links' => '連結',
      'settings.about.website' => '網站',
      'settings.about.github' => 'GitHub',
      'settings.about.report_issue' => '回報問題',
      'settings.about.license' => '授權條款',
      'settings.about.open_changelog' => '查看更新日誌',
      'settings.about.update' => '更新',
      _ => null,
    };
  }
}
