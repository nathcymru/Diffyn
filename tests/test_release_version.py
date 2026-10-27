"""Detect mismatched release metadata before a version is tagged."""

import importlib.util
import tempfile
import unittest
from pathlib import Path

SPEC = importlib.util.spec_from_file_location(
    "release_version", Path(__file__).resolve().parents[1] / "scripts/check-release-version.py"
)
VERSION = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(VERSION)


class ReleaseVersionTests(unittest.TestCase):
    def fixture(self, root):
        files = {
            "Cargo.toml": '[package]\nname="diffyn"\nversion="2.0.0"\n',
            "Cargo.lock": '[[package]]\nname="diffyn"\nversion="2.0.0"\n',
            "fuzz/Cargo.lock": '[[package]]\nname="diffyn"\nversion="2.0.0"\n',
            "fuzz/Cargo.toml": '[dependencies]\ndiffyn={path="..", version="=2.0.0"}\n',
            "CITATION.cff": "cff-version: 1.2.0\nversion: 2.0.0\n",
            "docs/releases/v2.0.0.md": "# Diffyn v2.0.0\n\nAdds communication workflows.\n",
            "README.md": "cargo install --tag v2.0.0 --locked diffyn\n",
            "docs/index.html": "<code>cargo install --tag v2.0.0 --locked diffyn</code>\n",
        }
        for name, content in files.items():
            path = root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding="utf-8")

    def test_current_tree_is_consistent(self):
        result = VERSION.validate(VERSION.ROOT)
        self.assertEqual(result["tag"], "v" + result["version"])

    def test_version_order_and_stable_channel(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.fixture(root)
            self.assertEqual(VERSION.validate(root, "1.9.9")["version"], "2.0.0")
            self.assertEqual(VERSION.validate(root, "2.0.0")["version"], "2.0.0")
            for previous in ("2.0.1", "not-a-version"):
                with self.subTest(previous=previous), self.assertRaises(ValueError):
                    VERSION.validate(root, previous)
            for invalid in ("2.1.0-rc.1", "02.0.0", "2.0", "../escape"):
                (root / "Cargo.toml").write_text(f'[package]\nversion="{invalid}"\n')
                with self.subTest(version=invalid), self.assertRaises(ValueError):
                    VERSION.validate(root)

    def test_stale_versions_and_missing_notes_are_rejected(self):
        for name in (
            "Cargo.lock",
            "fuzz/Cargo.lock",
            "fuzz/Cargo.toml",
            "CITATION.cff",
            "README.md",
            "docs/index.html",
            "docs/releases/v2.0.0.md",
        ):
            with self.subTest(file=name), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self.fixture(root)
                path = root / name
                path.write_text(path.read_text().replace("2.0.0", "0.1.0"))
                with self.assertRaises(ValueError):
                    VERSION.validate(root)
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.fixture(root)
            (root / "docs/releases/v2.0.0.md").unlink()
            with self.assertRaises(FileNotFoundError):
                VERSION.validate(root)


if __name__ == "__main__":
    unittest.main()
