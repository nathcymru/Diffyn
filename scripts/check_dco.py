#!/usr/bin/env python3
"""Check public PR commit metadata against DCO trailers without executing PR code."""

from __future__ import annotations

import json
import os
import re
import sys
import urllib.error
import urllib.request
from pathlib import Path

MAX_COMMITS = 250  # GitHub's pull-request commits endpoint returns at most 250.
MAX_RESPONSE_BYTES = 8 * 1024 * 1024
SHA = re.compile(r"[0-9a-f]{40}\Z")
REPOSITORY = re.compile(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+\Z")
TRAILER = re.compile(r"([A-Za-z0-9-]+):[ \t]*(.*)\Z")
IDENTITY = re.compile(r"([^<>\r\n]+?) <([^<>\s]+@[^<>\s]+)>\Z")


class CheckError(Exception):
    """Fixed, non-sensitive error suitable for an Actions log."""


class NoRedirect(urllib.request.HTTPRedirectHandler):
    """Never forward a token to a redirected host or repository endpoint."""

    def redirect_request(self, req, fp, code, msg, headers, newurl):
        raise CheckError("GitHub API redirected; check the repository configuration.")


def identity(name: object, email: object) -> tuple[str, str] | None:
    if not isinstance(name, str) or not isinstance(email, str):
        return None
    if any(ord(ch) < 32 or ord(ch) == 127 for ch in name + email):
        return None
    candidate = f"{name.strip()} <{email.strip()}>"
    matched = IDENTITY.fullmatch(candidate)
    if not matched:
        return None
    # GitHub preserves the stored identity. Do not mix an author's name with a
    # committer's email or treat a username as proof of a different identity.
    return matched.group(1), matched.group(2).casefold()


def signed_identities(message: str) -> set[tuple[str, str]]:
    """Read the final trailer paragraph, excluding quoted/body-only sign-offs."""
    paragraphs = re.split(r"\n[ \t]*\n", message.replace("\r\n", "\n").strip())
    # Require a commit subject followed by a separate trailer paragraph.
    if len(paragraphs) < 2:
        return set()
    result = set()
    for line in paragraphs[-1].splitlines():
        matched = TRAILER.fullmatch(line)
        if not matched:
            return set()
        if matched.group(1).casefold() == "signed-off-by":
            signoff = IDENTITY.fullmatch(matched.group(2))
            if signoff:
                parsed = identity(signoff.group(1), signoff.group(2))
                if parsed:
                    result.add(parsed)
    return result


def validate_commit(commit: object) -> str | None:
    """Return a fixed reason, or None when a commit has a matching sign-off."""
    if not isinstance(commit, dict) or not SHA.fullmatch(str(commit.get("sha", ""))):
        return "Commit metadata has no valid SHA."
    metadata = commit.get("commit")
    if not isinstance(metadata, dict) or not isinstance(metadata.get("message"), str):
        return "Commit metadata is incomplete."
    identities = set()
    for role in ("author", "committer"):
        value = metadata.get(role)
        if isinstance(value, dict):
            parsed = identity(value.get("name"), value.get("email"))
            if parsed:
                identities.add(parsed)
    if not identities:
        return "Commit metadata has no usable author or committer identity."
    if not identities.intersection(signed_identities(metadata["message"])):
        return "Missing final Signed-off-by trailer matching the author or committer."
    return None


def github_get(path: str, token: str) -> object:
    # Endpoints are constructed below from validated identifiers. Caller-owned
    # URLs, branch names and commit messages never become URLs or shell input.
    request = urllib.request.Request(
        "https://api.github.com" + path,
        headers={
            "Accept": "application/vnd.github+json",
            "Authorization": f"Bearer {token}",
            "X-GitHub-Api-Version": "2022-11-28",
            "User-Agent": "Diffyn-DCO-check",
        },
    )
    try:
        with urllib.request.build_opener(NoRedirect()).open(request, timeout=20) as response:
            content = response.read(MAX_RESPONSE_BYTES + 1)
        if len(content) > MAX_RESPONSE_BYTES:
            raise CheckError("GitHub API response exceeded the safety limit.")
        return json.loads(content)
    except urllib.error.HTTPError as error:
        raise CheckError(f"GitHub API request failed with HTTP {error.code}.") from None
    except (urllib.error.URLError, TimeoutError, OSError, ValueError):
        raise CheckError("GitHub API request failed or returned invalid JSON.") from None


def check_pull_request(repository: str, number: int, expected_head: str, get) -> list[str]:
    """Verify all pages and a stable head before returning per-commit failures."""
    if not REPOSITORY.fullmatch(repository) or any(part in (".", "..") for part in repository.split("/")):
        raise CheckError("Repository identifier is invalid.")
    if type(number) is not int or number < 1 or not SHA.fullmatch(expected_head):
        raise CheckError("Pull request event identifiers are invalid.")
    endpoint = f"/repos/{repository}/pulls/{number}"

    def snapshot():
        value = get(endpoint)
        if not isinstance(value, dict):
            raise CheckError("Pull request metadata is incomplete.")
        count = value.get("commits")
        head = value.get("head", {})
        sha = head.get("sha") if isinstance(head, dict) else None
        if type(count) is not int or not 1 <= count <= MAX_COMMITS:
            raise CheckError("Pull requests must contain 1–250 commits for complete DCO checking; split larger changes.")
        if sha != expected_head:
            raise CheckError("Pull request head changed; run the check for the current head.")
        return count, sha

    count, _ = snapshot()
    commits = []
    for page in range(1, (count + 99) // 100 + 1):
        values = get(f"{endpoint}/commits?per_page=100&page={page}")
        if not isinstance(values, list):
            raise CheckError("GitHub returned an invalid commit page.")
        commits.extend(values)
    if snapshot() != (count, expected_head):
        raise CheckError("Pull request changed while checking; rerun for the current head.")
    shas = [entry.get("sha") if isinstance(entry, dict) else None for entry in commits]
    if (len(commits) != count or any(not isinstance(sha, str) or not SHA.fullmatch(sha) for sha in shas)
            or len(set(shas)) != count or shas[-1] != expected_head):
        raise CheckError("Commit pages are incomplete or inconsistent; no partial DCO result is accepted.")
    failures = []
    for commit in commits:
        if reason := validate_commit(commit):
            failures.append(f"{commit['sha'][:12]}: {reason}")
    return failures


def main() -> int:
    try:
        repository = os.environ["GITHUB_REPOSITORY"]
        token = os.environ["GITHUB_TOKEN"]
        event = json.loads(Path(os.environ["GITHUB_EVENT_PATH"]).read_text(encoding="utf-8"))
        if not token:
            raise CheckError("A read-only GitHub token is required.")
        pull_request = event["pull_request"]
        if pull_request["base"]["repo"]["full_name"].casefold() != repository.casefold():
            raise CheckError("Pull request base does not match the workflow repository.")
        failures = check_pull_request(
            repository, pull_request["number"], pull_request["head"]["sha"],
            lambda path: github_get(path, token),
        )
    except CheckError as error:
        print(f"DCO check failed: {error}", file=sys.stderr)
        return 1
    except (KeyError, TypeError, ValueError, OSError):
        print("DCO check failed: required event metadata is missing or invalid.", file=sys.stderr)
        return 1
    if failures:
        print("\n".join(failures), file=sys.stderr)
        print("Read .github/DCO and the contribution guide before adding your own sign-off.", file=sys.stderr)
        return 1
    print("Every pull-request commit has a matching DCO sign-off.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
