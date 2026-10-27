#!/usr/bin/env python3
"""Check that a pull request owns a fresh version and an immutable HEAD candidate."""

import argparse
import base64
import json
import os
import re
import sys
import tomllib
import urllib.error
import urllib.parse
import urllib.request
from datetime import datetime
from pathlib import Path

SCHEMA = "diffyn.release-candidate.v1"
VERSION = re.compile(r"(0|[1-9][0-9]{0,19})\.(0|[1-9][0-9]{0,19})\.(0|[1-9][0-9]{0,19})\Z")
CANDIDATE = re.compile(
    r"v((?:0|[1-9][0-9]{0,19})\.(?:0|[1-9][0-9]{0,19})\.(?:0|[1-9][0-9]{0,19}))-rc\.([1-9][0-9]{0,5})\Z"
)
SHA = re.compile(r"[0-9a-f]{40}\Z")
REPOSITORY = re.compile(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+\Z")


def version_tuple(value):
    if not isinstance(value, str) or not VERSION.fullmatch(value):
        raise ValueError("Use a canonical stable major.minor.patch version")
    result = tuple(map(int, value.split(".")))
    if any(part > 2**64 - 1 for part in result):
        raise ValueError("Version components exceed the SemVer limit")
    return result


def repository_name(value):
    if (
        not isinstance(value, str)
        or not REPOSITORY.fullmatch(value)
        or any(part in (".", "..") for part in value.split("/"))
    ):
        raise ValueError("Use an owner/repository identifier")
    return value


def encoded(value):
    return urllib.parse.quote(value, safe="")


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *_args, **_kwargs):
        raise ValueError("GitHub API redirects are not accepted")


class GitHub:
    def __init__(self, token):
        if not token:
            raise ValueError("GH_TOKEN or GITHUB_TOKEN is required")
        self.token = token

    def request(self, path, method="GET", body=None, missing=False):
        if not path.startswith("/repos/") or "\r" in path or "\n" in path:
            raise ValueError("Invalid GitHub API path")
        request = urllib.request.Request(
            "https://api.github.com" + path,
            data=None if body is None else json.dumps(body).encode(),
            method=method,
            headers={
                "Accept": "application/vnd.github+json",
                "Authorization": f"Bearer {self.token}",
                "Content-Type": "application/json",
                "X-GitHub-Api-Version": "2022-11-28",
            },
        )
        try:
            with urllib.request.build_opener(NoRedirect()).open(request, timeout=30) as response:
                payload = response.read(8 * 1024 * 1024 + 1)
            if len(payload) > 8 * 1024 * 1024:
                raise ValueError("GitHub API response exceeded the safety limit")
            return json.loads(payload)
        except urllib.error.HTTPError as error:
            if missing and error.code == 404:
                return None
            raise ValueError(f"GitHub API request failed with HTTP {error.code}") from None
        except (urllib.error.URLError, TimeoutError, OSError):
            raise ValueError("GitHub API request failed") from None

    def pages(self, path):
        result = []
        for page in range(1, 101):
            values = self.request(f"{path}{'&' if '?' in path else '?'}per_page=100&page={page}")
            if not isinstance(values, list):
                raise ValueError("GitHub returned an invalid paginated response")
            result.extend(values)
            if len(values) < 100:
                return result
        raise ValueError("GitHub result limit exceeded; no partial version check is accepted")


def version_at(api, repository, reference, allow_missing=False):
    repository_name(repository)
    if not SHA.fullmatch(reference):
        raise ValueError("A complete source commit ID is required")
    file = api.request(
        f"/repos/{repository}/contents/Cargo.toml?ref={reference}", missing=allow_missing
    )
    if file is None and allow_missing:
        return None
    if file.get("encoding") != "base64" or file.get("size", 0) > 65536:
        raise ValueError("Cargo.toml has an unsupported encoding or size")
    manifest = tomllib.loads(base64.b64decode(file["content"]).decode("utf-8"))
    version = manifest["package"]["version"]
    version_tuple(version)
    return version


def tags(api, repository):
    # GitHub's matching-refs endpoint returns all matching references and has
    # no pagination parameters. The client's response bound fails closed.
    values = api.request(f"/repos/{repository}/git/matching-refs/tags/v")
    if not isinstance(values, list):
        raise ValueError("GitHub returned an invalid tag list")
    return {entry["ref"].removeprefix("refs/tags/"): entry["object"] for entry in values}


def branch_pulls(api, repository, head_repository, branch):
    owner = head_repository.split("/", 1)[0]
    pulls = api.pages(f"/repos/{repository}/pulls?state=all&head={encoded(owner + ':' + branch)}")
    return [
        pull
        for pull in pulls
        if pull.get("head", {}).get("ref") == branch
        and (pull["head"].get("repo") or {}).get("full_name", "").casefold()
        == head_repository.casefold()
    ]


def inspect(api, repository, branch, head_repository, head, base_branch, current_pr=None):
    """Validate source versions and reservation ownership without changing GitHub."""
    repository_name(repository)
    repository_name(head_repository)
    if not SHA.fullmatch(head) or not branch or not base_branch:
        raise ValueError("A branch, base branch and complete HEAD commit are required")
    base = api.request(f"/repos/{repository}/git/ref/heads/{encoded(base_branch)}")["object"]
    version = version_at(api, head_repository, head)
    number = version_tuple(version)
    if number <= version_tuple(version_at(api, repository, base["sha"])):
        raise ValueError(
            "Every pull request must increase the version above its current base branch"
        )
    references = tags(api, repository)
    if f"v{version}" in references:
        raise ValueError("The stable version tag already exists; choose a new version")
    published = [
        release for release in api.pages(f"/repos/{repository}/releases") if not release["draft"]
    ]
    for release in published:
        match = re.match(r"^v?([0-9]+\.[0-9]+\.[0-9]+)(?:[-+]|$)", release["tag_name"])
        if match and version_tuple(match[1]) == number:
            raise ValueError("The version was already used by a published release")
        if not release["prerelease"] and VERSION.fullmatch(release["tag_name"].removeprefix("v")):
            if number <= version_tuple(release["tag_name"].removeprefix("v")):
                raise ValueError("The version must exceed every published stable release")
    candidates = sorted(
        (int(match[2]), name, reference)
        for name, reference in references.items()
        if (match := CANDIDATE.fullmatch(name)) and match[1] == version
    )
    result = {
        "repository": repository,
        "head_repository": head_repository,
        "branch": branch,
        "base_branch": base_branch,
        "head": head,
        "version": version,
        "candidates": candidates,
    }
    for pull in api.pages(f"/repos/{repository}/pulls?state=all"):
        if (current_pr is None or pull["number"] < current_pr) and version_at(
            api, repository, pull["head"]["sha"], allow_missing=True
        ) == version:
            raise ValueError(
                "The version has already been used by an earlier open or closed pull request"
            )
    if not candidates:
        for name in references:
            candidate = CANDIDATE.fullmatch(name)
            previous = candidate[1] if candidate else name.removeprefix("v")
            if VERSION.fullmatch(previous) and number <= version_tuple(previous):
                raise ValueError(
                    "A new reservation must exceed every previously reserved or tagged version"
                )
        return result
    if candidates[0][0] != 1:
        raise ValueError("The immutable rc.1 reservation is missing")
    reservation = None
    expected = {
        "schema": SCHEMA,
        "repository": repository,
        "head_repository": head_repository,
        "branch": branch,
        "base_branch": base_branch,
        "version": version,
    }
    for expected_sequence, (sequence, name, reference) in enumerate(candidates, 1):
        if sequence != expected_sequence:
            raise ValueError("Candidate sequence contains a missing immutable tag")
        if reference["type"] != "tag" or not SHA.fullmatch(reference["sha"]):
            raise ValueError("Candidate references must be annotated tags")
        tag = api.request(f"/repos/{repository}/git/tags/{reference['sha']}")
        details = json.loads(tag["message"])
        if (
            any(details.get(key) != value for key, value in expected.items())
            or details.get("sequence") != sequence
        ):
            raise ValueError("This version reservation belongs to a different branch or repository")
        if (
            tag["tag"] != name
            or tag["object"]["type"] != "commit"
            or not SHA.fullmatch(tag["object"]["sha"])
        ):
            raise ValueError("The candidate annotation does not directly identify a source commit")
        if sequence == 1:
            reservation = tag
            if version_at(api, head_repository, tag["object"]["sha"]) != version:
                raise ValueError("The reserved source commit must contain the reserved version")
        elif details.get("reservation") != candidates[0][2]["sha"]:
            raise ValueError("The candidate does not identify its immutable rc.1 reservation")
        result["candidate"] = name
        result["candidate_head"] = tag["object"]["sha"]
    reserved_at = datetime.fromisoformat(reservation["tagger"]["date"].replace("Z", "+00:00"))
    for pull in branch_pulls(api, repository, head_repository, branch):
        if current_pr is not None and pull["number"] >= current_pr:
            continue
        created = datetime.fromisoformat(pull["created_at"].replace("Z", "+00:00"))
        if (
            created >= reserved_at
            or version_at(api, head_repository, pull["head"]["sha"]) == version
        ):
            raise ValueError(
                "This version reservation has already been used by another pull request"
            )
    result["reservation"] = candidates[0][2]["sha"]
    result["reserved_at"] = reserved_at.isoformat()
    return result


def check_pull(api, repository, number, expected_head=None):
    pull = api.request(f"/repos/{repository}/pulls/{number}")
    head = pull["head"]["sha"]
    if expected_head is not None and head != expected_head:
        raise ValueError("The pull request HEAD changed; rerun the check for the current commit")
    result = inspect(
        api,
        repository,
        pull["head"]["ref"],
        pull["head"]["repo"]["full_name"],
        head,
        pull["base"]["ref"],
        number,
    )
    if result.get("candidate_head") != head:
        raise ValueError("The current HEAD needs a matching annotated vVERSION-rc.N candidate tag")
    created = datetime.fromisoformat(pull["created_at"].replace("Z", "+00:00"))
    if datetime.fromisoformat(result["reserved_at"]) > created:
        raise ValueError(
            "The first candidate must predate the PR; close it and open a replacement with a fresh reserved version"
        )
    if api.request(f"/repos/{repository}/pulls/{number}")["head"]["sha"] != head:
        raise ValueError(
            "The pull request HEAD changed during validation; rerun for the current commit"
        )
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pr", type=int)
    args = parser.parse_args()
    repository = repository_name(os.environ["GITHUB_REPOSITORY"])
    api = GitHub(os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN"))
    event = json.loads(Path(os.environ["GITHUB_EVENT_PATH"]).read_text())
    if args.pr or "pull_request" in event:
        number = args.pr or event["pull_request"]["number"]
        expected = None if args.pr else event["pull_request"]["head"]["sha"]
        result = check_pull(api, repository, number, expected)
    else:
        tag_name = os.environ["GITHUB_REF"].removeprefix("refs/tags/")
        if not CANDIDATE.fullmatch(tag_name):
            raise ValueError("This event must identify a pull request or release-candidate tag")
        reference = tags(api, repository)[tag_name]
        tag = api.request(f"/repos/{repository}/git/tags/{reference['sha']}")
        details = json.loads(tag["message"])
        head = tag["object"]["sha"]
        matching = [
            pull
            for pull in branch_pulls(api, repository, details["head_repository"], details["branch"])
            if pull["state"] == "open"
        ]
        if len(matching) > 1:
            raise ValueError("A candidate branch cannot belong to multiple open pull requests")
        result = (
            check_pull(api, repository, matching[0]["number"], head)
            if matching
            else inspect(
                api,
                repository,
                details["branch"],
                details["head_repository"],
                head,
                details["base_branch"],
            )
        )
        if result.get("candidate_head") != head:
            raise ValueError("A newer candidate exists; validate the latest HEAD")
    print(json.dumps({key: result[key] for key in ("version", "head", "candidate")}))


if __name__ == "__main__":
    try:
        main()
    except (ValueError, KeyError, TypeError, OSError) as error:
        print(f"PR version validation failed: {error}", file=sys.stderr)
        sys.exit(1)
