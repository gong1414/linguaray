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
class TranslationsEs extends Translations
    with BaseTranslations<AppLocale, Translations> {
  /// You can call this constructor and build your own translation instance of this locale.
  /// Constructing via the enum [AppLocale.build] is preferred.
  TranslationsEs({
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
             locale: AppLocale.es,
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

  /// Metadata for the translations of <es>.
  final TranslationMetadata<AppLocale, Translations> _meta;
  @override
  TranslationMetadata<AppLocale, Translations> get $meta => _meta;

  /// Access flat map
  @override
  dynamic operator [](String key) => _meta.getTranslation(key) ?? super[key];

  late final TranslationsEs _root = this; // ignore: unused_field

  @override
  TranslationsEs $copyWith({
    TranslationMetadata<AppLocale, Translations>? meta,
  }) => TranslationsEs(meta: meta ?? this.$meta);

  // Translations
  @override
  late final _Translations$common$es common = _Translations$common$es._(_root);
  @override
  late final _Translations$app$es app = _Translations$app$es._(_root);
  @override
  late final _Translations$mini_translator$es mini_translator =
      _Translations$mini_translator$es._(_root);
  @override
  late final _Translations$workbench$es workbench =
      _Translations$workbench$es._(_root);
  @override
  late final _Translations$settings$es settings = _Translations$settings$es._(
    _root,
  );
}

// Path: common
class _Translations$common$es extends Translations$common$en {
  _Translations$common$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  late final _Translations$common$ui$es ui = _Translations$common$ui$es._(
    _root,
  );
  @override
  late final _Translations$common$language$es language =
      _Translations$common$language$es._(_root);
  @override
  late final _Translations$common$theme_mode$es theme_mode =
      _Translations$common$theme_mode$es._(_root);
  @override
  late final _Translations$common$theme_style$es theme_style =
      _Translations$common$theme_style$es._(_root);
  @override
  late final _Translations$common$provider$es provider =
      _Translations$common$provider$es._(_root);
}

// Path: app
class _Translations$app$es extends Translations$app$en {
  _Translations$app$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  late final _Translations$app$tray$es tray = _Translations$app$tray$es._(
    _root,
  );
}

// Path: mini_translator
class _Translations$mini_translator$es extends Translations$mini_translator$en {
  _Translations$mini_translator$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  late final _Translations$mini_translator$limited_banner$es limited_banner =
      _Translations$mini_translator$limited_banner$es._(_root);
  @override
  late final _Translations$mini_translator$input$es input =
      _Translations$mini_translator$input$es._(_root);
  @override
  late final _Translations$mini_translator$toolbar$es toolbar =
      _Translations$mini_translator$toolbar$es._(_root);
  @override
  late final _Translations$mini_translator$button$es button =
      _Translations$mini_translator$button$es._(_root);
  @override
  late final _Translations$mini_translator$language$es language =
      _Translations$mini_translator$language$es._(_root);
  @override
  late final _Translations$mini_translator$message$es message =
      _Translations$mini_translator$message$es._(_root);
  @override
  late final _Translations$mini_translator$result$es result =
      _Translations$mini_translator$result$es._(_root);
}

// Path: workbench
class _Translations$workbench$es extends Translations$workbench$en {
  _Translations$workbench$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get workspace => 'Espacio de trabajo';
  @override
  String get translate => 'Traducir';
  @override
  String get history => 'Historial';
  @override
  late final _Translations$workbench$history_page$es history_page =
      _Translations$workbench$history_page$es._(_root);
  @override
  String get glossary => 'Glosario';
  @override
  String get recent_languages => 'Idiomas recientes';
  @override
  String get not_configured => 'Sin configurar';
  @override
  late final _Translations$workbench$subtitle$es subtitle =
      _Translations$workbench$subtitle$es._(_root);
  @override
  late final _Translations$workbench$placeholder$es placeholder =
      _Translations$workbench$placeholder$es._(_root);
  @override
  late final _Translations$workbench$glossary_page$es glossary_page =
      _Translations$workbench$glossary_page$es._(_root);
  @override
  late final _Translations$workbench$translation$es translation =
      _Translations$workbench$translation$es._(_root);
  @override
  late final _Translations$workbench$status$es status =
      _Translations$workbench$status$es._(_root);
  @override
  String get version_latest => 'Actualizado';
  @override
  String get version_checking => 'Comprobando…';
  @override
  String get check_updates => 'Buscar actualizaciones';
}

// Path: settings
class _Translations$settings$es extends Translations$settings$en {
  _Translations$settings$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get version => 'v{} (Build {})';
  @override
  late final _Translations$settings$general$es general =
      _Translations$settings$general$es._(_root);
  @override
  late final _Translations$settings$appearance$es appearance =
      _Translations$settings$appearance$es._(_root);
  @override
  late final _Translations$settings$shortcuts$es shortcuts =
      _Translations$settings$shortcuts$es._(_root);
  @override
  late final _Translations$settings$advanced$es advanced =
      _Translations$settings$advanced$es._(_root);
  @override
  late final _Translations$settings$services$es services =
      _Translations$settings$services$es._(_root);
  @override
  late final _Translations$settings$providers$es providers =
      _Translations$settings$providers$es._(_root);
  @override
  late final _Translations$settings$layout$es layout =
      _Translations$settings$layout$es._(_root);
  @override
  late final _Translations$settings$about$es about =
      _Translations$settings$about$es._(_root);
}

// Path: common.ui
class _Translations$common$ui$es extends Translations$common$ui$en {
  _Translations$common$ui$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  late final _Translations$common$ui$button$es button =
      _Translations$common$ui$button$es._(_root);
  @override
  late final _Translations$common$ui$feedback$es feedback =
      _Translations$common$ui$feedback$es._(_root);
}

// Path: common.language
class _Translations$common$language$es extends Translations$common$language$en {
  _Translations$common$language$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get ar => 'Árabe';
  @override
  String get bn => 'Bengalí';
  @override
  String get de => 'Alemán';
  @override
  String get en => 'Inglés';
  @override
  String get es => 'Español';
  @override
  String get fa => 'Persa';
  @override
  String get fr => 'Francés';
  @override
  String get gu => 'Guyaratí';
  @override
  String get ha => 'Hausa';
  @override
  String get hi => 'Hindi';
  @override
  String get id => 'Indonesio';
  @override
  String get it => 'Italiano';
  @override
  String get ja => 'Japonés';
  @override
  String get jv => 'Javanés';
  @override
  String get ko => 'Coreano';
  @override
  String get ml => 'Malayalam';
  @override
  String get mr => 'Maratí';
  @override
  String get ms => 'Malayo';
  @override
  String get nl => 'Neerlandés';
  @override
  String get pa => 'Panyabí';
  @override
  String get pl => 'Polaco';
  @override
  String get pt => 'Portugués';
  @override
  String get ro => 'Rumano';
  @override
  String get ru => 'Ruso';
  @override
  String get sw => 'Suajili';
  @override
  String get ta => 'Tamil';
  @override
  String get te => 'Telugú';
  @override
  String get th => 'Tailandés';
  @override
  String get tr => 'Turco';
  @override
  String get uk => 'Ucraniano';
  @override
  String get ur => 'Urdu';
  @override
  String get vi => 'Vietnamita';
  @override
  String get yo => 'Yoruba';
  @override
  String get zh_hans => 'Chino simplificado';
  @override
  String get zh_hant => 'Chino tradicional';
}

// Path: common.theme_mode
class _Translations$common$theme_mode$es
    extends Translations$common$theme_mode$en {
  _Translations$common$theme_mode$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get light => 'Claro';
  @override
  String get dark => 'Oscuro';
  @override
  String get system => 'Sistema';
}

// Path: common.theme_style
class _Translations$common$theme_style$es
    extends Translations$common$theme_style$en {
  _Translations$common$theme_style$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get studio => 'Studio';
  @override
  String get bright => 'Bright';
}

// Path: common.provider
class _Translations$common$provider$es extends Translations$common$provider$en {
  _Translations$common$provider$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

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
  String get system => 'Sistema';
  @override
  String get tencent => 'Tencent';
  @override
  String get youdao => 'Youdao';
}

// Path: app.tray
class _Translations$app$tray$es extends Translations$app$tray$en {
  _Translations$app$tray$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  late final _Translations$app$tray$context_menu$es context_menu =
      _Translations$app$tray$context_menu$es._(_root);
}

// Path: mini_translator.limited_banner
class _Translations$mini_translator$limited_banner$es
    extends Translations$mini_translator$limited_banner$en {
  _Translations$mini_translator$limited_banner$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  late final _Translations$mini_translator$limited_banner$permission$es
  permission = _Translations$mini_translator$limited_banner$permission$es._(
    _root,
  );
  @override
  late final _Translations$mini_translator$limited_banner$instruction$es
  instruction = _Translations$mini_translator$limited_banner$instruction$es._(
    _root,
  );
  @override
  late final _Translations$mini_translator$limited_banner$action$es action =
      _Translations$mini_translator$limited_banner$action$es._(_root);
  @override
  late final _Translations$mini_translator$limited_banner$feedback$es feedback =
      _Translations$mini_translator$limited_banner$feedback$es._(_root);
  @override
  late final _Translations$mini_translator$limited_banner$tooltip$es tooltip =
      _Translations$mini_translator$limited_banner$tooltip$es._(_root);
}

// Path: mini_translator.input
class _Translations$mini_translator$input$es
    extends Translations$mini_translator$input$en {
  _Translations$mini_translator$input$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get hint => 'Escribe la palabra o texto aquí';
  @override
  String get extracting_text => 'Extrayendo texto...';
  @override
  String hint_translate_to({required Object language}) =>
      'Escribe una palabra o texto para traducir al ${language}';
}

// Path: mini_translator.toolbar
class _Translations$mini_translator$toolbar$es
    extends Translations$mini_translator$toolbar$en {
  _Translations$mini_translator$toolbar$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  late final _Translations$mini_translator$toolbar$tooltip$es tooltip =
      _Translations$mini_translator$toolbar$tooltip$es._(_root);
  @override
  late final _Translations$mini_translator$toolbar$menu$es menu =
      _Translations$mini_translator$toolbar$menu$es._(_root);
}

// Path: mini_translator.button
class _Translations$mini_translator$button$es
    extends Translations$mini_translator$button$en {
  _Translations$mini_translator$button$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get clear => 'Limpiar';
  @override
  String get translate => 'Traducir';
  @override
  String get copy => 'Copiar';
  @override
  String get copied => 'Copiado';
  @override
  String get bookmark => 'Guardar';
  @override
  String get bookmarked => 'Guardado';
}

// Path: mini_translator.language
class _Translations$mini_translator$language$es
    extends Translations$mini_translator$language$en {
  _Translations$mini_translator$language$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get auto_detect => 'Detección automática';
  @override
  String get auto_match => 'Coincidencia auto';
  @override
  String get switch_config => 'Cambiar destino';
  @override
  String get more_languages => 'Más idiomas...';
  @override
  String get manage_common_languages => 'Administrar idiomas comunes...';
  @override
  String get manage_targets => 'Administrar destinos de traducción...';
  @override
  String get add_target => 'Agregar destino de traducción...';
}

// Path: mini_translator.message
class _Translations$mini_translator$message$es
    extends Translations$mini_translator$message$en {
  _Translations$mini_translator$message$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get please_enter_word_or_text =>
      'No se ingresó texto o no se pudo extraer';
  @override
  String get capture_screen_area_canceled =>
      'La captura de área de pantalla ha sido cancelada';
  @override
  String get ocr_service_not_configured =>
      'No hay un servicio de reconocimiento de texto predeterminado configurado. Configúrelo en Ajustes.';
  @override
  String get ocr_recognition_failed => 'El reconocimiento de texto falló';
}

// Path: mini_translator.result
class _Translations$mini_translator$result$es
    extends Translations$mini_translator$result$en {
  _Translations$mini_translator$result$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get translating => 'Traduciendo…';
  @override
  String stale_requery({required Object key}) =>
      'Texto modificado · ${key} retraducir';
  @override
  String compare_services({required Object count}) =>
      'Comparar ${count} servicios';
  @override
  String get collapse_compare => 'Contraer';
  @override
  String get set_preferred => 'Establecer como preferido';
  @override
  String get retry => 'Reintentar';
  @override
  String get no_result =>
      'Ningún servicio devolvió resultados: revisa la red o prueba otro servicio.';
  @override
  String get no_result_note =>
      'El texto se conserva; reintentar no duplicará el historial.';
}

// Path: workbench.history_page
class _Translations$workbench$history_page$es
    extends Translations$workbench$history_page$en {
  _Translations$workbench$history_page$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get all => 'Todo';
  @override
  String get favorites => 'Favoritos';
  @override
  String get edited => 'Editados por mí';
  @override
  String get search => 'Buscar';
  @override
  String get search_placeholder => 'Buscar original, traducción o servicio';
  @override
  String get search_label => 'Buscar en el historial';
  @override
  String entry_count({required Object label, required Object count}) =>
      '${label} · ${count} entradas';
  @override
  String get by_time => 'Por fecha';
  @override
  String get loading => 'Cargando historial…';
  @override
  String get load_failed => 'No se pudo cargar el historial';
  @override
  String get retry => 'Reintentar';
  @override
  String get empty_title => 'Aún no hay historial de traducción';
  @override
  String get empty_description =>
      'La traducción preferida se guarda aquí al completarse.';
  @override
  String no_results({required Object query}) =>
      'No hay resultados para «${query}»';
  @override
  String get clear_search => 'Borrar búsqueda';
  @override
  String get select => 'Selección múltiple';
  @override
  String selected_count({required Object count}) => '${count} seleccionadas';
  @override
  String get exit_select => 'Salir de selección';
  @override
  String get add_to_glossary => 'Añadir al glosario';
  @override
  String get favorite => 'Favorito';
  @override
  String get unfavorite => 'Quitar favorito';
  @override
  String delete_confirm({required Object count}) =>
      '¿Eliminar las ${count} entradas seleccionadas? No se puede deshacer.';
  @override
  String get no_glossary => 'Crea primero un glosario';
  @override
  String added_to_glossary({required Object count}) =>
      '${count} entradas añadidas al glosario';
  @override
  String get favorite_flag => 'Favorito';
  @override
  String get edited_flag => 'Editado';
  @override
  String get edit_history_hint =>
      'La traducción editada se guardará en el historial';
}

// Path: workbench.subtitle
class _Translations$workbench$subtitle$es
    extends Translations$workbench$subtitle$en {
  _Translations$workbench$subtitle$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get translate => 'Mesa de trabajo · Comparación de servicios';
  @override
  String get settings => 'Ajustes';
}

// Path: workbench.placeholder
class _Translations$workbench$placeholder$es
    extends Translations$workbench$placeholder$en {
  _Translations$workbench$placeholder$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get history =>
      'Revisa, edita, guarda y reutiliza traducciones anteriores';
  @override
  String get glossary =>
      'Gestiona términos obligatorios y alternativas prohibidas';
}

// Path: workbench.glossary_page
class _Translations$workbench$glossary_page$es
    extends Translations$workbench$glossary_page$en {
  _Translations$workbench$glossary_page$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get add_entry => 'Nueva entrada';
  @override
  String get term => 'Origen';
  @override
  String get translation => 'Traducción obligatoria';
  @override
  String get forbidden => 'Prohibido';
  @override
  String get hits => 'Usos';
  @override
  String get term_placeholder => 'teacher forcing';
  @override
  String get translation_placeholder => 'forzado del maestro';
  @override
  String get forbidden_placeholder => 'enseñanza forzada';
  @override
  String get search => 'Buscar';
  @override
  String get search_placeholder =>
      'Buscar términos o traducciones obligatorias';
  @override
  String get search_label => 'Buscar en el glosario';
  @override
  String entry_count({required Object name, required Object count}) =>
      '${name} · ${count} términos';
  @override
  String get priority_note =>
      'El glosario tiene prioridad sobre cualquier servicio';
  @override
  String get new_book => 'Nuevo glosario';
  @override
  String get new_book_placeholder => 'Nombre del glosario';
  @override
  String get rename_book => 'Cambiar nombre';
  @override
  String delete_book_confirm({required Object name, required Object count}) =>
      '¿Eliminar «${name}» y sus ${count} términos?';
  @override
  String get disabled => 'Desactivado';
  @override
  String get enable => 'Activar';
  @override
  String get disable => 'Desactivar';
  @override
  String get empty_title => 'Este glosario está vacío';
  @override
  String get empty_description =>
      'El glosario tiene prioridad sobre cualquier servicio. Añade términos uno a uno, o suelta un CSV para combinarlos.';
  @override
  String no_results_title({required Object query}) =>
      'Ningún término coincide con «${query}»';
  @override
  String get no_results_description =>
      'Prueba otra palabra clave, o añade el término.';
  @override
  String get no_books_title => 'Todavía no hay glosarios';
  @override
  String get no_books_description =>
      'Un glosario mantiene tus traducciones coherentes en todos los servicios. Crea uno y empieza a añadir términos.';
  @override
  String get loading => 'Cargando…';
}

// Path: workbench.translation
class _Translations$workbench$translation$es
    extends Translations$workbench$translation$en {
  _Translations$workbench$translation$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get source => 'Origen';
  @override
  String get target => 'Traducción';
  @override
  String get input_hint => 'Escribe o pega el texto que quieras traducir';
  @override
  String get button => 'Traducir';
  @override
  String get auto_detected => 'Detectado automáticamente';
  @override
  String get loading_services => 'Cargando servicios de traducción…';
  @override
  String get no_services => 'Configura primero un servicio de traducción';
  @override
  String get translating => 'Traduciendo…';
  @override
  String get failed =>
      'La traducción falló. Revisa la configuración del servicio.';
  @override
  String get empty => 'La traducción aparecerá aquí';
  @override
  String get service_compare => 'Comparación de servicios';
  @override
  String get main_translation => 'Principal';
  @override
  String get service_unavailable => 'Servicio no disponible';
  @override
  String get waiting => 'Esperando traducción';
  @override
  String get copy => 'Copiar';
  @override
  String get favorite_unavailable =>
      'No se pudo actualizar el favorito. Inténtalo de nuevo.';
  @override
  String get preferred => 'Traducción preferida';
  @override
  String get other_services => 'Otros servicios';
  @override
  String get copy_result => 'Copiar traducción';
  @override
  String get copied => 'Copiado';
  @override
  String get favorite => 'Guardar';
  @override
  String get terms => 'Coincidencias del glosario';
  @override
  String get terms_hint => 'Los términos se comparan al escribir.';
  @override
  String get quality => 'Señales de calidad';
  @override
  String get quality_hint => 'Se calculan cuando llega la traducción.';
  @override
  String get shortcuts => 'Atajos';
  @override
  String get other_services_disabled =>
      'Los demás servicios están desactivados';
  @override
  String input_hint_translate_to({required Object language}) =>
      'Escribe o pega el texto para traducir al ${language}';
  @override
  String get clear => 'Borrar';
  @override
  String get swap_languages => 'Intercambiar idiomas';
  @override
  String get services => 'Servicios de traducción';
  @override
  String get configure_services => 'Configurar servicios';
  @override
  String get retry => 'Reintentar';
  @override
  String character_count({required Object count}) => '${count} caracteres';
}

// Path: workbench.status
class _Translations$workbench$status$es
    extends Translations$workbench$status$en {
  _Translations$workbench$status$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get runtime_ready => 'Motor de traducción listo';
  @override
  String get settings_synced => 'Ajustes sincronizados';
  @override
  String get shortcuts => '⌥Space Ventana rápida · ⌥⇧2 Captura';
}

// Path: settings.general
class _Translations$settings$general$es
    extends Translations$settings$general$en {
  _Translations$settings$general$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get title => 'General';
  @override
  late final _Translations$settings$general$section$es section =
      _Translations$settings$general$section$es._(_root);
  @override
  late final _Translations$settings$general$row$es row =
      _Translations$settings$general$row$es._(_root);
  @override
  late final _Translations$settings$general$button$es button =
      _Translations$settings$general$button$es._(_root);
  @override
  late final _Translations$settings$general$option$es option =
      _Translations$settings$general$option$es._(_root);
  @override
  late final _Translations$settings$general$editor$es editor =
      _Translations$settings$general$editor$es._(_root);
}

// Path: settings.appearance
class _Translations$settings$appearance$es
    extends Translations$settings$appearance$en {
  _Translations$settings$appearance$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get title => 'Apariencia';
  @override
  late final _Translations$settings$appearance$section$es section =
      _Translations$settings$appearance$section$es._(_root);
  @override
  String get footer => 'Los cambios se aplican de inmediato a toda la ventana.';
}

// Path: settings.shortcuts
class _Translations$settings$shortcuts$es
    extends Translations$settings$shortcuts$en {
  _Translations$settings$shortcuts$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get title => 'Atajos';
  @override
  late final _Translations$settings$shortcuts$section$es section =
      _Translations$settings$shortcuts$section$es._(_root);
  @override
  late final _Translations$settings$shortcuts$row$es row =
      _Translations$settings$shortcuts$row$es._(_root);
  @override
  late final _Translations$settings$shortcuts$reset_dialog$es reset_dialog =
      _Translations$settings$shortcuts$reset_dialog$es._(_root);
  @override
  late final _Translations$settings$shortcuts$group$es group =
      _Translations$settings$shortcuts$group$es._(_root);
  @override
  String get reset => 'Restaurar valores predeterminados...';
}

// Path: settings.advanced
class _Translations$settings$advanced$es
    extends Translations$settings$advanced$en {
  _Translations$settings$advanced$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get title => 'Avanzado';
  @override
  String get api_server => 'Servidor de API local';
  @override
  String get api_server_description =>
      'Exponer la API de traducción en 127.0.0.1 para integraciones locales.';
  @override
  String get enable => 'Habilitar';
  @override
  String get port => 'Puerto';
  @override
  String get running_at => 'Ejecutándose en {url}';
  @override
  String get disabled => 'Deshabilitado';
}

// Path: settings.services
class _Translations$settings$services$es
    extends Translations$settings$services$en {
  _Translations$settings$services$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get title => 'Servicios';
  @override
  late final _Translations$settings$services$button$es button =
      _Translations$settings$services$button$es._(_root);
  @override
  late final _Translations$settings$services$section$es section =
      _Translations$settings$services$section$es._(_root);
  @override
  late final _Translations$settings$services$editor$es editor =
      _Translations$settings$services$editor$es._(_root);
  @override
  late final _Translations$settings$services$detail$es detail =
      _Translations$settings$services$detail$es._(_root);
  @override
  String get make_default => 'Establecer como predeterminado';
  @override
  late final _Translations$settings$services$item$es item =
      _Translations$settings$services$item$es._(_root);
}

// Path: settings.providers
class _Translations$settings$providers$es
    extends Translations$settings$providers$en {
  _Translations$settings$providers$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get title => 'Proveedores';
  @override
  late final _Translations$settings$providers$section$es section =
      _Translations$settings$providers$section$es._(_root);
  @override
  late final _Translations$settings$providers$item$es item =
      _Translations$settings$providers$item$es._(_root);
  @override
  late final _Translations$settings$providers$button$es button =
      _Translations$settings$providers$button$es._(_root);
  @override
  late final _Translations$settings$providers$alert$es alert =
      _Translations$settings$providers$alert$es._(_root);
  @override
  late final _Translations$settings$providers$intro$es intro =
      _Translations$settings$providers$intro$es._(_root);
  @override
  late final _Translations$settings$providers$editor$es editor =
      _Translations$settings$providers$editor$es._(_root);
  @override
  late final _Translations$settings$providers$detail$es detail =
      _Translations$settings$providers$detail$es._(_root);
  @override
  late final _Translations$settings$providers$capability$es capability =
      _Translations$settings$providers$capability$es._(_root);
  @override
  late final _Translations$settings$providers$description$es description =
      _Translations$settings$providers$description$es._(_root);
  @override
  late final _Translations$settings$providers$delete_dialog$es delete_dialog =
      _Translations$settings$providers$delete_dialog$es._(_root);
}

// Path: settings.layout
class _Translations$settings$layout$es extends Translations$settings$layout$en {
  _Translations$settings$layout$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get title => 'Configuración';
  @override
  late final _Translations$settings$layout$empty$es empty =
      _Translations$settings$layout$empty$es._(_root);
  @override
  String get groups => 'Grupos';
  @override
  String get effect_hint => 'Los cambios se aplican al instante';
  @override
  String get footer_note =>
      'Las traducciones y claves se guardan solo en este equipo';
  @override
  String get support => 'Soporte';
}

// Path: settings.about
class _Translations$settings$about$es extends Translations$settings$about$en {
  _Translations$settings$about$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get title => 'Acerca de';
  @override
  String get copy_version_info => 'Copiar información de versión';
  @override
  String get up_to_date => 'Está actualizado.';
  @override
  String get check_again => 'Verificar de nuevo';
  @override
  String get links => 'Enlaces';
  @override
  String get website => 'Sitio web';
  @override
  String get github => 'GitHub';
  @override
  String get report_issue => 'Informar de un problema';
  @override
  String get license => 'Licencia';
  @override
  String get open_changelog => 'Abrir registro de cambios';
  @override
  String get update => 'Actualización';
}

// Path: common.ui.button
class _Translations$common$ui$button$es
    extends Translations$common$ui$button$en {
  _Translations$common$ui$button$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get ok => 'OK';
  @override
  String get cancel => 'Cancelar';
  @override
  String get add => 'Agregar';
  @override
  String get delete => 'Eliminar';
  @override
  String get edit => 'Editar';
  @override
  String get save => 'Guardar';
  @override
  String get manage => 'Administrar';
  @override
  String get kContinue => 'Continuar';
}

// Path: common.ui.feedback
class _Translations$common$ui$feedback$es
    extends Translations$common$ui$feedback$en {
  _Translations$common$ui$feedback$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get copied => 'Copiado';
}

// Path: app.tray.context_menu
class _Translations$app$tray$context_menu$es
    extends Translations$app$tray$context_menu$en {
  _Translations$app$tray$context_menu$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get show_window => 'Mostrar ventana';
  @override
  late final _Translations$app$tray$context_menu$dev_tools$es dev_tools =
      _Translations$app$tray$context_menu$dev_tools$es._(_root);
  @override
  String get check_for_updates => 'Buscar actualizaciones';
  @override
  String get settings => 'Configuración';
  @override
  String get quit => 'Salir';
}

// Path: mini_translator.limited_banner.permission
class _Translations$mini_translator$limited_banner$permission$es
    extends Translations$mini_translator$limited_banner$permission$en {
  _Translations$mini_translator$limited_banner$permission$es._(
    TranslationsEs root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get missing_both =>
      'Concede los permisos de Grabación de pantalla y Accesibilidad para habilitar todas las funciones.';
  @override
  String get missing_screen_capture =>
      'Concede el permiso de Grabación de pantalla para habilitar todas las funciones.';
  @override
  String get missing_accessibility =>
      'Concede el permiso de Accesibilidad para habilitar todas las funciones.';
}

// Path: mini_translator.limited_banner.instruction
class _Translations$mini_translator$limited_banner$instruction$es
    extends Translations$mini_translator$limited_banner$instruction$en {
  _Translations$mini_translator$limited_banner$instruction$es._(
    TranslationsEs root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get app_settings_prefix => 'Ve a ';
  @override
  String get follow_guide_prefix => ', sigue la guía y luego haz clic en ';
  @override
  String get suffix => '.';
}

// Path: mini_translator.limited_banner.action
class _Translations$mini_translator$limited_banner$action$es
    extends Translations$mini_translator$limited_banner$action$en {
  _Translations$mini_translator$limited_banner$action$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get app_settings => 'Configuración de la app';
  @override
  String get recheck => 'Verificar de nuevo';
}

// Path: mini_translator.limited_banner.feedback
class _Translations$mini_translator$limited_banner$feedback$es
    extends Translations$mini_translator$limited_banner$feedback$en {
  _Translations$mini_translator$limited_banner$feedback$es._(
    TranslationsEs root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get enabled => 'La extracción de texto de pantalla está habilitada.';
  @override
  String get still_missing =>
      'Los permisos necesarios aún faltan.\nRevisa tu configuración e inténtalo de nuevo.';
}

// Path: mini_translator.limited_banner.tooltip
class _Translations$mini_translator$limited_banner$tooltip$es
    extends Translations$mini_translator$limited_banner$tooltip$en {
  _Translations$mini_translator$limited_banner$tooltip$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get help => 'Ver ayuda';
}

// Path: mini_translator.toolbar.tooltip
class _Translations$mini_translator$toolbar$tooltip$es
    extends Translations$mini_translator$toolbar$tooltip$en {
  _Translations$mini_translator$toolbar$tooltip$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get extract_text_from_screen_capture =>
      'Capturar área de la pantalla y reconocer texto';
  @override
  String get extract_text_from_clipboard => 'Leer contenido del portapapeles';
  @override
  String get pin => 'Fijar ventana';
  @override
  String get more_actions => 'Más acciones';
}

// Path: mini_translator.toolbar.menu
class _Translations$mini_translator$toolbar$menu$es
    extends Translations$mini_translator$toolbar$menu$en {
  _Translations$mini_translator$toolbar$menu$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get extract_from_screen_capture => 'Capturar de pantalla';
  @override
  String get extract_from_clipboard => 'Desde el portapapeles';
  @override
  String get open_main_window => 'Abrir ventana principal';
  @override
  String get open_settings => 'Ajustes…';
}

// Path: settings.general.section
class _Translations$settings$general$section$es
    extends Translations$settings$general$section$en {
  _Translations$settings$general$section$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get permissions => 'Permisos del sistema';
  @override
  String get ocr => 'Reconocimiento de texto';
  @override
  String get directory => 'Directorio';
  @override
  String get translation => 'Traducción';
  @override
  String get translation_target => 'Idioma de destino';
  @override
  String get languages => 'Idiomas';
  @override
  String get input => 'Configuración de entrada';
  @override
  String get startup => 'Inicio e integración';
  @override
  String get ocr_behaviour => 'Comportamiento de captura';
  @override
  String get translation_behaviour => 'Comportamiento de traducción';
}

// Path: settings.general.row
class _Translations$settings$general$row$es
    extends Translations$settings$general$row$en {
  _Translations$settings$general$row$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get launch_at_login => 'Iniciar al iniciar sesión';
  @override
  String get show_in_menu_bar => 'Mostrar en la barra de menú';
  @override
  String get screen_capture_access =>
      'Conceder acceso de grabación de pantalla';
  @override
  String get screen_selection_access => 'Conceder acceso de accesibilidad';
  @override
  String get default_ocr_service =>
      'Servicio de reconocimiento de texto predeterminado';
  @override
  String get auto_copy_detected_text =>
      'Copiar texto detectado automáticamente';
  @override
  String get default_directory_service =>
      'Servicio de diccionario predeterminado';
  @override
  String get default_translation_service =>
      'Servicio de traducción predeterminado';
  @override
  String get translation_target_hint =>
      'Configura los pares de idiomas usados por el traductor.';
  @override
  String get common_languages => 'Idiomas comunes';
  @override
  String get common_languages_description =>
      'Los idiomas comunes aparecen al principio de los selectores de idioma.';
  @override
  String get common_languages_hint => 'Selecciona tus idiomas comunes:';
  @override
  String get common_languages_sort => 'Ordenar por código';
  @override
  String get common_languages_reset => 'Restablecer valores predeterminados';
  @override
  String get common_languages_reset_help =>
      'Restablecer el conjunto predeterminado de idiomas comunes';
  @override
  String get common_languages_search => 'Buscar idiomas...';
  @override
  String get common_languages_all => 'Todos los idiomas';
  @override
  String get double_click_copy_result =>
      'Doble clic para copiar el resultado de la traducción';
  @override
  String get submit_with_enter => 'Enviar con Enter';
  @override
  String get submit_with_meta_enter_mac => 'Enviar con ⌘ + Enter';
  @override
  String get screen_capture_access_hint =>
      'Capturar texto de la pantalla requiere leer su contenido.';
  @override
  String get screen_selection_access_hint =>
      'Capturar el texto seleccionado requiere leer selecciones de otras aplicaciones.';
  @override
  String get no_translation_targets =>
      'Aún no hay destinos de traducción; añade uno para fijar el idioma predeterminado.';
}

// Path: settings.general.button
class _Translations$settings$general$button$es
    extends Translations$settings$general$button$en {
  _Translations$settings$general$button$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get add_provider => 'Agregar...';
  @override
  String get add_target => 'Agregar destino...';
  @override
  String get manage_targets => 'Gestionar destinos de traducción...';
  @override
  String get manage_languages => 'Administrar idiomas comunes...';
  @override
  String get grant => 'Conceder';
}

// Path: settings.general.option
class _Translations$settings$general$option$es
    extends Translations$settings$general$option$en {
  _Translations$settings$general$option$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get none => 'Ninguno';
  @override
  String get no_services_available => 'No hay servicios disponibles';
  @override
  String get granted => 'Concedido';
  @override
  String get built_in_ocr => 'OCR integrado';
  @override
  String get tesseract => 'Tesseract';
  @override
  String get youdao_ocr => 'Youdao OCR';
}

// Path: settings.general.editor
class _Translations$settings$general$editor$es
    extends Translations$settings$general$editor$en {
  _Translations$settings$general$editor$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get add_target_title => 'Agregar destino de traducción';
  @override
  String get edit_target_title => 'Editar destino de traducción';
  @override
  late final _Translations$settings$general$editor$row$es row =
      _Translations$settings$general$editor$row$es._(_root);
  @override
  String get title_edit => 'Editar destino de traducción';
  @override
  String get subtitle =>
      'Decide a qué idioma se traduce un idioma de origen de forma predeterminada';
  @override
  String get same_language =>
      'El idioma de origen y el de destino son el mismo; elige otro destino.';
  @override
  String get duplicate => 'Ya existe un destino con esta combinación.';
  @override
  String get hint_auto => 'Traducir a {} cuando ninguna otra regla coincida.';
  @override
  String get hint_source => 'Traducir a {} cuando se detecte {}.';
}

// Path: settings.appearance.section
class _Translations$settings$appearance$section$es
    extends Translations$settings$appearance$section$en {
  _Translations$settings$appearance$section$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get app_language => 'Idioma de la interfaz';
  @override
  String get theme_mode => 'Modo de tema';
  @override
  String get theme_style => 'Estilo del tema';
}

// Path: settings.shortcuts.section
class _Translations$settings$shortcuts$section$es
    extends Translations$settings$shortcuts$section$en {
  _Translations$settings$shortcuts$section$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get text_extraction => 'Extracción de texto';
  @override
  String get input_assist => 'Función de asistencia de entrada';
  @override
  String get submit_mode => 'Enviar con';
}

// Path: settings.shortcuts.row
class _Translations$settings$shortcuts$row$es
    extends Translations$settings$shortcuts$row$en {
  _Translations$settings$shortcuts$row$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get toggle_mini_translator => 'Mostrar/Ocultar ventana';
  @override
  String get extract_text_from_screen_selection =>
      'Extraer texto de la selección de pantalla';
  @override
  String get extract_text_from_screen_capture =>
      'Extraer texto de la captura de pantalla';
  @override
  String get extract_text_from_clipboard => 'Extraer texto del portapapeles';
  @override
  String get translate_input => 'Traducir contenido de entrada';
}

// Path: settings.shortcuts.reset_dialog
class _Translations$settings$shortcuts$reset_dialog$es
    extends Translations$settings$shortcuts$reset_dialog$en {
  _Translations$settings$shortcuts$reset_dialog$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get title => 'Restablecer atajos';
  @override
  String get message =>
      '¿Estás seguro de que deseas restablecer todos los atajos a sus valores predeterminados?';
  @override
  String get confirm => 'Restablecer';
  @override
  String get cancel => 'Cancelar';
}

// Path: settings.shortcuts.group
class _Translations$settings$shortcuts$group$es
    extends Translations$settings$shortcuts$group$en {
  _Translations$settings$shortcuts$group$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  late final _Translations$settings$shortcuts$group$global$es global =
      _Translations$settings$shortcuts$group$global$es._(_root);
  @override
  late final _Translations$settings$shortcuts$group$in_app$es in_app =
      _Translations$settings$shortcuts$group$in_app$es._(_root);
}

// Path: settings.services.button
class _Translations$settings$services$button$es
    extends Translations$settings$services$button$en {
  _Translations$settings$services$button$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get add_service => 'Agregar Servicio...';
}

// Path: settings.services.section
class _Translations$settings$services$section$es
    extends Translations$settings$services$section$en {
  _Translations$settings$services$section$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get available_services => 'Servicios Disponibles';
}

// Path: settings.services.editor
class _Translations$settings$services$editor$es
    extends Translations$settings$services$editor$en {
  _Translations$settings$services$editor$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get title => 'Agregar servicio';
  @override
  String get subtitle => 'Agrega un servicio más a un proveedor configurado';
  @override
  late final _Translations$settings$services$editor$row$es row =
      _Translations$settings$services$editor$row$es._(_root);
  @override
  String get prompt_placeholder =>
      'Déjalo vacío para usar el prompt predeterminado de este tipo';
  @override
  String get variant_hint =>
      '{} ya tiene un servicio de {}; este se agrega junto al anterior como una segunda configuración.';
  @override
  String get traditional_note =>
      '{} es una interfaz tradicional: no hay modelo ni prompt que ajustar. Sus parámetros están en la página de detalle del proveedor.';
}

// Path: settings.services.detail
class _Translations$settings$services$detail$es
    extends Translations$settings$services$detail$en {
  _Translations$settings$services$detail$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  late final _Translations$settings$services$detail$row$es row =
      _Translations$settings$services$detail$row$es._(_root);
  @override
  late final _Translations$settings$services$detail$delete_dialog$es
  delete_dialog = _Translations$settings$services$detail$delete_dialog$es._(
    _root,
  );
  @override
  String get prompt_variables =>
      'Variables disponibles: {{sourceLanguage}}, {{targetLanguage}}, {{text}}';
}

// Path: settings.services.item
class _Translations$settings$services$item$es
    extends Translations$settings$services$item$en {
  _Translations$settings$services$item$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get none_of_kind => 'Aún no hay servicios de {} disponibles.';
}

// Path: settings.providers.section
class _Translations$settings$providers$section$es
    extends Translations$settings$providers$section$en {
  _Translations$settings$providers$section$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get services => 'Servicios disponibles';
  @override
  String get services_description =>
      'Consulta los servicios disponibles de los proveedores configurados y cambia entre tipos de servicio.';
}

// Path: settings.providers.item
class _Translations$settings$providers$item$es
    extends Translations$settings$providers$item$en {
  _Translations$settings$providers$item$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get empty =>
      'No hay proveedores configurados. Agrega uno para habilitar los servicios de traducción.';
  @override
  String get loading => 'Cargando proveedores...';
  @override
  String get no_services => 'No hay servicios disponibles.';
}

// Path: settings.providers.button
class _Translations$settings$providers$button$es
    extends Translations$settings$providers$button$en {
  _Translations$settings$providers$button$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get add => 'Agregar un proveedor...';
}

// Path: settings.providers.alert
class _Translations$settings$providers$alert$es
    extends Translations$settings$providers$alert$en {
  _Translations$settings$providers$alert$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get error => 'Error';
}

// Path: settings.providers.intro
class _Translations$settings$providers$intro$es
    extends Translations$settings$providers$intro$en {
  _Translations$settings$providers$intro$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get body => 'Gestiona los proveedores de servicios que usa la app.';
  @override
  String get warning =>
      'Los proveedores conectados pueden procesar el texto o las imágenes que envías. Activa solo servicios en los que confíes.';
}

// Path: settings.providers.editor
class _Translations$settings$providers$editor$es
    extends Translations$settings$providers$editor$en {
  _Translations$settings$providers$editor$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  late final _Translations$settings$providers$editor$row$es row =
      _Translations$settings$providers$editor$row$es._(_root);
  @override
  late final _Translations$settings$providers$editor$placeholder$es
  placeholder = _Translations$settings$providers$editor$placeholder$es._(_root);
  @override
  late final _Translations$settings$providers$editor$type_picker$es
  type_picker = _Translations$settings$providers$editor$type_picker$es._(_root);
  @override
  late final _Translations$settings$providers$editor$tooltip$es tooltip =
      _Translations$settings$providers$editor$tooltip$es._(_root);
  @override
  late final _Translations$settings$providers$editor$step$es step =
      _Translations$settings$providers$editor$step$es._(_root);
  @override
  String get add_title => 'Agregar {}';
  @override
  late final _Translations$settings$providers$editor$capability_note$es
  capability_note =
      _Translations$settings$providers$editor$capability_note$es._(_root);
  @override
  late final _Translations$settings$providers$editor$test$es test =
      _Translations$settings$providers$editor$test$es._(_root);
}

// Path: settings.providers.detail
class _Translations$settings$providers$detail$es
    extends Translations$settings$providers$detail$en {
  _Translations$settings$providers$detail$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  late final _Translations$settings$providers$detail$tooltip$es tooltip =
      _Translations$settings$providers$detail$tooltip$es._(_root);
  @override
  late final _Translations$settings$providers$detail$row$es row =
      _Translations$settings$providers$detail$row$es._(_root);
  @override
  late final _Translations$settings$providers$detail$section$es section =
      _Translations$settings$providers$detail$section$es._(_root);
  @override
  late final _Translations$settings$providers$detail$models$es models =
      _Translations$settings$providers$detail$models$es._(_root);
}

// Path: settings.providers.capability
class _Translations$settings$providers$capability$es
    extends Translations$settings$providers$capability$en {
  _Translations$settings$providers$capability$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get translation => 'Traducción';
  @override
  String get dictionary => 'Diccionario';
  @override
  String get ocr => 'OCR';
  @override
  String get llm => 'IA';
}

// Path: settings.providers.description
class _Translations$settings$providers$description$es
    extends Translations$settings$providers$description$en {
  _Translations$settings$providers$description$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get all => 'Proporciona búsqueda en diccionario y traducción de texto';
  @override
  String get dictionary =>
      'Proporciona búsqueda en diccionario y definiciones de palabras';
  @override
  String get translation => 'Proporciona traducción de texto entre idiomas';
  @override
  String get fallback => 'Proporciona servicios de traducción';
}

// Path: settings.providers.delete_dialog
class _Translations$settings$providers$delete_dialog$es
    extends Translations$settings$providers$delete_dialog$en {
  _Translations$settings$providers$delete_dialog$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get title => '¿Eliminar "{}"?';
  @override
  String get message => 'Esta acción no se puede deshacer.';
}

// Path: settings.layout.empty
class _Translations$settings$layout$empty$es
    extends Translations$settings$layout$empty$en {
  _Translations$settings$layout$empty$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get title => 'Selecciona una categoría';
  @override
  String get message =>
      'Elige una sección de configuración de la barra lateral.';
}

// Path: app.tray.context_menu.dev_tools
class _Translations$app$tray$context_menu$dev_tools$es
    extends Translations$app$tray$context_menu$dev_tools$en {
  _Translations$app$tray$context_menu$dev_tools$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get title => 'Herramientas de desarrollo';
  @override
  String get open_data_directory => 'Abrir directorio de datos';
}

// Path: settings.general.editor.row
class _Translations$settings$general$editor$row$es
    extends Translations$settings$general$editor$row$en {
  _Translations$settings$general$editor$row$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get source_language => 'Idioma de origen';
  @override
  String get target_language => 'Idioma de destino';
}

// Path: settings.shortcuts.group.global
class _Translations$settings$shortcuts$group$global$es
    extends Translations$settings$shortcuts$group$global$en {
  _Translations$settings$shortcuts$group$global$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get title => 'Atajos globales';
  @override
  String get description => 'Funcionan en cualquier aplicación.';
}

// Path: settings.shortcuts.group.in_app
class _Translations$settings$shortcuts$group$in_app$es
    extends Translations$settings$shortcuts$group$in_app$en {
  _Translations$settings$shortcuts$group$in_app$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get title => 'Teclas en la aplicación';
  @override
  String get description =>
      'Solo se aplican en los campos de texto de esta aplicación.';
}

// Path: settings.services.editor.row
class _Translations$settings$services$editor$row$es
    extends Translations$settings$services$editor$row$en {
  _Translations$settings$services$editor$row$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get model => 'Modelo';
  @override
  String get system_prompt => 'Prompt del sistema';
}

// Path: settings.services.detail.row
class _Translations$settings$services$detail$row$es
    extends Translations$settings$services$detail$row$en {
  _Translations$settings$services$detail$row$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get id => 'ID del servicio';
  @override
  String get name => 'Nombre';
  @override
  String get provider => 'Proveedor';
  @override
  String get type => 'Tipo';
}

// Path: settings.services.detail.delete_dialog
class _Translations$settings$services$detail$delete_dialog$es
    extends Translations$settings$services$detail$delete_dialog$en {
  _Translations$settings$services$detail$delete_dialog$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get title => '¿Eliminar "{}"?';
  @override
  String get message => 'Este servicio se eliminará del proveedor.';
}

// Path: settings.providers.editor.row
class _Translations$settings$providers$editor$row$es
    extends Translations$settings$providers$editor$row$en {
  _Translations$settings$providers$editor$row$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get id => 'ID del proveedor';
  @override
  String get type => 'Tipo de proveedor';
  @override
  String get default_model => 'Modelo predeterminado';
}

// Path: settings.providers.editor.placeholder
class _Translations$settings$providers$editor$placeholder$es
    extends Translations$settings$providers$editor$placeholder$en {
  _Translations$settings$providers$editor$placeholder$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get id => 'ej. deepl-main';
}

// Path: settings.providers.editor.type_picker
class _Translations$settings$providers$editor$type_picker$es
    extends Translations$settings$providers$editor$type_picker$en {
  _Translations$settings$providers$editor$type_picker$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get prompt => 'Selecciona el tipo de proveedor que deseas agregar:';
  @override
  String get section_llm => 'LLM';
  @override
  String get section_traditional => 'Tradicional';
}

// Path: settings.providers.editor.tooltip
class _Translations$settings$providers$editor$tooltip$es
    extends Translations$settings$providers$editor$tooltip$en {
  _Translations$settings$providers$editor$tooltip$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get help => 'Ayuda';
}

// Path: settings.providers.editor.step
class _Translations$settings$providers$editor$step$es
    extends Translations$settings$providers$editor$step$en {
  _Translations$settings$providers$editor$step$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get next => 'Continuar';
  @override
  String get back => 'Atrás';
}

// Path: settings.providers.editor.capability_note
class _Translations$settings$providers$editor$capability_note$es
    extends Translations$settings$providers$editor$capability_note$en {
  _Translations$settings$providers$editor$capability_note$es._(
    TranslationsEs root,
  ) : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get translation => 'Se suma a las traducciones candidatas';
  @override
  String get dictionary => 'Aporta definiciones de diccionario';
  @override
  String get ocr => 'Reconoce texto en imágenes';
}

// Path: settings.providers.editor.test
class _Translations$settings$providers$editor$test$es
    extends Translations$settings$providers$editor$test$en {
  _Translations$settings$providers$editor$test$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get run => 'Probar conexión';
  @override
  String get running => 'Probando la conexión · {}s transcurridos';
  @override
  String get passed_models => 'Conexión correcta · {} modelos disponibles';
  @override
  String get passed_service => 'Conexión correcta · servicio disponible';
  @override
  String get passed_footer => 'Prueba de conexión superada';
  @override
  String get retest => 'Probar de nuevo';
  @override
  String get tips_title => 'Puedes intentar';
  @override
  String get tips_llm =>
      '· Comprueba que la clave corresponda al tipo de proveedor elegido\n· Comprueba si la Base URL necesita /v1\n· Confirma que el modelo esté habilitado en la consola del proveedor';
  @override
  String get tips_traditional =>
      '· Comprueba que las credenciales correspondan al tipo de proveedor elegido\n· Confirma que el servicio esté habilitado en la consola del proveedor';
  @override
  String get failed_suffix => 'verificación fallida';
  @override
  String get passed_suffix => 'verificado';
}

// Path: settings.providers.detail.tooltip
class _Translations$settings$providers$detail$tooltip$es
    extends Translations$settings$providers$detail$tooltip$en {
  _Translations$settings$providers$detail$tooltip$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get edit => 'Editar proveedor';
}

// Path: settings.providers.detail.row
class _Translations$settings$providers$detail$row$es
    extends Translations$settings$providers$detail$row$en {
  _Translations$settings$providers$detail$row$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get id_hint => 'No se puede cambiar después de crearlo';
}

// Path: settings.providers.detail.section
class _Translations$settings$providers$detail$section$es
    extends Translations$settings$providers$detail$section$en {
  _Translations$settings$providers$detail$section$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get configuration => 'Configuración';
  @override
  String get models => 'Modelos';
}

// Path: settings.providers.detail.models
class _Translations$settings$providers$detail$models$es
    extends Translations$settings$providers$detail$models$en {
  _Translations$settings$providers$detail$models$es._(TranslationsEs root)
    : this._root = root,
      super.internal(root);

  final TranslationsEs _root; // ignore: unused_field

  // Translations
  @override
  String get loading => 'Cargando modelos...';
  @override
  String get empty => 'No se encontraron modelos.';
  @override
  String get retry => 'Reintentar';
  @override
  String get refresh => 'Actualizar lista';
  @override
  String get default_badge => 'Predeterminado';
  @override
  String get set_default => 'Establecer como predeterminado';
  @override
  String get fetch_error =>
      'No se pudieron obtener los modelos de la API del proveedor.';
}

/// The flat map containing all translations for locale <es>.
/// Only for edge cases! For simple maps, use the map function of this library.
///
/// The Dart AOT compiler has issues with very large switch statements,
/// so the map is split into smaller functions (512 entries each).
extension on TranslationsEs {
  dynamic _flatMapFunction(String path) {
    return switch (path) {
      'common.ui.button.ok' => 'OK',
      'common.ui.button.cancel' => 'Cancelar',
      'common.ui.button.add' => 'Agregar',
      'common.ui.button.delete' => 'Eliminar',
      'common.ui.button.edit' => 'Editar',
      'common.ui.button.save' => 'Guardar',
      'common.ui.button.manage' => 'Administrar',
      'common.ui.button.kContinue' => 'Continuar',
      'common.ui.feedback.copied' => 'Copiado',
      'common.language.ar' => 'Árabe',
      'common.language.bn' => 'Bengalí',
      'common.language.de' => 'Alemán',
      'common.language.en' => 'Inglés',
      'common.language.es' => 'Español',
      'common.language.fa' => 'Persa',
      'common.language.fr' => 'Francés',
      'common.language.gu' => 'Guyaratí',
      'common.language.ha' => 'Hausa',
      'common.language.hi' => 'Hindi',
      'common.language.id' => 'Indonesio',
      'common.language.it' => 'Italiano',
      'common.language.ja' => 'Japonés',
      'common.language.jv' => 'Javanés',
      'common.language.ko' => 'Coreano',
      'common.language.ml' => 'Malayalam',
      'common.language.mr' => 'Maratí',
      'common.language.ms' => 'Malayo',
      'common.language.nl' => 'Neerlandés',
      'common.language.pa' => 'Panyabí',
      'common.language.pl' => 'Polaco',
      'common.language.pt' => 'Portugués',
      'common.language.ro' => 'Rumano',
      'common.language.ru' => 'Ruso',
      'common.language.sw' => 'Suajili',
      'common.language.ta' => 'Tamil',
      'common.language.te' => 'Telugú',
      'common.language.th' => 'Tailandés',
      'common.language.tr' => 'Turco',
      'common.language.uk' => 'Ucraniano',
      'common.language.ur' => 'Urdu',
      'common.language.vi' => 'Vietnamita',
      'common.language.yo' => 'Yoruba',
      'common.language.zh_hans' => 'Chino simplificado',
      'common.language.zh_hant' => 'Chino tradicional',
      'common.theme_mode.light' => 'Claro',
      'common.theme_mode.dark' => 'Oscuro',
      'common.theme_mode.system' => 'Sistema',
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
      'common.provider.system' => 'Sistema',
      'common.provider.tencent' => 'Tencent',
      'common.provider.youdao' => 'Youdao',
      'app.tray.context_menu.show_window' => 'Mostrar ventana',
      'app.tray.context_menu.dev_tools.title' => 'Herramientas de desarrollo',
      'app.tray.context_menu.dev_tools.open_data_directory' =>
        'Abrir directorio de datos',
      'app.tray.context_menu.check_for_updates' => 'Buscar actualizaciones',
      'app.tray.context_menu.settings' => 'Configuración',
      'app.tray.context_menu.quit' => 'Salir',
      'mini_translator.limited_banner.permission.missing_both' => 'Concede los permisos de Grabación de pantalla y Accesibilidad para habilitar todas las funciones.',
      'mini_translator.limited_banner.permission.missing_screen_capture' => 'Concede el permiso de Grabación de pantalla para habilitar todas las funciones.',
      'mini_translator.limited_banner.permission.missing_accessibility' => 'Concede el permiso de Accesibilidad para habilitar todas las funciones.',
      'mini_translator.limited_banner.instruction.app_settings_prefix' =>
        'Ve a ',
      'mini_translator.limited_banner.instruction.follow_guide_prefix' =>
        ', sigue la guía y luego haz clic en ',
      'mini_translator.limited_banner.instruction.suffix' => '.',
      'mini_translator.limited_banner.action.app_settings' =>
        'Configuración de la app',
      'mini_translator.limited_banner.action.recheck' => 'Verificar de nuevo',
      'mini_translator.limited_banner.feedback.enabled' =>
        'La extracción de texto de pantalla está habilitada.',
      'mini_translator.limited_banner.feedback.still_missing' => 'Los permisos necesarios aún faltan.\nRevisa tu configuración e inténtalo de nuevo.',
      'mini_translator.limited_banner.tooltip.help' => 'Ver ayuda',
      'mini_translator.input.hint' => 'Escribe la palabra o texto aquí',
      'mini_translator.input.extracting_text' => 'Extrayendo texto...',
      'mini_translator.input.hint_translate_to' => ({
        required Object language,
      }) => 'Escribe una palabra o texto para traducir al ${language}',
      'mini_translator.toolbar.tooltip.extract_text_from_screen_capture' =>
        'Capturar área de la pantalla y reconocer texto',
      'mini_translator.toolbar.tooltip.extract_text_from_clipboard' =>
        'Leer contenido del portapapeles',
      'mini_translator.toolbar.tooltip.pin' => 'Fijar ventana',
      'mini_translator.toolbar.tooltip.more_actions' => 'Más acciones',
      'mini_translator.toolbar.menu.extract_from_screen_capture' =>
        'Capturar de pantalla',
      'mini_translator.toolbar.menu.extract_from_clipboard' =>
        'Desde el portapapeles',
      'mini_translator.toolbar.menu.open_main_window' =>
        'Abrir ventana principal',
      'mini_translator.toolbar.menu.open_settings' => 'Ajustes…',
      'mini_translator.button.clear' => 'Limpiar',
      'mini_translator.button.translate' => 'Traducir',
      'mini_translator.button.copy' => 'Copiar',
      'mini_translator.button.copied' => 'Copiado',
      'mini_translator.button.bookmark' => 'Guardar',
      'mini_translator.button.bookmarked' => 'Guardado',
      'mini_translator.language.auto_detect' => 'Detección automática',
      'mini_translator.language.auto_match' => 'Coincidencia auto',
      'mini_translator.language.switch_config' => 'Cambiar destino',
      'mini_translator.language.more_languages' => 'Más idiomas...',
      'mini_translator.language.manage_common_languages' =>
        'Administrar idiomas comunes...',
      'mini_translator.language.manage_targets' =>
        'Administrar destinos de traducción...',
      'mini_translator.language.add_target' =>
        'Agregar destino de traducción...',
      'mini_translator.message.please_enter_word_or_text' =>
        'No se ingresó texto o no se pudo extraer',
      'mini_translator.message.capture_screen_area_canceled' =>
        'La captura de área de pantalla ha sido cancelada',
      'mini_translator.message.ocr_service_not_configured' => 'No hay un servicio de reconocimiento de texto predeterminado configurado. Configúrelo en Ajustes.',
      'mini_translator.message.ocr_recognition_failed' =>
        'El reconocimiento de texto falló',
      'mini_translator.result.translating' => 'Traduciendo…',
      'mini_translator.result.stale_requery' => ({
        required Object key,
      }) => 'Texto modificado · ${key} retraducir',
      'mini_translator.result.compare_services' => ({
        required Object count,
      }) => 'Comparar ${count} servicios',
      'mini_translator.result.collapse_compare' => 'Contraer',
      'mini_translator.result.set_preferred' => 'Establecer como preferido',
      'mini_translator.result.retry' => 'Reintentar',
      'mini_translator.result.no_result' => 'Ningún servicio devolvió resultados: revisa la red o prueba otro servicio.',
      'mini_translator.result.no_result_note' =>
        'El texto se conserva; reintentar no duplicará el historial.',
      'workbench.workspace' => 'Espacio de trabajo',
      'workbench.translate' => 'Traducir',
      'workbench.history' => 'Historial',
      'workbench.history_page.all' => 'Todo',
      'workbench.history_page.favorites' => 'Favoritos',
      'workbench.history_page.edited' => 'Editados por mí',
      'workbench.history_page.search' => 'Buscar',
      'workbench.history_page.search_placeholder' =>
        'Buscar original, traducción o servicio',
      'workbench.history_page.search_label' => 'Buscar en el historial',
      'workbench.history_page.entry_count' => ({
        required Object label,
        required Object count,
      }) => '${label} · ${count} entradas',
      'workbench.history_page.by_time' => 'Por fecha',
      'workbench.history_page.loading' => 'Cargando historial…',
      'workbench.history_page.load_failed' => 'No se pudo cargar el historial',
      'workbench.history_page.retry' => 'Reintentar',
      'workbench.history_page.empty_title' =>
        'Aún no hay historial de traducción',
      'workbench.history_page.empty_description' =>
        'La traducción preferida se guarda aquí al completarse.',
      'workbench.history_page.no_results' => ({
        required Object query,
      }) => 'No hay resultados para «${query}»',
      'workbench.history_page.clear_search' => 'Borrar búsqueda',
      'workbench.history_page.select' => 'Selección múltiple',
      'workbench.history_page.selected_count' => ({
        required Object count,
      }) => '${count} seleccionadas',
      'workbench.history_page.exit_select' => 'Salir de selección',
      'workbench.history_page.add_to_glossary' => 'Añadir al glosario',
      'workbench.history_page.favorite' => 'Favorito',
      'workbench.history_page.unfavorite' => 'Quitar favorito',
      'workbench.history_page.delete_confirm' => ({
        required Object count,
      }) => '¿Eliminar las ${count} entradas seleccionadas? No se puede deshacer.',
      'workbench.history_page.no_glossary' => 'Crea primero un glosario',
      'workbench.history_page.added_to_glossary' => ({
        required Object count,
      }) => '${count} entradas añadidas al glosario',
      'workbench.history_page.favorite_flag' => 'Favorito',
      'workbench.history_page.edited_flag' => 'Editado',
      'workbench.history_page.edit_history_hint' =>
        'La traducción editada se guardará en el historial',
      'workbench.glossary' => 'Glosario',
      'workbench.recent_languages' => 'Idiomas recientes',
      'workbench.not_configured' => 'Sin configurar',
      'workbench.subtitle.translate' =>
        'Mesa de trabajo · Comparación de servicios',
      'workbench.subtitle.settings' => 'Ajustes',
      'workbench.placeholder.history' =>
        'Revisa, edita, guarda y reutiliza traducciones anteriores',
      'workbench.placeholder.glossary' =>
        'Gestiona términos obligatorios y alternativas prohibidas',
      'workbench.glossary_page.add_entry' => 'Nueva entrada',
      'workbench.glossary_page.term' => 'Origen',
      'workbench.glossary_page.translation' => 'Traducción obligatoria',
      'workbench.glossary_page.forbidden' => 'Prohibido',
      'workbench.glossary_page.hits' => 'Usos',
      'workbench.glossary_page.term_placeholder' => 'teacher forcing',
      'workbench.glossary_page.translation_placeholder' =>
        'forzado del maestro',
      'workbench.glossary_page.forbidden_placeholder' => 'enseñanza forzada',
      'workbench.glossary_page.search' => 'Buscar',
      'workbench.glossary_page.search_placeholder' =>
        'Buscar términos o traducciones obligatorias',
      'workbench.glossary_page.search_label' => 'Buscar en el glosario',
      'workbench.glossary_page.entry_count' => ({
        required Object name,
        required Object count,
      }) => '${name} · ${count} términos',
      'workbench.glossary_page.priority_note' =>
        'El glosario tiene prioridad sobre cualquier servicio',
      'workbench.glossary_page.new_book' => 'Nuevo glosario',
      'workbench.glossary_page.new_book_placeholder' => 'Nombre del glosario',
      'workbench.glossary_page.rename_book' => 'Cambiar nombre',
      'workbench.glossary_page.delete_book_confirm' => ({
        required Object name,
        required Object count,
      }) => '¿Eliminar «${name}» y sus ${count} términos?',
      'workbench.glossary_page.disabled' => 'Desactivado',
      'workbench.glossary_page.enable' => 'Activar',
      'workbench.glossary_page.disable' => 'Desactivar',
      'workbench.glossary_page.empty_title' => 'Este glosario está vacío',
      'workbench.glossary_page.empty_description' => 'El glosario tiene prioridad sobre cualquier servicio. Añade términos uno a uno, o suelta un CSV para combinarlos.',
      'workbench.glossary_page.no_results_title' => ({
        required Object query,
      }) => 'Ningún término coincide con «${query}»',
      'workbench.glossary_page.no_results_description' =>
        'Prueba otra palabra clave, o añade el término.',
      'workbench.glossary_page.no_books_title' => 'Todavía no hay glosarios',
      'workbench.glossary_page.no_books_description' => 'Un glosario mantiene tus traducciones coherentes en todos los servicios. Crea uno y empieza a añadir términos.',
      'workbench.glossary_page.loading' => 'Cargando…',
      'workbench.translation.source' => 'Origen',
      'workbench.translation.target' => 'Traducción',
      'workbench.translation.input_hint' =>
        'Escribe o pega el texto que quieras traducir',
      'workbench.translation.button' => 'Traducir',
      'workbench.translation.auto_detected' => 'Detectado automáticamente',
      'workbench.translation.loading_services' =>
        'Cargando servicios de traducción…',
      'workbench.translation.no_services' =>
        'Configura primero un servicio de traducción',
      'workbench.translation.translating' => 'Traduciendo…',
      'workbench.translation.failed' =>
        'La traducción falló. Revisa la configuración del servicio.',
      'workbench.translation.empty' => 'La traducción aparecerá aquí',
      'workbench.translation.service_compare' => 'Comparación de servicios',
      'workbench.translation.main_translation' => 'Principal',
      'workbench.translation.service_unavailable' => 'Servicio no disponible',
      'workbench.translation.waiting' => 'Esperando traducción',
      'workbench.translation.copy' => 'Copiar',
      'workbench.translation.favorite_unavailable' =>
        'No se pudo actualizar el favorito. Inténtalo de nuevo.',
      'workbench.translation.preferred' => 'Traducción preferida',
      'workbench.translation.other_services' => 'Otros servicios',
      'workbench.translation.copy_result' => 'Copiar traducción',
      'workbench.translation.copied' => 'Copiado',
      'workbench.translation.favorite' => 'Guardar',
      'workbench.translation.terms' => 'Coincidencias del glosario',
      'workbench.translation.terms_hint' =>
        'Los términos se comparan al escribir.',
      'workbench.translation.quality' => 'Señales de calidad',
      'workbench.translation.quality_hint' =>
        'Se calculan cuando llega la traducción.',
      'workbench.translation.shortcuts' => 'Atajos',
      'workbench.translation.other_services_disabled' =>
        'Los demás servicios están desactivados',
      'workbench.translation.input_hint_translate_to' => ({
        required Object language,
      }) => 'Escribe o pega el texto para traducir al ${language}',
      'workbench.translation.clear' => 'Borrar',
      'workbench.translation.swap_languages' => 'Intercambiar idiomas',
      'workbench.translation.services' => 'Servicios de traducción',
      'workbench.translation.configure_services' => 'Configurar servicios',
      'workbench.translation.retry' => 'Reintentar',
      'workbench.translation.character_count' => ({
        required Object count,
      }) => '${count} caracteres',
      'workbench.status.runtime_ready' => 'Motor de traducción listo',
      'workbench.status.settings_synced' => 'Ajustes sincronizados',
      'workbench.status.shortcuts' => '⌥Space Ventana rápida · ⌥⇧2 Captura',
      'workbench.version_latest' => 'Actualizado',
      'workbench.version_checking' => 'Comprobando…',
      'workbench.check_updates' => 'Buscar actualizaciones',
      'settings.version' => 'v{} (Build {})',
      'settings.general.title' => 'General',
      'settings.general.section.permissions' => 'Permisos del sistema',
      'settings.general.section.ocr' => 'Reconocimiento de texto',
      'settings.general.section.directory' => 'Directorio',
      'settings.general.section.translation' => 'Traducción',
      'settings.general.section.translation_target' => 'Idioma de destino',
      'settings.general.section.languages' => 'Idiomas',
      'settings.general.section.input' => 'Configuración de entrada',
      'settings.general.section.startup' => 'Inicio e integración',
      'settings.general.section.ocr_behaviour' => 'Comportamiento de captura',
      'settings.general.section.translation_behaviour' =>
        'Comportamiento de traducción',
      'settings.general.row.launch_at_login' => 'Iniciar al iniciar sesión',
      'settings.general.row.show_in_menu_bar' => 'Mostrar en la barra de menú',
      'settings.general.row.screen_capture_access' =>
        'Conceder acceso de grabación de pantalla',
      'settings.general.row.screen_selection_access' =>
        'Conceder acceso de accesibilidad',
      'settings.general.row.default_ocr_service' =>
        'Servicio de reconocimiento de texto predeterminado',
      'settings.general.row.auto_copy_detected_text' =>
        'Copiar texto detectado automáticamente',
      'settings.general.row.default_directory_service' =>
        'Servicio de diccionario predeterminado',
      'settings.general.row.default_translation_service' =>
        'Servicio de traducción predeterminado',
      'settings.general.row.translation_target_hint' =>
        'Configura los pares de idiomas usados por el traductor.',
      'settings.general.row.common_languages' => 'Idiomas comunes',
      'settings.general.row.common_languages_description' => 'Los idiomas comunes aparecen al principio de los selectores de idioma.',
      'settings.general.row.common_languages_hint' =>
        'Selecciona tus idiomas comunes:',
      'settings.general.row.common_languages_sort' => 'Ordenar por código',
      'settings.general.row.common_languages_reset' =>
        'Restablecer valores predeterminados',
      'settings.general.row.common_languages_reset_help' =>
        'Restablecer el conjunto predeterminado de idiomas comunes',
      'settings.general.row.common_languages_search' => 'Buscar idiomas...',
      'settings.general.row.common_languages_all' => 'Todos los idiomas',
      'settings.general.row.double_click_copy_result' =>
        'Doble clic para copiar el resultado de la traducción',
      'settings.general.row.submit_with_enter' => 'Enviar con Enter',
      'settings.general.row.submit_with_meta_enter_mac' =>
        'Enviar con ⌘ + Enter',
      'settings.general.row.screen_capture_access_hint' =>
        'Capturar texto de la pantalla requiere leer su contenido.',
      'settings.general.row.screen_selection_access_hint' => 'Capturar el texto seleccionado requiere leer selecciones de otras aplicaciones.',
      'settings.general.row.no_translation_targets' => 'Aún no hay destinos de traducción; añade uno para fijar el idioma predeterminado.',
      'settings.general.button.add_provider' => 'Agregar...',
      'settings.general.button.add_target' => 'Agregar destino...',
      'settings.general.button.manage_targets' =>
        'Gestionar destinos de traducción...',
      'settings.general.button.manage_languages' =>
        'Administrar idiomas comunes...',
      'settings.general.button.grant' => 'Conceder',
      'settings.general.option.none' => 'Ninguno',
      'settings.general.option.no_services_available' =>
        'No hay servicios disponibles',
      'settings.general.option.granted' => 'Concedido',
      'settings.general.option.built_in_ocr' => 'OCR integrado',
      'settings.general.option.tesseract' => 'Tesseract',
      'settings.general.option.youdao_ocr' => 'Youdao OCR',
      'settings.general.editor.add_target_title' =>
        'Agregar destino de traducción',
      'settings.general.editor.edit_target_title' =>
        'Editar destino de traducción',
      'settings.general.editor.row.source_language' => 'Idioma de origen',
      'settings.general.editor.row.target_language' => 'Idioma de destino',
      'settings.general.editor.title_edit' => 'Editar destino de traducción',
      'settings.general.editor.subtitle' => 'Decide a qué idioma se traduce un idioma de origen de forma predeterminada',
      'settings.general.editor.same_language' =>
        'El idioma de origen y el de destino son el mismo; elige otro destino.',
      'settings.general.editor.duplicate' =>
        'Ya existe un destino con esta combinación.',
      'settings.general.editor.hint_auto' =>
        'Traducir a {} cuando ninguna otra regla coincida.',
      'settings.general.editor.hint_source' =>
        'Traducir a {} cuando se detecte {}.',
      'settings.appearance.title' => 'Apariencia',
      'settings.appearance.section.app_language' => 'Idioma de la interfaz',
      'settings.appearance.section.theme_mode' => 'Modo de tema',
      'settings.appearance.section.theme_style' => 'Estilo del tema',
      'settings.appearance.footer' =>
        'Los cambios se aplican de inmediato a toda la ventana.',
      'settings.shortcuts.title' => 'Atajos',
      'settings.shortcuts.section.text_extraction' => 'Extracción de texto',
      'settings.shortcuts.section.input_assist' =>
        'Función de asistencia de entrada',
      'settings.shortcuts.section.submit_mode' => 'Enviar con',
      'settings.shortcuts.row.toggle_mini_translator' =>
        'Mostrar/Ocultar ventana',
      'settings.shortcuts.row.extract_text_from_screen_selection' =>
        'Extraer texto de la selección de pantalla',
      'settings.shortcuts.row.extract_text_from_screen_capture' =>
        'Extraer texto de la captura de pantalla',
      'settings.shortcuts.row.extract_text_from_clipboard' =>
        'Extraer texto del portapapeles',
      'settings.shortcuts.row.translate_input' =>
        'Traducir contenido de entrada',
      'settings.shortcuts.reset_dialog.title' => 'Restablecer atajos',
      'settings.shortcuts.reset_dialog.message' => '¿Estás seguro de que deseas restablecer todos los atajos a sus valores predeterminados?',
      'settings.shortcuts.reset_dialog.confirm' => 'Restablecer',
      'settings.shortcuts.reset_dialog.cancel' => 'Cancelar',
      'settings.shortcuts.group.global.title' => 'Atajos globales',
      'settings.shortcuts.group.global.description' =>
        'Funcionan en cualquier aplicación.',
      'settings.shortcuts.group.in_app.title' => 'Teclas en la aplicación',
      'settings.shortcuts.group.in_app.description' =>
        'Solo se aplican en los campos de texto de esta aplicación.',
      'settings.shortcuts.reset' => 'Restaurar valores predeterminados...',
      'settings.advanced.title' => 'Avanzado',
      'settings.advanced.api_server' => 'Servidor de API local',
      'settings.advanced.api_server_description' =>
        'Exponer la API de traducción en 127.0.0.1 para integraciones locales.',
      'settings.advanced.enable' => 'Habilitar',
      'settings.advanced.port' => 'Puerto',
      'settings.advanced.running_at' => 'Ejecutándose en {url}',
      'settings.advanced.disabled' => 'Deshabilitado',
      'settings.services.title' => 'Servicios',
      'settings.services.button.add_service' => 'Agregar Servicio...',
      'settings.services.section.available_services' => 'Servicios Disponibles',
      'settings.services.editor.title' => 'Agregar servicio',
      'settings.services.editor.subtitle' =>
        'Agrega un servicio más a un proveedor configurado',
      'settings.services.editor.row.model' => 'Modelo',
      'settings.services.editor.row.system_prompt' => 'Prompt del sistema',
      'settings.services.editor.prompt_placeholder' =>
        'Déjalo vacío para usar el prompt predeterminado de este tipo',
      'settings.services.editor.variant_hint' => '{} ya tiene un servicio de {}; este se agrega junto al anterior como una segunda configuración.',
      'settings.services.editor.traditional_note' => '{} es una interfaz tradicional: no hay modelo ni prompt que ajustar. Sus parámetros están en la página de detalle del proveedor.',
      'settings.services.detail.row.id' => 'ID del servicio',
      'settings.services.detail.row.name' => 'Nombre',
      'settings.services.detail.row.provider' => 'Proveedor',
      'settings.services.detail.row.type' => 'Tipo',
      'settings.services.detail.delete_dialog.title' => '¿Eliminar "{}"?',
      'settings.services.detail.delete_dialog.message' =>
        'Este servicio se eliminará del proveedor.',
      'settings.services.detail.prompt_variables' => 'Variables disponibles: {{sourceLanguage}}, {{targetLanguage}}, {{text}}',
      'settings.services.make_default' => 'Establecer como predeterminado',
      'settings.services.item.none_of_kind' =>
        'Aún no hay servicios de {} disponibles.',
      'settings.providers.title' => 'Proveedores',
      'settings.providers.section.services' => 'Servicios disponibles',
      'settings.providers.section.services_description' => 'Consulta los servicios disponibles de los proveedores configurados y cambia entre tipos de servicio.',
      'settings.providers.item.empty' => 'No hay proveedores configurados. Agrega uno para habilitar los servicios de traducción.',
      'settings.providers.item.loading' => 'Cargando proveedores...',
      'settings.providers.item.no_services' => 'No hay servicios disponibles.',
      'settings.providers.button.add' => 'Agregar un proveedor...',
      'settings.providers.alert.error' => 'Error',
      'settings.providers.intro.body' =>
        'Gestiona los proveedores de servicios que usa la app.',
      'settings.providers.intro.warning' => 'Los proveedores conectados pueden procesar el texto o las imágenes que envías. Activa solo servicios en los que confíes.',
      'settings.providers.editor.row.id' => 'ID del proveedor',
      'settings.providers.editor.row.type' => 'Tipo de proveedor',
      'settings.providers.editor.row.default_model' => 'Modelo predeterminado',
      'settings.providers.editor.placeholder.id' => 'ej. deepl-main',
      'settings.providers.editor.type_picker.prompt' =>
        'Selecciona el tipo de proveedor que deseas agregar:',
      'settings.providers.editor.type_picker.section_llm' => 'LLM',
      'settings.providers.editor.type_picker.section_traditional' =>
        'Tradicional',
      'settings.providers.editor.tooltip.help' => 'Ayuda',
      'settings.providers.editor.step.next' => 'Continuar',
      'settings.providers.editor.step.back' => 'Atrás',
      'settings.providers.editor.add_title' => 'Agregar {}',
      'settings.providers.editor.capability_note.translation' =>
        'Se suma a las traducciones candidatas',
      'settings.providers.editor.capability_note.dictionary' =>
        'Aporta definiciones de diccionario',
      'settings.providers.editor.capability_note.ocr' =>
        'Reconoce texto en imágenes',
      'settings.providers.editor.test.run' => 'Probar conexión',
      'settings.providers.editor.test.running' =>
        'Probando la conexión · {}s transcurridos',
      'settings.providers.editor.test.passed_models' =>
        'Conexión correcta · {} modelos disponibles',
      'settings.providers.editor.test.passed_service' =>
        'Conexión correcta · servicio disponible',
      'settings.providers.editor.test.passed_footer' =>
        'Prueba de conexión superada',
      'settings.providers.editor.test.retest' => 'Probar de nuevo',
      'settings.providers.editor.test.tips_title' => 'Puedes intentar',
      'settings.providers.editor.test.tips_llm' => '· Comprueba que la clave corresponda al tipo de proveedor elegido\n· Comprueba si la Base URL necesita /v1\n· Confirma que el modelo esté habilitado en la consola del proveedor',
      'settings.providers.editor.test.tips_traditional' => '· Comprueba que las credenciales correspondan al tipo de proveedor elegido\n· Confirma que el servicio esté habilitado en la consola del proveedor',
      'settings.providers.editor.test.failed_suffix' => 'verificación fallida',
      'settings.providers.editor.test.passed_suffix' => 'verificado',
      'settings.providers.detail.tooltip.edit' => 'Editar proveedor',
      'settings.providers.detail.row.id_hint' =>
        'No se puede cambiar después de crearlo',
      'settings.providers.detail.section.configuration' => 'Configuración',
      'settings.providers.detail.section.models' => 'Modelos',
      'settings.providers.detail.models.loading' => 'Cargando modelos...',
      'settings.providers.detail.models.empty' => 'No se encontraron modelos.',
      'settings.providers.detail.models.retry' => 'Reintentar',
      'settings.providers.detail.models.refresh' => 'Actualizar lista',
      'settings.providers.detail.models.default_badge' => 'Predeterminado',
      'settings.providers.detail.models.set_default' =>
        'Establecer como predeterminado',
      'settings.providers.detail.models.fetch_error' =>
        'No se pudieron obtener los modelos de la API del proveedor.',
      'settings.providers.capability.translation' => 'Traducción',
      'settings.providers.capability.dictionary' => 'Diccionario',
      'settings.providers.capability.ocr' => 'OCR',
      'settings.providers.capability.llm' => 'IA',
      'settings.providers.description.all' =>
        'Proporciona búsqueda en diccionario y traducción de texto',
      'settings.providers.description.dictionary' =>
        'Proporciona búsqueda en diccionario y definiciones de palabras',
      'settings.providers.description.translation' =>
        'Proporciona traducción de texto entre idiomas',
      'settings.providers.description.fallback' =>
        'Proporciona servicios de traducción',
      'settings.providers.delete_dialog.title' => '¿Eliminar "{}"?',
      'settings.providers.delete_dialog.message' =>
        'Esta acción no se puede deshacer.',
      'settings.layout.title' => 'Configuración',
      'settings.layout.empty.title' => 'Selecciona una categoría',
      'settings.layout.empty.message' =>
        'Elige una sección de configuración de la barra lateral.',
      'settings.layout.groups' => 'Grupos',
      'settings.layout.effect_hint' => 'Los cambios se aplican al instante',
      'settings.layout.footer_note' =>
        'Las traducciones y claves se guardan solo en este equipo',
      'settings.layout.support' => 'Soporte',
      'settings.about.title' => 'Acerca de',
      'settings.about.copy_version_info' => 'Copiar información de versión',
      'settings.about.up_to_date' => 'Está actualizado.',
      'settings.about.check_again' => 'Verificar de nuevo',
      'settings.about.links' => 'Enlaces',
      'settings.about.website' => 'Sitio web',
      'settings.about.github' => 'GitHub',
      'settings.about.report_issue' => 'Informar de un problema',
      'settings.about.license' => 'Licencia',
      'settings.about.open_changelog' => 'Abrir registro de cambios',
      'settings.about.update' => 'Actualización',
      _ => null,
    };
  }
}
