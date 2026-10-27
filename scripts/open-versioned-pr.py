#!/usr/bin/env python3
"""Reserve an immutable version candidate before opening or updating a pull request."""

import argparse
import importlib.util
import json
import os
import sys
from pathlib import Path

SPEC = importlib.util.spec_from_file_location(
    "pr_version", Path(__file__).with_name("check-pr-version.py")
)
policy = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(policy)


def prepare(api, repository, branch, base="main", head_repository=None):
    repository = policy.repository_name(repository)
    head_repository = policy.repository_name(head_repository or repository)
    if not branch or branch == base and head_repository == repository:
        raise ValueError("Use a feature branch separate from the base branch")
    reference_path = f"/repos/{head_repository}/git/ref/heads/{policy.encoded(branch)}"
    head = api.request(reference_path)["object"]["sha"]
    pulls = [
        pull
        for pull in policy.branch_pulls(api, repository, head_repository, branch)
        if pull["state"] == "open"
    ]
    if len(pulls) > 1:
        raise ValueError("The feature branch belongs to multiple open pull requests")
    current = pulls[0] if pulls else None
    result = policy.inspect(
        api, repository, branch, head_repository, head, base, current["number"] if current else None
    )
    if current and (
        not result.get("reserved_at")
        or policy.datetime.fromisoformat(result["reserved_at"])
        > policy.datetime.fromisoformat(current["created_at"].replace("Z", "+00:00"))
    ):
        raise ValueError(
            "This PR was opened before its version reservation; close it and prepare a fresh replacement"
        )
    if result.get("candidate_head") != head:
        sequence = result["candidates"][-1][0] + 1 if result["candidates"] else 1
        if sequence > 999999:
            raise ValueError("Candidate sequence limit reached; use a new version")
        name = f"v{result['version']}-rc.{sequence}"
        details = {
            "schema": policy.SCHEMA,
            "repository": repository,
            "head_repository": head_repository,
            "branch": branch,
            "base_branch": base,
            "version": result["version"],
            "sequence": sequence,
        }
        if sequence > 1:
            details["reservation"] = result["reservation"]
        tag = api.request(
            f"/repos/{repository}/git/tags",
            "POST",
            {
                "tag": name,
                "message": json.dumps(details, sort_keys=True),
                "object": head,
                "type": "commit",
            },
        )
        # Creating a ref is atomic. A concurrent reservation fails rather than
        # updating or deleting the first writer's immutable version tag.
        api.request(
            f"/repos/{repository}/git/refs", "POST", {"ref": f"refs/tags/{name}", "sha": tag["sha"]}
        )
        result["candidate"] = name
        result["candidate_head"] = head
        if sequence == 1:
            for other in policy.tags(api, repository):
                candidate = policy.CANDIDATE.fullmatch(other)
                previous = candidate[1] if candidate else other.removeprefix("v")
                if policy.VERSION.fullmatch(previous) and policy.version_tuple(
                    previous
                ) > policy.version_tuple(result["version"]):
                    raise ValueError(
                        "A higher version was reserved concurrently; retain this tag and prepare a fresh higher version"
                    )
    if api.request(reference_path)["object"]["sha"] != head:
        raise ValueError(
            "The branch changed during reservation; rerun to create its next candidate"
        )
    result["pull_request"] = current
    return result


def open_pull(api, result, title, body):
    if result["pull_request"]:
        return result["pull_request"]
    if not isinstance(title, str) or not title.strip():
        raise ValueError("A title is required to open a pull request")
    head = result["branch"]
    if result["head_repository"] != result["repository"]:
        head = result["head_repository"].split("/", 1)[0] + ":" + head
    pull = api.request(
        f"/repos/{result['repository']}/pulls",
        "POST",
        {
            "head": head,
            "base": result["base_branch"],
            "title": title,
            "body": body,
        },
    )
    policy.check_pull(api, result["repository"], pull["number"], result["head"])
    return pull


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", required=True)
    parser.add_argument("--branch", required=True)
    parser.add_argument("--base", default="main")
    parser.add_argument("--head-repo")
    parser.add_argument("--candidate-only", action="store_true")
    parser.add_argument("--title")
    parser.add_argument("--body-file", type=Path)
    args = parser.parse_args()
    if not args.candidate_only and not args.title:
        parser.error(
            "Use --title to open a pull request, or --candidate-only to reserve its version"
        )
    body = args.body_file.read_text(encoding="utf-8") if args.body_file else ""
    api = policy.GitHub(os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN"))
    result = prepare(api, args.repo, args.branch, args.base, args.head_repo)
    output = {key: result[key] for key in ("version", "head", "candidate")}
    if not args.candidate_only:
        output["pull_request"] = open_pull(api, result, args.title, body)["html_url"]
    print(json.dumps(output))


if __name__ == "__main__":
    try:
        main()
    except (ValueError, KeyError, TypeError, OSError) as error:
        print(f"Versioned pull request preparation failed: {error}", file=sys.stderr)
        sys.exit(1)
