"""Offline regressions for the pinned model catalog importer (Python 3.11+)."""
import tempfile
import unittest
from pathlib import Path

from update_provider_catalog import collect_provider, load_lab_models


class CatalogImportTests(unittest.TestCase):
    def test_namespaces_symlink_metadata_and_toml_strings_are_preserved(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            models = root / "providers" / "router" / "models"
            for vendor in ("one", "two"):
                path = models / vendor / "same.toml"
                path.parent.mkdir(parents=True)
                path.write_text('name = "Name # retained"\nrelease_date = 2026-01-01\n'
                                'modalities = { input = ["text"], output = ["text"] }\n')
            link = models / "alias.toml"
            link.write_text("../models/one/same.toml")
            (models / "broken.toml").write_text("../models/missing.toml")
            result = collect_provider(root, "router", load_lab_models(root))
            self.assertEqual({model["id"] for model in result},
                             {"one/same", "two/same", "alias"})
            self.assertTrue(all(model["name"] == "Name # retained" for model in result))
            self.assertTrue(all(model["release_date"] == "2026-01-01" for model in result))

    def test_non_text_models_do_not_enter_translation_reference_catalog(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            models = root / "providers" / "router" / "models"
            models.mkdir(parents=True)
            (models / "image.toml").write_text(
                'modalities = { input = ["text"], output = ["image"] }\n')
            self.assertEqual(collect_provider(root, "router", {}), [])


if __name__ == "__main__":
    unittest.main()
