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
class TranslationsJa extends Translations
    with BaseTranslations<AppLocale, Translations> {
  /// You can call this constructor and build your own translation instance of this locale.
  /// Constructing via the enum [AppLocale.build] is preferred.
  TranslationsJa({
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
             locale: AppLocale.ja,
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

  /// Metadata for the translations of <ja>.
  final TranslationMetadata<AppLocale, Translations> _meta;
  @override
  TranslationMetadata<AppLocale, Translations> get $meta => _meta;

  /// Access flat map
  @override
  dynamic operator [](String key) => _meta.getTranslation(key) ?? super[key];

  late final TranslationsJa _root = this; // ignore: unused_field

  @override
  TranslationsJa $copyWith({
    TranslationMetadata<AppLocale, Translations>? meta,
  }) => TranslationsJa(meta: meta ?? this.$meta);

  // Translations
  @override
  late final _Translations$common$ja common = _Translations$common$ja._(_root);
  @override
  late final _Translations$app$ja app = _Translations$app$ja._(_root);
  @override
  late final _Translations$mini_translator$ja mini_translator =
      _Translations$mini_translator$ja._(_root);
  @override
  late final _Translations$workbench$ja workbench =
      _Translations$workbench$ja._(_root);
  @override
  late final _Translations$settings$ja settings = _Translations$settings$ja._(
    _root,
  );
}

// Path: common
class _Translations$common$ja extends Translations$common$en {
  _Translations$common$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  late final _Translations$common$ui$ja ui = _Translations$common$ui$ja._(
    _root,
  );
  @override
  late final _Translations$common$language$ja language =
      _Translations$common$language$ja._(_root);
  @override
  late final _Translations$common$theme_mode$ja theme_mode =
      _Translations$common$theme_mode$ja._(_root);
  @override
  late final _Translations$common$theme_style$ja theme_style =
      _Translations$common$theme_style$ja._(_root);
  @override
  late final _Translations$common$provider$ja provider =
      _Translations$common$provider$ja._(_root);
}

// Path: app
class _Translations$app$ja extends Translations$app$en {
  _Translations$app$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  late final _Translations$app$tray$ja tray = _Translations$app$tray$ja._(
    _root,
  );
}

// Path: mini_translator
class _Translations$mini_translator$ja extends Translations$mini_translator$en {
  _Translations$mini_translator$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  late final _Translations$mini_translator$limited_banner$ja limited_banner =
      _Translations$mini_translator$limited_banner$ja._(_root);
  @override
  late final _Translations$mini_translator$input$ja input =
      _Translations$mini_translator$input$ja._(_root);
  @override
  late final _Translations$mini_translator$toolbar$ja toolbar =
      _Translations$mini_translator$toolbar$ja._(_root);
  @override
  late final _Translations$mini_translator$button$ja button =
      _Translations$mini_translator$button$ja._(_root);
  @override
  late final _Translations$mini_translator$language$ja language =
      _Translations$mini_translator$language$ja._(_root);
  @override
  late final _Translations$mini_translator$message$ja message =
      _Translations$mini_translator$message$ja._(_root);
  @override
  late final _Translations$mini_translator$result$ja result =
      _Translations$mini_translator$result$ja._(_root);
}

// Path: workbench
class _Translations$workbench$ja extends Translations$workbench$en {
  _Translations$workbench$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get workspace => 'ワークスペース';
  @override
  String get translate => '翻訳';
  @override
  String get history => '履歴';
  @override
  late final _Translations$workbench$history_page$ja history_page =
      _Translations$workbench$history_page$ja._(_root);
  @override
  String get glossary => '用語集';
  @override
  String get recent_languages => '最近の言語';
  @override
  String get not_configured => '未設定';
  @override
  late final _Translations$workbench$subtitle$ja subtitle =
      _Translations$workbench$subtitle$ja._(_root);
  @override
  late final _Translations$workbench$placeholder$ja placeholder =
      _Translations$workbench$placeholder$ja._(_root);
  @override
  late final _Translations$workbench$glossary_page$ja glossary_page =
      _Translations$workbench$glossary_page$ja._(_root);
  @override
  late final _Translations$workbench$translation$ja translation =
      _Translations$workbench$translation$ja._(_root);
  @override
  late final _Translations$workbench$status$ja status =
      _Translations$workbench$status$ja._(_root);
  @override
  String get version_latest => '最新です';
  @override
  String get version_checking => '確認中…';
  @override
  String get check_updates => 'アップデートを確認';
}

// Path: settings
class _Translations$settings$ja extends Translations$settings$en {
  _Translations$settings$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get version => 'v{} (Build {})';
  @override
  late final _Translations$settings$general$ja general =
      _Translations$settings$general$ja._(_root);
  @override
  late final _Translations$settings$appearance$ja appearance =
      _Translations$settings$appearance$ja._(_root);
  @override
  late final _Translations$settings$shortcuts$ja shortcuts =
      _Translations$settings$shortcuts$ja._(_root);
  @override
  late final _Translations$settings$advanced$ja advanced =
      _Translations$settings$advanced$ja._(_root);
  @override
  late final _Translations$settings$services$ja services =
      _Translations$settings$services$ja._(_root);
  @override
  late final _Translations$settings$providers$ja providers =
      _Translations$settings$providers$ja._(_root);
  @override
  late final _Translations$settings$layout$ja layout =
      _Translations$settings$layout$ja._(_root);
  @override
  late final _Translations$settings$about$ja about =
      _Translations$settings$about$ja._(_root);
}

// Path: common.ui
class _Translations$common$ui$ja extends Translations$common$ui$en {
  _Translations$common$ui$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  late final _Translations$common$ui$button$ja button =
      _Translations$common$ui$button$ja._(_root);
  @override
  late final _Translations$common$ui$feedback$ja feedback =
      _Translations$common$ui$feedback$ja._(_root);
}

// Path: common.language
class _Translations$common$language$ja extends Translations$common$language$en {
  _Translations$common$language$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get ar => 'アラビア語';
  @override
  String get bn => 'ベンガル語';
  @override
  String get de => 'ドイツ語';
  @override
  String get en => '英語';
  @override
  String get es => 'スペイン語';
  @override
  String get fa => 'ペルシア語';
  @override
  String get fr => 'フランス語';
  @override
  String get gu => 'グジャラート語';
  @override
  String get ha => 'ハウサ語';
  @override
  String get hi => 'ヒンディー語';
  @override
  String get id => 'インドネシア語';
  @override
  String get it => 'イタリア語';
  @override
  String get ja => '日本語';
  @override
  String get jv => 'ジャワ語';
  @override
  String get ko => '韓国語';
  @override
  String get ml => 'マラヤーラム語';
  @override
  String get mr => 'マラーティー語';
  @override
  String get ms => 'マレー語';
  @override
  String get nl => 'オランダ語';
  @override
  String get pa => 'パンジャブ語';
  @override
  String get pl => 'ポーランド語';
  @override
  String get pt => 'ポルトガル語';
  @override
  String get ro => 'ルーマニア語';
  @override
  String get ru => 'ロシア語';
  @override
  String get sw => 'スワヒリ語';
  @override
  String get ta => 'タミル語';
  @override
  String get te => 'テルグ語';
  @override
  String get th => 'タイ語';
  @override
  String get tr => 'トルコ語';
  @override
  String get uk => 'ウクライナ語';
  @override
  String get ur => 'ウルドゥー語';
  @override
  String get vi => 'ベトナム語';
  @override
  String get yo => 'ヨルバ語';
  @override
  String get zh_hans => '簡体字中国語';
  @override
  String get zh_hant => '繁体字中国語';
}

// Path: common.theme_mode
class _Translations$common$theme_mode$ja
    extends Translations$common$theme_mode$en {
  _Translations$common$theme_mode$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get light => 'ライト';
  @override
  String get dark => 'ダーク';
  @override
  String get system => 'システム';
}

// Path: common.theme_style
class _Translations$common$theme_style$ja
    extends Translations$common$theme_style$en {
  _Translations$common$theme_style$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get studio => 'Studio';
  @override
  String get bright => 'Bright';
}

// Path: common.provider
class _Translations$common$provider$ja extends Translations$common$provider$en {
  _Translations$common$provider$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get anthropic => 'Anthropic';
  @override
  String get baidu => 'Baidu';
  @override
  String get caiyun => 'Caiyun';
  @override
  String get deepl => 'DeepL';
  @override
  String get google => 'Google';
  @override
  String get ollama => 'Ollama';
  @override
  String get openai => 'OpenAI';
  @override
  String get sogou => 'Sogou';
  @override
  String get xai => 'xAI';
  @override
  String get system => 'システム';
  @override
  String get tencent => 'Tencent';
  @override
  String get youdao => 'Youda';
}

// Path: app.tray
class _Translations$app$tray$ja extends Translations$app$tray$en {
  _Translations$app$tray$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  late final _Translations$app$tray$context_menu$ja context_menu =
      _Translations$app$tray$context_menu$ja._(_root);
}

// Path: mini_translator.limited_banner
class _Translations$mini_translator$limited_banner$ja
    extends Translations$mini_translator$limited_banner$en {
  _Translations$mini_translator$limited_banner$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  late final _Translations$mini_translator$limited_banner$permission$ja
  permission = _Translations$mini_translator$limited_banner$permission$ja._(
    _root,
  );
  @override
  late final _Translations$mini_translator$limited_banner$instruction$ja
  instruction = _Translations$mini_translator$limited_banner$instruction$ja._(
    _root,
  );
  @override
  late final _Translations$mini_translator$limited_banner$action$ja action =
      _Translations$mini_translator$limited_banner$action$ja._(_root);
  @override
  late final _Translations$mini_translator$limited_banner$feedback$ja feedback =
      _Translations$mini_translator$limited_banner$feedback$ja._(_root);
  @override
  late final _Translations$mini_translator$limited_banner$tooltip$ja tooltip =
      _Translations$mini_translator$limited_banner$tooltip$ja._(_root);
}

// Path: mini_translator.input
class _Translations$mini_translator$input$ja
    extends Translations$mini_translator$input$en {
  _Translations$mini_translator$input$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get hint => 'ここに単語またはテキストを入力';
  @override
  String get extracting_text => 'テキストを抽出中...';
  @override
  String hint_translate_to({required Object language}) =>
      '単語またはテキストを入力して${language}に翻訳';
}

// Path: mini_translator.toolbar
class _Translations$mini_translator$toolbar$ja
    extends Translations$mini_translator$toolbar$en {
  _Translations$mini_translator$toolbar$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  late final _Translations$mini_translator$toolbar$tooltip$ja tooltip =
      _Translations$mini_translator$toolbar$tooltip$ja._(_root);
  @override
  late final _Translations$mini_translator$toolbar$menu$ja menu =
      _Translations$mini_translator$toolbar$menu$ja._(_root);
}

// Path: mini_translator.button
class _Translations$mini_translator$button$ja
    extends Translations$mini_translator$button$en {
  _Translations$mini_translator$button$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get clear => 'クリア';
  @override
  String get translate => '翻訳';
  @override
  String get copy => 'コピー';
  @override
  String get copied => 'コピー済み';
  @override
  String get bookmark => '保存';
  @override
  String get bookmarked => '保存済み';
}

// Path: mini_translator.language
class _Translations$mini_translator$language$ja
    extends Translations$mini_translator$language$en {
  _Translations$mini_translator$language$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get auto_detect => '自動検出';
  @override
  String get auto_match => '自動一致';
  @override
  String get switch_config => 'ターゲット切替';
  @override
  String get more_languages => 'その他の言語...';
  @override
  String get manage_common_languages => 'よく使う言語を管理...';
  @override
  String get manage_targets => '翻訳ターゲットを管理...';
  @override
  String get add_target => '翻訳ターゲットを追加...';
}

// Path: mini_translator.message
class _Translations$mini_translator$message$ja
    extends Translations$mini_translator$message$en {
  _Translations$mini_translator$message$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get please_enter_word_or_text => 'テキストが入力されていないか、テキストが抽出されていません';
  @override
  String get capture_screen_area_canceled => '画面領域のキャプチャがキャンセルされました';
  @override
  String get ocr_service_not_configured =>
      'デフォルトのテキスト認識サービスが設定されていません。設定で設定してください。';
  @override
  String get ocr_recognition_failed => 'テキスト認識に失敗しました';
}

// Path: mini_translator.result
class _Translations$mini_translator$result$ja
    extends Translations$mini_translator$result$en {
  _Translations$mini_translator$result$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get translating => '翻訳中…';
  @override
  String stale_requery({required Object key}) => '原文が変更されました · ${key} 再翻訳';
  @override
  String compare_services({required Object count}) => '${count} 個のサービスを比較';
  @override
  String get collapse_compare => '比較を閉じる';
  @override
  String get set_preferred => '優先に設定';
  @override
  String get retry => '再試行';
  @override
  String get no_result => 'どのサービスも結果を返しませんでした。ネットワークを確認するか、別のサービスをお試しください。';
  @override
  String get no_result_note => '原文は保持され、再試行しても履歴は重複しません。';
}

// Path: workbench.history_page
class _Translations$workbench$history_page$ja
    extends Translations$workbench$history_page$en {
  _Translations$workbench$history_page$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get all => 'すべて';
  @override
  String get favorites => 'お気に入り';
  @override
  String get edited => '編集済み';
  @override
  String get search => '検索';
  @override
  String get search_placeholder => '原文、訳文、サービスを検索';
  @override
  String get search_label => '履歴を検索';
  @override
  String entry_count({required Object label, required Object count}) =>
      '${label} · ${count} 件';
  @override
  String get by_time => '時間順';
  @override
  String get loading => '履歴を読み込み中…';
  @override
  String get load_failed => '履歴を読み込めませんでした';
  @override
  String get retry => '再試行';
  @override
  String get empty_title => '翻訳履歴はまだありません';
  @override
  String get empty_description => '翻訳が完了すると、優先訳がここに保存されます。';
  @override
  String no_results({required Object query}) => '「${query}」に一致する履歴はありません';
  @override
  String get clear_search => '検索をクリア';
  @override
  String get select => '複数選択';
  @override
  String selected_count({required Object count}) => '${count} 件選択中';
  @override
  String get exit_select => '選択を終了';
  @override
  String get add_to_glossary => '用語集に追加';
  @override
  String get favorite => 'お気に入り';
  @override
  String get unfavorite => 'お気に入り解除';
  @override
  String delete_confirm({required Object count}) =>
      '選択した ${count} 件の履歴を削除しますか？元に戻せません。';
  @override
  String get no_glossary => '先に用語集を作成してください';
  @override
  String added_to_glossary({required Object count}) => '${count} 件を用語集に追加しました';
  @override
  String get favorite_flag => 'お気に入り';
  @override
  String get edited_flag => '編集済み';
  @override
  String get edit_history_hint => '編集した訳文は履歴に保存されます';
}

// Path: workbench.subtitle
class _Translations$workbench$subtitle$ja
    extends Translations$workbench$subtitle$en {
  _Translations$workbench$subtitle$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get translate => 'ワークベンチ · サービス比較';
  @override
  String get settings => '設定';
}

// Path: workbench.placeholder
class _Translations$workbench$placeholder$ja
    extends Translations$workbench$placeholder$en {
  _Translations$workbench$placeholder$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get history => '以前の翻訳を確認、編集、お気に入り登録して再利用します';
  @override
  String get glossary => '指定訳と禁止する別訳を管理します';
}

// Path: workbench.glossary_page
class _Translations$workbench$glossary_page$ja
    extends Translations$workbench$glossary_page$en {
  _Translations$workbench$glossary_page$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get add_entry => '用語を追加';
  @override
  String get term => '原文';
  @override
  String get translation => '指定訳';
  @override
  String get forbidden => '禁止';
  @override
  String get hits => 'ヒット';
  @override
  String get term_placeholder => 'teacher forcing';
  @override
  String get translation_placeholder => '教師強制';
  @override
  String get forbidden_placeholder => '強制教育';
  @override
  String get search => '検索';
  @override
  String get search_placeholder => '用語または指定訳を検索';
  @override
  String get search_label => '用語集を検索';
  @override
  String entry_count({required Object name, required Object count}) =>
      '${name} · ${count} 件';
  @override
  String get priority_note => '用語集はどのサービスの出力よりも優先されます';
  @override
  String get new_book => '用語集を作成';
  @override
  String get new_book_placeholder => '用語集の名前';
  @override
  String get rename_book => '名前を変更';
  @override
  String delete_book_confirm({required Object name, required Object count}) =>
      '「${name}」と、その ${count} 件の用語を削除しますか？';
  @override
  String get disabled => '無効';
  @override
  String get enable => '有効にする';
  @override
  String get disable => '無効にする';
  @override
  String get empty_title => 'この用語集は空です';
  @override
  String get empty_description =>
      '用語集はどのサービスの出力よりも優先されます。1 件ずつ追加するか、CSV をドロップして取り込めます。';
  @override
  String no_results_title({required Object query}) => '「${query}」に一致する用語はありません';
  @override
  String get no_results_description => '別のキーワードを試すか、新しく追加してください。';
  @override
  String get no_books_title => '用語集がまだありません';
  @override
  String get no_books_description =>
      '用語集は、選んだ訳語をすべてのサービスで統一します。まず 1 つ作成して、用語を追加しましょう。';
  @override
  String get loading => '読み込み中…';
}

// Path: workbench.translation
class _Translations$workbench$translation$ja
    extends Translations$workbench$translation$en {
  _Translations$workbench$translation$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get source => '原文';
  @override
  String get target => '訳文';
  @override
  String get input_hint => '翻訳するテキストを入力または貼り付け';
  @override
  String get button => '翻訳';
  @override
  String get auto_detected => '自動検出';
  @override
  String get loading_services => '翻訳サービスを読み込み中…';
  @override
  String get no_services => '先に設定で翻訳サービスを構成してください';
  @override
  String get translating => '翻訳中…';
  @override
  String get failed => '翻訳に失敗しました。サービス設定を確認してください。';
  @override
  String get empty => '訳文がここに表示されます';
  @override
  String get service_compare => 'サービス比較';
  @override
  String get main_translation => 'メイン';
  @override
  String get service_unavailable => 'サービスを利用できません';
  @override
  String get waiting => '翻訳待ち';
  @override
  String get copy => 'コピー';
  @override
  String get favorite_unavailable => 'お気に入りを更新できませんでした。もう一度お試しください。';
  @override
  String get preferred => '優先訳文';
  @override
  String get other_services => '他のサービス';
  @override
  String get copy_result => '訳文をコピー';
  @override
  String get copied => 'コピー済み';
  @override
  String get favorite => '保存';
  @override
  String get terms => '用語ヒット';
  @override
  String get terms_hint => '入力すると用語集と照合されます。';
  @override
  String get quality => '品質シグナル';
  @override
  String get quality_hint => '訳文の生成後に計算されます。';
  @override
  String get shortcuts => 'ショートカット';
  @override
  String get other_services_disabled => '他のサービスは無効です';
  @override
  String input_hint_translate_to({required Object language}) =>
      '翻訳するテキストを入力または貼り付け、${language}に翻訳';
  @override
  String get clear => 'クリア';
  @override
  String get swap_languages => '言語を入れ替える';
  @override
  String get services => '翻訳サービス';
  @override
  String get configure_services => 'サービスを設定';
  @override
  String get retry => '再試行';
  @override
  String character_count({required Object count}) => '${count} 文字';
}

// Path: workbench.status
class _Translations$workbench$status$ja
    extends Translations$workbench$status$en {
  _Translations$workbench$status$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get runtime_ready => '翻訳ランタイム準備完了';
  @override
  String get settings_synced => '設定を同期済み';
  @override
  String get shortcuts => '⌥Space クイックウィンドウ · ⌥⇧2 キャプチャ';
}

// Path: settings.general
class _Translations$settings$general$ja
    extends Translations$settings$general$en {
  _Translations$settings$general$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get title => '一般';
  @override
  late final _Translations$settings$general$section$ja section =
      _Translations$settings$general$section$ja._(_root);
  @override
  late final _Translations$settings$general$row$ja row =
      _Translations$settings$general$row$ja._(_root);
  @override
  late final _Translations$settings$general$button$ja button =
      _Translations$settings$general$button$ja._(_root);
  @override
  late final _Translations$settings$general$option$ja option =
      _Translations$settings$general$option$ja._(_root);
  @override
  late final _Translations$settings$general$editor$ja editor =
      _Translations$settings$general$editor$ja._(_root);
}

// Path: settings.appearance
class _Translations$settings$appearance$ja
    extends Translations$settings$appearance$en {
  _Translations$settings$appearance$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get title => '外観';
  @override
  late final _Translations$settings$appearance$section$ja section =
      _Translations$settings$appearance$section$ja._(_root);
  @override
  String get footer => '変更はウインドウ全体にすぐ適用されます。';
}

// Path: settings.shortcuts
class _Translations$settings$shortcuts$ja
    extends Translations$settings$shortcuts$en {
  _Translations$settings$shortcuts$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get title => 'ショートカット';
  @override
  late final _Translations$settings$shortcuts$section$ja section =
      _Translations$settings$shortcuts$section$ja._(_root);
  @override
  late final _Translations$settings$shortcuts$row$ja row =
      _Translations$settings$shortcuts$row$ja._(_root);
  @override
  late final _Translations$settings$shortcuts$reset_dialog$ja reset_dialog =
      _Translations$settings$shortcuts$reset_dialog$ja._(_root);
  @override
  late final _Translations$settings$shortcuts$group$ja group =
      _Translations$settings$shortcuts$group$ja._(_root);
  @override
  String get reset => 'デフォルトに戻す...';
}

// Path: settings.advanced
class _Translations$settings$advanced$ja
    extends Translations$settings$advanced$en {
  _Translations$settings$advanced$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get title => '詳細設定';
  @override
  String get api_server => 'ローカルAPIサーバー';
  @override
  String get api_server_description =>
      'ローカルインテグレーションのために127.0.0.1で翻訳APIを公開します。';
  @override
  String get enable => '有効';
  @override
  String get port => 'ポート';
  @override
  String get running_at => '{url} で実行中';
  @override
  String get disabled => '無効';
}

// Path: settings.services
class _Translations$settings$services$ja
    extends Translations$settings$services$en {
  _Translations$settings$services$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get title => 'サービス';
  @override
  late final _Translations$settings$services$button$ja button =
      _Translations$settings$services$button$ja._(_root);
  @override
  late final _Translations$settings$services$section$ja section =
      _Translations$settings$services$section$ja._(_root);
  @override
  late final _Translations$settings$services$editor$ja editor =
      _Translations$settings$services$editor$ja._(_root);
  @override
  late final _Translations$settings$services$detail$ja detail =
      _Translations$settings$services$detail$ja._(_root);
  @override
  String get make_default => 'デフォルトにする';
  @override
  late final _Translations$settings$services$item$ja item =
      _Translations$settings$services$item$ja._(_root);
}

// Path: settings.providers
class _Translations$settings$providers$ja
    extends Translations$settings$providers$en {
  _Translations$settings$providers$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get title => 'プロバイダー';
  @override
  late final _Translations$settings$providers$section$ja section =
      _Translations$settings$providers$section$ja._(_root);
  @override
  late final _Translations$settings$providers$item$ja item =
      _Translations$settings$providers$item$ja._(_root);
  @override
  late final _Translations$settings$providers$button$ja button =
      _Translations$settings$providers$button$ja._(_root);
  @override
  late final _Translations$settings$providers$alert$ja alert =
      _Translations$settings$providers$alert$ja._(_root);
  @override
  late final _Translations$settings$providers$intro$ja intro =
      _Translations$settings$providers$intro$ja._(_root);
  @override
  late final _Translations$settings$providers$editor$ja editor =
      _Translations$settings$providers$editor$ja._(_root);
  @override
  late final _Translations$settings$providers$detail$ja detail =
      _Translations$settings$providers$detail$ja._(_root);
  @override
  late final _Translations$settings$providers$capability$ja capability =
      _Translations$settings$providers$capability$ja._(_root);
  @override
  late final _Translations$settings$providers$description$ja description =
      _Translations$settings$providers$description$ja._(_root);
  @override
  late final _Translations$settings$providers$delete_dialog$ja delete_dialog =
      _Translations$settings$providers$delete_dialog$ja._(_root);
}

// Path: settings.layout
class _Translations$settings$layout$ja extends Translations$settings$layout$en {
  _Translations$settings$layout$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get title => '設定';
  @override
  late final _Translations$settings$layout$empty$ja empty =
      _Translations$settings$layout$empty$ja._(_root);
  @override
  String get groups => '設定グループ';
  @override
  String get effect_hint => '変更は即時反映';
  @override
  String get footer_note => '訳文とキーはこのMacにのみ保存されます';
  @override
  String get support => 'サポート';
}

// Path: settings.about
class _Translations$settings$about$ja extends Translations$settings$about$en {
  _Translations$settings$about$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get title => '情報';
  @override
  String get copy_version_info => 'バージョン情報をコピー';
  @override
  String get up_to_date => '最新の状態です。';
  @override
  String get check_again => '再確認';
  @override
  String get links => 'リンク';
  @override
  String get website => 'ウェブサイト';
  @override
  String get github => 'GitHub';
  @override
  String get report_issue => '問題を報告';
  @override
  String get license => 'ライセンス';
  @override
  String get open_changelog => '更新履歴を開く';
  @override
  String get update => 'アップデート';
}

// Path: common.ui.button
class _Translations$common$ui$button$ja
    extends Translations$common$ui$button$en {
  _Translations$common$ui$button$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get ok => 'OK';
  @override
  String get cancel => 'キャンセル';
  @override
  String get add => '追加';
  @override
  String get delete => '削除';
  @override
  String get edit => '編集';
  @override
  String get save => '保存';
  @override
  String get manage => '管理';
  @override
  String get kContinue => '続行';
}

// Path: common.ui.feedback
class _Translations$common$ui$feedback$ja
    extends Translations$common$ui$feedback$en {
  _Translations$common$ui$feedback$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get copied => 'コピーしました';
}

// Path: app.tray.context_menu
class _Translations$app$tray$context_menu$ja
    extends Translations$app$tray$context_menu$en {
  _Translations$app$tray$context_menu$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get show_window => 'ウィンドウを表示';
  @override
  late final _Translations$app$tray$context_menu$dev_tools$ja dev_tools =
      _Translations$app$tray$context_menu$dev_tools$ja._(_root);
  @override
  String get check_for_updates => 'アップデートを確認';
  @override
  String get settings => '設定';
  @override
  String get quit => '終了';
}

// Path: mini_translator.limited_banner.permission
class _Translations$mini_translator$limited_banner$permission$ja
    extends Translations$mini_translator$limited_banner$permission$en {
  _Translations$mini_translator$limited_banner$permission$ja._(
    TranslationsJa root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get missing_both => 'すべての機能を有効にするには、画面収録とアクセシビリティの許可を付与してください。';
  @override
  String get missing_screen_capture => 'すべての機能を有効にするには、画面収録の許可を付与してください。';
  @override
  String get missing_accessibility => 'すべての機能を有効にするには、アクセシビリティの許可を付与してください。';
}

// Path: mini_translator.limited_banner.instruction
class _Translations$mini_translator$limited_banner$instruction$ja
    extends Translations$mini_translator$limited_banner$instruction$en {
  _Translations$mini_translator$limited_banner$instruction$ja._(
    TranslationsJa root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get app_settings_prefix => 'アプリ設定に移動';
  @override
  String get follow_guide_prefix => 'し、ガイドに従って';
  @override
  String get suffix => 'をクリックしてください。';
}

// Path: mini_translator.limited_banner.action
class _Translations$mini_translator$limited_banner$action$ja
    extends Translations$mini_translator$limited_banner$action$en {
  _Translations$mini_translator$limited_banner$action$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get app_settings => 'アプリ設定';
  @override
  String get recheck => '再確認';
}

// Path: mini_translator.limited_banner.feedback
class _Translations$mini_translator$limited_banner$feedback$ja
    extends Translations$mini_translator$limited_banner$feedback$en {
  _Translations$mini_translator$limited_banner$feedback$ja._(
    TranslationsJa root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get enabled => '画面テキスト抽出が有効です。';
  @override
  String get still_missing => '必要な権限がまだ不足しています。\n設定を確認して、もう一度お試しください。';
}

// Path: mini_translator.limited_banner.tooltip
class _Translations$mini_translator$limited_banner$tooltip$ja
    extends Translations$mini_translator$limited_banner$tooltip$en {
  _Translations$mini_translator$limited_banner$tooltip$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get help => 'ヘルプを表示';
}

// Path: mini_translator.toolbar.tooltip
class _Translations$mini_translator$toolbar$tooltip$ja
    extends Translations$mini_translator$toolbar$tooltip$en {
  _Translations$mini_translator$toolbar$tooltip$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get extract_text_from_screen_capture => '画面領域をキャプチャしてテキストを認識';
  @override
  String get extract_text_from_clipboard => 'クリップボードの内容を読み取る';
  @override
  String get pin => 'ウィンドウを固定';
  @override
  String get more_actions => 'その他の操作';
}

// Path: mini_translator.toolbar.menu
class _Translations$mini_translator$toolbar$menu$ja
    extends Translations$mini_translator$toolbar$menu$en {
  _Translations$mini_translator$toolbar$menu$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get extract_from_screen_capture => '画面から取得';
  @override
  String get extract_from_clipboard => 'クリップボードから取得';
  @override
  String get open_main_window => 'メインウィンドウを開く';
  @override
  String get open_settings => '設定…';
}

// Path: settings.general.section
class _Translations$settings$general$section$ja
    extends Translations$settings$general$section$en {
  _Translations$settings$general$section$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get permissions => 'システム権限';
  @override
  String get ocr => 'テキスト認識';
  @override
  String get directory => '辞書';
  @override
  String get translation => '翻訳';
  @override
  String get translation_target => '翻訳先';
  @override
  String get languages => '言語';
  @override
  String get input => '入力設定';
  @override
  String get startup => '起動と連携';
  @override
  String get ocr_behaviour => '取得の動作';
  @override
  String get translation_behaviour => '翻訳の動作';
}

// Path: settings.general.row
class _Translations$settings$general$row$ja
    extends Translations$settings$general$row$en {
  _Translations$settings$general$row$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get launch_at_login => 'ログイン時に起動';
  @override
  String get show_in_menu_bar => 'メニューバーに表示';
  @override
  String get screen_capture_access => '画面収録へのアクセスを許可';
  @override
  String get screen_selection_access => 'アクセシビリティへのアクセスを許可';
  @override
  String get default_ocr_service => 'デフォルトのテキスト認識サービス';
  @override
  String get auto_copy_detected_text => '検出したテキストを自動コピー';
  @override
  String get default_directory_service => 'デフォルトの辞書サービス';
  @override
  String get default_translation_service => 'デフォルトの翻訳サービス';
  @override
  String get translation_target_hint => '翻訳で使用する言語ペアを設定します。';
  @override
  String get common_languages => 'よく使う言語';
  @override
  String get common_languages_description => 'よく使う言語は言語選択リストの先頭に表示されます。';
  @override
  String get common_languages_hint => 'よく使う言語を選択してください：';
  @override
  String get common_languages_sort => 'コードで並べ替え';
  @override
  String get common_languages_reset => 'デフォルトにリセット';
  @override
  String get common_languages_reset_help => 'デフォルトのよく使う言語セットにリセット';
  @override
  String get common_languages_search => '言語を検索...';
  @override
  String get common_languages_all => 'すべての言語';
  @override
  String get double_click_copy_result => 'ダブルクリックで翻訳結果をコピー';
  @override
  String get submit_with_enter => 'Enterで送信';
  @override
  String get submit_with_meta_enter_mac => '⌘ + Enterで送信';
  @override
  String get screen_capture_access_hint => '画面からの文字取得には画面内容の読み取りが必要です。';
  @override
  String get screen_selection_access_hint => '選択テキストの取得には他アプリの選択内容の読み取りが必要です。';
  @override
  String get no_translation_targets => '翻訳ターゲットがありません。既定の翻訳先を決めるには追加してください。';
}

// Path: settings.general.button
class _Translations$settings$general$button$ja
    extends Translations$settings$general$button$en {
  _Translations$settings$general$button$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get add_provider => '追加...';
  @override
  String get add_target => '対象を追加...';
  @override
  String get manage_targets => '翻訳ターゲットを管理...';
  @override
  String get manage_languages => 'よく使う言語を管理...';
  @override
  String get grant => '許可';
}

// Path: settings.general.option
class _Translations$settings$general$option$ja
    extends Translations$settings$general$option$en {
  _Translations$settings$general$option$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get none => 'なし';
  @override
  String get no_services_available => '利用可能なサービスがありません';
  @override
  String get granted => '許可済み';
  @override
  String get built_in_ocr => '内蔵OCR';
  @override
  String get tesseract => 'Tesseract';
  @override
  String get youdao_ocr => 'Youdao OCR';
}

// Path: settings.general.editor
class _Translations$settings$general$editor$ja
    extends Translations$settings$general$editor$en {
  _Translations$settings$general$editor$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get add_target_title => '翻訳ターゲットを追加';
  @override
  String get edit_target_title => '翻訳ターゲットを編集';
  @override
  late final _Translations$settings$general$editor$row$ja row =
      _Translations$settings$general$editor$row$ja._(_root);
  @override
  String get title_edit => '翻訳ターゲットを編集';
  @override
  String get subtitle => 'ある元言語を既定でどの言語に翻訳するかを決めます';
  @override
  String get same_language => '元言語と翻訳先が同じです。別の翻訳先を選んでください。';
  @override
  String get duplicate => '同じ組み合わせのターゲットがすでにあります。';
  @override
  String get hint_auto => '他のルールに一致しない場合は{}に翻訳します。';
  @override
  String get hint_source => '{}を検出したら{}に翻訳します。';
}

// Path: settings.appearance.section
class _Translations$settings$appearance$section$ja
    extends Translations$settings$appearance$section$en {
  _Translations$settings$appearance$section$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get app_language => '表示言語';
  @override
  String get theme_mode => 'テーマモード';
  @override
  String get theme_style => 'テーマスタイル';
}

// Path: settings.shortcuts.section
class _Translations$settings$shortcuts$section$ja
    extends Translations$settings$shortcuts$section$en {
  _Translations$settings$shortcuts$section$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get text_extraction => 'テキスト抽出';
  @override
  String get input_assist => '入力補助機能';
  @override
  String get submit_mode => '送信方法';
}

// Path: settings.shortcuts.row
class _Translations$settings$shortcuts$row$ja
    extends Translations$settings$shortcuts$row$en {
  _Translations$settings$shortcuts$row$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get toggle_mini_translator => 'ウィンドウの表示/非表示';
  @override
  String get extract_text_from_screen_selection => '画面選択からテキストを抽出';
  @override
  String get extract_text_from_screen_capture => '画面キャプチャからテキストを抽出';
  @override
  String get extract_text_from_clipboard => 'クリップボードからテキストを抽出';
  @override
  String get translate_input => '入力内容を翻訳';
}

// Path: settings.shortcuts.reset_dialog
class _Translations$settings$shortcuts$reset_dialog$ja
    extends Translations$settings$shortcuts$reset_dialog$en {
  _Translations$settings$shortcuts$reset_dialog$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get title => 'ショートカットをリセット';
  @override
  String get message => 'すべてのショートカットをデフォルト値にリセットしますか？';
  @override
  String get confirm => 'リセット';
  @override
  String get cancel => 'キャンセル';
}

// Path: settings.shortcuts.group
class _Translations$settings$shortcuts$group$ja
    extends Translations$settings$shortcuts$group$en {
  _Translations$settings$shortcuts$group$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  late final _Translations$settings$shortcuts$group$global$ja global =
      _Translations$settings$shortcuts$group$global$ja._(_root);
  @override
  late final _Translations$settings$shortcuts$group$in_app$ja in_app =
      _Translations$settings$shortcuts$group$in_app$ja._(_root);
}

// Path: settings.services.button
class _Translations$settings$services$button$ja
    extends Translations$settings$services$button$en {
  _Translations$settings$services$button$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get add_service => 'サービスを追加...';
}

// Path: settings.services.section
class _Translations$settings$services$section$ja
    extends Translations$settings$services$section$en {
  _Translations$settings$services$section$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get available_services => '利用可能なサービス';
}

// Path: settings.services.editor
class _Translations$settings$services$editor$ja
    extends Translations$settings$services$editor$en {
  _Translations$settings$services$editor$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get title => 'サービスを追加';
  @override
  String get subtitle => '設定済みのプロバイダーにサービスを追加します';
  @override
  late final _Translations$settings$services$editor$row$ja row =
      _Translations$settings$services$editor$row$ja._(_root);
  @override
  String get prompt_placeholder => '空欄にすると、この種類の既定プロンプトを使います';
  @override
  String get variant_hint => '{} にはすでに{}サービスがあります。これは並列の別設定として追加されます。';
  @override
  String get traditional_note =>
      '{} は従来型のインターフェースで、調整できるモデルやプロンプトはありません。パラメーターはプロバイダー詳細で設定します。';
}

// Path: settings.services.detail
class _Translations$settings$services$detail$ja
    extends Translations$settings$services$detail$en {
  _Translations$settings$services$detail$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  late final _Translations$settings$services$detail$row$ja row =
      _Translations$settings$services$detail$row$ja._(_root);
  @override
  late final _Translations$settings$services$detail$delete_dialog$ja
  delete_dialog = _Translations$settings$services$detail$delete_dialog$ja._(
    _root,
  );
  @override
  String get prompt_variables =>
      '利用可能な変数: {{sourceLanguage}}, {{targetLanguage}}, {{text}}';
}

// Path: settings.services.item
class _Translations$settings$services$item$ja
    extends Translations$settings$services$item$en {
  _Translations$settings$services$item$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get none_of_kind => '利用できる{}サービスがまだありません。';
}

// Path: settings.providers.section
class _Translations$settings$providers$section$ja
    extends Translations$settings$providers$section$en {
  _Translations$settings$providers$section$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get services => '利用可能なサービス';
  @override
  String get services_description => '設定済みプロバイダーで利用できるサービスを確認し、種類ごとに切り替えます。';
}

// Path: settings.providers.item
class _Translations$settings$providers$item$ja
    extends Translations$settings$providers$item$en {
  _Translations$settings$providers$item$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get empty => 'プロバイダーが設定されていません。追加して翻訳サービスを有効にしてください。';
  @override
  String get loading => 'プロバイダーを読み込み中...';
  @override
  String get no_services => '利用可能なサービスがありません。';
}

// Path: settings.providers.button
class _Translations$settings$providers$button$ja
    extends Translations$settings$providers$button$en {
  _Translations$settings$providers$button$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get add => 'プロバイダーを追加...';
}

// Path: settings.providers.alert
class _Translations$settings$providers$alert$ja
    extends Translations$settings$providers$alert$en {
  _Translations$settings$providers$alert$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get error => 'エラー';
}

// Path: settings.providers.intro
class _Translations$settings$providers$intro$ja
    extends Translations$settings$providers$intro$en {
  _Translations$settings$providers$intro$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get body => 'アプリで使用するサービスプロバイダーを管理します。';
  @override
  String get warning =>
      '接続したプロバイダーは送信したテキストや画像を処理する場合があります。信頼できるサービスのみ有効にしてください。';
}

// Path: settings.providers.editor
class _Translations$settings$providers$editor$ja
    extends Translations$settings$providers$editor$en {
  _Translations$settings$providers$editor$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  late final _Translations$settings$providers$editor$row$ja row =
      _Translations$settings$providers$editor$row$ja._(_root);
  @override
  late final _Translations$settings$providers$editor$placeholder$ja
  placeholder = _Translations$settings$providers$editor$placeholder$ja._(_root);
  @override
  late final _Translations$settings$providers$editor$type_picker$ja
  type_picker = _Translations$settings$providers$editor$type_picker$ja._(_root);
  @override
  late final _Translations$settings$providers$editor$tooltip$ja tooltip =
      _Translations$settings$providers$editor$tooltip$ja._(_root);
  @override
  late final _Translations$settings$providers$editor$step$ja step =
      _Translations$settings$providers$editor$step$ja._(_root);
  @override
  String get add_title => '{} を追加';
  @override
  late final _Translations$settings$providers$editor$capability_note$ja
  capability_note =
      _Translations$settings$providers$editor$capability_note$ja._(_root);
  @override
  late final _Translations$settings$providers$editor$test$ja test =
      _Translations$settings$providers$editor$test$ja._(_root);
}

// Path: settings.providers.detail
class _Translations$settings$providers$detail$ja
    extends Translations$settings$providers$detail$en {
  _Translations$settings$providers$detail$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  late final _Translations$settings$providers$detail$tooltip$ja tooltip =
      _Translations$settings$providers$detail$tooltip$ja._(_root);
  @override
  late final _Translations$settings$providers$detail$row$ja row =
      _Translations$settings$providers$detail$row$ja._(_root);
  @override
  late final _Translations$settings$providers$detail$section$ja section =
      _Translations$settings$providers$detail$section$ja._(_root);
  @override
  late final _Translations$settings$providers$detail$models$ja models =
      _Translations$settings$providers$detail$models$ja._(_root);
}

// Path: settings.providers.capability
class _Translations$settings$providers$capability$ja
    extends Translations$settings$providers$capability$en {
  _Translations$settings$providers$capability$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get translation => '翻訳';
  @override
  String get dictionary => '辞書';
  @override
  String get ocr => 'OCR';
  @override
  String get llm => 'AI';
}

// Path: settings.providers.description
class _Translations$settings$providers$description$ja
    extends Translations$settings$providers$description$en {
  _Translations$settings$providers$description$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get all => '辞書検索とテキスト翻訳を提供';
  @override
  String get dictionary => '辞書検索と単語定義を提供';
  @override
  String get translation => '言語間のテキスト翻訳を提供';
  @override
  String get fallback => '翻訳サービスを提供';
}

// Path: settings.providers.delete_dialog
class _Translations$settings$providers$delete_dialog$ja
    extends Translations$settings$providers$delete_dialog$en {
  _Translations$settings$providers$delete_dialog$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get title => '"{}" を削除しますか？';
  @override
  String get message => 'この操作は元に戻せません。';
}

// Path: settings.layout.empty
class _Translations$settings$layout$empty$ja
    extends Translations$settings$layout$empty$en {
  _Translations$settings$layout$empty$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get title => 'カテゴリを選択';
  @override
  String get message => 'サイドバーから設定セクションを選択してください。';
}

// Path: app.tray.context_menu.dev_tools
class _Translations$app$tray$context_menu$dev_tools$ja
    extends Translations$app$tray$context_menu$dev_tools$en {
  _Translations$app$tray$context_menu$dev_tools$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get title => '開発ツール';
  @override
  String get open_data_directory => 'データディレクトリを開く';
}

// Path: settings.general.editor.row
class _Translations$settings$general$editor$row$ja
    extends Translations$settings$general$editor$row$en {
  _Translations$settings$general$editor$row$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get source_language => 'ソース言語';
  @override
  String get target_language => 'ターゲット言語';
}

// Path: settings.shortcuts.group.global
class _Translations$settings$shortcuts$group$global$ja
    extends Translations$settings$shortcuts$group$global$en {
  _Translations$settings$shortcuts$group$global$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get title => 'グローバルショートカット';
  @override
  String get description => 'どのアプリでも動作します。';
}

// Path: settings.shortcuts.group.in_app
class _Translations$settings$shortcuts$group$in_app$ja
    extends Translations$settings$shortcuts$group$in_app$en {
  _Translations$settings$shortcuts$group$in_app$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get title => 'アプリ内キー';
  @override
  String get description => 'このアプリの入力欄でのみ有効です。';
}

// Path: settings.services.editor.row
class _Translations$settings$services$editor$row$ja
    extends Translations$settings$services$editor$row$en {
  _Translations$settings$services$editor$row$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get model => 'モデル';
  @override
  String get system_prompt => 'システムプロンプト';
}

// Path: settings.services.detail.row
class _Translations$settings$services$detail$row$ja
    extends Translations$settings$services$detail$row$en {
  _Translations$settings$services$detail$row$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get id => 'サービス ID';
  @override
  String get name => '名前';
  @override
  String get provider => 'プロバイダー';
  @override
  String get type => '種類';
}

// Path: settings.services.detail.delete_dialog
class _Translations$settings$services$detail$delete_dialog$ja
    extends Translations$settings$services$detail$delete_dialog$en {
  _Translations$settings$services$detail$delete_dialog$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get title => '「{}」を削除しますか？';
  @override
  String get message => 'このサービスはプロバイダーから削除されます。';
}

// Path: settings.providers.editor.row
class _Translations$settings$providers$editor$row$ja
    extends Translations$settings$providers$editor$row$en {
  _Translations$settings$providers$editor$row$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get id => 'プロバイダーID';
  @override
  String get type => 'プロバイダーの種類';
  @override
  String get default_model => 'デフォルトモデル';
}

// Path: settings.providers.editor.placeholder
class _Translations$settings$providers$editor$placeholder$ja
    extends Translations$settings$providers$editor$placeholder$en {
  _Translations$settings$providers$editor$placeholder$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get id => '例: deepl-main';
}

// Path: settings.providers.editor.type_picker
class _Translations$settings$providers$editor$type_picker$ja
    extends Translations$settings$providers$editor$type_picker$en {
  _Translations$settings$providers$editor$type_picker$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get prompt => '追加するプロバイダーの種類を選択してください：';
  @override
  String get section_llm => 'LLM';
  @override
  String get section_traditional => '従来型';
}

// Path: settings.providers.editor.tooltip
class _Translations$settings$providers$editor$tooltip$ja
    extends Translations$settings$providers$editor$tooltip$en {
  _Translations$settings$providers$editor$tooltip$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get help => 'ヘルプ';
}

// Path: settings.providers.editor.step
class _Translations$settings$providers$editor$step$ja
    extends Translations$settings$providers$editor$step$en {
  _Translations$settings$providers$editor$step$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get next => '続ける';
  @override
  String get back => '前へ';
}

// Path: settings.providers.editor.capability_note
class _Translations$settings$providers$editor$capability_note$ja
    extends Translations$settings$providers$editor$capability_note$en {
  _Translations$settings$providers$editor$capability_note$ja._(
    TranslationsJa root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get translation => '翻訳候補に加わります';
  @override
  String get dictionary => '辞書の語義を提供します';
  @override
  String get ocr => '画像内の文字を認識します';
}

// Path: settings.providers.editor.test
class _Translations$settings$providers$editor$test$ja
    extends Translations$settings$providers$editor$test$en {
  _Translations$settings$providers$editor$test$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get run => '接続をテスト';
  @override
  String get running => '接続をテスト中 · 経過 {}s';
  @override
  String get passed_models => '接続は正常 · {} 個のモデルが利用可能';
  @override
  String get passed_service => '接続は正常 · サービスを利用できます';
  @override
  String get passed_footer => '接続テストに合格しました';
  @override
  String get retest => '再テスト';
  @override
  String get tips_title => '試せること';
  @override
  String get tips_llm =>
      '· キーが選んだプロバイダーの種類と一致しているか確認\n· Base URL に /v1 が必要か確認\n· プロバイダーのコンソールでそのモデルが有効か確認';
  @override
  String get tips_traditional =>
      '· 認証情報が選んだプロバイダーの種類と一致しているか確認\n· プロバイダーのコンソールでサービスが有効か確認';
  @override
  String get failed_suffix => '検証に失敗';
  @override
  String get passed_suffix => '検証済み';
}

// Path: settings.providers.detail.tooltip
class _Translations$settings$providers$detail$tooltip$ja
    extends Translations$settings$providers$detail$tooltip$en {
  _Translations$settings$providers$detail$tooltip$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get edit => 'プロバイダーを編集';
}

// Path: settings.providers.detail.row
class _Translations$settings$providers$detail$row$ja
    extends Translations$settings$providers$detail$row$en {
  _Translations$settings$providers$detail$row$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get id_hint => '作成後は変更できません';
}

// Path: settings.providers.detail.section
class _Translations$settings$providers$detail$section$ja
    extends Translations$settings$providers$detail$section$en {
  _Translations$settings$providers$detail$section$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get configuration => '設定';
  @override
  String get models => 'モデル';
}

// Path: settings.providers.detail.models
class _Translations$settings$providers$detail$models$ja
    extends Translations$settings$providers$detail$models$en {
  _Translations$settings$providers$detail$models$ja._(TranslationsJa root)
    : this._root = root,
      super.internal(root);

  final TranslationsJa _root; // ignore: unused_field

  // Translations
  @override
  String get loading => 'モデルを読み込み中...';
  @override
  String get empty => 'モデルが見つかりません。';
  @override
  String get retry => '再試行';
  @override
  String get refresh => 'リストを更新';
  @override
  String get default_badge => 'デフォルト';
  @override
  String get set_default => 'デフォルトに設定';
  @override
  String get fetch_error => 'プロバイダーAPIからモデルを取得できませんでした。';
}

/// The flat map containing all translations for locale <ja>.
/// Only for edge cases! For simple maps, use the map function of this library.
///
/// The Dart AOT compiler has issues with very large switch statements,
/// so the map is split into smaller functions (512 entries each).
extension on TranslationsJa {
  dynamic _flatMapFunction(String path) {
    return switch (path) {
      'common.ui.button.ok' => 'OK',
      'common.ui.button.cancel' => 'キャンセル',
      'common.ui.button.add' => '追加',
      'common.ui.button.delete' => '削除',
      'common.ui.button.edit' => '編集',
      'common.ui.button.save' => '保存',
      'common.ui.button.manage' => '管理',
      'common.ui.button.kContinue' => '続行',
      'common.ui.feedback.copied' => 'コピーしました',
      'common.language.ar' => 'アラビア語',
      'common.language.bn' => 'ベンガル語',
      'common.language.de' => 'ドイツ語',
      'common.language.en' => '英語',
      'common.language.es' => 'スペイン語',
      'common.language.fa' => 'ペルシア語',
      'common.language.fr' => 'フランス語',
      'common.language.gu' => 'グジャラート語',
      'common.language.ha' => 'ハウサ語',
      'common.language.hi' => 'ヒンディー語',
      'common.language.id' => 'インドネシア語',
      'common.language.it' => 'イタリア語',
      'common.language.ja' => '日本語',
      'common.language.jv' => 'ジャワ語',
      'common.language.ko' => '韓国語',
      'common.language.ml' => 'マラヤーラム語',
      'common.language.mr' => 'マラーティー語',
      'common.language.ms' => 'マレー語',
      'common.language.nl' => 'オランダ語',
      'common.language.pa' => 'パンジャブ語',
      'common.language.pl' => 'ポーランド語',
      'common.language.pt' => 'ポルトガル語',
      'common.language.ro' => 'ルーマニア語',
      'common.language.ru' => 'ロシア語',
      'common.language.sw' => 'スワヒリ語',
      'common.language.ta' => 'タミル語',
      'common.language.te' => 'テルグ語',
      'common.language.th' => 'タイ語',
      'common.language.tr' => 'トルコ語',
      'common.language.uk' => 'ウクライナ語',
      'common.language.ur' => 'ウルドゥー語',
      'common.language.vi' => 'ベトナム語',
      'common.language.yo' => 'ヨルバ語',
      'common.language.zh_hans' => '簡体字中国語',
      'common.language.zh_hant' => '繁体字中国語',
      'common.theme_mode.light' => 'ライト',
      'common.theme_mode.dark' => 'ダーク',
      'common.theme_mode.system' => 'システム',
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
      'common.provider.system' => 'システム',
      'common.provider.tencent' => 'Tencent',
      'common.provider.youdao' => 'Youda',
      'app.tray.context_menu.show_window' => 'ウィンドウを表示',
      'app.tray.context_menu.dev_tools.title' => '開発ツール',
      'app.tray.context_menu.dev_tools.open_data_directory' => 'データディレクトリを開く',
      'app.tray.context_menu.check_for_updates' => 'アップデートを確認',
      'app.tray.context_menu.settings' => '設定',
      'app.tray.context_menu.quit' => '終了',
      'mini_translator.limited_banner.permission.missing_both' =>
        'すべての機能を有効にするには、画面収録とアクセシビリティの許可を付与してください。',
      'mini_translator.limited_banner.permission.missing_screen_capture' =>
        'すべての機能を有効にするには、画面収録の許可を付与してください。',
      'mini_translator.limited_banner.permission.missing_accessibility' =>
        'すべての機能を有効にするには、アクセシビリティの許可を付与してください。',
      'mini_translator.limited_banner.instruction.app_settings_prefix' =>
        'アプリ設定に移動',
      'mini_translator.limited_banner.instruction.follow_guide_prefix' =>
        'し、ガイドに従って',
      'mini_translator.limited_banner.instruction.suffix' => 'をクリックしてください。',
      'mini_translator.limited_banner.action.app_settings' => 'アプリ設定',
      'mini_translator.limited_banner.action.recheck' => '再確認',
      'mini_translator.limited_banner.feedback.enabled' => '画面テキスト抽出が有効です。',
      'mini_translator.limited_banner.feedback.still_missing' =>
        '必要な権限がまだ不足しています。\n設定を確認して、もう一度お試しください。',
      'mini_translator.limited_banner.tooltip.help' => 'ヘルプを表示',
      'mini_translator.input.hint' => 'ここに単語またはテキストを入力',
      'mini_translator.input.extracting_text' => 'テキストを抽出中...',
      'mini_translator.input.hint_translate_to' => ({
        required Object language,
      }) => '単語またはテキストを入力して${language}に翻訳',
      'mini_translator.toolbar.tooltip.extract_text_from_screen_capture' =>
        '画面領域をキャプチャしてテキストを認識',
      'mini_translator.toolbar.tooltip.extract_text_from_clipboard' =>
        'クリップボードの内容を読み取る',
      'mini_translator.toolbar.tooltip.pin' => 'ウィンドウを固定',
      'mini_translator.toolbar.tooltip.more_actions' => 'その他の操作',
      'mini_translator.toolbar.menu.extract_from_screen_capture' => '画面から取得',
      'mini_translator.toolbar.menu.extract_from_clipboard' => 'クリップボードから取得',
      'mini_translator.toolbar.menu.open_main_window' => 'メインウィンドウを開く',
      'mini_translator.toolbar.menu.open_settings' => '設定…',
      'mini_translator.button.clear' => 'クリア',
      'mini_translator.button.translate' => '翻訳',
      'mini_translator.button.copy' => 'コピー',
      'mini_translator.button.copied' => 'コピー済み',
      'mini_translator.button.bookmark' => '保存',
      'mini_translator.button.bookmarked' => '保存済み',
      'mini_translator.language.auto_detect' => '自動検出',
      'mini_translator.language.auto_match' => '自動一致',
      'mini_translator.language.switch_config' => 'ターゲット切替',
      'mini_translator.language.more_languages' => 'その他の言語...',
      'mini_translator.language.manage_common_languages' => 'よく使う言語を管理...',
      'mini_translator.language.manage_targets' => '翻訳ターゲットを管理...',
      'mini_translator.language.add_target' => '翻訳ターゲットを追加...',
      'mini_translator.message.please_enter_word_or_text' =>
        'テキストが入力されていないか、テキストが抽出されていません',
      'mini_translator.message.capture_screen_area_canceled' =>
        '画面領域のキャプチャがキャンセルされました',
      'mini_translator.message.ocr_service_not_configured' =>
        'デフォルトのテキスト認識サービスが設定されていません。設定で設定してください。',
      'mini_translator.message.ocr_recognition_failed' => 'テキスト認識に失敗しました',
      'mini_translator.result.translating' => '翻訳中…',
      'mini_translator.result.stale_requery' => ({
        required Object key,
      }) => '原文が変更されました · ${key} 再翻訳',
      'mini_translator.result.compare_services' => ({
        required Object count,
      }) => '${count} 個のサービスを比較',
      'mini_translator.result.collapse_compare' => '比較を閉じる',
      'mini_translator.result.set_preferred' => '優先に設定',
      'mini_translator.result.retry' => '再試行',
      'mini_translator.result.no_result' =>
        'どのサービスも結果を返しませんでした。ネットワークを確認するか、別のサービスをお試しください。',
      'mini_translator.result.no_result_note' => '原文は保持され、再試行しても履歴は重複しません。',
      'workbench.workspace' => 'ワークスペース',
      'workbench.translate' => '翻訳',
      'workbench.history' => '履歴',
      'workbench.history_page.all' => 'すべて',
      'workbench.history_page.favorites' => 'お気に入り',
      'workbench.history_page.edited' => '編集済み',
      'workbench.history_page.search' => '検索',
      'workbench.history_page.search_placeholder' => '原文、訳文、サービスを検索',
      'workbench.history_page.search_label' => '履歴を検索',
      'workbench.history_page.entry_count' => ({
        required Object label,
        required Object count,
      }) => '${label} · ${count} 件',
      'workbench.history_page.by_time' => '時間順',
      'workbench.history_page.loading' => '履歴を読み込み中…',
      'workbench.history_page.load_failed' => '履歴を読み込めませんでした',
      'workbench.history_page.retry' => '再試行',
      'workbench.history_page.empty_title' => '翻訳履歴はまだありません',
      'workbench.history_page.empty_description' => '翻訳が完了すると、優先訳がここに保存されます。',
      'workbench.history_page.no_results' => ({
        required Object query,
      }) => '「${query}」に一致する履歴はありません',
      'workbench.history_page.clear_search' => '検索をクリア',
      'workbench.history_page.select' => '複数選択',
      'workbench.history_page.selected_count' => ({
        required Object count,
      }) => '${count} 件選択中',
      'workbench.history_page.exit_select' => '選択を終了',
      'workbench.history_page.add_to_glossary' => '用語集に追加',
      'workbench.history_page.favorite' => 'お気に入り',
      'workbench.history_page.unfavorite' => 'お気に入り解除',
      'workbench.history_page.delete_confirm' => ({
        required Object count,
      }) => '選択した ${count} 件の履歴を削除しますか？元に戻せません。',
      'workbench.history_page.no_glossary' => '先に用語集を作成してください',
      'workbench.history_page.added_to_glossary' => ({
        required Object count,
      }) => '${count} 件を用語集に追加しました',
      'workbench.history_page.favorite_flag' => 'お気に入り',
      'workbench.history_page.edited_flag' => '編集済み',
      'workbench.history_page.edit_history_hint' => '編集した訳文は履歴に保存されます',
      'workbench.glossary' => '用語集',
      'workbench.recent_languages' => '最近の言語',
      'workbench.not_configured' => '未設定',
      'workbench.subtitle.translate' => 'ワークベンチ · サービス比較',
      'workbench.subtitle.settings' => '設定',
      'workbench.placeholder.history' => '以前の翻訳を確認、編集、お気に入り登録して再利用します',
      'workbench.placeholder.glossary' => '指定訳と禁止する別訳を管理します',
      'workbench.glossary_page.add_entry' => '用語を追加',
      'workbench.glossary_page.term' => '原文',
      'workbench.glossary_page.translation' => '指定訳',
      'workbench.glossary_page.forbidden' => '禁止',
      'workbench.glossary_page.hits' => 'ヒット',
      'workbench.glossary_page.term_placeholder' => 'teacher forcing',
      'workbench.glossary_page.translation_placeholder' => '教師強制',
      'workbench.glossary_page.forbidden_placeholder' => '強制教育',
      'workbench.glossary_page.search' => '検索',
      'workbench.glossary_page.search_placeholder' => '用語または指定訳を検索',
      'workbench.glossary_page.search_label' => '用語集を検索',
      'workbench.glossary_page.entry_count' => ({
        required Object name,
        required Object count,
      }) => '${name} · ${count} 件',
      'workbench.glossary_page.priority_note' => '用語集はどのサービスの出力よりも優先されます',
      'workbench.glossary_page.new_book' => '用語集を作成',
      'workbench.glossary_page.new_book_placeholder' => '用語集の名前',
      'workbench.glossary_page.rename_book' => '名前を変更',
      'workbench.glossary_page.delete_book_confirm' => ({
        required Object name,
        required Object count,
      }) => '「${name}」と、その ${count} 件の用語を削除しますか？',
      'workbench.glossary_page.disabled' => '無効',
      'workbench.glossary_page.enable' => '有効にする',
      'workbench.glossary_page.disable' => '無効にする',
      'workbench.glossary_page.empty_title' => 'この用語集は空です',
      'workbench.glossary_page.empty_description' =>
        '用語集はどのサービスの出力よりも優先されます。1 件ずつ追加するか、CSV をドロップして取り込めます。',
      'workbench.glossary_page.no_results_title' => ({
        required Object query,
      }) => '「${query}」に一致する用語はありません',
      'workbench.glossary_page.no_results_description' =>
        '別のキーワードを試すか、新しく追加してください。',
      'workbench.glossary_page.no_books_title' => '用語集がまだありません',
      'workbench.glossary_page.no_books_description' =>
        '用語集は、選んだ訳語をすべてのサービスで統一します。まず 1 つ作成して、用語を追加しましょう。',
      'workbench.glossary_page.loading' => '読み込み中…',
      'workbench.translation.source' => '原文',
      'workbench.translation.target' => '訳文',
      'workbench.translation.input_hint' => '翻訳するテキストを入力または貼り付け',
      'workbench.translation.button' => '翻訳',
      'workbench.translation.auto_detected' => '自動検出',
      'workbench.translation.loading_services' => '翻訳サービスを読み込み中…',
      'workbench.translation.no_services' => '先に設定で翻訳サービスを構成してください',
      'workbench.translation.translating' => '翻訳中…',
      'workbench.translation.failed' => '翻訳に失敗しました。サービス設定を確認してください。',
      'workbench.translation.empty' => '訳文がここに表示されます',
      'workbench.translation.service_compare' => 'サービス比較',
      'workbench.translation.main_translation' => 'メイン',
      'workbench.translation.service_unavailable' => 'サービスを利用できません',
      'workbench.translation.waiting' => '翻訳待ち',
      'workbench.translation.copy' => 'コピー',
      'workbench.translation.favorite_unavailable' =>
        'お気に入りを更新できませんでした。もう一度お試しください。',
      'workbench.translation.preferred' => '優先訳文',
      'workbench.translation.other_services' => '他のサービス',
      'workbench.translation.copy_result' => '訳文をコピー',
      'workbench.translation.copied' => 'コピー済み',
      'workbench.translation.favorite' => '保存',
      'workbench.translation.terms' => '用語ヒット',
      'workbench.translation.terms_hint' => '入力すると用語集と照合されます。',
      'workbench.translation.quality' => '品質シグナル',
      'workbench.translation.quality_hint' => '訳文の生成後に計算されます。',
      'workbench.translation.shortcuts' => 'ショートカット',
      'workbench.translation.other_services_disabled' => '他のサービスは無効です',
      'workbench.translation.input_hint_translate_to' => ({
        required Object language,
      }) => '翻訳するテキストを入力または貼り付け、${language}に翻訳',
      'workbench.translation.clear' => 'クリア',
      'workbench.translation.swap_languages' => '言語を入れ替える',
      'workbench.translation.services' => '翻訳サービス',
      'workbench.translation.configure_services' => 'サービスを設定',
      'workbench.translation.retry' => '再試行',
      'workbench.translation.character_count' => ({
        required Object count,
      }) => '${count} 文字',
      'workbench.status.runtime_ready' => '翻訳ランタイム準備完了',
      'workbench.status.settings_synced' => '設定を同期済み',
      'workbench.status.shortcuts' => '⌥Space クイックウィンドウ · ⌥⇧2 キャプチャ',
      'workbench.version_latest' => '最新です',
      'workbench.version_checking' => '確認中…',
      'workbench.check_updates' => 'アップデートを確認',
      'settings.version' => 'v{} (Build {})',
      'settings.general.title' => '一般',
      'settings.general.section.permissions' => 'システム権限',
      'settings.general.section.ocr' => 'テキスト認識',
      'settings.general.section.directory' => '辞書',
      'settings.general.section.translation' => '翻訳',
      'settings.general.section.translation_target' => '翻訳先',
      'settings.general.section.languages' => '言語',
      'settings.general.section.input' => '入力設定',
      'settings.general.section.startup' => '起動と連携',
      'settings.general.section.ocr_behaviour' => '取得の動作',
      'settings.general.section.translation_behaviour' => '翻訳の動作',
      'settings.general.row.launch_at_login' => 'ログイン時に起動',
      'settings.general.row.show_in_menu_bar' => 'メニューバーに表示',
      'settings.general.row.screen_capture_access' => '画面収録へのアクセスを許可',
      'settings.general.row.screen_selection_access' => 'アクセシビリティへのアクセスを許可',
      'settings.general.row.default_ocr_service' => 'デフォルトのテキスト認識サービス',
      'settings.general.row.auto_copy_detected_text' => '検出したテキストを自動コピー',
      'settings.general.row.default_directory_service' => 'デフォルトの辞書サービス',
      'settings.general.row.default_translation_service' => 'デフォルトの翻訳サービス',
      'settings.general.row.translation_target_hint' => '翻訳で使用する言語ペアを設定します。',
      'settings.general.row.common_languages' => 'よく使う言語',
      'settings.general.row.common_languages_description' =>
        'よく使う言語は言語選択リストの先頭に表示されます。',
      'settings.general.row.common_languages_hint' => 'よく使う言語を選択してください：',
      'settings.general.row.common_languages_sort' => 'コードで並べ替え',
      'settings.general.row.common_languages_reset' => 'デフォルトにリセット',
      'settings.general.row.common_languages_reset_help' =>
        'デフォルトのよく使う言語セットにリセット',
      'settings.general.row.common_languages_search' => '言語を検索...',
      'settings.general.row.common_languages_all' => 'すべての言語',
      'settings.general.row.double_click_copy_result' => 'ダブルクリックで翻訳結果をコピー',
      'settings.general.row.submit_with_enter' => 'Enterで送信',
      'settings.general.row.submit_with_meta_enter_mac' => '⌘ + Enterで送信',
      'settings.general.row.screen_capture_access_hint' =>
        '画面からの文字取得には画面内容の読み取りが必要です。',
      'settings.general.row.screen_selection_access_hint' =>
        '選択テキストの取得には他アプリの選択内容の読み取りが必要です。',
      'settings.general.row.no_translation_targets' =>
        '翻訳ターゲットがありません。既定の翻訳先を決めるには追加してください。',
      'settings.general.button.add_provider' => '追加...',
      'settings.general.button.add_target' => '対象を追加...',
      'settings.general.button.manage_targets' => '翻訳ターゲットを管理...',
      'settings.general.button.manage_languages' => 'よく使う言語を管理...',
      'settings.general.button.grant' => '許可',
      'settings.general.option.none' => 'なし',
      'settings.general.option.no_services_available' => '利用可能なサービスがありません',
      'settings.general.option.granted' => '許可済み',
      'settings.general.option.built_in_ocr' => '内蔵OCR',
      'settings.general.option.tesseract' => 'Tesseract',
      'settings.general.option.youdao_ocr' => 'Youdao OCR',
      'settings.general.editor.add_target_title' => '翻訳ターゲットを追加',
      'settings.general.editor.edit_target_title' => '翻訳ターゲットを編集',
      'settings.general.editor.row.source_language' => 'ソース言語',
      'settings.general.editor.row.target_language' => 'ターゲット言語',
      'settings.general.editor.title_edit' => '翻訳ターゲットを編集',
      'settings.general.editor.subtitle' => 'ある元言語を既定でどの言語に翻訳するかを決めます',
      'settings.general.editor.same_language' => '元言語と翻訳先が同じです。別の翻訳先を選んでください。',
      'settings.general.editor.duplicate' => '同じ組み合わせのターゲットがすでにあります。',
      'settings.general.editor.hint_auto' => '他のルールに一致しない場合は{}に翻訳します。',
      'settings.general.editor.hint_source' => '{}を検出したら{}に翻訳します。',
      'settings.appearance.title' => '外観',
      'settings.appearance.section.app_language' => '表示言語',
      'settings.appearance.section.theme_mode' => 'テーマモード',
      'settings.appearance.section.theme_style' => 'テーマスタイル',
      'settings.appearance.footer' => '変更はウインドウ全体にすぐ適用されます。',
      'settings.shortcuts.title' => 'ショートカット',
      'settings.shortcuts.section.text_extraction' => 'テキスト抽出',
      'settings.shortcuts.section.input_assist' => '入力補助機能',
      'settings.shortcuts.section.submit_mode' => '送信方法',
      'settings.shortcuts.row.toggle_mini_translator' => 'ウィンドウの表示/非表示',
      'settings.shortcuts.row.extract_text_from_screen_selection' =>
        '画面選択からテキストを抽出',
      'settings.shortcuts.row.extract_text_from_screen_capture' =>
        '画面キャプチャからテキストを抽出',
      'settings.shortcuts.row.extract_text_from_clipboard' =>
        'クリップボードからテキストを抽出',
      'settings.shortcuts.row.translate_input' => '入力内容を翻訳',
      'settings.shortcuts.reset_dialog.title' => 'ショートカットをリセット',
      'settings.shortcuts.reset_dialog.message' =>
        'すべてのショートカットをデフォルト値にリセットしますか？',
      'settings.shortcuts.reset_dialog.confirm' => 'リセット',
      'settings.shortcuts.reset_dialog.cancel' => 'キャンセル',
      'settings.shortcuts.group.global.title' => 'グローバルショートカット',
      'settings.shortcuts.group.global.description' => 'どのアプリでも動作します。',
      'settings.shortcuts.group.in_app.title' => 'アプリ内キー',
      'settings.shortcuts.group.in_app.description' => 'このアプリの入力欄でのみ有効です。',
      'settings.shortcuts.reset' => 'デフォルトに戻す...',
      'settings.advanced.title' => '詳細設定',
      'settings.advanced.api_server' => 'ローカルAPIサーバー',
      'settings.advanced.api_server_description' =>
        'ローカルインテグレーションのために127.0.0.1で翻訳APIを公開します。',
      'settings.advanced.enable' => '有効',
      'settings.advanced.port' => 'ポート',
      'settings.advanced.running_at' => '{url} で実行中',
      'settings.advanced.disabled' => '無効',
      'settings.services.title' => 'サービス',
      'settings.services.button.add_service' => 'サービスを追加...',
      'settings.services.section.available_services' => '利用可能なサービス',
      'settings.services.editor.title' => 'サービスを追加',
      'settings.services.editor.subtitle' => '設定済みのプロバイダーにサービスを追加します',
      'settings.services.editor.row.model' => 'モデル',
      'settings.services.editor.row.system_prompt' => 'システムプロンプト',
      'settings.services.editor.prompt_placeholder' =>
        '空欄にすると、この種類の既定プロンプトを使います',
      'settings.services.editor.variant_hint' =>
        '{} にはすでに{}サービスがあります。これは並列の別設定として追加されます。',
      'settings.services.editor.traditional_note' =>
        '{} は従来型のインターフェースで、調整できるモデルやプロンプトはありません。パラメーターはプロバイダー詳細で設定します。',
      'settings.services.detail.row.id' => 'サービス ID',
      'settings.services.detail.row.name' => '名前',
      'settings.services.detail.row.provider' => 'プロバイダー',
      'settings.services.detail.row.type' => '種類',
      'settings.services.detail.delete_dialog.title' => '「{}」を削除しますか？',
      'settings.services.detail.delete_dialog.message' =>
        'このサービスはプロバイダーから削除されます。',
      'settings.services.detail.prompt_variables' =>
        '利用可能な変数: {{sourceLanguage}}, {{targetLanguage}}, {{text}}',
      'settings.services.make_default' => 'デフォルトにする',
      'settings.services.item.none_of_kind' => '利用できる{}サービスがまだありません。',
      'settings.providers.title' => 'プロバイダー',
      'settings.providers.section.services' => '利用可能なサービス',
      'settings.providers.section.services_description' =>
        '設定済みプロバイダーで利用できるサービスを確認し、種類ごとに切り替えます。',
      'settings.providers.item.empty' =>
        'プロバイダーが設定されていません。追加して翻訳サービスを有効にしてください。',
      'settings.providers.item.loading' => 'プロバイダーを読み込み中...',
      'settings.providers.item.no_services' => '利用可能なサービスがありません。',
      'settings.providers.button.add' => 'プロバイダーを追加...',
      'settings.providers.alert.error' => 'エラー',
      'settings.providers.intro.body' => 'アプリで使用するサービスプロバイダーを管理します。',
      'settings.providers.intro.warning' =>
        '接続したプロバイダーは送信したテキストや画像を処理する場合があります。信頼できるサービスのみ有効にしてください。',
      'settings.providers.editor.row.id' => 'プロバイダーID',
      'settings.providers.editor.row.type' => 'プロバイダーの種類',
      'settings.providers.editor.row.default_model' => 'デフォルトモデル',
      'settings.providers.editor.placeholder.id' => '例: deepl-main',
      'settings.providers.editor.type_picker.prompt' =>
        '追加するプロバイダーの種類を選択してください：',
      'settings.providers.editor.type_picker.section_llm' => 'LLM',
      'settings.providers.editor.type_picker.section_traditional' => '従来型',
      'settings.providers.editor.tooltip.help' => 'ヘルプ',
      'settings.providers.editor.step.next' => '続ける',
      'settings.providers.editor.step.back' => '前へ',
      'settings.providers.editor.add_title' => '{} を追加',
      'settings.providers.editor.capability_note.translation' => '翻訳候補に加わります',
      'settings.providers.editor.capability_note.dictionary' => '辞書の語義を提供します',
      'settings.providers.editor.capability_note.ocr' => '画像内の文字を認識します',
      'settings.providers.editor.test.run' => '接続をテスト',
      'settings.providers.editor.test.running' => '接続をテスト中 · 経過 {}s',
      'settings.providers.editor.test.passed_models' => '接続は正常 · {} 個のモデルが利用可能',
      'settings.providers.editor.test.passed_service' => '接続は正常 · サービスを利用できます',
      'settings.providers.editor.test.passed_footer' => '接続テストに合格しました',
      'settings.providers.editor.test.retest' => '再テスト',
      'settings.providers.editor.test.tips_title' => '試せること',
      'settings.providers.editor.test.tips_llm' => '· キーが選んだプロバイダーの種類と一致しているか確認\n· Base URL に /v1 が必要か確認\n· プロバイダーのコンソールでそのモデルが有効か確認',
      'settings.providers.editor.test.tips_traditional' =>
        '· 認証情報が選んだプロバイダーの種類と一致しているか確認\n· プロバイダーのコンソールでサービスが有効か確認',
      'settings.providers.editor.test.failed_suffix' => '検証に失敗',
      'settings.providers.editor.test.passed_suffix' => '検証済み',
      'settings.providers.detail.tooltip.edit' => 'プロバイダーを編集',
      'settings.providers.detail.row.id_hint' => '作成後は変更できません',
      'settings.providers.detail.section.configuration' => '設定',
      'settings.providers.detail.section.models' => 'モデル',
      'settings.providers.detail.models.loading' => 'モデルを読み込み中...',
      'settings.providers.detail.models.empty' => 'モデルが見つかりません。',
      'settings.providers.detail.models.retry' => '再試行',
      'settings.providers.detail.models.refresh' => 'リストを更新',
      'settings.providers.detail.models.default_badge' => 'デフォルト',
      'settings.providers.detail.models.set_default' => 'デフォルトに設定',
      'settings.providers.detail.models.fetch_error' =>
        'プロバイダーAPIからモデルを取得できませんでした。',
      'settings.providers.capability.translation' => '翻訳',
      'settings.providers.capability.dictionary' => '辞書',
      'settings.providers.capability.ocr' => 'OCR',
      'settings.providers.capability.llm' => 'AI',
      'settings.providers.description.all' => '辞書検索とテキスト翻訳を提供',
      'settings.providers.description.dictionary' => '辞書検索と単語定義を提供',
      'settings.providers.description.translation' => '言語間のテキスト翻訳を提供',
      'settings.providers.description.fallback' => '翻訳サービスを提供',
      'settings.providers.delete_dialog.title' => '"{}" を削除しますか？',
      'settings.providers.delete_dialog.message' => 'この操作は元に戻せません。',
      'settings.layout.title' => '設定',
      'settings.layout.empty.title' => 'カテゴリを選択',
      'settings.layout.empty.message' => 'サイドバーから設定セクションを選択してください。',
      'settings.layout.groups' => '設定グループ',
      'settings.layout.effect_hint' => '変更は即時反映',
      'settings.layout.footer_note' => '訳文とキーはこのMacにのみ保存されます',
      'settings.layout.support' => 'サポート',
      'settings.about.title' => '情報',
      'settings.about.copy_version_info' => 'バージョン情報をコピー',
      'settings.about.up_to_date' => '最新の状態です。',
      'settings.about.check_again' => '再確認',
      'settings.about.links' => 'リンク',
      'settings.about.website' => 'ウェブサイト',
      'settings.about.github' => 'GitHub',
      'settings.about.report_issue' => '問題を報告',
      'settings.about.license' => 'ライセンス',
      'settings.about.open_changelog' => '更新履歴を開く',
      'settings.about.update' => 'アップデート',
      _ => null,
    };
  }
}
