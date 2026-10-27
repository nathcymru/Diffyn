#!/usr/bin/env python3
"""Validate the version shared by source, release notes and installation guides."""

import argparse
import json
import re
import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VERSION = re.compile(r"(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\Z")


def validate(root, base_version=None):
    root = Path(root)
    manifest = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))
    version = manifest["package"]["version"]
    if not isinstance(version, str) or not VERSION.fullmatch(version):
        raise ValueError("The release version must be stable numeric major.minor.patch")
    if base_version is not None:
        if not VERSION.fullmatch(base_version):
            raise ValueError("The base version must be stable numeric major.minor.patch")
        if tuple(map(int, version.split("."))) < tuple(map(int, base_version.split("."))):
            raise ValueError("The package version cannot move backwards")

    for filename in ("Cargo.lock", "fuzz/Cargo.lock"):
        lock = tomllib.loads((root / filename).read_text(encoding="utf-8"))
        entries = [p for p in lock["package"] if p["name"] == "diffyn"]
        if len(entries) != 1 or entries[0]["version"] != version:
            raise ValueError(f"{filename} must contain the current Diffyn version exactly once")
    fuzz = tomllib.loads((root / "fuzz/Cargo.toml").read_text(encoding="utf-8"))
    if fuzz["dependencies"]["diffyn"]["version"] != "=" + version:
        raise ValueError("The fuzz workspace must require the exact Diffyn version")
    citation = (root / "CITATION.cff").read_text(encoding="utf-8")
    if re.findall(r"^version:\s*(\S+)\s*$", citation, re.MULTILINE) != [version]:
        raise ValueError("CITATION.cff must identify the current Diffyn version")

    tag = "v" + version
    notes = Path("docs/releases") / f"{tag}.md"
    content = (root / notes).read_text(encoding="utf-8")
    if not content.startswith(f"# Diffyn {tag}\n") or len(content.splitlines()) < 3:
        raise ValueError("Release notes must identify the version and describe its changes")
    for filename in ("README.md", "docs/index.html"):
        content = (root / filename).read_text(encoding="utf-8")
        tags = re.findall(r"--tag\s+(v[0-9]+\.[0-9]+\.[0-9]+)\b", content)
        if not tags or any(value != tag for value in tags):
            raise ValueError(f"{filename} must install the current version tag")
    return {"version": version, "tag": tag, "release_notes": notes.as_posix()}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument("--base-ref")
    args = parser.parse_args()
    base_version = None
    if args.base_ref:
        if not re.fullmatch(r"[A-Za-z0-9_./-]+", args.base_ref) or args.base_ref.startswith("-"):
            parser.error("Use a Git reference or full commit identifier")
        result = subprocess.run(
            ["git", "show", f"{args.base_ref}:Cargo.toml"],
            cwd=args.root,
            check=True,
            capture_output=True,
            text=True,
            timeout=30,
        )
        base_version = tomllib.loads(result.stdout)["package"]["version"]
    print(json.dumps(validate(args.root, base_version)))


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        print(f"Release metadata validation failed: {error}", file=sys.stderr)
        sys.exit(1)
