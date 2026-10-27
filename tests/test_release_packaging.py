"""Check portable skill paths and release archive safety on both archive formats."""

import importlib.util
import os
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest.mock import patch
import zipfile

SPEC = importlib.util.spec_from_file_location(
    "release_packaging", Path(__file__).resolve().parents[1] / "scripts" / "package-release.py"
)
PACKAGING = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PACKAGING)


class ReleasePackagingTests(unittest.TestCase):
    def prepare(self, root, target):
        binary = root / "target" / target / "release" / ("diffyn.exe" if "windows" in target else "diffyn")
        binary.parent.mkdir(parents=True)
        binary.write_bytes(b"test executable")
        (root / "LICENSE").write_text("MIT test license\n")
        (root / "docs").mkdir()
        (root / "docs" / "skills.md").write_text(
            "# Skills\n\n[Workflow](../skills/diffyn-task-support/SKILL.md)\n"
        )
        for name in PACKAGING.SKILL_NAMES:
            skill = root / "skills" / name / "SKILL.md"
            skill.parent.mkdir(parents=True)
            skill.write_text(f"---\nname: {name}\n---\nWorkflow instructions.\n")
        destination = root / "dist"
        destination.mkdir()
        return destination

    def test_archives_contain_identical_skills_and_resolvable_guide_links(self):
        for target in ("aarch64-apple-darwin", "x86_64-pc-windows-msvc"):
            with self.subTest(target=target), tempfile.TemporaryDirectory() as directory:
                root = Path(directory).resolve()
                destination = self.prepare(root, target)
                with patch.object(PACKAGING, "ROOT", root), patch.dict(os.environ, {"RELEASE_TARGET": target}), patch.object(PACKAGING, "third_party_notices", return_value=b"third party"), patch.object(PACKAGING, "rust_library_notices", return_value=(b"rust licenses", b"rust metadata")):
                    PACKAGING.package_binary("2.0.0", destination)
                archive = destination / PACKAGING.archive_name("2.0.0", target)
                if target.endswith("msvc"):
                    with zipfile.ZipFile(archive) as package:
                        contents = {name: package.read(name) for name in package.namelist()}
                else:
                    with tarfile.open(archive) as package:
                        contents = {member.name: package.extractfile(member).read() for member in package.getmembers()}
                for name in PACKAGING.SKILL_NAMES:
                    relative = f"skills/{name}/SKILL.md"
                    self.assertEqual(contents[relative], (root / relative).read_bytes())
                self.assertIn(b"(skills/diffyn-task-support/SKILL.md)", contents["SKILLS.md"])
                self.assertNotIn(b"(../skills/", contents["SKILLS.md"])
                self.assertIn("THIRD_PARTY_NOTICES.txt", contents)
                self.assertIn("RUST_LIBRARY_LICENSES.html", contents)

    def test_linked_skill_files_cannot_enter_release_archive(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            target = "aarch64-apple-darwin"
            destination = self.prepare(root, target)
            skill = root / "skills" / PACKAGING.SKILL_NAMES[0] / "SKILL.md"
            skill.unlink()
            skill.symlink_to(root / "LICENSE")
            with patch.object(PACKAGING, "ROOT", root), patch.dict(os.environ, {"RELEASE_TARGET": target}), patch.object(PACKAGING, "third_party_notices", return_value=b"third party"), patch.object(PACKAGING, "rust_library_notices", return_value=(b"rust licenses", b"rust metadata")):
                with self.assertRaisesRegex(ValueError, "missing or unsafe"):
                    PACKAGING.package_binary("2.0.0", destination)


if __name__ == "__main__":
    unittest.main()
