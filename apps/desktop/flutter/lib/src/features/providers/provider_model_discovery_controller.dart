import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:linguaray_application/linguaray_application.dart';

/// Debounces credential edits and prevents an older configuration's response
/// from replacing the current provider's model list. No credentials are cached.
class ProviderModelDiscoveryController extends ChangeNotifier {
  ProviderModelDiscoveryController(this._discover);

  final Future<ProviderModelDiscovery> Function(ProviderDraft) _discover;
  ProviderModelDiscovery? result;
  bool loading = false;
  Timer? _timer;
  int _generation = 0;
  bool _disposed = false;

  void schedule(ProviderDraft? draft, {bool immediately = false}) {
    _timer?.cancel();
    final generation = ++_generation;
    result = null;
    loading = draft != null;
    notifyListeners();
    if (draft == null) return;
    if (immediately) {
      unawaited(_fetch(draft, generation));
    } else {
      _timer = Timer(const Duration(milliseconds: 700), () {
        unawaited(_fetch(draft, generation));
      });
    }
  }

  Future<void> _fetch(ProviderDraft draft, int generation) async {
    ProviderModelDiscovery next;
    try {
      next = await _discover(draft);
    } catch (_) {
      next = const ProviderModelDiscovery(errorCode: 'network_error');
    }
    if (_disposed || generation != _generation) return;
    result = next;
    loading = false;
    notifyListeners();
  }

  @override
  void dispose() {
    _disposed = true;
    _generation++;
    _timer?.cancel();
    super.dispose();
  }
}
