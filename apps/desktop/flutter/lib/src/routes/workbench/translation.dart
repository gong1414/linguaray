import 'package:flutter/widgets.dart';

import '../../ui/translation/translation_screen.dart';

/// Route-level compatibility name for the workbench shell.
///
/// The feature implementation lives under `ui/translation`; routing should
/// know how to mount it, not how translation is performed.
class WorkbenchTranslationPage extends StatelessWidget {
  const WorkbenchTranslationPage({super.key});

  @override
  Widget build(BuildContext context) => const TranslationScreen();
}
