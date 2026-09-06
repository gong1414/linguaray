from pathlib import Path
from tempfile import TemporaryDirectory
import unittest

from check_dart_architecture import directives, inspect


class DartArchitectureTest(unittest.TestCase):
    def setUp(self):
        temporary = TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.write("apps/desktop/flutter/pubspec.yaml", "name: desktop\n")
        self.write("packages/application/pubspec.yaml", "name: linguaray_application\n")
        self.write("packages/ui_flutter/pubspec.yaml", "name: linguaray_ui\n")

    def write(self, name, source):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(source)

    def test_application_rejects_io_network_flutter_and_runtime_even_through_exports(self):
        self.write("packages/application/lib/port.dart", "export 'leak.dart';")
        self.write("packages/application/lib/leak.dart", "export 'dart:io';\nexport 'package:flutter/widgets.dart';\nexport 'package:linguaray_runtime/linguaray_runtime.dart';\nexport 'package:http/http.dart';")
        found = inspect(self.root)
        self.assertEqual(len(found), 8)
        self.assertEqual({v.rule for v in found}, {"application-purity"})

    def test_widgets_cannot_hide_plugins_behind_conditional_or_barrel_imports(self):
        self.write("apps/desktop/flutter/lib/src/features/test/screen.dart", "import 'bridge.dart';\nclass Screen extends StatelessWidget {}")
        self.write("apps/desktop/flutter/lib/src/features/test/bridge.dart", "export 'stub.dart' if (dart.library.io) 'native.dart';")
        self.write("apps/desktop/flutter/lib/src/features/test/stub.dart", "")
        self.write("apps/desktop/flutter/lib/src/features/test/native.dart", "export 'package:nativeapi/nativeapi.dart';")
        found = inspect(self.root)
        self.assertEqual(len(found), 1)
        self.assertEqual(found[0].target, "package:nativeapi/nativeapi.dart")

    def test_view_models_cannot_import_data_adapters(self):
        self.write("apps/desktop/flutter/lib/src/features/test/test_view_model.dart", "import 'data/repository.dart';")
        self.write("apps/desktop/flutter/lib/src/features/test/data/repository.dart", "")
        self.assertEqual(inspect(self.root)[0].rule, "presentation-port")

    def test_composition_and_controllers_can_use_native_capabilities(self):
        self.write("apps/desktop/flutter/lib/src/app/app_host.dart", "import 'package:nativeapi/nativeapi.dart';\nclass Root extends StatelessWidget {}")
        self.write("apps/desktop/flutter/lib/src/features/test/window_coordinator.dart", "import 'package:nativeapi/nativeapi.dart';")
        self.write("apps/desktop/flutter/lib/src/features/test/screen.dart", "import 'window_coordinator.dart';\nclass Screen extends StatelessWidget {}")
        self.assertEqual(inspect(self.root), [])

    def test_design_system_and_desktop_directory_rules(self):
        self.write("packages/ui_flutter/lib/button.dart", "import 'package:linguaray_application/port.dart';")
        self.write("apps/desktop/flutter/lib/src/services/old.dart", "")
        self.assertEqual({v.rule for v in inspect(self.root)}, {"design-system-purity", "desktop-directory"})

    def test_comments_are_not_dependencies_and_conditional_branches_are_included(self):
        source = '''/* outer /* inner */
import 'fake.dart';
*/
// import 'fake2.dart';
import 'stub.dart' if (dart.library.io) 'native.dart';
export 'https://example.invalid/file.dart';
part of 'owner.dart';
'''
        self.assertEqual(directives(source), [("import", "stub.dart"), ("import", "native.dart"), ("export", "https://example.invalid/file.dart")])


if __name__ == "__main__":
    unittest.main()
