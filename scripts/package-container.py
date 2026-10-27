#!/usr/bin/env python3
"""Prepare and publish the tested Linux release as a local stdio container."""

import argparse
import base64
import hashlib
import json
import os
import re
import subprocess
import sys
import tarfile
import tomllib
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MAX_ARCHIVE_BYTES = 256 * 1024 * 1024
SKILLS = (
    "diffyn-understand-document",
    "diffyn-task-support",
    "diffyn-repair-communication",
    "diffyn-write-in-my-voice",
    "diffyn-make-decision",
    "diffyn-prepare-conversation",
)
CONTENTS = {
    "diffyn",
    "LICENSE",
    "THIRD_PARTY_NOTICES.txt",
    "RUST_LIBRARY_LICENSES.html",
    "SKILLS.md",
    *(f"skills/{name}/SKILL.md" for name in SKILLS),
}
MANIFEST_ACCEPT = ", ".join(
    (
        "application/vnd.oci.image.manifest.v1+json",
        "application/vnd.docker.distribution.manifest.v2+json",
    )
)


def command(*arguments):
    return subprocess.check_output(arguments, cwd=ROOT, text=True).strip()


def validate_metadata(value):
    if not isinstance(value, dict) or set(value) != {
        "version",
        "revision",
        "packaging_revision",
        "archive",
        "tag",
        "repository",
    }:
        raise ValueError("Invalid container source metadata")
    if any(not isinstance(item, str) for item in value.values()):
        raise ValueError("Container source metadata fields must be strings")
    if not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", value["version"]):
        raise ValueError("Invalid container version")
    if not re.fullmatch(r"[0-9a-f]{40}", value["revision"]):
        raise ValueError("Invalid source commit")
    if not re.fullmatch(r"[0-9a-f]{40}", value["packaging_revision"]):
        raise ValueError("Invalid container packaging commit")
    if value["tag"] != f"v{value['version']}":
        raise ValueError("The source tag must match the package version")
    if value["archive"] != f"diffyn-{value['version']}-x86_64-unknown-linux-gnu.tar.gz":
        raise ValueError("Invalid Linux package name")
    if value["repository"] != "nathcymru/Diffyn":
        raise ValueError("Unexpected source repository")
    return value


def metadata(source_tag):
    if source_tag:
        if not re.fullmatch(r"v[0-9]+\.[0-9]+\.[0-9]+", source_tag):
            raise ValueError("Use a numeric vMAJOR.MINOR.PATCH release tag")
        revision = command("git", "rev-parse", "--verify", f"refs/tags/{source_tag}^{{commit}}")
        subprocess.run(
            ["git", "merge-base", "--is-ancestor", revision, "origin/main"],
            cwd=ROOT,
            check=True,
        )
        manifest = command("git", "show", f"{revision}:Cargo.toml")
    else:
        revision = command("git", "rev-parse", "HEAD")
        manifest = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    version = tomllib.loads(manifest)["package"]["version"]
    return validate_metadata(
        {
            "version": version,
            "revision": revision,
            "packaging_revision": command("git", "rev-parse", "HEAD"),
            "archive": f"diffyn-{version}-x86_64-unknown-linux-gnu.tar.gz",
            "tag": source_tag or f"v{version}",
            "repository": "nathcymru/Diffyn",
        }
    )


def verify_release(source, release, directory):
    """Verify an immutable release and the selected archive before extraction."""
    validate_metadata(source)
    if not isinstance(release, dict) or release.get("tagName") != source["tag"]:
        raise ValueError("The published release does not match the requested tag")
    if release.get("isDraft") is not False or release.get("isImmutable") is not True:
        raise ValueError("Container publication requires a published immutable release")
    if release.get("isPrerelease") is not False or not release.get("publishedAt"):
        raise ValueError("Container publication requires a stable published release")
    version = source["version"]
    expected = {
        source["archive"],
        f"diffyn-{version}.crate",
        "SHA256SUMS",
        f"diffyn-{version}-aarch64-apple-darwin.tar.gz",
        f"diffyn-{version}-x86_64-apple-darwin.tar.gz",
        f"diffyn-{version}-x86_64-pc-windows-msvc.zip",
    }
    assets = release.get("assets", [])
    if not isinstance(assets, list) or any(not isinstance(item, dict) for item in assets):
        raise ValueError("Invalid published asset metadata")
    if len(assets) != len(expected) or {item.get("name") for item in assets} != expected:
        raise ValueError("The release does not contain the complete package set")
    sizes = {item["name"]: item.get("size") for item in assets}
    for name in (source["archive"], "SHA256SUMS"):
        path = directory / name
        if path.is_symlink() or not path.is_file() or path.stat().st_size != sizes[name]:
            raise ValueError("A downloaded package does not match the release metadata")
    checksums = {}
    for line in (directory / "SHA256SUMS").read_text(encoding="utf-8").splitlines():
        match = re.fullmatch(r"([0-9a-f]{64})  ([A-Za-z0-9_.-]+)", line)
        if match is None or match[2] in checksums:
            raise ValueError("Invalid or duplicate package checksum")
        checksums[match[2]] = match[1]
    if set(checksums) != expected - {"SHA256SUMS"}:
        raise ValueError("The checksum manifest does not cover the package set")
    archive = directory / source["archive"]
    if archive.stat().st_size > MAX_ARCHIVE_BYTES:
        raise ValueError("The Linux package exceeds its size limit")
    with archive.open("rb") as stream:
        digest = hashlib.file_digest(stream, "sha256").hexdigest()
    if digest != checksums[source["archive"]]:
        raise ValueError("The Linux package checksum does not match")


def extract(archive, destination):
    """Copy only the exact release files; never extract links or archive paths."""
    if archive.is_symlink() or not archive.is_file() or archive.stat().st_size > MAX_ARCHIVE_BYTES:
        raise ValueError("The Linux package is missing or unsafe")
    if destination.exists():
        raise ValueError("The container context must be a new directory")
    with tarfile.open(archive, "r:gz") as package:
        members = package.getmembers()
        if len(members) != len(CONTENTS) or {item.name for item in members} != CONTENTS:
            raise ValueError("The Linux package contains unexpected or missing files")
        if any(not item.isfile() or item.size <= 0 for item in members):
            raise ValueError("Every packaged entry must be a non-empty regular file")
        if sum(item.size for item in members) > MAX_ARCHIVE_BYTES:
            raise ValueError("The extracted package exceeds its size limit")
        destination.mkdir(parents=True)
        for member in members:
            path = destination / member.name
            path.parent.mkdir(parents=True, exist_ok=True)
            with package.extractfile(member) as source, path.open("xb") as output:
                while block := source.read(1024 * 1024):
                    output.write(block)
            path.chmod(0o755 if member.name == "diffyn" else 0o644)
            os.utime(path, (0, 0))
        # Extraction time must not change a published version's image digest.
        for path in sorted(destination.rglob("*"), reverse=True):
            if path.is_dir():
                os.utime(path, (0, 0))
        os.utime(destination, (0, 0))


def image_config(image):
    return json.loads(command("docker", "image", "inspect", image))[0]


def verify_image(image, source):
    validate_metadata(source)
    inspected = image_config(image)
    configuration = inspected["Config"]
    if inspected.get("Os") != "linux" or inspected.get("Architecture") != "amd64":
        raise ValueError("The container must target Linux amd64")
    if configuration.get("User") != "65532:65532":
        raise ValueError("The container must run as its unprivileged user")
    if configuration.get("Entrypoint") != ["/opt/diffyn/diffyn"]:
        raise ValueError("The container must start the stdio server directly")
    if configuration.get("Cmd"):
        raise ValueError("The container must not append a shell or command arguments")
    if configuration.get("ExposedPorts") or configuration.get("Volumes"):
        raise ValueError("The stdio container must not expose ports or declare volumes")
    labels = configuration.get("Labels", {})
    if labels.get("io.diffyn.packaging.revision") != source["packaging_revision"]:
        raise ValueError("Container packaging metadata does not match the workflow source")
    for name, expected in {
        "source": "https://github.com/nathcymru/Diffyn",
        "version": source["version"],
        "revision": source["revision"],
        "licenses": "MIT",
    }.items():
        if labels.get(f"org.opencontainers.image.{name}") != expected:
            raise ValueError("Container metadata does not match its release source")
    # Recent containerd-backed Docker stores expose the manifest as Id, while
    # classic stores expose the config digest. Compare config content on both.
    identifier = (
        inspected.get("Descriptor", {})
        .get("annotations", {})
        .get(
            "config.digest",
            inspected.get("Id", ""),
        )
    )
    if not re.fullmatch(r"sha256:[0-9a-f]{64}", identifier):
        raise ValueError("Invalid local image identifier")
    return identifier


def registry_token(repository):
    credentials = f"{os.environ['GITHUB_ACTOR']}:{os.environ['GH_TOKEN']}".encode()
    query = urllib.parse.urlencode(
        {
            "service": "ghcr.io",
            "scope": f"repository:{repository}:pull,push",
        }
    )
    request = urllib.request.Request(
        f"https://ghcr.io/token?{query}",
        headers={"Authorization": "Basic " + base64.b64encode(credentials).decode()},
    )
    with urllib.request.urlopen(request, timeout=60) as response:
        result = json.load(response)
    return result["token"]


def registry_manifest(repository, version, token):
    request = urllib.request.Request(
        f"https://ghcr.io/v2/{repository}/manifests/{version}",
        headers={"Authorization": f"Bearer {token}", "Accept": MANIFEST_ACCEPT},
    )
    try:
        with urllib.request.urlopen(request, timeout=60) as response:
            content = response.read(MAX_ARCHIVE_BYTES + 1)
            if len(content) > MAX_ARCHIVE_BYTES:
                raise ValueError("The registry manifest exceeds its size limit")
            digest = response.headers.get("Docker-Content-Digest", "")
            if digest != "sha256:" + hashlib.sha256(content).hexdigest():
                raise ValueError("The registry manifest digest is inconsistent")
            return digest, json.loads(content)
    except urllib.error.HTTPError as error:
        if error.code == 404:
            error.close()
            return None
        raise


def publish(image, source):
    """Never overwrite a version tag with different image content."""
    local_config = verify_image(image, source)
    repository = source["repository"].lower()
    version = source["version"]
    reference = f"ghcr.io/{repository}:{version}"
    token = registry_token(repository)
    existing = registry_manifest(repository, version, token)
    if existing is not None:
        if existing[1].get("config", {}).get("digest") != local_config:
            raise ValueError(
                "This container version already exists with different content; publish a new version"
            )
        return existing[0]
    subprocess.run(["docker", "tag", image, reference], check=True)
    subprocess.run(["docker", "push", reference], check=True)
    published = registry_manifest(repository, version, token)
    if published is None or published[1].get("config", {}).get("digest") != local_config:
        raise ValueError("The published image does not match the tested container")
    return published[0]


def promote_latest(source, digest):
    """Move the convenience alias only when this is GitHub's latest stable release."""
    validate_metadata(source)
    if not re.fullmatch(r"sha256:[0-9a-f]{64}", digest):
        raise ValueError("Invalid published image digest")
    release = json.loads(
        command(
            "gh",
            "release",
            "view",
            "--repo",
            source["repository"],
            "--json",
            "tagName,isDraft,isImmutable,isPrerelease",
        )
    )
    if release.get("tagName") != source["tag"]:
        print("Retained latest: a different release is the current stable version")
        return
    if (
        release.get("isDraft") is not False
        or release.get("isImmutable") is not True
        or release.get("isPrerelease") is not False
    ):
        raise ValueError("The latest alias requires a published immutable stable release")
    repository = source["repository"].lower()
    source_image = f"ghcr.io/{repository}@{digest}"
    # Preserve the tested single-platform manifest instead of creating a new index.
    subprocess.run(
        [
            "docker",
            "buildx",
            "imagetools",
            "create",
            "--prefer-index=false",
            "--tag",
            f"ghcr.io/{repository}:latest",
            source_image,
        ],
        check=True,
    )
    latest = registry_manifest(repository, "latest", registry_token(repository))
    if latest is None or latest[0] != digest:
        raise ValueError("The latest alias does not match the verified version digest")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "mode",
        choices=(
            "metadata",
            "verify-release",
            "extract",
            "verify-image",
            "publish",
            "promote-latest",
        ),
    )
    parser.add_argument("--source-tag", default="")
    parser.add_argument("--metadata", type=Path, required=True)
    parser.add_argument("--release", type=Path)
    parser.add_argument("--directory", type=Path, default=ROOT / "dist")
    parser.add_argument("--context", type=Path)
    parser.add_argument("--image", default="diffyn-container:candidate")
    parser.add_argument("--digest")
    arguments = parser.parse_args()
    if arguments.mode == "metadata":
        source = metadata(arguments.source_tag)
        arguments.metadata.write_text(json.dumps(source, indent=2) + "\n", encoding="utf-8")
        if os.environ.get("GITHUB_OUTPUT"):
            with open(os.environ["GITHUB_OUTPUT"], "a", encoding="utf-8") as output:
                for key, value in source.items():
                    output.write(f"{key}={value}\n")
        return
    source = validate_metadata(json.loads(arguments.metadata.read_text(encoding="utf-8")))
    if arguments.mode == "verify-release":
        if arguments.release is None:
            raise ValueError("Release metadata is required")
        verify_release(
            source, json.loads(arguments.release.read_text(encoding="utf-8")), arguments.directory
        )
    elif arguments.mode == "extract":
        if arguments.context is None:
            raise ValueError("A container context path is required")
        extract(arguments.directory / source["archive"], arguments.context)
    elif arguments.mode == "verify-image":
        verify_image(arguments.image, source)
    elif arguments.mode == "promote-latest":
        promote_latest(source, arguments.digest or "")
    else:
        digest = publish(arguments.image, source)
        if os.environ.get("GITHUB_OUTPUT"):
            with open(os.environ["GITHUB_OUTPUT"], "a", encoding="utf-8") as output:
                output.write(f"digest={digest}\n")
        print(f"Verified ghcr.io/{source['repository'].lower()}@{digest}")


if __name__ == "__main__":
    try:
        main()
    except (
        ValueError,
        KeyError,
        TypeError,
        OSError,
        subprocess.CalledProcessError,
        tarfile.TarError,
    ) as error:
        print(f"Container packaging failed: {error}", file=sys.stderr)
        sys.exit(1)
