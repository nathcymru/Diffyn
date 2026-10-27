"""Verify that automatic release tags cannot escape the validated main commit."""

import importlib.util
import json
import os
import subprocess
import unittest
from pathlib import Path
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location(
    "start_release", Path(__file__).resolve().parents[1] / "scripts" / "start-release.py"
)
RELEASE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(RELEASE)
COMMIT = "a" * 40
ENVIRONMENT = {
    "GITHUB_REPOSITORY": "nathcymru/Diffyn",
    "GITHUB_REF": "refs/heads/main",
    "GITHUB_SHA": COMMIT,
    "RELEASE_COMMIT": COMMIT,
    "EXPECTED_VERSION": "2.1.0",
}
VALIDATED = json.dumps({"version": "2.1.0", "tag": "v2.1.0"})


class StartReleaseTests(unittest.TestCase):
    def test_rejects_non_main_untrusted_repository_or_invalid_dispatch_before_remote_writes(self):
        for key, value in (
            ("GITHUB_REF", "refs/heads/feature"),
            ("GITHUB_REPOSITORY", "other/repo"),
            ("RELEASE_COMMIT", "main"),
            ("EXPECTED_VERSION", "2.1.0-beta.1"),
            ("GITHUB_SHA", "b" * 40),
        ):
            with (
                self.subTest(key=key),
                patch.dict(os.environ, ENVIRONMENT | {key: value}),
                patch.object(RELEASE.subprocess, "run") as run,
                self.assertRaises(ValueError),
            ):
                RELEASE.main()
            run.assert_not_called()

    def test_rejects_main_advancing_after_dispatch(self):
        with (
            patch.dict(os.environ, ENVIRONMENT),
            patch.object(RELEASE, "output", side_effect=[COMMIT, "b" * 40]),
            patch.object(RELEASE.subprocess, "run") as run,
            self.assertRaisesRegex(ValueError, "current main"),
        ):
            RELEASE.main()
        run.assert_not_called()

    def test_rejects_version_drift_before_tag_creation(self):
        wrong_version = json.dumps({"version": "2.2.0", "tag": "v2.2.0"})
        with (
            patch.dict(os.environ, ENVIRONMENT),
            patch.object(RELEASE, "output", side_effect=[COMMIT, COMMIT, wrong_version]),
            patch.object(RELEASE.subprocess, "run") as run,
            self.assertRaisesRegex(ValueError, "validated source"),
        ):
            RELEASE.main()
        run.assert_not_called()

    def test_existing_tag_cannot_be_moved(self):
        existing = subprocess.CompletedProcess([], 0, stdout="b" * 40 + "\n")
        with (
            patch.dict(os.environ, ENVIRONMENT),
            patch.object(RELEASE, "output", side_effect=[COMMIT, COMMIT, VALIDATED]),
            patch.object(RELEASE.subprocess, "run", return_value=existing) as run,
            patch.object(RELEASE.subprocess, "check_output") as api,
            self.assertRaisesRegex(ValueError, "another commit"),
        ):
            RELEASE.main()
        self.assertEqual(run.call_count, 1)
        api.assert_not_called()

    def test_retry_retains_matching_tag(self):
        existing = subprocess.CompletedProcess([], 0, stdout=COMMIT + "\n")
        with (
            patch.dict(os.environ, ENVIRONMENT),
            patch.object(RELEASE, "output", side_effect=[COMMIT, COMMIT, VALIDATED]),
            patch.object(RELEASE.subprocess, "run", return_value=existing) as run,
            patch.object(RELEASE.subprocess, "check_output") as api,
        ):
            RELEASE.main()
        self.assertEqual(run.call_count, 1)
        api.assert_not_called()

    def test_new_version_creates_an_annotated_tag_at_exact_commit(self):
        missing = subprocess.CompletedProcess([], 1, stdout="")
        tag_object = "c" * 40
        with (
            patch.dict(os.environ, ENVIRONMENT),
            patch.object(RELEASE, "output", side_effect=[COMMIT, COMMIT, VALIDATED]),
            patch.object(RELEASE.subprocess, "run", return_value=missing) as run,
            patch.object(
                RELEASE.subprocess, "check_output", return_value=json.dumps({"sha": tag_object})
            ) as api,
        ):
            RELEASE.main()
        annotation = json.loads(api.call_args.kwargs["input"])
        self.assertEqual(
            annotation,
            {
                "tag": "v2.1.0",
                "message": "Diffyn v2.1.0",
                "object": COMMIT,
                "type": "commit",
            },
        )
        self.assertEqual(
            json.loads(run.call_args.kwargs["input"]),
            {
                "ref": "refs/tags/v2.1.0",
                "sha": tag_object,
            },
        )
        self.assertNotIn("--force", run.call_args.args[0])


if __name__ == "__main__":
    unittest.main()
