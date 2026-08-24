import 'package:flutter/foundation.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../data/github_update_repository.dart';
import '../services/network_proxy.dart';
import '../services/settings_store.dart';

class StartupUpdateController {
  bool _started = false;
  final ValueNotifier<UpdateState?> result = ValueNotifier(null);

  Future<UpdateState?> check() async {
    if (_started || !settingsStore.advanced.checkUpdatesOnLaunch) {
      return result.value;
    }
    _started = true;
    final repository = GitHubUpdateRepository(
      client: createNetworkHttpClient(),
    );
    try {
      final state = await CheckForUpdate(repository)();
      result.value = state;
      return state;
    } finally {
      repository.close();
    }
  }
}

final startupUpdateController = StartupUpdateController();
