import 'dart:async';

/// Silent checks continue while the app lives in the menu bar. The preference
/// is read for every attempt, including after resume and settings changes.
class AutomaticUpdateSchedule {
  AutomaticUpdateSchedule({
    required this._enabled,
    required this._runCheck,
    DateTime Function()? now,
    this.interval = const Duration(hours: 6),
  }) : _now = now ?? DateTime.now;

  final bool Function() _enabled;
  final Future<void> Function() _runCheck;
  final DateTime Function() _now;
  final Duration interval;
  DateTime? _lastCheck;
  Timer? _timer;
  Future<void>? _pending;
  bool _disposed = false;

  void start() {
    if (_disposed) return;
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
  }

  Future<void> check() {
    if (_pending != null) return _pending!;
    if (_disposed ||
        !_enabled() ||
        (_lastCheck != null && _now().difference(_lastCheck!) < interval)) {
      return Future.value();
    }
    _lastCheck = _now();
    return _pending = Future<void>.sync(_runCheck)
        .whenComplete(() => _pending = null);
  }
}
