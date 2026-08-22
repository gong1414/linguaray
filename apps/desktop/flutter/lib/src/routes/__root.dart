import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../platform/onboarding_controller.dart';

part '__root.g.dart';

@TypedGoRoute<RootRoute>(path: '/')
class RootRoute extends GoRouteData with $RootRoute {
  const RootRoute();

  @override
  String? redirect(BuildContext context, GoRouterState state) {
    return onboardingController.isComplete ? '/translate' : '/welcome';
  }
}
