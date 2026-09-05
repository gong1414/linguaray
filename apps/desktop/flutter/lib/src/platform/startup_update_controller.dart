import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../data/github_update_repository.dart';
import '../services/network_proxy.dart';
import '../services/settings_store.dart';

/// Silent checks continue while the app lives in the menu bar. The preference
/// is read for every attempt, including after resume and settings changes.
class StartupUpdateController {
  StartupUpdateController({
    bool Function()? enabled,
    Future<UpdateState> Function()? runCheck,
    DateTime Function()? now,
    this.interval = const Duration(hours: 6),
  }) : _enabled =
           enabled ?? (() => settingsStore.advanced.checkUpdatesOnLaunch),
       _runCheck = runCheck ?? _checkRepository,
       _now = now ?? DateTime.now;

  final bool Function() _enabled;
  final Future<UpdateState> Function() _runCheck;
  final DateTime Function() _now;
  final Duration interval;
  final ValueNotifier<UpdateState?> result = ValueNotifier(null);
  DateTime? _lastCheck;
  Timer? _timer;
  Future<UpdateState?>? _pending;
  bool _disposed = false;

  void start() {
    _timer ??= Timer.periodic(interval, (_) => unawaited(check()));
    unawaited(check());
  }

  void stop() {
    _timer?.cancel();
    _timer = null;
  }

  void dispose() {
    stop();
    _disposed = true;
    result.dispose();
  }

  Future<UpdateState?> check() {
    if (_pending != null) return _pending!;
    if (_disposed ||
        !_enabled() ||
        (_lastCheck != null && _now().difference(_lastCheck!) < interval)) {
      return Future.value(result.value);
    }
    _lastCheck = _now();
    return _pending = _performCheck().whenComplete(() => _pending = null);
  }

  Future<UpdateState?> _performCheck() async {
    final state = await _runCheck();
    if (!_disposed) result.value = state;
    return state;
  }

  static Future<UpdateState> _checkRepository() async {
    final repository = GitHubUpdateRepository(
      client: createNetworkHttpClient(),
    );
    try {
      return await CheckForUpdate(repository)();
    } finally {
      repository.close();
    }
  }
}

final startupUpdateController = StartupUpdateController();
