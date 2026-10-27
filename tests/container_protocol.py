#!/usr/bin/env python3
"""Run the complete stdio protocol suite against an isolated local container."""

import io
import os
import re
import shlex
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
IMAGE = os.environ.get("DIFFYN_IMAGE", "diffyn-container:candidate")


def protocol_source(destination, revision):
    """Use the versioned protocol suite and fixtures shipped with its binary."""
    if not revision:
        return ROOT
    if not re.fullmatch(r"[0-9a-f]{40}", revision):
        raise ValueError("The protocol source must be a full commit identifier")
    current = subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True, timeout=30
    ).strip()
    if revision != current:
        subprocess.run(
            ["git", "merge-base", "--is-ancestor", revision, "origin/main"],
            cwd=ROOT,
            check=True,
            timeout=30,
        )
    archive = subprocess.check_output(
        [
            "git",
            "archive",
            "--format=tar",
            revision,
            "Cargo.toml",
            "tests/stdio_protocol.py",
            "skills",
        ],
        cwd=ROOT,
        timeout=30,
    )
    required = {"Cargo.toml", "tests/stdio_protocol.py"}
    exported = set()
    with tarfile.open(fileobj=io.BytesIO(archive), mode="r:") as source:
        destination.mkdir()
        for entry in source:
            path = Path(entry.name)
            if entry.isdir():
                continue
            if (
                not entry.isfile()
                or path.is_absolute()
                or ".." in path.parts
                or (entry.name not in required and not entry.name.startswith("skills/"))
            ):
                raise ValueError("The protocol snapshot contains an unexpected file")
            target = destination / path
            target.parent.mkdir(parents=True, exist_ok=True)
            with source.extractfile(entry) as content, target.open("xb") as output:
                output.write(content.read())
            exported.add(entry.name)
    if not required.issubset(exported):
        raise ValueError("The protocol snapshot is missing its test or version manifest")
    return destination


def main():
    # The wrapper preserves the protocol suite's subprocess and signal checks.
    # No terminal allocation: Docker must relay the JSON-RPC byte streams intact.
    with tempfile.TemporaryDirectory(prefix="diffyn-container-tests-") as directory:
        source = protocol_source(
            Path(directory) / "source", os.environ.get("DIFFYN_SOURCE_REVISION", "")
        )
        wrapper = Path(directory) / "diffyn-container"
        arguments = [
            "docker",
            "run",
            "--rm",
            "--interactive",
            "--pull=never",
            "--platform=linux/amd64",
            "--network=none",
            "--read-only",
            "--cap-drop=ALL",
            "--security-opt=no-new-privileges",
            IMAGE,
        ]
        wrapper.write_text("#!/bin/sh\nexec " + shlex.join(arguments) + "\n", encoding="utf-8")
        wrapper.chmod(0o755)
        environment = os.environ | {"DIFFYN_BINARY": str(wrapper)}
        subprocess.run(
            [sys.executable, "tests/stdio_protocol.py"],
            cwd=source,
            env=environment,
            check=True,
            timeout=300,
        )


if __name__ == "__main__":
    main()
