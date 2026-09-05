import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../app/dependencies.dart';

final aboutViewModelProvider = NotifierProvider<AboutViewModel, AboutInfo?>(
  AboutViewModel.new,
);

final class AboutViewModel extends Notifier<AboutInfo?> {
  @override
  AboutInfo? build() {
    scheduleMicrotask(() async {
      state = await ref.read(appInfoRepositoryProvider).loadAbout();
    });
    return null;
  }
}
