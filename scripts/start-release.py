#!/usr/bin/env python3
"""Create one annotated release tag for the validated current main commit."""

import json
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def output(*arguments):
    return subprocess.check_output(arguments, cwd=ROOT, text=True).strip()


def main():
    commit = os.environ["RELEASE_COMMIT"]
    version = os.environ["EXPECTED_VERSION"]
    if not re.fullmatch(r"[0-9a-f]{40}", commit) or not re.fullmatch(
        r"[0-9]+\.[0-9]+\.[0-9]+", version
    ):
        raise ValueError("Release dispatch requires a full commit ID and numeric stable version")
    if os.environ.get("GITHUB_REPOSITORY") != "nathcymru/Diffyn":
        raise ValueError("Release tags can only be created in the source repository")
    if os.environ.get("GITHUB_REF") != "refs/heads/main":
        raise ValueError("Automatic version publication must run from main")
    if (
        os.environ.get("GITHUB_SHA") != commit
        or output("git", "rev-parse", "HEAD") != commit
        or output("git", "rev-parse", "origin/main") != commit
    ):
        raise ValueError("The release commit must still be the current main commit")
    checked = json.loads(output(sys.executable, str(ROOT / "scripts/check-release-version.py")))
    if checked["version"] != version or checked["tag"] != f"v{version}":
        raise ValueError("The requested version differs from the validated source")
    tag = checked["tag"]
    existing = subprocess.run(
        ["git", "rev-parse", "--verify", f"refs/tags/{tag}^{{commit}}"],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    if existing.returncode == 0:
        if existing.stdout.strip() != commit:
            raise ValueError("The version tag already belongs to another commit")
        print(f"Retained existing {tag} at the validated commit")
        return
    repository = os.environ["GITHUB_REPOSITORY"]
    tag_object = json.loads(
        subprocess.check_output(
            ["gh", "api", f"repos/{repository}/git/tags", "--input", "-"],
            input=json.dumps(
                {
                    "tag": tag,
                    "message": f"Diffyn {tag}",
                    "object": commit,
                    "type": "commit",
                }
            ),
            text=True,
        )
    )
    subprocess.run(
        ["gh", "api", f"repos/{repository}/git/refs", "--input", "-"],
        input=json.dumps({"ref": f"refs/tags/{tag}", "sha": tag_object["sha"]}),
        text=True,
        stdout=subprocess.DEVNULL,
        check=True,
    )
    print(f"Created {tag} at {commit}")


if __name__ == "__main__":
    try:
        main()
    except (ValueError, KeyError, OSError, subprocess.CalledProcessError) as error:
        print(f"Release preparation failed: {error}", file=sys.stderr)
        sys.exit(1)
