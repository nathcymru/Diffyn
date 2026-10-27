#!/usr/bin/env python3
"""Build release archives and SHA-256 manifests using the standard library."""

import hashlib
import io
import json
import os
import re
import subprocess
import sys
import tarfile
import tomllib
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TARGETS = (
    "x86_64-unknown-linux-gnu",
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "x86_64-pc-windows-msvc",
)
SKILL_NAMES = (
    "diffyn-understand-document",
    "diffyn-task-support",
    "diffyn-repair-communication",
    "diffyn-write-in-my-voice",
    "diffyn-make-decision",
    "diffyn-prepare-conversation",
)


def release_version():
    version = tomllib.loads((ROOT / "Cargo.toml").read_text())["package"]["version"]
    if not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", version):
        raise ValueError("Release packaging requires a numeric major.minor.patch version")
    if os.environ.get("RELEASE_REF_TYPE") == "tag":
        if os.environ.get("RELEASE_REF_NAME") != f"v{version}":
            raise ValueError("The release tag must match the Cargo package version")
    return version


def archive_name(version, target):
    extension = "zip" if "windows" in target else "tar.gz"
    return f"diffyn-{version}-{target}.{extension}"


def license_documents(package):
    """Read the crate's own license and notice files without following links."""
    root = Path(package["manifest_path"]).parent.resolve()
    prefixes = ("LICENSE", "LICENCE", "COPYING", "NOTICE", "COPYRIGHT")
    candidates = set()
    for entry in root.iterdir():
        if entry.name.upper().startswith(prefixes):
            if entry.is_symlink():
                raise ValueError(f"Linked license path in {package['name']}")
            if entry.is_dir():
                candidates.update(entry.rglob("*"))
            else:
                candidates.add(entry)
    declared = package.get("license_file")
    declared_path = None
    if declared:
        path = Path(declared)
        declared_path = path if path.is_absolute() else root / path
        candidates.add(declared_path)

    documents = []
    has_license = False
    for path in sorted(candidates):
        if path.is_symlink() or not path.resolve().is_relative_to(root):
            raise ValueError(f"Unsafe license path in {package['name']}")
        if path.is_dir():
            continue
        if not path.is_file():
            raise ValueError(f"Missing license file in {package['name']}")
        content = path.read_text(encoding="utf-8")
        if not content.strip():
            raise ValueError(f"Empty license file in {package['name']}")
        relative = path.relative_to(root)
        if path == declared_path or any(
            part.upper().startswith(prefixes[:3]) for part in relative.parts
        ):
            has_license = True
        documents.append((relative.as_posix(), content))
    if not has_license:
        raise ValueError(f"No packaged license text found for {package['name']}")
    return documents


def third_party_notices(target):
    """Collect notices from the locked, already-downloaded target dependency graph."""
    result = subprocess.run(
        [
            "cargo",
            "metadata",
            "--locked",
            "--offline",
            "--format-version",
            "1",
            "--filter-platform",
            target,
        ],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
        timeout=60,
    )
    metadata = json.loads(result.stdout)
    resolution = metadata.get("resolve") or {}
    root_id = resolution.get("root")
    nodes = {node["id"]: node for node in resolution.get("nodes", [])}
    if not root_id or root_id not in nodes:
        raise ValueError("Cargo did not return the package dependency graph")
    reachable = set()
    pending = [root_id]
    while pending:
        package_id = pending.pop()
        if package_id not in reachable:
            reachable.add(package_id)
            pending.extend(nodes[package_id]["dependencies"])
    packages = [p for p in metadata["packages"] if p["id"] in reachable and p["id"] != root_id]
    if len(packages) != len(reachable) - 1:
        raise ValueError("Cargo returned incomplete dependency metadata")

    sections = [
        "Diffyn third-party dependency notices\n",
        f"Target: {target}\n",
        "These documents are reproduced from packages in the locked Cargo dependency\n"
        "graph for this target, including build-time dependencies. License expressions\n"
        "are package metadata; the original license and notice texts follow.\n",
    ]
    for package in sorted(packages, key=lambda p: (p["name"], p["version"])):
        source = package.get("source") or ""
        if source != "registry+https://github.com/rust-lang/crates.io-index":
            raise ValueError(f"Unsupported dependency source for {package['name']}")
        sections.extend(
            [
                "=" * 78,
                f"Package: {package['name']} {package['version']}",
                f"License metadata: {package.get('license') or 'See declared license file'}",
                f"Source: {source}",
                f"Package URL: https://crates.io/crates/{package['name']}/{package['version']}",
            ]
        )
        if package.get("repository"):
            sections.append(f"Repository: {package['repository']}")
        for name, content in license_documents(package):
            sections.extend([f"\n--- {name} ---\n", content])
    return "\n".join(sections).encode("utf-8")


def rust_library_notices():
    """Preserve the copyright document installed by this build's rustc component."""

    def rustc(*arguments):
        return subprocess.run(
            ["rustc", *arguments],
            cwd=ROOT,
            check=True,
            capture_output=True,
            text=True,
            timeout=60,
        ).stdout.strip()

    sysroot = Path(rustc("--print", "sysroot"))
    path = sysroot / "share" / "doc" / "rust" / "COPYRIGHT-library.html"
    if path.is_symlink() or not path.is_file():
        raise ValueError("The Rust standard-library copyright document is missing")
    document = path.read_bytes()
    if not document.strip():
        raise ValueError("The Rust standard-library copyright document is empty")
    version = rustc("--version", "--verbose")
    fields = dict(line.split(": ", 1) for line in version.splitlines() if ": " in line)
    commit = fields.get("commit-hash", "")
    if not re.fullmatch(r"[0-9a-f]{40}", commit):
        raise ValueError("The Rust toolchain source commit could not be identified")
    metadata = (
        "\n" + "=" * 78 + "\n"
        f"Rust toolchain: {version.splitlines()[0]}\n"
        f"Source: https://github.com/rust-lang/rust/tree/{commit}\n"
        "The original COPYRIGHT-library.html document from this toolchain is\n"
        "included separately as RUST_LIBRARY_LICENSES.html.\n"
    ).encode("utf-8")
    return document, metadata


def package_binary(version, destination):
    target = os.environ.get("RELEASE_TARGET", "")
    if target not in TARGETS:
        raise ValueError("Unsupported release target")
    executable = "diffyn.exe" if "windows" in target else "diffyn"
    binary = ROOT / "target" / target / "release" / executable
    if not binary.is_file() or binary.is_symlink():
        raise ValueError("The release binary is missing or is a symbolic link")
    notices = third_party_notices(target)
    rust_notices, rust_metadata = rust_library_notices()
    bundled_notices = (
        ("THIRD_PARTY_NOTICES.txt", notices + rust_metadata),
        ("RUST_LIBRARY_LICENSES.html", rust_notices),
        (
            "SKILLS.md",
            (ROOT / "docs" / "skills.md")
            .read_text(encoding="utf-8")
            .replace("(../skills/", "(skills/")
            .encode("utf-8"),
        ),
    )
    archive = destination / archive_name(version, target)
    files = [(binary, executable), (ROOT / "LICENSE", "LICENSE")]
    for name in SKILL_NAMES:
        source = ROOT / "skills" / name / "SKILL.md"
        if source.is_symlink() or not source.is_file() or not source.resolve().is_relative_to(ROOT):
            raise ValueError("A bundled workflow skill is missing or unsafe")
        files.append((source, f"skills/{name}/SKILL.md"))
    if "windows" in target:
        with zipfile.ZipFile(archive, "w", compression=zipfile.ZIP_DEFLATED) as output:
            for source, name in files:
                output.write(source, arcname=name)
            for name, content in bundled_notices:
                output.writestr(name, content)
    else:
        with tarfile.open(archive, "w:gz") as output:
            for source, name in files:
                output.add(source, arcname=name, recursive=False)
            for name, content in bundled_notices:
                entry = tarfile.TarInfo(name)
                entry.size = len(content)
                entry.mode = 0o644
                output.addfile(entry, io.BytesIO(content))
    print(f"Packaged {archive.name}")


def write_checksums(version, destination):
    expected = {archive_name(version, target) for target in TARGETS}
    expected.add(f"diffyn-{version}.crate")
    actual = {path.name for path in destination.iterdir()}
    if actual != expected:
        raise ValueError("Release artifacts do not match the complete expected package set")
    lines = []
    for name in sorted(expected):
        path = destination / name
        if path.is_symlink() or not path.is_file() or path.stat().st_size == 0:
            raise ValueError("Every release artifact must be a non-empty regular file")
        with path.open("rb") as stream:
            digest = hashlib.file_digest(stream, "sha256").hexdigest()
        lines.append(f"{digest}  {name}\n")
    (destination / "SHA256SUMS").write_text("".join(lines), encoding="utf-8")
    print("Verified five release packages and wrote SHA256SUMS")


def verify_release(version, destination, release):
    """Require all six uploaded assets before making an immutable release public."""
    if not isinstance(release, dict):
        raise ValueError("The release metadata must be a JSON object")
    if release.get("tagName") != f"v{version}" or release.get("isDraft") is not True:
        raise ValueError("The release must be a draft with the expected version tag")
    expected = {archive_name(version, target) for target in TARGETS}
    expected.update((f"diffyn-{version}.crate", "SHA256SUMS"))
    assets = release.get("assets")
    if not isinstance(assets, list) or len(assets) != len(expected):
        raise ValueError("The draft release does not contain exactly six assets")
    if any(
        not isinstance(asset, dict) or not isinstance(asset.get("name"), str) for asset in assets
    ):
        raise ValueError("The draft release contains invalid asset metadata")
    if {asset.get("name") for asset in assets} != expected:
        raise ValueError("The draft release asset names do not match the package set")
    for asset in assets:
        path = destination / asset["name"]
        if path.is_symlink() or not path.is_file() or path.stat().st_size == 0:
            raise ValueError("A local release asset is missing or invalid")
        if asset.get("size") != path.stat().st_size:
            raise ValueError("An uploaded asset size does not match the local package")
    print("Verified the draft release's six uploaded asset names and sizes")


def publish_draft(version, destination):
    """Publish verified assets without moving latest behind a newer stable version."""
    repository = "nathcymru/Diffyn"
    release = json.loads(
        subprocess.check_output(
            [
                "gh",
                "release",
                "view",
                f"v{version}",
                "--repo",
                repository,
                "--json",
                "assets,isDraft,tagName",
            ],
            text=True,
        )
    )
    verify_release(version, destination, release)
    pages = json.loads(
        subprocess.check_output(
            [
                "gh",
                "api",
                "--paginate",
                "--slurp",
                f"repos/{repository}/releases?per_page=100",
            ],
            text=True,
        )
    )
    if not isinstance(pages, list) or any(not isinstance(page, list) for page in pages):
        raise ValueError("Invalid stable release history")
    current = tuple(map(int, version.split(".")))
    newer_stable = False
    for page in pages:
        for published in page:
            if not isinstance(published, dict):
                raise ValueError("Invalid published release metadata")
            if published.get("draft") or published.get("prerelease"):
                continue
            tag = published.get("tag_name", "")
            if not isinstance(tag, str) or not re.fullmatch(r"v[0-9]+\.[0-9]+\.[0-9]+", tag):
                raise ValueError("Stable release history contains a non-numeric version")
            if tuple(map(int, tag[1:].split("."))) > current:
                newer_stable = True
    subprocess.run(
        [
            "gh",
            "release",
            "edit",
            f"v{version}",
            "--repo",
            repository,
            "--draft=false",
            f"--latest={'false' if newer_stable else 'true'}",
        ],
        check=True,
    )


if __name__ == "__main__":
    try:
        mode = sys.argv[1] if len(sys.argv) == 2 else ""
        version = release_version()
        destination = ROOT / "dist"
        destination.mkdir(exist_ok=True)
        if mode == "binary":
            package_binary(version, destination)
        elif mode == "checksums":
            write_checksums(version, destination)
        elif mode == "verify-release":
            verify_release(version, destination, json.load(sys.stdin))
        elif mode == "publish-release":
            publish_draft(version, destination)
        else:
            raise ValueError("Use binary, checksums, verify-release or publish-release mode")
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        print(f"Packaging failed: {error}", file=sys.stderr)
        sys.exit(1)
