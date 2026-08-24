import 'package:flutter/material.dart';

/// Pushes a modal route into the active Flutter navigator instead of asking
/// the desktop multi-window host to create another native window.
Future<T?> showDialogInCurrentWindow<T>({
  required BuildContext context,
  required WidgetBuilder builder,
  bool barrierDismissible = true,
  Color? barrierColor,
  String? barrierLabel,
  bool useSafeArea = true,
  RouteSettings? routeSettings,
  Offset? anchorPoint,
  TraversalEdgeBehavior? traversalEdgeBehavior,
  bool fullscreenDialog = false,
  bool? requestFocus,
  AnimationStyle? animationStyle,
}) {
  final navigator = Navigator.of(context, rootNavigator: true);
  final inherited = InheritedTheme.capture(
    from: context,
    to: navigator.context,
  );

  final route = DialogRoute<T>(
    context: context,
    builder: builder,
    themes: inherited,
    barrierColor: barrierColor ?? Colors.black54,
    barrierDismissible: barrierDismissible,
    barrierLabel: barrierLabel,
    useSafeArea: useSafeArea,
    settings: routeSettings,
    anchorPoint: anchorPoint,
    traversalEdgeBehavior:
        traversalEdgeBehavior ?? TraversalEdgeBehavior.closedLoop,
    requestFocus: requestFocus,
    animationStyle: animationStyle,
    fullscreenDialog: fullscreenDialog,
  );
  return navigator.push<T>(route);
}
