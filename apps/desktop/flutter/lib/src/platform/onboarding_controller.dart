import 'package:flutter/foundation.dart';
import 'package:nativeapi/nativeapi.dart';

const _kOnboardingScope = 'io.github.gong1414.linguaray.v2.ui';
const _kCompletedKey = 'onboarding.completed';

class OnboardingController extends ChangeNotifier {
  OnboardingController({Preferences? preferences})
    : _preferences = preferences ?? Preferences.withScope(_kOnboardingScope) {
    _isComplete = _preferences.get(_kCompletedKey, 'false') == 'true';
  }

  final Preferences _preferences;
  late bool _isComplete;

  bool get isComplete => _isComplete;

  void complete() {
    if (!_preferences.set(_kCompletedKey, 'true')) {
      throw StateError('Could not save onboarding state.');
    }
    if (_isComplete) return;
    _isComplete = true;
    notifyListeners();
  }
}

late final OnboardingController onboardingController;

void initOnboardingController() {
  onboardingController = OnboardingController();
}
