"""The DCO API boundary must fail safely without exposing response or token data."""

import contextlib
import importlib.util
import io
import json
import os
import tempfile
import unittest
import urllib.error
from pathlib import Path
from unittest.mock import MagicMock, patch

SPEC = importlib.util.spec_from_file_location(
    "dco_network", Path(__file__).resolve().parents[1] / "scripts/check_dco.py"
)
dco = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(dco)


class DcoNetworkTests(unittest.TestCase):
    def test_api_uses_authenticated_fixed_origin_and_bounded_read(self):
        response = MagicMock()
        response.__enter__.return_value = response
        response.read.return_value = b'{"commits": 1}'
        opener = MagicMock()
        opener.open.return_value = response
        with patch.object(dco.urllib.request, "build_opener", return_value=opener):
            self.assertEqual(
                dco.github_get("/repos/owner/repo/pulls/1", "test-token"), {"commits": 1}
            )
        request = opener.open.call_args.args[0]
        self.assertEqual(request.full_url, "https://api.github.com/repos/owner/repo/pulls/1")
        self.assertEqual(request.get_header("Authorization"), "Bearer test-token")
        response.read.assert_called_once_with(dco.MAX_RESPONSE_BYTES + 1)

    def test_oversize_invalid_json_network_and_http_failures_are_sanitized(self):
        for content in [b"PRIVATE-CANARY", b"x" * (dco.MAX_RESPONSE_BYTES + 1)]:
            response = MagicMock()
            response.__enter__.return_value = response
            response.read.return_value = content
            with patch.object(dco.urllib.request, "build_opener") as build:
                build.return_value.open.return_value = response
                with self.assertRaises(dco.CheckError) as caught:
                    dco.github_get("/repos/owner/repo/pulls/1", "PRIVATE-TOKEN")
                self.assertNotIn("PRIVATE", str(caught.exception))
        for error in [
            urllib.error.URLError("PRIVATE-CANARY"),
            urllib.error.HTTPError("https://example.invalid", 403, "PRIVATE-CANARY", {}, None),
        ]:
            with patch.object(dco.urllib.request, "build_opener") as build:
                build.return_value.open.side_effect = error
                with self.assertRaises(dco.CheckError) as caught:
                    dco.github_get("/repos/owner/repo/pulls/1", "PRIVATE-TOKEN")
                self.assertNotIn("PRIVATE", str(caught.exception))
            if isinstance(error, urllib.error.HTTPError):
                error.close()

    def test_main_binds_event_to_repository_and_propagates_signed_or_unsigned_result(self):
        with tempfile.TemporaryDirectory() as directory:
            event = Path(directory) / "event.json"
            payload = {
                "pull_request": {
                    "base": {"repo": {"full_name": "owner/repo"}},
                    "number": 1,
                    "head": {"sha": "a" * 40},
                }
            }
            event.write_text(json.dumps(payload))
            environment = {
                "GITHUB_REPOSITORY": "owner/repo",
                "GITHUB_TOKEN": "test-token",
                "GITHUB_EVENT_PATH": str(event),
            }
            with (
                patch.dict(os.environ, environment),
                contextlib.redirect_stdout(io.StringIO()),
                contextlib.redirect_stderr(io.StringIO()),
            ):
                with patch.object(dco, "check_pull_request", return_value=[]) as check:
                    self.assertEqual(dco.main(), 0)
                    self.assertEqual(check.call_args.args[:3], ("owner/repo", 1, "a" * 40))
                with patch.object(dco, "check_pull_request", return_value=["Missing sign-off"]):
                    self.assertEqual(dco.main(), 1)
                payload["pull_request"]["base"]["repo"]["full_name"] = "other/repo"
                event.write_text(json.dumps(payload))
                with patch.object(dco, "check_pull_request") as check:
                    self.assertEqual(dco.main(), 1)
                    check.assert_not_called()
                event.write_text("not valid JSON")
                self.assertEqual(dco.main(), 1)


if __name__ == "__main__":
    unittest.main()
