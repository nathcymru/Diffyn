"""Coverage accounting must not inflate production results with test execution."""

import importlib.util
import tempfile
import unittest
from pathlib import Path

SPEC = importlib.util.spec_from_file_location(
    "rust_coverage", Path(__file__).resolve().parents[1] / "scripts/check-rust-coverage.py"
)
coverage = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(coverage)


class RustCoverageTests(unittest.TestCase):
    def test_instances_union_without_test_or_gap_inflation(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            functions = [
                {
                    "name": "_RNv6diffyn7compute",
                    "filenames": [str(root / "main.rs")],
                    "regions": [[1, 1, 2, 1, 0, 0, 0, 0], [3, 1, 4, 1, 0, 0, 0, 0]],
                },
                {
                    "name": "_RINv6diffyn7compute",
                    "filenames": [str(root / "main.rs")],
                    "regions": [[1, 1, 2, 1, 4, 0, 0, 0], [5, 1, 9, 1, 8, 0, 0, 3]],
                },
                {
                    "name": "_RNvNt6diffyn5tests4test",
                    "filenames": [str(root / "main.rs")],
                    "regions": [[10, 1, 99, 1, 1, 0, 0, 0]],
                },
                {
                    "name": "helper",
                    "filenames": [str(root / "tools/tests.rs")],
                    "regions": [[1, 1, 99, 1, 1, 0, 0, 0]],
                },
                {
                    "name": "dependency",
                    "filenames": [str(root.parent / "dependency.rs")],
                    "regions": [[1, 1, 99, 1, 1, 0, 0, 0]],
                },
            ]
            result = coverage.summarize({"data": [{"functions": functions}]}, root)
            self.assertEqual(result["regions"], 2)
            self.assertEqual(result["covered"], 1)
            self.assertEqual(result["percent"], 50)
            self.assertEqual(result["files"], {"main.rs": {"regions": 2, "covered": 1}})

    def test_empty_report_cannot_be_a_passing_measurement(self):
        with self.assertRaisesRegex(ValueError, "No production"):
            coverage.summarize({"data": [{"functions": []}]}, Path("src"))


if __name__ == "__main__":
    unittest.main()
