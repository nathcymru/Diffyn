"""Keep container backfill verification tied to the published source version."""

import importlib.util
import io
import tarfile
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location(
    "container_protocol", Path(__file__).with_name("container_protocol.py")
)
PROTOCOL = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PROTOCOL)


class ContainerProtocolSourceTests(unittest.TestCase):
    def archive(self, extra=None):
        stream = io.BytesIO()
        with tarfile.open(fileobj=stream, mode="w:") as archive:
            for name, contents in {
                "Cargo.toml": b'[package]\nversion = "2.0.0"\n',
                "tests/stdio_protocol.py": b'print("historical protocol suite")\n',
                "skills/diffyn-task-support/SKILL.md": b"Historical skill\n",
            }.items():
                entry = tarfile.TarInfo(name)
                entry.size = len(contents)
                archive.addfile(entry, io.BytesIO(contents))
            if extra:
                archive.addfile(extra)
        return stream.getvalue()

    def test_snapshot_uses_committed_version_and_skill_fixtures(self):
        with tempfile.TemporaryDirectory() as directory:
            destination = Path(directory) / "source"
            revision = "a" * 40
            with (
                patch.object(
                    PROTOCOL.subprocess, "check_output", side_effect=["b" * 40, self.archive()]
                ),
                patch.object(PROTOCOL.subprocess, "run") as ancestry,
            ):
                self.assertEqual(PROTOCOL.protocol_source(destination, revision), destination)
            self.assertIn(b'"2.0.0"', (destination / "Cargo.toml").read_bytes())
            self.assertEqual(
                (destination / "skills/diffyn-task-support/SKILL.md").read_bytes(),
                b"Historical skill\n",
            )
            self.assertIn(revision, ancestry.call_args.args[0])

    def test_default_uses_working_tree_but_ambiguous_revision_cannot_reach_git(self):
        with patch.object(PROTOCOL.subprocess, "check_output") as command:
            self.assertEqual(PROTOCOL.protocol_source(Path("unused"), ""), PROTOCOL.ROOT)
            with self.assertRaises(ValueError):
                PROTOCOL.protocol_source(Path("unused"), "main:untrusted")
        command.assert_not_called()

    def test_archive_paths_and_links_cannot_escape_the_snapshot(self):
        for name, kind in (
            ("../escape", tarfile.REGTYPE),
            ("skills/link", tarfile.SYMTYPE),
            ("scripts/unrelated.py", tarfile.REGTYPE),
        ):
            entry = tarfile.TarInfo(name)
            entry.type = kind
            entry.linkname = "/etc/passwd" if kind == tarfile.SYMTYPE else ""
            with (
                self.subTest(name=name),
                tempfile.TemporaryDirectory() as directory,
                patch.object(
                    PROTOCOL.subprocess, "check_output", side_effect=["a" * 40, self.archive(entry)]
                ),
                self.assertRaises(ValueError),
            ):
                PROTOCOL.protocol_source(Path(directory) / "source", "a" * 40)


if __name__ == "__main__":
    unittest.main()
