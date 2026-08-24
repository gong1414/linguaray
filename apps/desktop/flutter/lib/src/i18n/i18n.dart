import 'package:flutter/widgets.dart';
import 'package:flutter_localizations/flutter_localizations.dart';

import 'strings.g.dart';
export 'strings.g.dart';

String formatTranslation(String value, {List<String> args = const []}) {
  return args.fold(value, (text, argument) => text.replaceFirst('{}', argument));
}

/// Invalidates descendants after a Slang locale change without replacing
/// their state objects. This also reaches router overlay entries whose widgets
/// read the global [t] accessor.
class LocaleRebuildScope extends StatefulWidget {
  const LocaleRebuildScope({super.key, required this.child});

  final Widget child;

  @override
  State<LocaleRebuildScope> createState() => _LocaleRebuildScopeState();
}

class _LocaleRebuildScopeState extends State<LocaleRebuildScope> {
  AppLocale? _lastLocale;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final current = TranslationProvider.of(context).locale;
    final changed = _lastLocale != null && _lastLocale != current;
    _lastLocale = current;
    if (changed) (context as Element).visitChildren(_invalidate);
  }

  static void _invalidate(Element element) {
    element.markNeedsBuild();
    element.visitChildren(_invalidate);
  }

  @override
  Widget build(BuildContext context) => widget.child;
}

extension SlangBuildContextExtension on BuildContext {
  Iterable<LocalizationsDelegate<dynamic>> get localizationDelegates =>
      GlobalMaterialLocalizations.delegates;

  List<Locale> get supportedLocales => AppLocaleUtils.supportedLocales;

  Locale get locale => LocaleSettings.currentLocale.flutterLocale;

  Future<void> setLocale(Locale locale) async {
    await LocaleSettings.setLocaleRaw(locale.toLanguageTag());
  }
}
