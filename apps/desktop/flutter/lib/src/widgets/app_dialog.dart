import 'package:flutter/widgets.dart';

import 'ui.dart'
    show Dialog, DialogBody, DialogFooter, DialogHeader, DialogTone;

/// Product-level composition for confirmation and settings dialogs.
class AppDialog extends StatelessWidget {
  const AppDialog({
    super.key,
    required this.title,
    this.subtitle,
    this.width = 440,
    this.tone = DialogTone.standard,
    this.content,
    this.actions = const <Widget>[],
  });

  final Widget title;
  final Widget? subtitle;
  final double width;
  final DialogTone tone;
  final Widget? content;
  final List<Widget> actions;

  @override
  Widget build(BuildContext context) {
    final sections = <Widget>[
      DialogHeader(title: title, subtitle: subtitle),
      if (content != null) DialogBody(children: <Widget>[content!]),
      if (actions.isNotEmpty)
        DialogFooter(
          children: <Widget>[const Spacer(), ...actions],
        ),
    ];

    return Semantics(
      scopesRoute: true,
      explicitChildNodes: true,
      child: Center(
        child: ConstrainedBox(
          constraints: BoxConstraints(maxWidth: width),
          child: Dialog(width: width, tone: tone, children: sections),
        ),
      ),
    );
  }
}
