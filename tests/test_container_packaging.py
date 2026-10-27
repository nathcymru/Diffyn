"""Exercise archive, publication and provenance boundaries without a registry."""

import hashlib
import importlib.util
import io
import json
import os
import subprocess
import sys
import tarfile
import tempfile
import unittest
import urllib.error
from pathlib import Path
from unittest.mock import MagicMock, patch

SPEC = importlib.util.spec_from_file_location(
    "container_packaging", Path(__file__).resolve().parents[1] / "scripts" / "package-container.py"
)
PACKAGING = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PACKAGING)
SOURCE = {
    "version": "2.0.0",
    "tag": "v2.0.0",
    "revision": "a" * 40,
    "packaging_revision": "f" * 40,
    "repository": "nathcymru/Diffyn",
    "archive": "diffyn-2.0.0-x86_64-unknown-linux-gnu.tar.gz",
}
IMAGE_ID = "sha256:" + "b" * 64
MANIFEST_DIGEST = "sha256:" + "c" * 64


class ContainerPackagingTests(unittest.TestCase):
    def archive(self, directory, entries=None):
        path = directory / SOURCE["archive"]
        with tarfile.open(path, "w:gz") as package:
            for name in sorted(PACKAGING.CONTENTS) if entries is None else entries:
                entry = tarfile.TarInfo(name)
                content = b"test package file\n"
                entry.size = len(content)
                package.addfile(entry, io.BytesIO(content))
        return path

    def release(self, directory):
        self.archive(directory)
        names = {
            SOURCE["archive"],
            "diffyn-2.0.0.crate",
            "diffyn-2.0.0-aarch64-apple-darwin.tar.gz",
            "diffyn-2.0.0-x86_64-apple-darwin.tar.gz",
            "diffyn-2.0.0-x86_64-pc-windows-msvc.zip",
        }
        for name in names - {SOURCE["archive"]}:
            (directory / name).write_bytes(b"another platform\n")
        manifest = "".join(
            f"{hashlib.sha256((directory / name).read_bytes()).hexdigest()}  {name}\n"
            for name in sorted(names)
        )
        (directory / "SHA256SUMS").write_text(manifest)
        return {
            "tagName": SOURCE["tag"],
            "isDraft": False,
            "isImmutable": True,
            "isPrerelease": False,
            "publishedAt": "2026-10-27T12:00:00Z",
            "assets": [
                {"name": name, "size": (directory / name).stat().st_size}
                for name in names | {"SHA256SUMS"}
            ],
        }

    def image(self):
        return {
            "Id": IMAGE_ID,
            "Os": "linux",
            "Architecture": "amd64",
            "Config": {
                "User": "65532:65532",
                "Entrypoint": ["/opt/diffyn/diffyn"],
                "Cmd": None,
                "Labels": {
                    "org.opencontainers.image.source": "https://github.com/nathcymru/Diffyn",
                    "org.opencontainers.image.version": SOURCE["version"],
                    "org.opencontainers.image.revision": SOURCE["revision"],
                    "org.opencontainers.image.licenses": "MIT",
                    "io.diffyn.packaging.revision": SOURCE["packaging_revision"],
                },
            },
        }

    def test_metadata_rejects_tag_commit_and_repository_substitution(self):
        for field, value in (
            ("tag", "v9.0.0"),
            ("revision", "HEAD"),
            ("repository", "other/repo"),
            ("version", "2.0.0\nunsafe=yes"),
            ("archive", "../archive.tar.gz"),
            ("version", None),
        ):
            with self.subTest(field=field, value=value), self.assertRaises(ValueError):
                PACKAGING.validate_metadata(SOURCE | {field: value})

    def test_metadata_and_cli_bind_the_archive_to_an_actual_main_tag(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(
                ["git", "init", "--quiet", "--initial-branch=main", str(root)], check=True
            )
            (root / "Cargo.toml").write_text('[package]\nversion = "2.0.0"\n')
            subprocess.run(["git", "add", "Cargo.toml"], cwd=root, check=True)
            subprocess.run(
                [
                    "git",
                    "-c",
                    "user.name=Release test",
                    "-c",
                    "user.email=release@example.invalid",
                    "commit",
                    "--quiet",
                    "-m",
                    "Release fixture",
                ],
                cwd=root,
                check=True,
            )
            subprocess.run(
                ["git", "update-ref", "refs/remotes/origin/main", "HEAD"], cwd=root, check=True
            )
            subprocess.run(["git", "tag", "v2.0.0"], cwd=root, check=True)
            with patch.object(PACKAGING, "ROOT", root):
                tagged = PACKAGING.metadata("v2.0.0")
                self.assertEqual(tagged, PACKAGING.metadata(""))
                self.assertEqual(tagged["revision"], tagged["packaging_revision"])
                with self.assertRaises(ValueError):
                    PACKAGING.metadata("v2.0.0; untrusted")
                destination = root / "metadata.json"
                outputs = root / "outputs.txt"
                with (
                    patch.object(
                        sys,
                        "argv",
                        [
                            "package-container.py",
                            "metadata",
                            "--source-tag",
                            "v2.0.0",
                            "--metadata",
                            str(destination),
                        ],
                    ),
                    patch.dict(os.environ, {"GITHUB_OUTPUT": str(outputs)}),
                ):
                    PACKAGING.main()
                self.assertEqual(json.loads(destination.read_text()), tagged)
                self.assertIn("tag=v2.0.0\n", outputs.read_text())

    def test_cli_checks_release_before_extracting_the_container_context(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "metadata.json"
            source.write_text(json.dumps(SOURCE))
            release = root / "release.json"
            release.write_text(json.dumps(self.release(root)))
            arguments = [
                "package-container.py",
                "verify-release",
                "--metadata",
                str(source),
                "--release",
                str(release),
                "--directory",
                str(root),
            ]
            with patch.object(sys, "argv", arguments):
                PACKAGING.main()
            with patch.object(
                sys,
                "argv",
                [
                    "package-container.py",
                    "extract",
                    "--metadata",
                    str(source),
                    "--directory",
                    str(root),
                    "--context",
                    str(root / "context"),
                ],
            ):
                PACKAGING.main()
            self.assertTrue((root / "context/diffyn").is_file())
            with (
                patch.object(
                    sys, "argv", ["package-container.py", "verify-image", "--metadata", str(source)]
                ),
                patch.object(PACKAGING, "image_config", return_value=self.image()),
            ):
                PACKAGING.main()

    def test_cli_emits_only_the_verified_publication_digest(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "metadata.json"
            source.write_text(json.dumps(SOURCE))
            outputs = root / "outputs.txt"
            with (
                patch.object(
                    sys, "argv", ["package-container.py", "publish", "--metadata", str(source)]
                ),
                patch.object(PACKAGING, "publish", return_value=MANIFEST_DIGEST),
                patch.dict(os.environ, {"GITHUB_OUTPUT": str(outputs)}),
            ):
                PACKAGING.main()
            self.assertEqual(outputs.read_text(), f"digest={MANIFEST_DIGEST}\n")
            with (
                patch.object(
                    sys,
                    "argv",
                    [
                        "package-container.py",
                        "promote-latest",
                        "--metadata",
                        str(source),
                        "--digest",
                        MANIFEST_DIGEST,
                    ],
                ),
                patch.object(PACKAGING, "promote_latest") as promote,
            ):
                PACKAGING.main()
            promote.assert_called_once_with(SOURCE, MANIFEST_DIGEST)

    def test_context_preserves_exact_files_and_removes_archive_permissions(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            PACKAGING.extract(self.archive(root), root / "context")
            actual = {
                str(path.relative_to(root / "context"))
                for path in (root / "context").rglob("*")
                if path.is_file()
            }
            self.assertEqual(actual, PACKAGING.CONTENTS)
            self.assertEqual((root / "context/diffyn").stat().st_mode & 0o7777, 0o755)
            self.assertEqual((root / "context/LICENSE").stat().st_mode & 0o7777, 0o644)
            self.assertTrue(
                all(path.stat().st_mtime == 0 for path in (root / "context").rglob("*"))
            )
            self.assertEqual((root / "context").stat().st_mtime, 0)

    def test_archive_rejects_traversal_duplicate_and_missing_entries(self):
        entries = sorted(PACKAGING.CONTENTS)
        for changed in (entries + ["../escape"], entries + [entries[0]], entries[1:]):
            with self.subTest(entries=changed), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                with self.assertRaises(ValueError):
                    PACKAGING.extract(self.archive(root, changed), root / "context")
                self.assertFalse((root / "context").exists())

    def test_archive_rejects_links_before_writing_files(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            archive = root / SOURCE["archive"]
            with tarfile.open(archive, "w:gz") as package:
                for name in PACKAGING.CONTENTS:
                    entry = tarfile.TarInfo(name)
                    entry.type = tarfile.SYMTYPE
                    entry.linkname = "/etc/passwd"
                    package.addfile(entry)
            with self.assertRaises(ValueError):
                PACKAGING.extract(archive, root / "context")
            self.assertFalse((root / "context").exists())

    def test_release_requires_published_immutable_matching_assets_and_checksums(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            release = self.release(root)
            PACKAGING.verify_release(SOURCE, release, root)
            for field, value in (
                ("isDraft", True),
                ("isImmutable", False),
                ("isPrerelease", True),
                ("tagName", "v1.0.0"),
                ("assets", []),
            ):
                with self.subTest(field=field), self.assertRaises(ValueError):
                    PACKAGING.verify_release(SOURCE, release | {field: value}, root)
            archive = root / SOURCE["archive"]
            content = archive.read_bytes()
            archive.write_bytes(bytes([content[0] ^ 1]) + content[1:])
            with self.assertRaisesRegex(ValueError, "checksum"):
                PACKAGING.verify_release(SOURCE, release, root)

    def test_image_rejects_root_ports_shell_and_wrong_source(self):
        for field, value in (
            ("User", "0"),
            ("ExposedPorts", {"8080/tcp": {}}),
            ("Cmd", ["bash"]),
            ("Entrypoint", ["bash"]),
            ("Volumes", {"/data": {}}),
        ):
            image = self.image()
            image["Config"][field] = value
            with (
                self.subTest(field=field),
                patch.object(PACKAGING, "image_config", return_value=image),
                self.assertRaises(ValueError),
            ):
                PACKAGING.verify_image("candidate", SOURCE)
        image = self.image()
        image["Config"]["Labels"]["org.opencontainers.image.revision"] = "d" * 40
        with (
            patch.object(PACKAGING, "image_config", return_value=image),
            self.assertRaises(ValueError),
        ):
            PACKAGING.verify_image("candidate", SOURCE)

    def test_image_config_digest_supports_containerd_and_classic_docker_stores(self):
        image = self.image()
        with patch.object(PACKAGING, "image_config", return_value=image):
            self.assertEqual(PACKAGING.verify_image("candidate", SOURCE), IMAGE_ID)
        image["Id"] = MANIFEST_DIGEST
        image["Descriptor"] = {"annotations": {"config.digest": IMAGE_ID}}
        with patch.object(PACKAGING, "image_config", return_value=image):
            self.assertEqual(PACKAGING.verify_image("candidate", SOURCE), IMAGE_ID)

    def test_publication_never_overwrites_a_different_version(self):
        existing = (MANIFEST_DIGEST, {"config": {"digest": "sha256:" + "e" * 64}})
        with (
            patch.object(PACKAGING, "verify_image", return_value=IMAGE_ID),
            patch.object(PACKAGING, "registry_token", return_value="token"),
            patch.object(PACKAGING, "registry_manifest", return_value=existing),
            patch.object(PACKAGING.subprocess, "run") as run,
        ):
            with self.assertRaisesRegex(ValueError, "different content"):
                PACKAGING.publish("candidate", SOURCE)
            run.assert_not_called()

    def test_same_version_content_is_reused_without_push(self):
        existing = (MANIFEST_DIGEST, {"config": {"digest": IMAGE_ID}})
        with (
            patch.object(PACKAGING, "verify_image", return_value=IMAGE_ID),
            patch.object(PACKAGING, "registry_token", return_value="token"),
            patch.object(PACKAGING, "registry_manifest", return_value=existing),
            patch.object(PACKAGING.subprocess, "run") as run,
        ):
            self.assertEqual(PACKAGING.publish("candidate", SOURCE), MANIFEST_DIGEST)
            run.assert_not_called()

    def test_new_version_is_checked_again_after_push(self):
        published = (MANIFEST_DIGEST, {"config": {"digest": IMAGE_ID}})
        with (
            patch.object(PACKAGING, "verify_image", return_value=IMAGE_ID),
            patch.object(PACKAGING, "registry_token", return_value="token"),
            patch.object(PACKAGING, "registry_manifest", side_effect=[None, published]),
            patch.object(PACKAGING.subprocess, "run") as run,
        ):
            self.assertEqual(PACKAGING.publish("candidate", SOURCE), MANIFEST_DIGEST)
            self.assertEqual(
                run.call_args_list[0].args[0],
                ["docker", "tag", "candidate", "ghcr.io/nathcymru/diffyn:2.0.0"],
            )
            self.assertEqual(
                run.call_args_list[1].args[0], ["docker", "push", "ghcr.io/nathcymru/diffyn:2.0.0"]
            )
        with (
            patch.object(PACKAGING, "verify_image", return_value=IMAGE_ID),
            patch.object(PACKAGING, "registry_token", return_value="token"),
            patch.object(PACKAGING, "registry_manifest", side_effect=[None, None]),
            patch.object(PACKAGING.subprocess, "run"),
            self.assertRaisesRegex(ValueError, "does not match"),
        ):
            PACKAGING.publish("candidate", SOURCE)

    def test_registry_replies_require_a_matching_content_digest(self):
        content = json.dumps({"config": {"digest": IMAGE_ID}}).encode()
        digest = "sha256:" + hashlib.sha256(content).hexdigest()
        response = MagicMock()
        response.__enter__.return_value = response
        response.read.return_value = content
        response.headers = {"Docker-Content-Digest": digest}
        with patch.object(PACKAGING.urllib.request, "urlopen", return_value=response):
            self.assertEqual(
                PACKAGING.registry_manifest("nathcymru/diffyn", "2.0.0", "token"),
                (digest, json.loads(content)),
            )
            response.headers["Docker-Content-Digest"] = MANIFEST_DIGEST
            with self.assertRaisesRegex(ValueError, "inconsistent"):
                PACKAGING.registry_manifest("nathcymru/diffyn", "2.0.0", "token")

    def test_registry_credentials_stay_in_the_authorization_header(self):
        response = MagicMock()
        response.__enter__.return_value = response
        response.read.return_value = '{"token":"registry-bearer"}'
        with (
            patch.dict(
                os.environ, {"GITHUB_ACTOR": "release-bot", "GH_TOKEN": "credential-canary"}
            ),
            patch.object(PACKAGING.urllib.request, "urlopen", return_value=response) as request,
        ):
            self.assertEqual(PACKAGING.registry_token("nathcymru/diffyn"), "registry-bearer")
        sent = request.call_args.args[0]
        self.assertNotIn("credential-canary", sent.full_url)
        self.assertTrue(sent.headers["Authorization"].startswith("Basic "))
        self.assertIn("scope=repository%3Anathcymru%2Fdiffyn%3Apull%2Cpush", sent.full_url)

    def test_registry_authentication_and_network_failures_are_not_treated_as_missing_tags(self):
        for status in (401, 403, 500):
            error = urllib.error.HTTPError("https://ghcr.io/", status, "failure", {}, None)
            with (
                self.subTest(status=status),
                patch.object(PACKAGING.urllib.request, "urlopen", side_effect=error),
                self.assertRaises(urllib.error.HTTPError),
            ):
                PACKAGING.registry_manifest("nathcymru/diffyn", "2.0.0", "token")
            error.close()
        missing = urllib.error.HTTPError("https://ghcr.io/", 404, "not found", {}, None)
        with patch.object(PACKAGING.urllib.request, "urlopen", side_effect=missing):
            self.assertIsNone(PACKAGING.registry_manifest("nathcymru/diffyn", "2.0.0", "token"))

    def test_older_backfill_cannot_move_latest(self):
        with (
            patch.object(PACKAGING, "command", return_value=json.dumps({"tagName": "v2.1.0"})),
            patch.object(PACKAGING.subprocess, "run") as run,
        ):
            PACKAGING.promote_latest(SOURCE, MANIFEST_DIGEST)
            run.assert_not_called()

    def test_latest_points_to_the_verified_digest_after_stable_release_check(self):
        release = {
            "tagName": SOURCE["tag"],
            "isDraft": False,
            "isImmutable": True,
            "isPrerelease": False,
        }
        with (
            patch.object(PACKAGING, "command", return_value=json.dumps(release)),
            patch.object(PACKAGING.subprocess, "run") as run,
            patch.object(PACKAGING, "registry_token", return_value="token"),
            patch.object(PACKAGING, "registry_manifest", return_value=(MANIFEST_DIGEST, {})),
        ):
            PACKAGING.promote_latest(SOURCE, MANIFEST_DIGEST)
        self.assertEqual(run.call_args.args[0][-1], f"ghcr.io/nathcymru/diffyn@{MANIFEST_DIGEST}")
        self.assertIn("--prefer-index=false", run.call_args.args[0])
        with (
            patch.object(
                PACKAGING, "command", return_value=json.dumps(release | {"isImmutable": False})
            ),
            patch.object(PACKAGING.subprocess, "run") as run,
            self.assertRaises(ValueError),
        ):
            PACKAGING.promote_latest(SOURCE, MANIFEST_DIGEST)
        run.assert_not_called()


if __name__ == "__main__":
    unittest.main()
