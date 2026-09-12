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
class TranslationsFr extends Translations
    with BaseTranslations<AppLocale, Translations> {
  /// You can call this constructor and build your own translation instance of this locale.
  /// Constructing via the enum [AppLocale.build] is preferred.
  TranslationsFr({
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
             locale: AppLocale.fr,
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

  /// Metadata for the translations of <fr>.
  final TranslationMetadata<AppLocale, Translations> _meta;
  @override
  TranslationMetadata<AppLocale, Translations> get $meta => _meta;

  /// Access flat map
  @override
  dynamic operator [](String key) => _meta.getTranslation(key) ?? super[key];

  late final TranslationsFr _root = this; // ignore: unused_field

  @override
  TranslationsFr $copyWith({
    TranslationMetadata<AppLocale, Translations>? meta,
  }) => TranslationsFr(meta: meta ?? this.$meta);

  // Translations
  @override
  late final _Translations$common$fr common = _Translations$common$fr._(_root);
  @override
  late final _Translations$app$fr app = _Translations$app$fr._(_root);
  @override
  late final _Translations$mini_translator$fr mini_translator =
      _Translations$mini_translator$fr._(_root);
  @override
  late final _Translations$workbench$fr workbench =
      _Translations$workbench$fr._(_root);
  @override
  late final _Translations$settings$fr settings = _Translations$settings$fr._(
    _root,
  );
}

// Path: common
class _Translations$common$fr extends Translations$common$en {
  _Translations$common$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  late final _Translations$common$ui$fr ui = _Translations$common$ui$fr._(
    _root,
  );
  @override
  late final _Translations$common$language$fr language =
      _Translations$common$language$fr._(_root);
  @override
  late final _Translations$common$theme_mode$fr theme_mode =
      _Translations$common$theme_mode$fr._(_root);
  @override
  late final _Translations$common$theme_style$fr theme_style =
      _Translations$common$theme_style$fr._(_root);
  @override
  late final _Translations$common$provider$fr provider =
      _Translations$common$provider$fr._(_root);
}

// Path: app
class _Translations$app$fr extends Translations$app$en {
  _Translations$app$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  late final _Translations$app$tray$fr tray = _Translations$app$tray$fr._(
    _root,
  );
}

// Path: mini_translator
class _Translations$mini_translator$fr extends Translations$mini_translator$en {
  _Translations$mini_translator$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  late final _Translations$mini_translator$limited_banner$fr limited_banner =
      _Translations$mini_translator$limited_banner$fr._(_root);
  @override
  late final _Translations$mini_translator$input$fr input =
      _Translations$mini_translator$input$fr._(_root);
  @override
  late final _Translations$mini_translator$toolbar$fr toolbar =
      _Translations$mini_translator$toolbar$fr._(_root);
  @override
  late final _Translations$mini_translator$button$fr button =
      _Translations$mini_translator$button$fr._(_root);
  @override
  late final _Translations$mini_translator$language$fr language =
      _Translations$mini_translator$language$fr._(_root);
  @override
  late final _Translations$mini_translator$message$fr message =
      _Translations$mini_translator$message$fr._(_root);
  @override
  late final _Translations$mini_translator$result$fr result =
      _Translations$mini_translator$result$fr._(_root);
}

// Path: workbench
class _Translations$workbench$fr extends Translations$workbench$en {
  _Translations$workbench$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get workspace => 'Espace de travail';
  @override
  String get translate => 'Traduire';
  @override
  String get history => 'Historique';
  @override
  late final _Translations$workbench$history_page$fr history_page =
      _Translations$workbench$history_page$fr._(_root);
  @override
  String get glossary => 'Glossaire';
  @override
  String get recent_languages => 'Langues récentes';
  @override
  String get not_configured => 'Non configuré';
  @override
  late final _Translations$workbench$subtitle$fr subtitle =
      _Translations$workbench$subtitle$fr._(_root);
  @override
  late final _Translations$workbench$placeholder$fr placeholder =
      _Translations$workbench$placeholder$fr._(_root);
  @override
  late final _Translations$workbench$glossary_page$fr glossary_page =
      _Translations$workbench$glossary_page$fr._(_root);
  @override
  late final _Translations$workbench$translation$fr translation =
      _Translations$workbench$translation$fr._(_root);
  @override
  late final _Translations$workbench$status$fr status =
      _Translations$workbench$status$fr._(_root);
  @override
  String get version_latest => 'À jour';
  @override
  String get version_checking => 'Vérification…';
  @override
  String get check_updates => 'Rechercher des mises à jour';
}

// Path: settings
class _Translations$settings$fr extends Translations$settings$en {
  _Translations$settings$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get version => 'v{} (Build {})';
  @override
  late final _Translations$settings$general$fr general =
      _Translations$settings$general$fr._(_root);
  @override
  late final _Translations$settings$appearance$fr appearance =
      _Translations$settings$appearance$fr._(_root);
  @override
  late final _Translations$settings$shortcuts$fr shortcuts =
      _Translations$settings$shortcuts$fr._(_root);
  @override
  late final _Translations$settings$advanced$fr advanced =
      _Translations$settings$advanced$fr._(_root);
  @override
  late final _Translations$settings$services$fr services =
      _Translations$settings$services$fr._(_root);
  @override
  late final _Translations$settings$providers$fr providers =
      _Translations$settings$providers$fr._(_root);
  @override
  late final _Translations$settings$layout$fr layout =
      _Translations$settings$layout$fr._(_root);
  @override
  late final _Translations$settings$about$fr about =
      _Translations$settings$about$fr._(_root);
}

// Path: common.ui
class _Translations$common$ui$fr extends Translations$common$ui$en {
  _Translations$common$ui$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  late final _Translations$common$ui$button$fr button =
      _Translations$common$ui$button$fr._(_root);
  @override
  late final _Translations$common$ui$feedback$fr feedback =
      _Translations$common$ui$feedback$fr._(_root);
}

// Path: common.language
class _Translations$common$language$fr extends Translations$common$language$en {
  _Translations$common$language$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get ar => 'Arabe';
  @override
  String get bn => 'Bengali';
  @override
  String get de => 'Allemand';
  @override
  String get en => 'Anglais';
  @override
  String get es => 'Espagnol';
  @override
  String get fa => 'Persan';
  @override
  String get fr => 'Français';
  @override
  String get gu => 'Gujarati';
  @override
  String get ha => 'Haoussa';
  @override
  String get hi => 'Hindi';
  @override
  String get id => 'Indonésien';
  @override
  String get it => 'Italien';
  @override
  String get ja => 'Japonais';
  @override
  String get jv => 'Javanais';
  @override
  String get ko => 'Coréen';
  @override
  String get ml => 'Malayalam';
  @override
  String get mr => 'Marathi';
  @override
  String get ms => 'Malais';
  @override
  String get nl => 'Néerlandais';
  @override
  String get pa => 'Pendjabi';
  @override
  String get pl => 'Polonais';
  @override
  String get pt => 'Portugais';
  @override
  String get ro => 'Roumain';
  @override
  String get ru => 'Russe';
  @override
  String get sw => 'Swahili';
  @override
  String get ta => 'Tamoul';
  @override
  String get te => 'Télougou';
  @override
  String get th => 'Thaï';
  @override
  String get tr => 'Turc';
  @override
  String get uk => 'Ukrainien';
  @override
  String get ur => 'Ourdou';
  @override
  String get vi => 'Vietnamien';
  @override
  String get yo => 'Yoruba';
  @override
  String get zh_hans => 'Chinois simplifié';
  @override
  String get zh_hant => 'Chinois traditionnel';
}

// Path: common.theme_mode
class _Translations$common$theme_mode$fr
    extends Translations$common$theme_mode$en {
  _Translations$common$theme_mode$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get light => 'Clair';
  @override
  String get dark => 'Sombre';
  @override
  String get system => 'Système';
}

// Path: common.theme_style
class _Translations$common$theme_style$fr
    extends Translations$common$theme_style$en {
  _Translations$common$theme_style$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get studio => 'Studio';
  @override
  String get bright => 'Bright';
}

// Path: common.provider
class _Translations$common$provider$fr extends Translations$common$provider$en {
  _Translations$common$provider$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

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
  String get system => 'Système';
  @override
  String get tencent => 'Tencent';
  @override
  String get youdao => 'Youdao';
}

// Path: app.tray
class _Translations$app$tray$fr extends Translations$app$tray$en {
  _Translations$app$tray$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  late final _Translations$app$tray$context_menu$fr context_menu =
      _Translations$app$tray$context_menu$fr._(_root);
}

// Path: mini_translator.limited_banner
class _Translations$mini_translator$limited_banner$fr
    extends Translations$mini_translator$limited_banner$en {
  _Translations$mini_translator$limited_banner$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  late final _Translations$mini_translator$limited_banner$permission$fr
  permission = _Translations$mini_translator$limited_banner$permission$fr._(
    _root,
  );
  @override
  late final _Translations$mini_translator$limited_banner$instruction$fr
  instruction = _Translations$mini_translator$limited_banner$instruction$fr._(
    _root,
  );
  @override
  late final _Translations$mini_translator$limited_banner$action$fr action =
      _Translations$mini_translator$limited_banner$action$fr._(_root);
  @override
  late final _Translations$mini_translator$limited_banner$feedback$fr feedback =
      _Translations$mini_translator$limited_banner$feedback$fr._(_root);
  @override
  late final _Translations$mini_translator$limited_banner$tooltip$fr tooltip =
      _Translations$mini_translator$limited_banner$tooltip$fr._(_root);
}

// Path: mini_translator.input
class _Translations$mini_translator$input$fr
    extends Translations$mini_translator$input$en {
  _Translations$mini_translator$input$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get hint => 'Saisissez le mot ou le texte ici';
  @override
  String get extracting_text => 'Extraction du texte...';
  @override
  String hint_translate_to({required Object language}) =>
      'Saisissez un mot ou un texte à traduire en ${language}';
}

// Path: mini_translator.toolbar
class _Translations$mini_translator$toolbar$fr
    extends Translations$mini_translator$toolbar$en {
  _Translations$mini_translator$toolbar$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  late final _Translations$mini_translator$toolbar$tooltip$fr tooltip =
      _Translations$mini_translator$toolbar$tooltip$fr._(_root);
  @override
  late final _Translations$mini_translator$toolbar$menu$fr menu =
      _Translations$mini_translator$toolbar$menu$fr._(_root);
}

// Path: mini_translator.button
class _Translations$mini_translator$button$fr
    extends Translations$mini_translator$button$en {
  _Translations$mini_translator$button$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get clear => 'Effacer';
  @override
  String get translate => 'Traduire';
  @override
  String get copy => 'Copier';
  @override
  String get copied => 'Copié';
  @override
  String get bookmark => 'Enregistrer';
  @override
  String get bookmarked => 'Enregistré';
}

// Path: mini_translator.language
class _Translations$mini_translator$language$fr
    extends Translations$mini_translator$language$en {
  _Translations$mini_translator$language$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get auto_detect => 'Détection automatique';
  @override
  String get auto_match => 'Correspondance auto';
  @override
  String get switch_config => 'Changer de cible';
  @override
  String get more_languages => 'Plus de langues...';
  @override
  String get manage_common_languages => 'Gérer les langues courantes...';
  @override
  String get manage_targets => 'Gérer les cibles de traduction...';
  @override
  String get add_target => 'Ajouter une cible de traduction...';
}

// Path: mini_translator.message
class _Translations$mini_translator$message$fr
    extends Translations$mini_translator$message$en {
  _Translations$mini_translator$message$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get please_enter_word_or_text =>
      'Aucun texte saisi ou texte non extrait';
  @override
  String get capture_screen_area_canceled =>
      'La capture de la zone d\'écran a été annulée';
  @override
  String get ocr_service_not_configured =>
      'Aucun service de reconnaissance de texte par défaut configuré. Veuillez en définir un dans les Paramètres.';
  @override
  String get ocr_recognition_failed => 'La reconnaissance de texte a échoué';
}

// Path: mini_translator.result
class _Translations$mini_translator$result$fr
    extends Translations$mini_translator$result$en {
  _Translations$mini_translator$result$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get translating => 'Traduction…';
  @override
  String stale_requery({required Object key}) =>
      'Texte modifié · ${key} retraduire';
  @override
  String compare_services({required Object count}) =>
      'Comparer ${count} services';
  @override
  String get collapse_compare => 'Réduire';
  @override
  String get set_preferred => 'Définir comme préféré';
  @override
  String get retry => 'Réessayer';
  @override
  String get no_result =>
      'Aucun service n’a renvoyé de résultat : vérifiez le réseau ou essayez un autre service.';
  @override
  String get no_result_note =>
      'Le texte est conservé ; réessayer ne dupliquera pas l’historique.';
}

// Path: workbench.history_page
class _Translations$workbench$history_page$fr
    extends Translations$workbench$history_page$en {
  _Translations$workbench$history_page$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get all => 'Tout';
  @override
  String get favorites => 'Favoris';
  @override
  String get edited => 'Modifiés par moi';
  @override
  String get search => 'Rechercher';
  @override
  String get search_placeholder =>
      'Rechercher le texte, la traduction ou le service';
  @override
  String get search_label => 'Rechercher dans l’historique';
  @override
  String entry_count({required Object label, required Object count}) =>
      '${label} · ${count} entrées';
  @override
  String get by_time => 'Par date';
  @override
  String get loading => 'Chargement de l’historique…';
  @override
  String get load_failed => 'Échec du chargement de l’historique';
  @override
  String get retry => 'Réessayer';
  @override
  String get empty_title => 'Aucun historique de traduction';
  @override
  String get empty_description =>
      'La traduction préférée est enregistrée ici après chaque réussite.';
  @override
  String no_results({required Object query}) =>
      'Aucun résultat pour « ${query} »';
  @override
  String get clear_search => 'Effacer la recherche';
  @override
  String get select => 'Sélection multiple';
  @override
  String selected_count({required Object count}) => '${count} sélectionnées';
  @override
  String get exit_select => 'Quitter la sélection';
  @override
  String get add_to_glossary => 'Ajouter au glossaire';
  @override
  String get favorite => 'Ajouter aux favoris';
  @override
  String get unfavorite => 'Retirer des favoris';
  @override
  String delete_confirm({required Object count}) =>
      'Supprimer les ${count} entrées sélectionnées ? Cette action est irréversible.';
  @override
  String get no_glossary => 'Créez d’abord un glossaire';
  @override
  String added_to_glossary({required Object count}) =>
      '${count} entrées ajoutées au glossaire';
  @override
  String get favorite_flag => 'Favori';
  @override
  String get edited_flag => 'Modifié';
  @override
  String get edit_history_hint =>
      'La traduction modifiée sera enregistrée dans l’historique';
}

// Path: workbench.subtitle
class _Translations$workbench$subtitle$fr
    extends Translations$workbench$subtitle$en {
  _Translations$workbench$subtitle$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get translate => 'Atelier · Comparaison des services';
  @override
  String get settings => 'Réglages';
}

// Path: workbench.placeholder
class _Translations$workbench$placeholder$fr
    extends Translations$workbench$placeholder$en {
  _Translations$workbench$placeholder$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get history =>
      'Consultez, modifiez, ajoutez aux favoris et réutilisez vos traductions';
  @override
  String get glossary => 'Gérez les termes imposés et les variantes interdites';
}

// Path: workbench.glossary_page
class _Translations$workbench$glossary_page$fr
    extends Translations$workbench$glossary_page$en {
  _Translations$workbench$glossary_page$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get add_entry => 'Nouvelle entrée';
  @override
  String get term => 'Source';
  @override
  String get translation => 'Traduction imposée';
  @override
  String get forbidden => 'Interdit';
  @override
  String get hits => 'Occur.';
  @override
  String get term_placeholder => 'teacher forcing';
  @override
  String get translation_placeholder => 'forçage enseignant';
  @override
  String get forbidden_placeholder => 'enseignement forcé';
  @override
  String get search => 'Rechercher';
  @override
  String get search_placeholder =>
      'Rechercher un terme ou une traduction imposée';
  @override
  String get search_label => 'Rechercher dans le glossaire';
  @override
  String entry_count({required Object name, required Object count}) =>
      '${name} · ${count} termes';
  @override
  String get priority_note => 'Le glossaire prime sur toute sortie de service';
  @override
  String get new_book => 'Nouveau glossaire';
  @override
  String get new_book_placeholder => 'Nom du glossaire';
  @override
  String get rename_book => 'Renommer';
  @override
  String delete_book_confirm({required Object name, required Object count}) =>
      'Supprimer « ${name} » et ses ${count} termes ?';
  @override
  String get disabled => 'Désactivé';
  @override
  String get enable => 'Activer';
  @override
  String get disable => 'Désactiver';
  @override
  String get empty_title => 'Ce glossaire est vide';
  @override
  String get empty_description =>
      'Le glossaire prime sur toute sortie de service. Ajoutez les termes un à un, ou déposez un CSV pour les fusionner.';
  @override
  String no_results_title({required Object query}) =>
      'Aucun terme ne correspond à « ${query} »';
  @override
  String get no_results_description =>
      'Essayez un autre mot-clé, ou ajoutez le terme.';
  @override
  String get no_books_title => 'Aucun glossaire pour l’instant';
  @override
  String get no_books_description =>
      'Un glossaire garde vos choix de traduction cohérents sur tous les services. Créez-en un, puis ajoutez des termes.';
  @override
  String get loading => 'Chargement…';
}

// Path: workbench.translation
class _Translations$workbench$translation$fr
    extends Translations$workbench$translation$en {
  _Translations$workbench$translation$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get source => 'Source';
  @override
  String get target => 'Traduction';
  @override
  String get input_hint => 'Saisissez ou collez le texte à traduire';
  @override
  String get button => 'Traduire';
  @override
  String get auto_detected => 'Détecté automatiquement';
  @override
  String get loading_services => 'Chargement des services de traduction…';
  @override
  String get no_services => 'Configurez d’abord un service de traduction';
  @override
  String get translating => 'Traduction…';
  @override
  String get failed =>
      'Échec de la traduction. Vérifiez la configuration du service.';
  @override
  String get empty => 'La traduction apparaîtra ici';
  @override
  String get service_compare => 'Comparaison des services';
  @override
  String get main_translation => 'Principale';
  @override
  String get service_unavailable => 'Service indisponible';
  @override
  String get waiting => 'En attente';
  @override
  String get copy => 'Copier';
  @override
  String get favorite_unavailable =>
      'Impossible de mettre à jour le favori. Réessayez.';
  @override
  String get preferred => 'Traduction préférée';
  @override
  String get other_services => 'Autres services';
  @override
  String get copy_result => 'Copier la traduction';
  @override
  String get copied => 'Copié';
  @override
  String get favorite => 'Enregistrer';
  @override
  String get terms => 'Correspondances du glossaire';
  @override
  String get terms_hint => 'Les termes sont comparés à la saisie.';
  @override
  String get quality => 'Signaux de qualité';
  @override
  String get quality_hint => 'Calculés une fois la traduction reçue.';
  @override
  String get shortcuts => 'Raccourcis';
  @override
  String get other_services_disabled => 'Les autres services sont désactivés';
  @override
  String input_hint_translate_to({required Object language}) =>
      'Saisissez ou collez le texte à traduire en ${language}';
  @override
  String get clear => 'Effacer';
  @override
  String get swap_languages => 'Inverser les langues';
  @override
  String get services => 'Services de traduction';
  @override
  String get configure_services => 'Configurer les services';
  @override
  String get retry => 'Réessayer';
  @override
  String character_count({required Object count}) => '${count} caractères';
}

// Path: workbench.status
class _Translations$workbench$status$fr
    extends Translations$workbench$status$en {
  _Translations$workbench$status$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get runtime_ready => 'Moteur de traduction prêt';
  @override
  String get settings_synced => 'Réglages synchronisés';
  @override
  String get shortcuts => '⌥Space Fenêtre rapide · ⌥⇧2 Capture';
}

// Path: settings.general
class _Translations$settings$general$fr
    extends Translations$settings$general$en {
  _Translations$settings$general$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get title => 'Général';
  @override
  late final _Translations$settings$general$section$fr section =
      _Translations$settings$general$section$fr._(_root);
  @override
  late final _Translations$settings$general$row$fr row =
      _Translations$settings$general$row$fr._(_root);
  @override
  late final _Translations$settings$general$button$fr button =
      _Translations$settings$general$button$fr._(_root);
  @override
  late final _Translations$settings$general$option$fr option =
      _Translations$settings$general$option$fr._(_root);
  @override
  late final _Translations$settings$general$editor$fr editor =
      _Translations$settings$general$editor$fr._(_root);
}

// Path: settings.appearance
class _Translations$settings$appearance$fr
    extends Translations$settings$appearance$en {
  _Translations$settings$appearance$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get title => 'Apparence';
  @override
  late final _Translations$settings$appearance$section$fr section =
      _Translations$settings$appearance$section$fr._(_root);
  @override
  String get footer =>
      'Les modifications s’appliquent immédiatement à toute la fenêtre.';
}

// Path: settings.shortcuts
class _Translations$settings$shortcuts$fr
    extends Translations$settings$shortcuts$en {
  _Translations$settings$shortcuts$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get title => 'Raccourcis';
  @override
  late final _Translations$settings$shortcuts$section$fr section =
      _Translations$settings$shortcuts$section$fr._(_root);
  @override
  late final _Translations$settings$shortcuts$row$fr row =
      _Translations$settings$shortcuts$row$fr._(_root);
  @override
  late final _Translations$settings$shortcuts$reset_dialog$fr reset_dialog =
      _Translations$settings$shortcuts$reset_dialog$fr._(_root);
  @override
  late final _Translations$settings$shortcuts$group$fr group =
      _Translations$settings$shortcuts$group$fr._(_root);
  @override
  String get reset => 'Rétablir les valeurs par défaut...';
}

// Path: settings.advanced
class _Translations$settings$advanced$fr
    extends Translations$settings$advanced$en {
  _Translations$settings$advanced$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get title => 'Avancé';
  @override
  String get api_server => 'Serveur API local';
  @override
  String get api_server_description =>
      'Expose l\'API de traduction sur 127.0.0.1 pour les intégrations locales.';
  @override
  String get enable => 'Activer';
  @override
  String get port => 'Port';
  @override
  String get running_at => 'En cours d\'exécution à {url}';
  @override
  String get disabled => 'Désactivé';
}

// Path: settings.services
class _Translations$settings$services$fr
    extends Translations$settings$services$en {
  _Translations$settings$services$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get title => 'Services';
  @override
  late final _Translations$settings$services$button$fr button =
      _Translations$settings$services$button$fr._(_root);
  @override
  late final _Translations$settings$services$section$fr section =
      _Translations$settings$services$section$fr._(_root);
  @override
  late final _Translations$settings$services$editor$fr editor =
      _Translations$settings$services$editor$fr._(_root);
  @override
  late final _Translations$settings$services$detail$fr detail =
      _Translations$settings$services$detail$fr._(_root);
  @override
  String get make_default => 'Définir par défaut';
  @override
  late final _Translations$settings$services$item$fr item =
      _Translations$settings$services$item$fr._(_root);
}

// Path: settings.providers
class _Translations$settings$providers$fr
    extends Translations$settings$providers$en {
  _Translations$settings$providers$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get title => 'Fournisseurs';
  @override
  late final _Translations$settings$providers$section$fr section =
      _Translations$settings$providers$section$fr._(_root);
  @override
  late final _Translations$settings$providers$item$fr item =
      _Translations$settings$providers$item$fr._(_root);
  @override
  late final _Translations$settings$providers$button$fr button =
      _Translations$settings$providers$button$fr._(_root);
  @override
  late final _Translations$settings$providers$alert$fr alert =
      _Translations$settings$providers$alert$fr._(_root);
  @override
  late final _Translations$settings$providers$intro$fr intro =
      _Translations$settings$providers$intro$fr._(_root);
  @override
  late final _Translations$settings$providers$editor$fr editor =
      _Translations$settings$providers$editor$fr._(_root);
  @override
  late final _Translations$settings$providers$detail$fr detail =
      _Translations$settings$providers$detail$fr._(_root);
  @override
  late final _Translations$settings$providers$capability$fr capability =
      _Translations$settings$providers$capability$fr._(_root);
  @override
  late final _Translations$settings$providers$description$fr description =
      _Translations$settings$providers$description$fr._(_root);
  @override
  late final _Translations$settings$providers$delete_dialog$fr delete_dialog =
      _Translations$settings$providers$delete_dialog$fr._(_root);
}

// Path: settings.layout
class _Translations$settings$layout$fr extends Translations$settings$layout$en {
  _Translations$settings$layout$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get title => 'Paramètres';
  @override
  late final _Translations$settings$layout$empty$fr empty =
      _Translations$settings$layout$empty$fr._(_root);
  @override
  String get groups => 'Groupes';
  @override
  String get effect_hint => 'Les changements prennent effet immédiatement';
  @override
  String get footer_note => 'Traductions et clés restent sur cet appareil';
  @override
  String get support => 'Assistance';
}

// Path: settings.about
class _Translations$settings$about$fr extends Translations$settings$about$en {
  _Translations$settings$about$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get title => 'À propos';
  @override
  String get copy_version_info => 'Copier les informations de version';
  @override
  String get up_to_date => 'Vous êtes à jour.';
  @override
  String get check_again => 'Vérifier à nouveau';
  @override
  String get links => 'Liens';
  @override
  String get website => 'Site web';
  @override
  String get github => 'GitHub';
  @override
  String get report_issue => 'Signaler un problème';
  @override
  String get license => 'Licence';
  @override
  String get open_changelog => 'Ouvrir le journal des modifications';
  @override
  String get update => 'Mise à jour';
}

// Path: common.ui.button
class _Translations$common$ui$button$fr
    extends Translations$common$ui$button$en {
  _Translations$common$ui$button$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get ok => 'OK';
  @override
  String get cancel => 'Annuler';
  @override
  String get add => 'Ajouter';
  @override
  String get delete => 'Supprimer';
  @override
  String get edit => 'Modifier';
  @override
  String get save => 'Enregistrer';
  @override
  String get manage => 'Gérer';
  @override
  String get kContinue => 'Continuer';
}

// Path: common.ui.feedback
class _Translations$common$ui$feedback$fr
    extends Translations$common$ui$feedback$en {
  _Translations$common$ui$feedback$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get copied => 'Copié';
}

// Path: app.tray.context_menu
class _Translations$app$tray$context_menu$fr
    extends Translations$app$tray$context_menu$en {
  _Translations$app$tray$context_menu$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get show_window => 'Afficher la fenêtre';
  @override
  late final _Translations$app$tray$context_menu$dev_tools$fr dev_tools =
      _Translations$app$tray$context_menu$dev_tools$fr._(_root);
  @override
  String get check_for_updates => 'Vérifier les mises à jour';
  @override
  String get settings => 'Paramètres';
  @override
  String get quit => 'Quitter';
}

// Path: mini_translator.limited_banner.permission
class _Translations$mini_translator$limited_banner$permission$fr
    extends Translations$mini_translator$limited_banner$permission$en {
  _Translations$mini_translator$limited_banner$permission$fr._(
    TranslationsFr root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get missing_both =>
      'Accordez les autorisations d\'enregistrement d\'écran et d\'accessibilité pour activer toutes les fonctionnalités.';
  @override
  String get missing_screen_capture =>
      'Accordez l\'autorisation d\'enregistrement d\'écran pour activer toutes les fonctionnalités.';
  @override
  String get missing_accessibility =>
      'Accordez l\'autorisation d\'accessibilité pour activer toutes les fonctionnalités.';
}

// Path: mini_translator.limited_banner.instruction
class _Translations$mini_translator$limited_banner$instruction$fr
    extends Translations$mini_translator$limited_banner$instruction$en {
  _Translations$mini_translator$limited_banner$instruction$fr._(
    TranslationsFr root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get app_settings_prefix => 'Accédez aux ';
  @override
  String get follow_guide_prefix => ', suivez le guide, puis cliquez sur ';
  @override
  String get suffix => '.';
}

// Path: mini_translator.limited_banner.action
class _Translations$mini_translator$limited_banner$action$fr
    extends Translations$mini_translator$limited_banner$action$en {
  _Translations$mini_translator$limited_banner$action$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get app_settings => 'Paramètres de l\'application';
  @override
  String get recheck => 'Re-vérifier';
}

// Path: mini_translator.limited_banner.feedback
class _Translations$mini_translator$limited_banner$feedback$fr
    extends Translations$mini_translator$limited_banner$feedback$en {
  _Translations$mini_translator$limited_banner$feedback$fr._(
    TranslationsFr root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get enabled => 'L\'extraction de texte d\'écran est activée.';
  @override
  String get still_missing =>
      'Les autorisations requises sont toujours manquantes.\nVeuillez vérifier vos paramètres et réessayer.';
}

// Path: mini_translator.limited_banner.tooltip
class _Translations$mini_translator$limited_banner$tooltip$fr
    extends Translations$mini_translator$limited_banner$tooltip$en {
  _Translations$mini_translator$limited_banner$tooltip$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get help => 'Voir l\'aide';
}

// Path: mini_translator.toolbar.tooltip
class _Translations$mini_translator$toolbar$tooltip$fr
    extends Translations$mini_translator$toolbar$tooltip$en {
  _Translations$mini_translator$toolbar$tooltip$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get extract_text_from_screen_capture =>
      'Capturer une zone de l\'écran et reconnaître le texte';
  @override
  String get extract_text_from_clipboard => 'Lire le contenu du presse-papiers';
  @override
  String get pin => 'Épingler la fenêtre';
  @override
  String get more_actions => 'Plus d’actions';
}

// Path: mini_translator.toolbar.menu
class _Translations$mini_translator$toolbar$menu$fr
    extends Translations$mini_translator$toolbar$menu$en {
  _Translations$mini_translator$toolbar$menu$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get extract_from_screen_capture => 'Capturer depuis l’écran';
  @override
  String get extract_from_clipboard => 'Depuis le presse-papiers';
  @override
  String get open_main_window => 'Ouvrir la fenêtre principale';
  @override
  String get open_settings => 'Réglages…';
}

// Path: settings.general.section
class _Translations$settings$general$section$fr
    extends Translations$settings$general$section$en {
  _Translations$settings$general$section$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get permissions => 'Autorisations système';
  @override
  String get ocr => 'Reconnaissance de texte';
  @override
  String get directory => 'Répertoire';
  @override
  String get translation => 'Traduction';
  @override
  String get translation_target => 'Cible de traduction';
  @override
  String get languages => 'Langues';
  @override
  String get input => 'Paramètres de saisie';
  @override
  String get startup => 'Démarrage et intégration';
  @override
  String get ocr_behaviour => 'Comportement de capture';
  @override
  String get translation_behaviour => 'Comportement de traduction';
}

// Path: settings.general.row
class _Translations$settings$general$row$fr
    extends Translations$settings$general$row$en {
  _Translations$settings$general$row$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get launch_at_login => 'Lancer à l\'ouverture de session';
  @override
  String get show_in_menu_bar => 'Afficher dans la barre de menus';
  @override
  String get screen_capture_access =>
      'Accorder l\'accès à l\'enregistrement d\'écran';
  @override
  String get screen_selection_access => 'Accorder l\'accès à l\'accessibilité';
  @override
  String get default_ocr_service =>
      'Service de reconnaissance de texte par défaut';
  @override
  String get auto_copy_detected_text =>
      'Copier automatiquement le texte détecté';
  @override
  String get default_directory_service => 'Service de répertoire par défaut';
  @override
  String get default_translation_service => 'Service de traduction par défaut';
  @override
  String get translation_target_hint =>
      'Configurez les paires de langues utilisées par le traducteur.';
  @override
  String get common_languages => 'Langues courantes';
  @override
  String get common_languages_description =>
      'Les langues courantes apparaissent en haut des sélecteurs de langue.';
  @override
  String get common_languages_hint => 'Sélectionnez vos langues courantes:';
  @override
  String get common_languages_sort => 'Trier par code';
  @override
  String get common_languages_reset => 'Réinitialiser';
  @override
  String get common_languages_reset_help =>
      'Rétablir l\'ensemble par défaut des langues courantes';
  @override
  String get common_languages_search => 'Rechercher des langues...';
  @override
  String get common_languages_all => 'Toutes les langues';
  @override
  String get double_click_copy_result =>
      'Double-cliquer pour copier le résultat de la traduction';
  @override
  String get submit_with_enter => 'Valider avec Entrée';
  @override
  String get submit_with_meta_enter_mac => 'Valider avec ⌘ + Entrée';
  @override
  String get screen_capture_access_hint =>
      'La capture de texte à l’écran nécessite de lire son contenu.';
  @override
  String get screen_selection_access_hint =>
      'La capture du texte sélectionné nécessite de lire les sélections des autres applications.';
  @override
  String get no_translation_targets =>
      'Aucune cible de traduction — ajoutez-en une pour définir la langue par défaut.';
}

// Path: settings.general.button
class _Translations$settings$general$button$fr
    extends Translations$settings$general$button$en {
  _Translations$settings$general$button$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get add_provider => 'Ajouter...';
  @override
  String get add_target => 'Ajouter une cible...';
  @override
  String get manage_targets => 'Gérer les cibles de traduction...';
  @override
  String get manage_languages => 'Gérer les langues courantes...';
  @override
  String get grant => 'Accorder';
}

// Path: settings.general.option
class _Translations$settings$general$option$fr
    extends Translations$settings$general$option$en {
  _Translations$settings$general$option$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get none => 'Aucun';
  @override
  String get no_services_available => 'Aucun service disponible';
  @override
  String get granted => 'Accordée';
  @override
  String get built_in_ocr => 'OCR intégré';
  @override
  String get tesseract => 'Tesseract';
  @override
  String get youdao_ocr => 'OCR Youdao';
}

// Path: settings.general.editor
class _Translations$settings$general$editor$fr
    extends Translations$settings$general$editor$en {
  _Translations$settings$general$editor$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get add_target_title => 'Ajouter une cible de traduction';
  @override
  String get edit_target_title => 'Modifier la cible de traduction';
  @override
  late final _Translations$settings$general$editor$row$fr row =
      _Translations$settings$general$editor$row$fr._(_root);
  @override
  String get title_edit => 'Modifier la cible de traduction';
  @override
  String get subtitle =>
      'Définit vers quelle langue une langue source est traduite par défaut';
  @override
  String get same_language =>
      'La langue source et la cible sont identiques — choisissez une autre cible.';
  @override
  String get duplicate => 'Une cible avec cette paire existe déjà.';
  @override
  String get hint_auto =>
      'Traduire vers {} lorsque aucune autre règle ne correspond.';
  @override
  String get hint_source => 'Traduire vers {} lorsque {} est détecté.';
}

// Path: settings.appearance.section
class _Translations$settings$appearance$section$fr
    extends Translations$settings$appearance$section$en {
  _Translations$settings$appearance$section$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get app_language => 'Langue d\'affichage';
  @override
  String get theme_mode => 'Mode du thème';
  @override
  String get theme_style => 'Style de thème';
}

// Path: settings.shortcuts.section
class _Translations$settings$shortcuts$section$fr
    extends Translations$settings$shortcuts$section$en {
  _Translations$settings$shortcuts$section$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get text_extraction => 'Extraction de texte';
  @override
  String get input_assist => 'Fonction d\'aide à la saisie';
  @override
  String get submit_mode => 'Envoyer avec';
}

// Path: settings.shortcuts.row
class _Translations$settings$shortcuts$row$fr
    extends Translations$settings$shortcuts$row$en {
  _Translations$settings$shortcuts$row$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get toggle_mini_translator => 'Afficher/Masquer la fenêtre';
  @override
  String get extract_text_from_screen_selection =>
      'Extraire le texte d\'une sélection d\'écran';
  @override
  String get extract_text_from_screen_capture =>
      'Extraire le texte d\'une capture d\'écran';
  @override
  String get extract_text_from_clipboard =>
      'Extraire le texte du presse-papiers';
  @override
  String get translate_input => 'Traduire le contenu saisi';
}

// Path: settings.shortcuts.reset_dialog
class _Translations$settings$shortcuts$reset_dialog$fr
    extends Translations$settings$shortcuts$reset_dialog$en {
  _Translations$settings$shortcuts$reset_dialog$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get title => 'Réinitialiser les raccourcis';
  @override
  String get message =>
      'Êtes-vous sûr de vouloir réinitialiser tous les raccourcis à leurs valeurs par défaut ?';
  @override
  String get confirm => 'Réinitialiser';
  @override
  String get cancel => 'Annuler';
}

// Path: settings.shortcuts.group
class _Translations$settings$shortcuts$group$fr
    extends Translations$settings$shortcuts$group$en {
  _Translations$settings$shortcuts$group$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  late final _Translations$settings$shortcuts$group$global$fr global =
      _Translations$settings$shortcuts$group$global$fr._(_root);
  @override
  late final _Translations$settings$shortcuts$group$in_app$fr in_app =
      _Translations$settings$shortcuts$group$in_app$fr._(_root);
}

// Path: settings.services.button
class _Translations$settings$services$button$fr
    extends Translations$settings$services$button$en {
  _Translations$settings$services$button$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get add_service => 'Ajouter un service...';
}

// Path: settings.services.section
class _Translations$settings$services$section$fr
    extends Translations$settings$services$section$en {
  _Translations$settings$services$section$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get available_services => 'Services disponibles';
}

// Path: settings.services.editor
class _Translations$settings$services$editor$fr
    extends Translations$settings$services$editor$en {
  _Translations$settings$services$editor$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get title => 'Ajouter un service';
  @override
  String get subtitle => 'Ajoutez un service à un fournisseur déjà configuré';
  @override
  late final _Translations$settings$services$editor$row$fr row =
      _Translations$settings$services$editor$row$fr._(_root);
  @override
  String get prompt_placeholder =>
      'Laissez vide pour utiliser le prompt par défaut de ce type';
  @override
  String get variant_hint =>
      '{} a déjà un service de {} ; celui-ci vient s’y ajouter comme seconde configuration.';
  @override
  String get traditional_note =>
      '{} est une interface traditionnelle : ni modèle ni prompt à régler. Ses paramètres se trouvent sur la page de détail du fournisseur.';
}

// Path: settings.services.detail
class _Translations$settings$services$detail$fr
    extends Translations$settings$services$detail$en {
  _Translations$settings$services$detail$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  late final _Translations$settings$services$detail$row$fr row =
      _Translations$settings$services$detail$row$fr._(_root);
  @override
  late final _Translations$settings$services$detail$delete_dialog$fr
  delete_dialog = _Translations$settings$services$detail$delete_dialog$fr._(
    _root,
  );
  @override
  String get prompt_variables =>
      'Variables disponibles : {{sourceLanguage}}, {{targetLanguage}}, {{text}}';
}

// Path: settings.services.item
class _Translations$settings$services$item$fr
    extends Translations$settings$services$item$en {
  _Translations$settings$services$item$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get none_of_kind => 'Aucun service de {} disponible pour l’instant.';
}

// Path: settings.providers.section
class _Translations$settings$providers$section$fr
    extends Translations$settings$providers$section$en {
  _Translations$settings$providers$section$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get services => 'Services disponibles';
  @override
  String get services_description =>
      'Consultez les services disponibles des fournisseurs configurés et changez de type de service.';
}

// Path: settings.providers.item
class _Translations$settings$providers$item$fr
    extends Translations$settings$providers$item$en {
  _Translations$settings$providers$item$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get empty =>
      'Aucun fournisseur configuré. Ajoutez-en un pour activer les services de traduction.';
  @override
  String get loading => 'Chargement des fournisseurs...';
  @override
  String get no_services => 'Aucun service disponible.';
}

// Path: settings.providers.button
class _Translations$settings$providers$button$fr
    extends Translations$settings$providers$button$en {
  _Translations$settings$providers$button$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get add => 'Ajouter un fournisseur...';
}

// Path: settings.providers.alert
class _Translations$settings$providers$alert$fr
    extends Translations$settings$providers$alert$en {
  _Translations$settings$providers$alert$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get error => 'Erreur';
}

// Path: settings.providers.intro
class _Translations$settings$providers$intro$fr
    extends Translations$settings$providers$intro$en {
  _Translations$settings$providers$intro$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get body =>
      'Gérez les fournisseurs de services utilisés par l’application.';
  @override
  String get warning =>
      'Les fournisseurs connectés peuvent traiter le texte ou les images que vous envoyez. Activez uniquement les services de confiance.';
}

// Path: settings.providers.editor
class _Translations$settings$providers$editor$fr
    extends Translations$settings$providers$editor$en {
  _Translations$settings$providers$editor$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  late final _Translations$settings$providers$editor$row$fr row =
      _Translations$settings$providers$editor$row$fr._(_root);
  @override
  late final _Translations$settings$providers$editor$placeholder$fr
  placeholder = _Translations$settings$providers$editor$placeholder$fr._(_root);
  @override
  late final _Translations$settings$providers$editor$type_picker$fr
  type_picker = _Translations$settings$providers$editor$type_picker$fr._(_root);
  @override
  late final _Translations$settings$providers$editor$tooltip$fr tooltip =
      _Translations$settings$providers$editor$tooltip$fr._(_root);
  @override
  late final _Translations$settings$providers$editor$step$fr step =
      _Translations$settings$providers$editor$step$fr._(_root);
  @override
  String get add_title => 'Ajouter {}';
  @override
  late final _Translations$settings$providers$editor$capability_note$fr
  capability_note =
      _Translations$settings$providers$editor$capability_note$fr._(_root);
  @override
  late final _Translations$settings$providers$editor$test$fr test =
      _Translations$settings$providers$editor$test$fr._(_root);
}

// Path: settings.providers.detail
class _Translations$settings$providers$detail$fr
    extends Translations$settings$providers$detail$en {
  _Translations$settings$providers$detail$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  late final _Translations$settings$providers$detail$tooltip$fr tooltip =
      _Translations$settings$providers$detail$tooltip$fr._(_root);
  @override
  late final _Translations$settings$providers$detail$row$fr row =
      _Translations$settings$providers$detail$row$fr._(_root);
  @override
  late final _Translations$settings$providers$detail$section$fr section =
      _Translations$settings$providers$detail$section$fr._(_root);
  @override
  late final _Translations$settings$providers$detail$models$fr models =
      _Translations$settings$providers$detail$models$fr._(_root);
}

// Path: settings.providers.capability
class _Translations$settings$providers$capability$fr
    extends Translations$settings$providers$capability$en {
  _Translations$settings$providers$capability$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get translation => 'Traduction';
  @override
  String get dictionary => 'Dictionnaire';
  @override
  String get ocr => 'OCR';
  @override
  String get llm => 'IA';
}

// Path: settings.providers.description
class _Translations$settings$providers$description$fr
    extends Translations$settings$providers$description$en {
  _Translations$settings$providers$description$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get all =>
      'Fournit la recherche dans le dictionnaire et la traduction de texte';
  @override
  String get dictionary =>
      'Fournit la recherche dans le dictionnaire et les définitions de mots';
  @override
  String get translation => 'Fournit la traduction de texte entre les langues';
  @override
  String get fallback => 'Fournit des services de traduction';
}

// Path: settings.providers.delete_dialog
class _Translations$settings$providers$delete_dialog$fr
    extends Translations$settings$providers$delete_dialog$en {
  _Translations$settings$providers$delete_dialog$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get title => 'Supprimer « {} » ?';
  @override
  String get message => 'Cette action est irréversible.';
}

// Path: settings.layout.empty
class _Translations$settings$layout$empty$fr
    extends Translations$settings$layout$empty$en {
  _Translations$settings$layout$empty$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get title => 'Sélectionnez une catégorie';
  @override
  String get message =>
      'Choisissez une section de paramètres dans la barre latérale.';
}

// Path: app.tray.context_menu.dev_tools
class _Translations$app$tray$context_menu$dev_tools$fr
    extends Translations$app$tray$context_menu$dev_tools$en {
  _Translations$app$tray$context_menu$dev_tools$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get title => 'Outils de développement';
  @override
  String get open_data_directory => 'Ouvrir le répertoire de données';
}

// Path: settings.general.editor.row
class _Translations$settings$general$editor$row$fr
    extends Translations$settings$general$editor$row$en {
  _Translations$settings$general$editor$row$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get source_language => 'Langue source';
  @override
  String get target_language => 'Langue cible';
}

// Path: settings.shortcuts.group.global
class _Translations$settings$shortcuts$group$global$fr
    extends Translations$settings$shortcuts$group$global$en {
  _Translations$settings$shortcuts$group$global$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get title => 'Raccourcis globaux';
  @override
  String get description => 'Fonctionnent dans toutes les applications.';
}

// Path: settings.shortcuts.group.in_app
class _Translations$settings$shortcuts$group$in_app$fr
    extends Translations$settings$shortcuts$group$in_app$en {
  _Translations$settings$shortcuts$group$in_app$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get title => 'Touches de l’application';
  @override
  String get description =>
      'S’appliquent uniquement aux champs de saisie de cette application.';
}

// Path: settings.services.editor.row
class _Translations$settings$services$editor$row$fr
    extends Translations$settings$services$editor$row$en {
  _Translations$settings$services$editor$row$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get model => 'Modèle';
  @override
  String get system_prompt => 'Prompt système';
}

// Path: settings.services.detail.row
class _Translations$settings$services$detail$row$fr
    extends Translations$settings$services$detail$row$en {
  _Translations$settings$services$detail$row$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get id => 'ID du service';
  @override
  String get name => 'Nom';
  @override
  String get provider => 'Fournisseur';
  @override
  String get type => 'Type';
}

// Path: settings.services.detail.delete_dialog
class _Translations$settings$services$detail$delete_dialog$fr
    extends Translations$settings$services$detail$delete_dialog$en {
  _Translations$settings$services$detail$delete_dialog$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get title => 'Supprimer "{}" ?';
  @override
  String get message => 'Ce service sera supprimé du fournisseur.';
}

// Path: settings.providers.editor.row
class _Translations$settings$providers$editor$row$fr
    extends Translations$settings$providers$editor$row$en {
  _Translations$settings$providers$editor$row$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get id => 'Identifiant du fournisseur';
  @override
  String get type => 'Type de fournisseur';
  @override
  String get default_model => 'Modèle par défaut';
}

// Path: settings.providers.editor.placeholder
class _Translations$settings$providers$editor$placeholder$fr
    extends Translations$settings$providers$editor$placeholder$en {
  _Translations$settings$providers$editor$placeholder$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get id => 'p. ex. deepl-main';
}

// Path: settings.providers.editor.type_picker
class _Translations$settings$providers$editor$type_picker$fr
    extends Translations$settings$providers$editor$type_picker$en {
  _Translations$settings$providers$editor$type_picker$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get prompt =>
      'Sélectionnez le type de fournisseur que vous souhaitez ajouter :';
  @override
  String get section_llm => 'LLM';
  @override
  String get section_traditional => 'Traditionnel';
}

// Path: settings.providers.editor.tooltip
class _Translations$settings$providers$editor$tooltip$fr
    extends Translations$settings$providers$editor$tooltip$en {
  _Translations$settings$providers$editor$tooltip$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get help => 'Aide';
}

// Path: settings.providers.editor.step
class _Translations$settings$providers$editor$step$fr
    extends Translations$settings$providers$editor$step$en {
  _Translations$settings$providers$editor$step$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get next => 'Continuer';
  @override
  String get back => 'Précédent';
}

// Path: settings.providers.editor.capability_note
class _Translations$settings$providers$editor$capability_note$fr
    extends Translations$settings$providers$editor$capability_note$en {
  _Translations$settings$providers$editor$capability_note$fr._(
    TranslationsFr root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get translation => 'Rejoint les traductions candidates';
  @override
  String get dictionary => 'Fournit des définitions de dictionnaire';
  @override
  String get ocr => 'Reconnaît le texte des images';
}

// Path: settings.providers.editor.test
class _Translations$settings$providers$editor$test$fr
    extends Translations$settings$providers$editor$test$en {
  _Translations$settings$providers$editor$test$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get run => 'Tester la connexion';
  @override
  String get running => 'Test de la connexion · {}s écoulées';
  @override
  String get passed_models => 'Connexion OK · {} modèles disponibles';
  @override
  String get passed_service => 'Connexion OK · service disponible';
  @override
  String get passed_footer => 'Test de connexion réussi';
  @override
  String get retest => 'Retester';
  @override
  String get tips_title => 'À essayer';
  @override
  String get tips_llm =>
      '· Vérifiez que la clé correspond au type de fournisseur choisi\n· Vérifiez si la Base URL doit contenir /v1\n· Confirmez que le modèle est activé dans la console du fournisseur';
  @override
  String get tips_traditional =>
      '· Vérifiez que les identifiants correspondent au type de fournisseur choisi\n· Confirmez que le service est activé dans la console du fournisseur';
  @override
  String get failed_suffix => 'échec de la vérification';
  @override
  String get passed_suffix => 'vérifié';
}

// Path: settings.providers.detail.tooltip
class _Translations$settings$providers$detail$tooltip$fr
    extends Translations$settings$providers$detail$tooltip$en {
  _Translations$settings$providers$detail$tooltip$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get edit => 'Modifier le fournisseur';
}

// Path: settings.providers.detail.row
class _Translations$settings$providers$detail$row$fr
    extends Translations$settings$providers$detail$row$en {
  _Translations$settings$providers$detail$row$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get id_hint => 'Non modifiable après la création';
}

// Path: settings.providers.detail.section
class _Translations$settings$providers$detail$section$fr
    extends Translations$settings$providers$detail$section$en {
  _Translations$settings$providers$detail$section$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get configuration => 'Configuration';
  @override
  String get models => 'Modèles';
}

// Path: settings.providers.detail.models
class _Translations$settings$providers$detail$models$fr
    extends Translations$settings$providers$detail$models$en {
  _Translations$settings$providers$detail$models$fr._(TranslationsFr root)
    : this._root = root,
      super.internal(root);

  final TranslationsFr _root; // ignore: unused_field

  // Translations
  @override
  String get loading => 'Chargement des modèles...';
  @override
  String get empty => 'Aucun modèle trouvé.';
  @override
  String get retry => 'Réessayer';
  @override
  String get refresh => 'Actualiser la liste';
  @override
  String get default_badge => 'Par défaut';
  @override
  String get set_default => 'Définir par défaut';
  @override
  String get fetch_error =>
      'Impossible de récupérer les modèles depuis l\'API du fournisseur.';
}

/// The flat map containing all translations for locale <fr>.
/// Only for edge cases! For simple maps, use the map function of this library.
///
/// The Dart AOT compiler has issues with very large switch statements,
/// so the map is split into smaller functions (512 entries each).
extension on TranslationsFr {
  dynamic _flatMapFunction(String path) {
    return switch (path) {
      'common.ui.button.ok' => 'OK',
      'common.ui.button.cancel' => 'Annuler',
      'common.ui.button.add' => 'Ajouter',
      'common.ui.button.delete' => 'Supprimer',
      'common.ui.button.edit' => 'Modifier',
      'common.ui.button.save' => 'Enregistrer',
      'common.ui.button.manage' => 'Gérer',
      'common.ui.button.kContinue' => 'Continuer',
      'common.ui.feedback.copied' => 'Copié',
      'common.language.ar' => 'Arabe',
      'common.language.bn' => 'Bengali',
      'common.language.de' => 'Allemand',
      'common.language.en' => 'Anglais',
      'common.language.es' => 'Espagnol',
      'common.language.fa' => 'Persan',
      'common.language.fr' => 'Français',
      'common.language.gu' => 'Gujarati',
      'common.language.ha' => 'Haoussa',
      'common.language.hi' => 'Hindi',
      'common.language.id' => 'Indonésien',
      'common.language.it' => 'Italien',
      'common.language.ja' => 'Japonais',
      'common.language.jv' => 'Javanais',
      'common.language.ko' => 'Coréen',
      'common.language.ml' => 'Malayalam',
      'common.language.mr' => 'Marathi',
      'common.language.ms' => 'Malais',
      'common.language.nl' => 'Néerlandais',
      'common.language.pa' => 'Pendjabi',
      'common.language.pl' => 'Polonais',
      'common.language.pt' => 'Portugais',
      'common.language.ro' => 'Roumain',
      'common.language.ru' => 'Russe',
      'common.language.sw' => 'Swahili',
      'common.language.ta' => 'Tamoul',
      'common.language.te' => 'Télougou',
      'common.language.th' => 'Thaï',
      'common.language.tr' => 'Turc',
      'common.language.uk' => 'Ukrainien',
      'common.language.ur' => 'Ourdou',
      'common.language.vi' => 'Vietnamien',
      'common.language.yo' => 'Yoruba',
      'common.language.zh_hans' => 'Chinois simplifié',
      'common.language.zh_hant' => 'Chinois traditionnel',
      'common.theme_mode.light' => 'Clair',
      'common.theme_mode.dark' => 'Sombre',
      'common.theme_mode.system' => 'Système',
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
      'common.provider.system' => 'Système',
      'common.provider.tencent' => 'Tencent',
      'common.provider.youdao' => 'Youdao',
      'app.tray.context_menu.show_window' => 'Afficher la fenêtre',
      'app.tray.context_menu.dev_tools.title' => 'Outils de développement',
      'app.tray.context_menu.dev_tools.open_data_directory' =>
        'Ouvrir le répertoire de données',
      'app.tray.context_menu.check_for_updates' => 'Vérifier les mises à jour',
      'app.tray.context_menu.settings' => 'Paramètres',
      'app.tray.context_menu.quit' => 'Quitter',
      'mini_translator.limited_banner.permission.missing_both' => 'Accordez les autorisations d\'enregistrement d\'écran et d\'accessibilité pour activer toutes les fonctionnalités.',
      'mini_translator.limited_banner.permission.missing_screen_capture' => 'Accordez l\'autorisation d\'enregistrement d\'écran pour activer toutes les fonctionnalités.',
      'mini_translator.limited_banner.permission.missing_accessibility' => 'Accordez l\'autorisation d\'accessibilité pour activer toutes les fonctionnalités.',
      'mini_translator.limited_banner.instruction.app_settings_prefix' =>
        'Accédez aux ',
      'mini_translator.limited_banner.instruction.follow_guide_prefix' =>
        ', suivez le guide, puis cliquez sur ',
      'mini_translator.limited_banner.instruction.suffix' => '.',
      'mini_translator.limited_banner.action.app_settings' =>
        'Paramètres de l\'application',
      'mini_translator.limited_banner.action.recheck' => 'Re-vérifier',
      'mini_translator.limited_banner.feedback.enabled' =>
        'L\'extraction de texte d\'écran est activée.',
      'mini_translator.limited_banner.feedback.still_missing' => 'Les autorisations requises sont toujours manquantes.\nVeuillez vérifier vos paramètres et réessayer.',
      'mini_translator.limited_banner.tooltip.help' => 'Voir l\'aide',
      'mini_translator.input.hint' => 'Saisissez le mot ou le texte ici',
      'mini_translator.input.extracting_text' => 'Extraction du texte...',
      'mini_translator.input.hint_translate_to' => ({
        required Object language,
      }) => 'Saisissez un mot ou un texte à traduire en ${language}',
      'mini_translator.toolbar.tooltip.extract_text_from_screen_capture' =>
        'Capturer une zone de l\'écran et reconnaître le texte',
      'mini_translator.toolbar.tooltip.extract_text_from_clipboard' =>
        'Lire le contenu du presse-papiers',
      'mini_translator.toolbar.tooltip.pin' => 'Épingler la fenêtre',
      'mini_translator.toolbar.tooltip.more_actions' => 'Plus d’actions',
      'mini_translator.toolbar.menu.extract_from_screen_capture' =>
        'Capturer depuis l’écran',
      'mini_translator.toolbar.menu.extract_from_clipboard' =>
        'Depuis le presse-papiers',
      'mini_translator.toolbar.menu.open_main_window' =>
        'Ouvrir la fenêtre principale',
      'mini_translator.toolbar.menu.open_settings' => 'Réglages…',
      'mini_translator.button.clear' => 'Effacer',
      'mini_translator.button.translate' => 'Traduire',
      'mini_translator.button.copy' => 'Copier',
      'mini_translator.button.copied' => 'Copié',
      'mini_translator.button.bookmark' => 'Enregistrer',
      'mini_translator.button.bookmarked' => 'Enregistré',
      'mini_translator.language.auto_detect' => 'Détection automatique',
      'mini_translator.language.auto_match' => 'Correspondance auto',
      'mini_translator.language.switch_config' => 'Changer de cible',
      'mini_translator.language.more_languages' => 'Plus de langues...',
      'mini_translator.language.manage_common_languages' =>
        'Gérer les langues courantes...',
      'mini_translator.language.manage_targets' =>
        'Gérer les cibles de traduction...',
      'mini_translator.language.add_target' =>
        'Ajouter une cible de traduction...',
      'mini_translator.message.please_enter_word_or_text' =>
        'Aucun texte saisi ou texte non extrait',
      'mini_translator.message.capture_screen_area_canceled' =>
        'La capture de la zone d\'écran a été annulée',
      'mini_translator.message.ocr_service_not_configured' => 'Aucun service de reconnaissance de texte par défaut configuré. Veuillez en définir un dans les Paramètres.',
      'mini_translator.message.ocr_recognition_failed' =>
        'La reconnaissance de texte a échoué',
      'mini_translator.result.translating' => 'Traduction…',
      'mini_translator.result.stale_requery' => ({
        required Object key,
      }) => 'Texte modifié · ${key} retraduire',
      'mini_translator.result.compare_services' => ({
        required Object count,
      }) => 'Comparer ${count} services',
      'mini_translator.result.collapse_compare' => 'Réduire',
      'mini_translator.result.set_preferred' => 'Définir comme préféré',
      'mini_translator.result.retry' => 'Réessayer',
      'mini_translator.result.no_result' => 'Aucun service n’a renvoyé de résultat : vérifiez le réseau ou essayez un autre service.',
      'mini_translator.result.no_result_note' =>
        'Le texte est conservé ; réessayer ne dupliquera pas l’historique.',
      'workbench.workspace' => 'Espace de travail',
      'workbench.translate' => 'Traduire',
      'workbench.history' => 'Historique',
      'workbench.history_page.all' => 'Tout',
      'workbench.history_page.favorites' => 'Favoris',
      'workbench.history_page.edited' => 'Modifiés par moi',
      'workbench.history_page.search' => 'Rechercher',
      'workbench.history_page.search_placeholder' =>
        'Rechercher le texte, la traduction ou le service',
      'workbench.history_page.search_label' => 'Rechercher dans l’historique',
      'workbench.history_page.entry_count' => ({
        required Object label,
        required Object count,
      }) => '${label} · ${count} entrées',
      'workbench.history_page.by_time' => 'Par date',
      'workbench.history_page.loading' => 'Chargement de l’historique…',
      'workbench.history_page.load_failed' =>
        'Échec du chargement de l’historique',
      'workbench.history_page.retry' => 'Réessayer',
      'workbench.history_page.empty_title' => 'Aucun historique de traduction',
      'workbench.history_page.empty_description' =>
        'La traduction préférée est enregistrée ici après chaque réussite.',
      'workbench.history_page.no_results' => ({
        required Object query,
      }) => 'Aucun résultat pour « ${query} »',
      'workbench.history_page.clear_search' => 'Effacer la recherche',
      'workbench.history_page.select' => 'Sélection multiple',
      'workbench.history_page.selected_count' => ({
        required Object count,
      }) => '${count} sélectionnées',
      'workbench.history_page.exit_select' => 'Quitter la sélection',
      'workbench.history_page.add_to_glossary' => 'Ajouter au glossaire',
      'workbench.history_page.favorite' => 'Ajouter aux favoris',
      'workbench.history_page.unfavorite' => 'Retirer des favoris',
      'workbench.history_page.delete_confirm' => ({
        required Object count,
      }) => 'Supprimer les ${count} entrées sélectionnées ? Cette action est irréversible.',
      'workbench.history_page.no_glossary' => 'Créez d’abord un glossaire',
      'workbench.history_page.added_to_glossary' => ({
        required Object count,
      }) => '${count} entrées ajoutées au glossaire',
      'workbench.history_page.favorite_flag' => 'Favori',
      'workbench.history_page.edited_flag' => 'Modifié',
      'workbench.history_page.edit_history_hint' =>
        'La traduction modifiée sera enregistrée dans l’historique',
      'workbench.glossary' => 'Glossaire',
      'workbench.recent_languages' => 'Langues récentes',
      'workbench.not_configured' => 'Non configuré',
      'workbench.subtitle.translate' => 'Atelier · Comparaison des services',
      'workbench.subtitle.settings' => 'Réglages',
      'workbench.placeholder.history' => 'Consultez, modifiez, ajoutez aux favoris et réutilisez vos traductions',
      'workbench.placeholder.glossary' =>
        'Gérez les termes imposés et les variantes interdites',
      'workbench.glossary_page.add_entry' => 'Nouvelle entrée',
      'workbench.glossary_page.term' => 'Source',
      'workbench.glossary_page.translation' => 'Traduction imposée',
      'workbench.glossary_page.forbidden' => 'Interdit',
      'workbench.glossary_page.hits' => 'Occur.',
      'workbench.glossary_page.term_placeholder' => 'teacher forcing',
      'workbench.glossary_page.translation_placeholder' => 'forçage enseignant',
      'workbench.glossary_page.forbidden_placeholder' => 'enseignement forcé',
      'workbench.glossary_page.search' => 'Rechercher',
      'workbench.glossary_page.search_placeholder' =>
        'Rechercher un terme ou une traduction imposée',
      'workbench.glossary_page.search_label' => 'Rechercher dans le glossaire',
      'workbench.glossary_page.entry_count' => ({
        required Object name,
        required Object count,
      }) => '${name} · ${count} termes',
      'workbench.glossary_page.priority_note' =>
        'Le glossaire prime sur toute sortie de service',
      'workbench.glossary_page.new_book' => 'Nouveau glossaire',
      'workbench.glossary_page.new_book_placeholder' => 'Nom du glossaire',
      'workbench.glossary_page.rename_book' => 'Renommer',
      'workbench.glossary_page.delete_book_confirm' => ({
        required Object name,
        required Object count,
      }) => 'Supprimer « ${name} » et ses ${count} termes ?',
      'workbench.glossary_page.disabled' => 'Désactivé',
      'workbench.glossary_page.enable' => 'Activer',
      'workbench.glossary_page.disable' => 'Désactiver',
      'workbench.glossary_page.empty_title' => 'Ce glossaire est vide',
      'workbench.glossary_page.empty_description' => 'Le glossaire prime sur toute sortie de service. Ajoutez les termes un à un, ou déposez un CSV pour les fusionner.',
      'workbench.glossary_page.no_results_title' => ({
        required Object query,
      }) => 'Aucun terme ne correspond à « ${query} »',
      'workbench.glossary_page.no_results_description' =>
        'Essayez un autre mot-clé, ou ajoutez le terme.',
      'workbench.glossary_page.no_books_title' =>
        'Aucun glossaire pour l’instant',
      'workbench.glossary_page.no_books_description' => 'Un glossaire garde vos choix de traduction cohérents sur tous les services. Créez-en un, puis ajoutez des termes.',
      'workbench.glossary_page.loading' => 'Chargement…',
      'workbench.translation.source' => 'Source',
      'workbench.translation.target' => 'Traduction',
      'workbench.translation.input_hint' =>
        'Saisissez ou collez le texte à traduire',
      'workbench.translation.button' => 'Traduire',
      'workbench.translation.auto_detected' => 'Détecté automatiquement',
      'workbench.translation.loading_services' =>
        'Chargement des services de traduction…',
      'workbench.translation.no_services' =>
        'Configurez d’abord un service de traduction',
      'workbench.translation.translating' => 'Traduction…',
      'workbench.translation.failed' =>
        'Échec de la traduction. Vérifiez la configuration du service.',
      'workbench.translation.empty' => 'La traduction apparaîtra ici',
      'workbench.translation.service_compare' => 'Comparaison des services',
      'workbench.translation.main_translation' => 'Principale',
      'workbench.translation.service_unavailable' => 'Service indisponible',
      'workbench.translation.waiting' => 'En attente',
      'workbench.translation.copy' => 'Copier',
      'workbench.translation.favorite_unavailable' =>
        'Impossible de mettre à jour le favori. Réessayez.',
      'workbench.translation.preferred' => 'Traduction préférée',
      'workbench.translation.other_services' => 'Autres services',
      'workbench.translation.copy_result' => 'Copier la traduction',
      'workbench.translation.copied' => 'Copié',
      'workbench.translation.favorite' => 'Enregistrer',
      'workbench.translation.terms' => 'Correspondances du glossaire',
      'workbench.translation.terms_hint' =>
        'Les termes sont comparés à la saisie.',
      'workbench.translation.quality' => 'Signaux de qualité',
      'workbench.translation.quality_hint' =>
        'Calculés une fois la traduction reçue.',
      'workbench.translation.shortcuts' => 'Raccourcis',
      'workbench.translation.other_services_disabled' =>
        'Les autres services sont désactivés',
      'workbench.translation.input_hint_translate_to' => ({
        required Object language,
      }) => 'Saisissez ou collez le texte à traduire en ${language}',
      'workbench.translation.clear' => 'Effacer',
      'workbench.translation.swap_languages' => 'Inverser les langues',
      'workbench.translation.services' => 'Services de traduction',
      'workbench.translation.configure_services' => 'Configurer les services',
      'workbench.translation.retry' => 'Réessayer',
      'workbench.translation.character_count' => ({
        required Object count,
      }) => '${count} caractères',
      'workbench.status.runtime_ready' => 'Moteur de traduction prêt',
      'workbench.status.settings_synced' => 'Réglages synchronisés',
      'workbench.status.shortcuts' => '⌥Space Fenêtre rapide · ⌥⇧2 Capture',
      'workbench.version_latest' => 'À jour',
      'workbench.version_checking' => 'Vérification…',
      'workbench.check_updates' => 'Rechercher des mises à jour',
      'settings.version' => 'v{} (Build {})',
      'settings.general.title' => 'Général',
      'settings.general.section.permissions' => 'Autorisations système',
      'settings.general.section.ocr' => 'Reconnaissance de texte',
      'settings.general.section.directory' => 'Répertoire',
      'settings.general.section.translation' => 'Traduction',
      'settings.general.section.translation_target' => 'Cible de traduction',
      'settings.general.section.languages' => 'Langues',
      'settings.general.section.input' => 'Paramètres de saisie',
      'settings.general.section.startup' => 'Démarrage et intégration',
      'settings.general.section.ocr_behaviour' => 'Comportement de capture',
      'settings.general.section.translation_behaviour' =>
        'Comportement de traduction',
      'settings.general.row.launch_at_login' =>
        'Lancer à l\'ouverture de session',
      'settings.general.row.show_in_menu_bar' =>
        'Afficher dans la barre de menus',
      'settings.general.row.screen_capture_access' =>
        'Accorder l\'accès à l\'enregistrement d\'écran',
      'settings.general.row.screen_selection_access' =>
        'Accorder l\'accès à l\'accessibilité',
      'settings.general.row.default_ocr_service' =>
        'Service de reconnaissance de texte par défaut',
      'settings.general.row.auto_copy_detected_text' =>
        'Copier automatiquement le texte détecté',
      'settings.general.row.default_directory_service' =>
        'Service de répertoire par défaut',
      'settings.general.row.default_translation_service' =>
        'Service de traduction par défaut',
      'settings.general.row.translation_target_hint' =>
        'Configurez les paires de langues utilisées par le traducteur.',
      'settings.general.row.common_languages' => 'Langues courantes',
      'settings.general.row.common_languages_description' =>
        'Les langues courantes apparaissent en haut des sélecteurs de langue.',
      'settings.general.row.common_languages_hint' =>
        'Sélectionnez vos langues courantes:',
      'settings.general.row.common_languages_sort' => 'Trier par code',
      'settings.general.row.common_languages_reset' => 'Réinitialiser',
      'settings.general.row.common_languages_reset_help' =>
        'Rétablir l\'ensemble par défaut des langues courantes',
      'settings.general.row.common_languages_search' =>
        'Rechercher des langues...',
      'settings.general.row.common_languages_all' => 'Toutes les langues',
      'settings.general.row.double_click_copy_result' =>
        'Double-cliquer pour copier le résultat de la traduction',
      'settings.general.row.submit_with_enter' => 'Valider avec Entrée',
      'settings.general.row.submit_with_meta_enter_mac' =>
        'Valider avec ⌘ + Entrée',
      'settings.general.row.screen_capture_access_hint' =>
        'La capture de texte à l’écran nécessite de lire son contenu.',
      'settings.general.row.screen_selection_access_hint' => 'La capture du texte sélectionné nécessite de lire les sélections des autres applications.',
      'settings.general.row.no_translation_targets' => 'Aucune cible de traduction — ajoutez-en une pour définir la langue par défaut.',
      'settings.general.button.add_provider' => 'Ajouter...',
      'settings.general.button.add_target' => 'Ajouter une cible...',
      'settings.general.button.manage_targets' =>
        'Gérer les cibles de traduction...',
      'settings.general.button.manage_languages' =>
        'Gérer les langues courantes...',
      'settings.general.button.grant' => 'Accorder',
      'settings.general.option.none' => 'Aucun',
      'settings.general.option.no_services_available' =>
        'Aucun service disponible',
      'settings.general.option.granted' => 'Accordée',
      'settings.general.option.built_in_ocr' => 'OCR intégré',
      'settings.general.option.tesseract' => 'Tesseract',
      'settings.general.option.youdao_ocr' => 'OCR Youdao',
      'settings.general.editor.add_target_title' =>
        'Ajouter une cible de traduction',
      'settings.general.editor.edit_target_title' =>
        'Modifier la cible de traduction',
      'settings.general.editor.row.source_language' => 'Langue source',
      'settings.general.editor.row.target_language' => 'Langue cible',
      'settings.general.editor.title_edit' => 'Modifier la cible de traduction',
      'settings.general.editor.subtitle' =>
        'Définit vers quelle langue une langue source est traduite par défaut',
      'settings.general.editor.same_language' => 'La langue source et la cible sont identiques — choisissez une autre cible.',
      'settings.general.editor.duplicate' =>
        'Une cible avec cette paire existe déjà.',
      'settings.general.editor.hint_auto' =>
        'Traduire vers {} lorsque aucune autre règle ne correspond.',
      'settings.general.editor.hint_source' =>
        'Traduire vers {} lorsque {} est détecté.',
      'settings.appearance.title' => 'Apparence',
      'settings.appearance.section.app_language' => 'Langue d\'affichage',
      'settings.appearance.section.theme_mode' => 'Mode du thème',
      'settings.appearance.section.theme_style' => 'Style de thème',
      'settings.appearance.footer' =>
        'Les modifications s’appliquent immédiatement à toute la fenêtre.',
      'settings.shortcuts.title' => 'Raccourcis',
      'settings.shortcuts.section.text_extraction' => 'Extraction de texte',
      'settings.shortcuts.section.input_assist' =>
        'Fonction d\'aide à la saisie',
      'settings.shortcuts.section.submit_mode' => 'Envoyer avec',
      'settings.shortcuts.row.toggle_mini_translator' =>
        'Afficher/Masquer la fenêtre',
      'settings.shortcuts.row.extract_text_from_screen_selection' =>
        'Extraire le texte d\'une sélection d\'écran',
      'settings.shortcuts.row.extract_text_from_screen_capture' =>
        'Extraire le texte d\'une capture d\'écran',
      'settings.shortcuts.row.extract_text_from_clipboard' =>
        'Extraire le texte du presse-papiers',
      'settings.shortcuts.row.translate_input' => 'Traduire le contenu saisi',
      'settings.shortcuts.reset_dialog.title' => 'Réinitialiser les raccourcis',
      'settings.shortcuts.reset_dialog.message' => 'Êtes-vous sûr de vouloir réinitialiser tous les raccourcis à leurs valeurs par défaut ?',
      'settings.shortcuts.reset_dialog.confirm' => 'Réinitialiser',
      'settings.shortcuts.reset_dialog.cancel' => 'Annuler',
      'settings.shortcuts.group.global.title' => 'Raccourcis globaux',
      'settings.shortcuts.group.global.description' =>
        'Fonctionnent dans toutes les applications.',
      'settings.shortcuts.group.in_app.title' => 'Touches de l’application',
      'settings.shortcuts.group.in_app.description' =>
        'S’appliquent uniquement aux champs de saisie de cette application.',
      'settings.shortcuts.reset' => 'Rétablir les valeurs par défaut...',
      'settings.advanced.title' => 'Avancé',
      'settings.advanced.api_server' => 'Serveur API local',
      'settings.advanced.api_server_description' => 'Expose l\'API de traduction sur 127.0.0.1 pour les intégrations locales.',
      'settings.advanced.enable' => 'Activer',
      'settings.advanced.port' => 'Port',
      'settings.advanced.running_at' => 'En cours d\'exécution à {url}',
      'settings.advanced.disabled' => 'Désactivé',
      'settings.services.title' => 'Services',
      'settings.services.button.add_service' => 'Ajouter un service...',
      'settings.services.section.available_services' => 'Services disponibles',
      'settings.services.editor.title' => 'Ajouter un service',
      'settings.services.editor.subtitle' =>
        'Ajoutez un service à un fournisseur déjà configuré',
      'settings.services.editor.row.model' => 'Modèle',
      'settings.services.editor.row.system_prompt' => 'Prompt système',
      'settings.services.editor.prompt_placeholder' =>
        'Laissez vide pour utiliser le prompt par défaut de ce type',
      'settings.services.editor.variant_hint' => '{} a déjà un service de {} ; celui-ci vient s’y ajouter comme seconde configuration.',
      'settings.services.editor.traditional_note' => '{} est une interface traditionnelle : ni modèle ni prompt à régler. Ses paramètres se trouvent sur la page de détail du fournisseur.',
      'settings.services.detail.row.id' => 'ID du service',
      'settings.services.detail.row.name' => 'Nom',
      'settings.services.detail.row.provider' => 'Fournisseur',
      'settings.services.detail.row.type' => 'Type',
      'settings.services.detail.delete_dialog.title' => 'Supprimer "{}" ?',
      'settings.services.detail.delete_dialog.message' =>
        'Ce service sera supprimé du fournisseur.',
      'settings.services.detail.prompt_variables' => 'Variables disponibles : {{sourceLanguage}}, {{targetLanguage}}, {{text}}',
      'settings.services.make_default' => 'Définir par défaut',
      'settings.services.item.none_of_kind' =>
        'Aucun service de {} disponible pour l’instant.',
      'settings.providers.title' => 'Fournisseurs',
      'settings.providers.section.services' => 'Services disponibles',
      'settings.providers.section.services_description' => 'Consultez les services disponibles des fournisseurs configurés et changez de type de service.',
      'settings.providers.item.empty' => 'Aucun fournisseur configuré. Ajoutez-en un pour activer les services de traduction.',
      'settings.providers.item.loading' => 'Chargement des fournisseurs...',
      'settings.providers.item.no_services' => 'Aucun service disponible.',
      'settings.providers.button.add' => 'Ajouter un fournisseur...',
      'settings.providers.alert.error' => 'Erreur',
      'settings.providers.intro.body' =>
        'Gérez les fournisseurs de services utilisés par l’application.',
      'settings.providers.intro.warning' => 'Les fournisseurs connectés peuvent traiter le texte ou les images que vous envoyez. Activez uniquement les services de confiance.',
      'settings.providers.editor.row.id' => 'Identifiant du fournisseur',
      'settings.providers.editor.row.type' => 'Type de fournisseur',
      'settings.providers.editor.row.default_model' => 'Modèle par défaut',
      'settings.providers.editor.placeholder.id' => 'p. ex. deepl-main',
      'settings.providers.editor.type_picker.prompt' =>
        'Sélectionnez le type de fournisseur que vous souhaitez ajouter :',
      'settings.providers.editor.type_picker.section_llm' => 'LLM',
      'settings.providers.editor.type_picker.section_traditional' =>
        'Traditionnel',
      'settings.providers.editor.tooltip.help' => 'Aide',
      'settings.providers.editor.step.next' => 'Continuer',
      'settings.providers.editor.step.back' => 'Précédent',
      'settings.providers.editor.add_title' => 'Ajouter {}',
      'settings.providers.editor.capability_note.translation' =>
        'Rejoint les traductions candidates',
      'settings.providers.editor.capability_note.dictionary' =>
        'Fournit des définitions de dictionnaire',
      'settings.providers.editor.capability_note.ocr' =>
        'Reconnaît le texte des images',
      'settings.providers.editor.test.run' => 'Tester la connexion',
      'settings.providers.editor.test.running' =>
        'Test de la connexion · {}s écoulées',
      'settings.providers.editor.test.passed_models' =>
        'Connexion OK · {} modèles disponibles',
      'settings.providers.editor.test.passed_service' =>
        'Connexion OK · service disponible',
      'settings.providers.editor.test.passed_footer' =>
        'Test de connexion réussi',
      'settings.providers.editor.test.retest' => 'Retester',
      'settings.providers.editor.test.tips_title' => 'À essayer',
      'settings.providers.editor.test.tips_llm' => '· Vérifiez que la clé correspond au type de fournisseur choisi\n· Vérifiez si la Base URL doit contenir /v1\n· Confirmez que le modèle est activé dans la console du fournisseur',
      'settings.providers.editor.test.tips_traditional' => '· Vérifiez que les identifiants correspondent au type de fournisseur choisi\n· Confirmez que le service est activé dans la console du fournisseur',
      'settings.providers.editor.test.failed_suffix' =>
        'échec de la vérification',
      'settings.providers.editor.test.passed_suffix' => 'vérifié',
      'settings.providers.detail.tooltip.edit' => 'Modifier le fournisseur',
      'settings.providers.detail.row.id_hint' =>
        'Non modifiable après la création',
      'settings.providers.detail.section.configuration' => 'Configuration',
      'settings.providers.detail.section.models' => 'Modèles',
      'settings.providers.detail.models.loading' => 'Chargement des modèles...',
      'settings.providers.detail.models.empty' => 'Aucun modèle trouvé.',
      'settings.providers.detail.models.retry' => 'Réessayer',
      'settings.providers.detail.models.refresh' => 'Actualiser la liste',
      'settings.providers.detail.models.default_badge' => 'Par défaut',
      'settings.providers.detail.models.set_default' => 'Définir par défaut',
      'settings.providers.detail.models.fetch_error' =>
        'Impossible de récupérer les modèles depuis l\'API du fournisseur.',
      'settings.providers.capability.translation' => 'Traduction',
      'settings.providers.capability.dictionary' => 'Dictionnaire',
      'settings.providers.capability.ocr' => 'OCR',
      'settings.providers.capability.llm' => 'IA',
      'settings.providers.description.all' =>
        'Fournit la recherche dans le dictionnaire et la traduction de texte',
      'settings.providers.description.dictionary' =>
        'Fournit la recherche dans le dictionnaire et les définitions de mots',
      'settings.providers.description.translation' =>
        'Fournit la traduction de texte entre les langues',
      'settings.providers.description.fallback' =>
        'Fournit des services de traduction',
      'settings.providers.delete_dialog.title' => 'Supprimer « {} » ?',
      'settings.providers.delete_dialog.message' =>
        'Cette action est irréversible.',
      'settings.layout.title' => 'Paramètres',
      'settings.layout.empty.title' => 'Sélectionnez une catégorie',
      'settings.layout.empty.message' =>
        'Choisissez une section de paramètres dans la barre latérale.',
      'settings.layout.groups' => 'Groupes',
      'settings.layout.effect_hint' =>
        'Les changements prennent effet immédiatement',
      'settings.layout.footer_note' =>
        'Traductions et clés restent sur cet appareil',
      'settings.layout.support' => 'Assistance',
      'settings.about.title' => 'À propos',
      'settings.about.copy_version_info' =>
        'Copier les informations de version',
      'settings.about.up_to_date' => 'Vous êtes à jour.',
      'settings.about.check_again' => 'Vérifier à nouveau',
      'settings.about.links' => 'Liens',
      'settings.about.website' => 'Site web',
      'settings.about.github' => 'GitHub',
      'settings.about.report_issue' => 'Signaler un problème',
      'settings.about.license' => 'Licence',
      'settings.about.open_changelog' => 'Ouvrir le journal des modifications',
      'settings.about.update' => 'Mise à jour',
      _ => null,
    };
  }
}
