import 'package:flutter/foundation.dart';

enum GeneralSettingsIntent {
  manageCommonLanguages,
  manageTranslationTargets,
  addTranslationTarget,
}

/// Carries a one-shot settings action while the app switches from the quick
/// translator surface to the settings surface.
final class GeneralSettingsIntentController extends ChangeNotifier {
  GeneralSettingsIntent? _pending;

  bool get hasPending => _pending != null;

  void request(GeneralSettingsIntent intent) {
    _pending = intent;
    notifyListeners();
  }

  GeneralSettingsIntent? takePending() {
    final intent = _pending;
    _pending = null;
    return intent;
  }
}

final generalSettingsIntentController = GeneralSettingsIntentController();
