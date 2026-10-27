"""Release integrity checks use synthetic dependency graphs and archive files."""

import hashlib
import importlib.util
import json
import os
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location(
    "release_integrity", Path(__file__).resolve().parents[1] / "scripts/package-release.py"
)
release = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(release)


class ReleaseIntegrityTests(unittest.TestCase):
    def test_version_must_match_release_tag(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "Cargo.toml").write_text('[package]\nversion = "2.0.0"\n')
            with (
                patch.object(release, "ROOT", root),
                patch.dict(
                    os.environ,
                    {
                        "RELEASE_REF_TYPE": "tag",
                        "RELEASE_REF_NAME": "v2.0.0",
                    },
                ),
            ):
                self.assertEqual(release.release_version(), "2.0.0")
                with patch.dict(os.environ, {"RELEASE_REF_NAME": "v2.1.0"}):
                    with self.assertRaisesRegex(ValueError, "match"):
                        release.release_version()
                (root / "Cargo.toml").write_text('[package]\nversion = "2.0.0-rc1"\n')
                with self.assertRaisesRegex(ValueError, "numeric"):
                    release.release_version()

    def test_license_text_and_notices_are_retained_and_unsafe_paths_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            package = {"name": "example", "manifest_path": str(root / "Cargo.toml")}
            (root / "LICENSE").write_text("MIT example\n")
            (root / "NOTICE").write_text("Example attribution\n")
            self.assertEqual(
                dict(release.license_documents(package)),
                {
                    "LICENSE": "MIT example\n",
                    "NOTICE": "Example attribution\n",
                },
            )
            (root / "LICENSE").write_text("")
            with self.assertRaisesRegex(ValueError, "Empty"):
                release.license_documents(package)
            (root / "LICENSE").unlink()
            with self.assertRaisesRegex(ValueError, "No packaged license"):
                release.license_documents(package)
            (root / "LICENSE").symlink_to(root / "NOTICE")
            with self.assertRaisesRegex(ValueError, "Linked"):
                release.license_documents(package)
            (root / "LICENSE").unlink()
            with self.assertRaisesRegex(ValueError, "Unsafe"):
                release.license_documents({**package, "license_file": "../outside"})
            (root / "legal").mkdir()
            (root / "legal/terms.txt").write_text("Declared license\n")
            self.assertIn(
                ("legal/terms.txt", "Declared license\n"),
                release.license_documents({**package, "license_file": "legal/terms.txt"}),
            )

    def test_dependency_notices_follow_only_complete_locked_reachable_graph(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "LICENSE").write_text("MIT example\n")
            dependency = {
                "id": "dep",
                "name": "example",
                "version": "1.0.0",
                "license": "MIT",
                "source": "registry+https://github.com/rust-lang/crates.io-index",
                "repository": "https://example.invalid/source",
                "manifest_path": str(root / "Cargo.toml"),
            }
            graph = {
                "resolve": {
                    "root": "root",
                    "nodes": [
                        {"id": "root", "dependencies": ["dep"]},
                        {"id": "dep", "dependencies": []},
                    ],
                },
                "packages": [{"id": "root"}, dependency, {"id": "unused"}],
            }
            with patch.object(
                release.subprocess, "run", return_value=SimpleNamespace(stdout=json.dumps(graph))
            ) as run:
                notices = release.third_party_notices("aarch64-apple-darwin")
                self.assertIn(b"MIT example", notices)
                self.assertIn(b"example 1.0.0", notices)
                self.assertNotIn(b"unused", notices)
                self.assertIn("--locked", run.call_args.args[0])
                self.assertIn("--offline", run.call_args.args[0])
            graph["packages"] = [{"id": "root"}]
            with patch.object(
                release.subprocess, "run", return_value=SimpleNamespace(stdout=json.dumps(graph))
            ):
                with self.assertRaisesRegex(ValueError, "incomplete"):
                    release.third_party_notices("aarch64-apple-darwin")
            dependency["source"] = "git+https://example.invalid/unreviewed"
            graph["packages"] = [{"id": "root"}, dependency]
            with patch.object(
                release.subprocess, "run", return_value=SimpleNamespace(stdout=json.dumps(graph))
            ):
                with self.assertRaisesRegex(ValueError, "Unsupported"):
                    release.third_party_notices("aarch64-apple-darwin")

    def test_rust_notices_include_toolchain_source_and_reject_missing_text(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            license_file = root / "share/doc/rust/COPYRIGHT-library.html"
            license_file.parent.mkdir(parents=True)
            license_file.write_bytes(b"Original Rust library licenses")

            def rustc(command, **_kwargs):
                output = (
                    str(root) if "--print" in command else "rustc 1.99.0\ncommit-hash: " + "a" * 40
                )
                return SimpleNamespace(stdout=output)

            with patch.object(release.subprocess, "run", side_effect=rustc):
                document, metadata = release.rust_library_notices()
                self.assertEqual(document, license_file.read_bytes())
                self.assertIn(b"https://github.com/rust-lang/rust/tree/" + b"a" * 40, metadata)
                license_file.write_bytes(b"")
                with self.assertRaisesRegex(ValueError, "empty"):
                    release.rust_library_notices()
                license_file.unlink()
                with self.assertRaisesRegex(ValueError, "missing"):
                    release.rust_library_notices()

    def test_release_manifest_authenticates_complete_set_and_rejects_upload_mismatch(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            names = [release.archive_name("2.0.0", target) for target in release.TARGETS]
            names.append("diffyn-2.0.0.crate")
            for name in names:
                (root / name).write_bytes(name.encode())
            release.write_checksums("2.0.0", root)
            for line in (root / "SHA256SUMS").read_text().splitlines():
                digest, name = line.split("  ")
                self.assertEqual(digest, hashlib.sha256((root / name).read_bytes()).hexdigest())
            metadata = {
                "tagName": "v2.0.0",
                "isDraft": True,
                "assets": [
                    {"name": path.name, "size": path.stat().st_size} for path in root.iterdir()
                ],
            }
            release.verify_release("2.0.0", root, metadata)
            metadata["assets"][0]["size"] += 1
            with self.assertRaisesRegex(ValueError, "size"):
                release.verify_release("2.0.0", root, metadata)
            metadata["isDraft"] = False
            with self.assertRaisesRegex(ValueError, "draft"):
                release.verify_release("2.0.0", root, metadata)
            with self.assertRaisesRegex(ValueError, "complete expected"):
                release.write_checksums("2.0.0", root)


if __name__ == "__main__":
    unittest.main()
