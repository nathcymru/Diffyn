"""Keep stable latest publication ordered even when older versions are backfilled."""

import importlib.util
import json
import unittest
from pathlib import Path
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location(
    "release_publication", Path(__file__).resolve().parents[1] / "scripts" / "package-release.py"
)
RELEASE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(RELEASE)


class ReleasePublicationTests(unittest.TestCase):
    def publication(self, pages):
        with (
            patch.object(RELEASE, "verify_release") as verify,
            patch.object(RELEASE.subprocess, "check_output", side_effect=["{}", json.dumps(pages)]),
            patch.object(RELEASE.subprocess, "run") as publish,
        ):
            RELEASE.publish_draft("2.0.0", Path("dist"))
        verify.assert_called_once_with("2.0.0", Path("dist"), {})
        return publish.call_args.args[0]

    def test_newest_stable_and_first_release_become_latest(self):
        for pages in ([[]], [[{"tag_name": "v0.1.0", "draft": False, "prerelease": False}]]):
            with self.subTest(pages=pages):
                self.assertIn("--latest=true", self.publication(pages))

    def test_older_release_cannot_displace_latest_across_pages(self):
        self.assertIn(
            "--latest=false",
            self.publication(
                [
                    [{"tag_name": "v0.1.0", "draft": False, "prerelease": False}],
                    [{"tag_name": "v2.10.0", "draft": False, "prerelease": False}],
                ]
            ),
        )

    def test_drafts_and_prereleases_do_not_change_stable_latest(self):
        self.assertIn(
            "--latest=true",
            self.publication(
                [
                    [
                        {"tag_name": "v9.0.0", "draft": True, "prerelease": False},
                        {"tag_name": "v9.0.0-beta.1", "draft": False, "prerelease": True},
                    ]
                ]
            ),
        )

    def test_failed_asset_verification_prevents_publication(self):
        with (
            patch.object(RELEASE, "verify_release", side_effect=ValueError("bad asset")),
            patch.object(RELEASE.subprocess, "check_output", return_value="{}"),
            patch.object(RELEASE.subprocess, "run") as publish,
            self.assertRaises(ValueError),
        ):
            RELEASE.publish_draft("2.0.0", Path("dist"))
        publish.assert_not_called()

    def test_malformed_release_history_cannot_move_latest(self):
        for pages in (
            {},
            [{}],
            [[None]],
            [[{"tag_name": "unversioned", "draft": False, "prerelease": False}]],
        ):
            with (
                self.subTest(pages=pages),
                patch.object(RELEASE, "verify_release"),
                patch.object(
                    RELEASE.subprocess, "check_output", side_effect=["{}", json.dumps(pages)]
                ),
                patch.object(RELEASE.subprocess, "run") as publish,
                self.assertRaises(ValueError),
            ):
                RELEASE.publish_draft("2.0.0", Path("dist"))
            publish.assert_not_called()


if __name__ == "__main__":
    unittest.main()
