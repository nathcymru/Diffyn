"""Synthetic positive and negative cases for the trusted DCO metadata checker."""

import importlib.util
import unittest
from pathlib import Path

SPEC = importlib.util.spec_from_file_location(
    "check_dco", Path(__file__).resolve().parents[1] / "scripts/check_dco.py"
)
dco = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(dco)


def commit(
    index=1, message="Improve a test\n\nSigned-off-by: Example Person <example@example.invalid>"
):
    return {
        "sha": f"{index:040x}",
        "commit": {
            "message": message,
            "author": {"name": "Example Person", "email": "example@example.invalid"},
            "committer": {"name": "Other Person", "email": "other@example.invalid"},
        },
    }


class DcoTests(unittest.TestCase):
    def test_matching_author_and_committer_are_supported(self):
        self.assertIsNone(dco.validate_commit(commit()))
        self.assertIsNone(
            dco.validate_commit(
                commit(message="Change\n\nSigned-off-by: Other Person <other@example.invalid>")
            )
        )

    def test_multiple_trailers_unicode_and_noreply_identity(self):
        value = commit(
            message="Change\n\nCo-authored-by: Another <another@example.invalid>\nSigned-off-by: Siôn Example <123+example@users.noreply.github.com>"
        )
        value["commit"]["author"] = {
            "name": "Siôn Example",
            "email": "123+example@users.noreply.github.com",
        }
        self.assertIsNone(dco.validate_commit(value))

    def test_body_quotes_code_and_missing_signoffs_fail(self):
        for message in [
            "Change",
            "Signed-off-by: Example Person <example@example.invalid>",
            "Change\n\n> Signed-off-by: Example Person <example@example.invalid>",
            "Change\n\nSigned-off-by: Example Person <example@example.invalid>\n\nMore text",
            "Change\n\n```\nSigned-off-by: Example Person <example@example.invalid>\n```",
        ]:
            with self.subTest(message=message):
                self.assertIsNotNone(dco.validate_commit(commit(message=message)))

    def test_mismatched_cross_paired_and_forged_control_identity_fail(self):
        for signoff in [
            "Unrelated <unrelated@example.invalid>",
            "Example Person <other@example.invalid>",
            "Example\x00 Person <example@example.invalid>",
            "Example Person <not-an-email>",
        ]:
            self.assertIsNotNone(
                dco.validate_commit(commit(message=f"Change\n\nSigned-off-by: {signoff}"))
            )

    def test_empty_and_malformed_metadata_fail(self):
        for value in [None, {}, {"sha": "bad"}, {"sha": "a" * 40, "commit": {"message": "Change"}}]:
            self.assertIsNotNone(dco.validate_commit(value))

    def api(self, commits, *, count=None, after_head=None, pages=None):
        calls = []
        head = commits[-1]["sha"]

        def get(path):
            calls.append(path)
            if "/commits?" in path:
                page = int(path.rsplit("=", 1)[1])
                return (
                    pages[page - 1] if pages is not None else commits[(page - 1) * 100 : page * 100]
                )
            return {
                "commits": len(commits) if count is None else count,
                "head": {"sha": after_head if after_head and len(calls) > 1 else head},
            }

        return get, calls, head

    def test_all_pages_and_every_commit_are_checked(self):
        commits = [commit(i) for i in range(1, 206)]
        commits[101]["commit"]["message"] = "Unsigned change on page two"
        get, calls, head = self.api(commits)
        failures = dco.check_pull_request("owner/repository", 1, head, get)
        self.assertEqual(len(failures), 1)
        self.assertIn(commits[101]["sha"][:12], failures[0])
        self.assertEqual(len(calls), 5)

    def test_complete_signed_pull_request_passes(self):
        get, _, head = self.api([commit(1), commit(2)])
        self.assertEqual(dco.check_pull_request("owner/repository", 1, head, get), [])

    def test_incomplete_duplicate_and_out_of_order_pages_fail_closed(self):
        commits = [commit(1), commit(2)]
        for page in [[commits[1]], [commits[1], commits[1]], list(reversed(commits))]:
            get, _, head = self.api(commits, pages=[page])
            with self.assertRaises(dco.CheckError):
                dco.check_pull_request("owner/repository", 1, head, get)

    def test_changed_head_and_endpoint_limit_fail_closed(self):
        for changes in [{"after_head": "f" * 40}, {"count": 251}, {"count": 0}]:
            get, _, head = self.api([commit()], **changes)
            with self.assertRaises(dco.CheckError):
                dco.check_pull_request("owner/repository", 1, head, get)

    def test_untrusted_identifiers_never_reach_the_api(self):
        for repository, number, head in [
            ("owner/../other", 1, "a" * 40),
            ("owner/repo", True, "a" * 40),
            ("owner/repo", 1, "$(echo injected)"),
        ]:
            with self.assertRaises(dco.CheckError):
                dco.check_pull_request(
                    repository, number, head, lambda _: self.fail("No API call expected")
                )

    def test_redirects_are_refused(self):
        with self.assertRaises(dco.CheckError):
            dco.NoRedirect().redirect_request(None, None, 302, "", {}, "https://example.invalid")

    def test_failure_output_does_not_include_submitted_message_or_email(self):
        commits = [
            commit(message="PRIVATE-CANARY\n\nSigned-off-by: Wrong <private@example.invalid>")
        ]
        get, _, head = self.api(commits)
        failures = dco.check_pull_request("owner/repository", 1, head, get)
        self.assertNotIn("PRIVATE-CANARY", str(failures))
        self.assertNotIn("private@example.invalid", str(failures))


if __name__ == "__main__":
    unittest.main()
