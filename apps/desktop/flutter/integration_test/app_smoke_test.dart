import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:linguaray_desktop/main.dart' as app;
import 'package:linguaray_desktop/src/platform/permission_controller.dart';
import 'package:linguaray_desktop/src/platform/platform_types.dart';
import 'package:linguaray_desktop/src/platform/trigger_controller.dart';
import 'package:linguaray_desktop/src/routes/mini_translator/mini_translator.dart';
import 'package:linguaray_desktop/src/services/app_windows.dart';
import 'package:linguaray_desktop/src/services/shortcut_service/shortcut_service.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('launches a styled LinguaRay workbench surface', (tester) async {
    await app.main();
    await tester.pumpAndSettle();

    expect(find.textContaining('LinguaRay'), findsWidgets);
    expect(find.byType(ErrorWidget), findsNothing);

    expect(ShortcutService.instance.bindings, hasLength(4));
    expect(
      ShortcutService.instance.bindings.every(
        (binding) => binding.state == ShortcutRegistrationState.registered,
      ),
      isTrue,
    );

    final permissions = await permissionController.refresh();
    expect(permissions.accessibility, isNot(PermissionState.unknown));
    expect(permissions.screenRecording, isNot(PermissionState.unknown));

    final showQuickWindow =
        triggerController.trigger(TriggerAction.toggleQuickWindow);
    await tester.pump();
    await showQuickWindow;
    await tester.pumpAndSettle();
    expect(appSurface.value, AppSurface.miniTranslator);
    expect(find.byType(MiniTranslatorPage), findsOneWidget);
    expect(find.byType(ErrorWidget), findsNothing);

    showWorkbenchWindow();
    await tester.pumpAndSettle();
    expect(appSurface.value, AppSurface.workbench);
    expect(find.byType(ErrorWidget), findsNothing);
  });
}
