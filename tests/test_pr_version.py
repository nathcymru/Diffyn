"""Version reservations remain unique, immutable and bound to the PR's exact HEAD."""

import base64
import contextlib
import copy
import importlib.util
import io
import json
import os
import tempfile
import unittest
import urllib.error
from pathlib import Path
from unittest.mock import MagicMock, patch

ROOT = Path(__file__).resolve().parents[1]


def load(name, filename):
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / filename)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


policy = load("version_policy", "check-pr-version.py")
opening = load("version_opening", "open-versioned-pr.py")
REPO = "owner/project"
BRANCH = "codex/change"
HEAD = "a" * 40
BASE = "b" * 40
NEXT = "c" * 40


class FakeGitHub:
    def __init__(self):
        self.head = HEAD
        self.versions = {BASE: "2.0.0", HEAD: "2.0.1", NEXT: "2.0.1"}
        self.refs = {"v2.0.0": {"type": "commit", "sha": BASE}}
        self.objects = {}
        self.pulls = []
        self.releases = [{"draft": False, "prerelease": False, "tag_name": "v2.0.0"}]
        self.writes = []
        self.collide = False
        self.change_after_write = False

    def pages(self, path):
        if path.endswith("/releases"):
            return copy.deepcopy(self.releases)
        if "/pulls?state=all" in path:
            return copy.deepcopy(self.pulls)
        raise AssertionError(path)

    def request(self, path, method="GET", body=None, missing=False):
        if method == "POST":
            self.writes.append((path, copy.deepcopy(body)))
            if path.endswith("/git/tags"):
                sha = f"{len(self.objects) + 1:040x}"
                self.objects[sha] = {
                    "sha": sha,
                    "tag": body["tag"],
                    "message": body["message"],
                    "object": {"type": body["type"], "sha": body["object"]},
                    "tagger": {"date": "2026-10-27T02:00:00Z"},
                }
                return {"sha": sha}
            if path.endswith("/git/refs"):
                name = body["ref"].removeprefix("refs/tags/")
                if self.collide or name in self.refs:
                    raise ValueError("GitHub API request failed with HTTP 422")
                self.refs[name] = {"type": "tag", "sha": body["sha"]}
                if self.change_after_write:
                    self.head = NEXT
                return {}
            if path.endswith("/pulls"):
                pull = self.pull(len(self.pulls) + 1)
                self.pulls.append(pull)
                return copy.deepcopy(pull)
            raise AssertionError(path)
        if "/git/ref/heads/" in path:
            return {
                "object": {"type": "commit", "sha": BASE if path.endswith("/main") else self.head}
            }
        if "/contents/Cargo.toml?ref=" in path:
            sha = path.rsplit("=", 1)[1]
            if sha not in self.versions and missing:
                return None
            content = f'[package]\nname="diffyn"\nversion="{self.versions[sha]}"\n'.encode()
            return {
                "encoding": "base64",
                "size": len(content),
                "content": base64.b64encode(content).decode(),
            }
        if path.endswith("/git/matching-refs/tags/v"):
            return [
                {"ref": f"refs/tags/{name}", "object": copy.deepcopy(value)}
                for name, value in self.refs.items()
            ]
        if "/git/tags/" in path:
            return copy.deepcopy(self.objects[path.rsplit("/", 1)[1]])
        if "/pulls/" in path:
            return copy.deepcopy(
                next(pull for pull in self.pulls if pull["number"] == int(path.rsplit("/", 1)[1]))
            )
        raise AssertionError(path)

    def pull(self, number=1, state="open", created="2026-10-27T02:01:00Z"):
        return {
            "number": number,
            "state": state,
            "created_at": created,
            "html_url": f"https://github.com/{REPO}/pull/{number}",
            "head": {"sha": self.head, "ref": BRANCH, "repo": {"full_name": REPO}},
            "base": {"ref": "main"},
        }

    def reserve(self):
        return opening.prepare(self, REPO, BRANCH)


class VersionPolicyTests(unittest.TestCase):
    def test_reserves_before_opening_and_preserves_exact_branch_identity(self):
        api = FakeGitHub()
        result = api.reserve()
        self.assertEqual(result["candidate"], "v2.0.1-rc.1")
        self.assertEqual(api.pulls, [])
        annotation = json.loads(api.objects[api.refs["v2.0.1-rc.1"]["sha"]]["message"])
        self.assertEqual(annotation["branch"], BRANCH)
        pull = opening.open_pull(api, result, "Versioned change", "Public description\n")
        self.assertEqual(pull["number"], 1)
        self.assertEqual(policy.check_pull(api, REPO, 1, HEAD)["head"], HEAD)
        self.assertEqual(
            opening.open_pull(api, opening.prepare(api, REPO, BRANCH), "Ignored", ""), pull
        )

    def test_same_head_is_idempotent_and_new_head_appends_a_candidate(self):
        api = FakeGitHub()
        api.reserve()
        before = copy.deepcopy(api.refs)
        api.reserve()
        self.assertEqual(len(api.writes), 2)
        api.pulls.append(api.pull())
        api.head = NEXT
        api.pulls[0]["head"]["sha"] = NEXT
        result = api.reserve()
        self.assertEqual(result["candidate"], "v2.0.1-rc.2")
        self.assertEqual(api.refs["v2.0.1-rc.1"], before["v2.0.1-rc.1"])
        self.assertEqual(policy.check_pull(api, REPO, 1)["candidate_head"], NEXT)

    def test_existing_pr_without_prior_reservation_cannot_be_repaired_in_place(self):
        api = FakeGitHub()
        api.pulls.append(api.pull(created="2026-10-27T01:00:00Z"))
        with self.assertRaisesRegex(ValueError, "opened before"):
            api.reserve()
        self.assertEqual(api.writes, [])
        api.pulls.clear()
        api.reserve()
        api.pulls.append(api.pull(created="2026-10-27T01:00:00Z"))
        with self.assertRaisesRegex(ValueError, "predate"):
            policy.check_pull(api, REPO, 1)

    def test_same_or_lower_base_and_stable_releases_are_rejected(self):
        for version in ("2.0.0", "1.99.0"):
            api = FakeGitHub()
            api.versions[HEAD] = version
            with self.assertRaisesRegex(ValueError, "increase"):
                api.reserve()
        for tag, prerelease in (("v2.0.1", False), ("v2.0.1-beta.1", True), ("v2.1.0", False)):
            api = FakeGitHub()
            api.releases.append({"tag_name": tag, "draft": False, "prerelease": prerelease})
            with self.assertRaisesRegex(ValueError, "published"):
                api.reserve()
        api = FakeGitHub()
        api.refs["v2.0.1"] = {"type": "commit", "sha": HEAD}
        with self.assertRaisesRegex(ValueError, "stable version tag"):
            api.reserve()

    def test_prior_reservations_and_earlier_open_or_closed_prs_cannot_be_reused(self):
        api = FakeGitHub()
        api.refs["v2.0.2-rc.1"] = {"type": "tag", "sha": "d" * 40}
        with self.assertRaisesRegex(ValueError, "previously reserved"):
            api.reserve()
        for state in ("open", "closed"):
            api = FakeGitHub()
            earlier = api.pull(state=state)
            earlier["head"]["ref"] = "different-branch"
            api.pulls.append(earlier)
            with self.assertRaisesRegex(ValueError, "earlier open or closed"):
                api.reserve()
        api = FakeGitHub()
        api.reserve()
        api.pulls.append(api.pull(state="closed"))
        with self.assertRaisesRegex(ValueError, "earlier open or closed"):
            api.reserve()

    def test_later_invalid_duplicate_does_not_block_the_original_pr(self):
        api = FakeGitHub()
        api.reserve()
        api.pulls.extend([api.pull(1), api.pull(2)])
        self.assertEqual(policy.check_pull(api, REPO, 1)["candidate_head"], HEAD)
        with self.assertRaisesRegex(ValueError, "earlier open or closed"):
            policy.check_pull(api, REPO, 2)

    def test_reservation_and_candidate_annotations_cannot_change_ownership(self):
        for mutate, message in (
            (lambda tag: tag.update(message=json.dumps({})), "different branch"),
            (lambda tag: tag["object"].update(type="tag"), "directly"),
            (lambda tag: tag.update(tag="other"), "directly"),
        ):
            api = FakeGitHub()
            api.reserve()
            mutate(api.objects[api.refs["v2.0.1-rc.1"]["sha"]])
            with self.assertRaisesRegex(ValueError, message):
                api.reserve()
        api = FakeGitHub()
        api.reserve()
        api.refs["v2.0.1-rc.1"]["type"] = "commit"
        with self.assertRaisesRegex(ValueError, "annotated"):
            api.reserve()

    def test_atomic_ref_collision_and_moving_head_fail_without_retagging(self):
        api = FakeGitHub()
        api.collide = True
        with self.assertRaisesRegex(ValueError, "422"):
            api.reserve()
        self.assertNotIn("v2.0.1-rc.1", api.refs)
        api = FakeGitHub()
        api.change_after_write = True
        with self.assertRaisesRegex(ValueError, "branch changed"):
            api.reserve()
        self.assertEqual(api.objects[api.refs["v2.0.1-rc.1"]["sha"]]["object"]["sha"], HEAD)
        self.assertTrue(all(path.endswith(("/git/tags", "/git/refs")) for path, _ in api.writes))

    def test_candidate_must_match_current_head_and_event_snapshot(self):
        api = FakeGitHub()
        api.pulls.append(api.pull())
        with self.assertRaisesRegex(ValueError, "matching annotated"):
            policy.check_pull(api, REPO, 1)
        api.pulls.clear()
        api.reserve()
        api.pulls.append(api.pull())
        with self.assertRaisesRegex(ValueError, "HEAD changed"):
            policy.check_pull(api, REPO, 1, NEXT)
        api.pulls[0]["head"]["sha"] = NEXT
        with self.assertRaisesRegex(ValueError, "matching annotated"):
            policy.check_pull(api, REPO, 1)

    def test_missing_candidate_sequence_and_wrong_reservation_binding_fail(self):
        api = FakeGitHub()
        api.reserve()
        api.head = NEXT
        api.reserve()
        second = api.objects[api.refs["v2.0.1-rc.2"]["sha"]]
        details = json.loads(second["message"])
        details["reservation"] = "f" * 40
        second["message"] = json.dumps(details)
        with self.assertRaisesRegex(ValueError, "immutable rc.1"):
            api.reserve()
        details["sequence"] = 3
        second["message"] = json.dumps(details)
        second["tag"] = "v2.0.1-rc.3"
        api.refs["v2.0.1-rc.3"] = api.refs.pop("v2.0.1-rc.2")
        with self.assertRaisesRegex(ValueError, "missing immutable"):
            api.reserve()

    def test_concurrent_higher_reservation_and_changing_pr_head_are_detected(self):
        api = FakeGitHub()
        original = api.request

        def concurrent(path, method="GET", body=None, missing=False):
            result = original(path, method, body, missing)
            if method == "POST" and path.endswith("/git/refs"):
                api.refs["v2.0.2-rc.1"] = {"type": "tag", "sha": "f" * 40}
            return result

        with patch.object(api, "request", side_effect=concurrent):
            with self.assertRaisesRegex(ValueError, "concurrently"):
                api.reserve()
        api = FakeGitHub()
        api.reserve()
        api.pulls.append(api.pull())
        original = api.request
        reads = 0

        def changed(path, method="GET", body=None, missing=False):
            nonlocal reads
            if path.endswith("/pulls/1"):
                reads += 1
                if reads == 2:
                    api.pulls[0]["head"]["sha"] = NEXT
            return original(path, method, body, missing)

        with patch.object(api, "request", side_effect=changed):
            with self.assertRaisesRegex(ValueError, "changed during"):
                policy.check_pull(api, REPO, 1)

    def test_canonical_versions_and_repository_names_are_bounded(self):
        for value in ("01.0.0", "2.0.1-rc.1", "2.0.0\n", "99999999999999999999.0.0", None):
            with self.assertRaises(ValueError):
                policy.version_tuple(value)
        for value in ("../repo", "owner/repo/other", "owner/..", "owner/repo\n", None):
            with self.assertRaises(ValueError):
                policy.repository_name(value)
        with self.assertRaisesRegex(ValueError, "feature branch"):
            opening.prepare(FakeGitHub(), REPO, "main")

    def test_read_only_cli_checks_pr_and_pre_pr_candidate_events(self):
        api = FakeGitHub()
        api.reserve()
        with tempfile.TemporaryDirectory() as directory:
            event = Path(directory) / "event.json"
            event.write_text("{}")
            environment = {
                "GITHUB_REPOSITORY": REPO,
                "GITHUB_EVENT_PATH": str(event),
                "GITHUB_REF": "refs/tags/v2.0.1-rc.1",
                "GH_TOKEN": "test-token",
            }
            with (
                patch.dict(os.environ, environment),
                patch.object(policy, "GitHub", return_value=api),
                patch("sys.argv", ["check"]),
                contextlib.redirect_stdout(io.StringIO()) as output,
            ):
                policy.main()
                self.assertIn("v2.0.1-rc.1", output.getvalue())
                api.pulls.append(api.pull())
                policy.main()
                event.write_text(json.dumps({"pull_request": api.pulls[0]}))
                policy.main()
                with patch("sys.argv", ["check", "--pr", "1"]):
                    policy.main()
        self.assertEqual(len(api.writes), 2)

    def test_preflight_cli_creates_candidate_before_optional_pr(self):
        api = FakeGitHub()
        with (
            patch.object(opening.policy, "GitHub", return_value=api),
            patch.dict(os.environ, {"GH_TOKEN": "test-token"}),
            contextlib.redirect_stdout(io.StringIO()),
        ):
            with patch(
                "sys.argv", ["open", "--repo", REPO, "--branch", BRANCH, "--candidate-only"]
            ):
                opening.main()
                self.assertEqual(api.pulls, [])
            with patch(
                "sys.argv",
                ["open", "--repo", REPO, "--branch", BRANCH, "--title", "Prepared change"],
            ):
                opening.main()
                self.assertEqual(len(api.pulls), 1)


class GitHubBoundaryTests(unittest.TestCase):
    def test_fixed_origin_bounded_response_and_http_errors(self):
        with self.assertRaisesRegex(ValueError, "required"):
            policy.GitHub("")
        api = policy.GitHub("PRIVATE-TOKEN")
        response = MagicMock()
        response.__enter__.return_value = response
        response.read.return_value = b'{"ok": true}'
        with patch.object(policy.urllib.request, "build_opener") as opener:
            opener.return_value.open.return_value = response
            self.assertEqual(api.request("/repos/owner/project/pulls"), {"ok": True})
            request = opener.return_value.open.call_args.args[0]
            self.assertEqual(request.full_url, "https://api.github.com/repos/owner/project/pulls")
            response.read.assert_called_once_with(8 * 1024 * 1024 + 1)
            for code in (404, 403):
                error = urllib.error.HTTPError(request.full_url, code, "PRIVATE-BODY", {}, None)
                opener.return_value.open.side_effect = error
                if code == 404:
                    self.assertIsNone(api.request("/repos/owner/project/pulls", missing=True))
                else:
                    with self.assertRaisesRegex(ValueError, "HTTP 403") as caught:
                        api.request("/repos/owner/project/pulls")
                    self.assertNotIn("PRIVATE", str(caught.exception))
                error.close()
        with self.assertRaisesRegex(ValueError, "Invalid"):
            api.request("https://example.invalid")
        with self.assertRaisesRegex(ValueError, "redirect"):
            policy.NoRedirect().redirect_request()

    def test_pagination_never_accepts_incomplete_or_invalid_results(self):
        api = policy.GitHub("test-token")
        with patch.object(api, "request", side_effect=[[{}] * 100, [{"last": True}]]) as request:
            self.assertEqual(len(api.pages("/repos/owner/project/pulls?state=all")), 101)
            self.assertTrue(request.call_args.args[0].endswith("&per_page=100&page=2"))
        with patch.object(api, "request", return_value={}):
            with self.assertRaisesRegex(ValueError, "invalid paginated"):
                api.pages("/repos/owner/project/releases")
        with patch.object(api, "request", return_value=[{}] * 100):
            with self.assertRaisesRegex(ValueError, "no partial"):
                api.pages("/repos/owner/project/releases")


if __name__ == "__main__":
    unittest.main()
