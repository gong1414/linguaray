from pathlib import Path
from tempfile import TemporaryDirectory
import unittest

from check_dart_reachability import inspect


class DartReachabilityTest(unittest.TestCase):
    def setUp(self):
        self.temporary = TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        self.app = self.root / "apps/desktop/flutter"
        self.write("apps/desktop/flutter/pubspec.yaml", "name: desktop\n")
        self.write("apps/desktop/flutter/lib/main.dart", "void main() {}\n")

    def write(self, path, source):
        file = self.root / path
        file.parent.mkdir(parents=True, exist_ok=True)
        file.write_text(source)
        return file

    def test_tests_do_not_keep_orphan_product_libraries_alive(self):
        orphan = self.write("apps/desktop/flutter/lib/old_widget.dart", "class OldWidget {}")
        self.write("apps/desktop/flutter/test/old_test.dart", "import 'package:desktop/old_widget.dart';")
        self.assertEqual(inspect(self.root), ([orphan], []))

    def test_package_exports_relative_parts_and_conditional_imports(self):
        self.write("packages/ui/pubspec.yaml", "name: ui\n")
        self.write("packages/ui/lib/ui.dart", "export 'theme.dart';")
        self.write("packages/ui/lib/theme.dart", "part 'colors.dart';")
        self.write("packages/ui/lib/colors.dart", "part of 'theme.dart';")
        self.write("apps/desktop/flutter/lib/main.dart", "import 'package:ui/ui.dart';\nimport 'stub.dart' if (dart.library.io) 'native.dart';")
        self.write("apps/desktop/flutter/lib/stub.dart", "")
        self.write("apps/desktop/flutter/lib/native.dart", "")
        self.assertEqual(inspect(self.root), ([], []))

    def test_catalog_is_only_reachable_from_development_entry(self):
        preview = self.write("apps/desktop/flutter/lib/src/ui/catalog/preview.dart", "")
        self.write("apps/desktop/flutter/lib/widgetbook.dart", "import 'src/ui/catalog/preview.dart';")
        self.assertEqual(inspect(self.root), ([], []))
        self.write("apps/desktop/flutter/lib/main.dart", "import 'src/ui/catalog/preview.dart';")
        self.assertEqual(inspect(self.root), ([], [preview]))


if __name__ == "__main__":
    unittest.main()
