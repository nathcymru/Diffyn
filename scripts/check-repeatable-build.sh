#!/usr/bin/env bash
# Compare clean builds of the committed source in one controlled environment.
set -euo pipefail

root=$(git rev-parse --show-toplevel)
report="$root/target/verification"
mkdir -p "$report"
work=$(mktemp -d "${TMPDIR:-/tmp}/diffyn-repeatable.XXXXXXXX")
trap 'rm -rf "$work"' EXIT

git -C "$root" archive --format=tar HEAD > "$work/source.tar"
export SOURCE_DATE_EPOCH
SOURCE_DATE_EPOCH=$(git -C "$root" show -s --format=%ct HEAD)
host=$(rustc -vV | sed -n 's/^host: //p')
case "$host" in
  *-apple-darwin|*-unknown-linux-gnu) ;;
  *) printf '%s\n' 'Repeatability check currently supports macOS and Linux hosts.' >&2; exit 1 ;;
esac

for iteration in first second; do
  source_dir="$work/$iteration"
  mkdir "$source_dir"
  tar -xf "$work/source.tar" -C "$source_dir"
  (
    cd "$source_dir"
    export CARGO_TARGET_DIR="$source_dir/target"
    if [[ -n "${CARGO_ENCODED_RUSTFLAGS:-}" ]]; then
      export CARGO_ENCODED_RUSTFLAGS="${CARGO_ENCODED_RUSTFLAGS}"$'\x1f'"--remap-path-prefix=$source_dir=/workspace"
    else
      export RUSTFLAGS="${RUSTFLAGS:-} --remap-path-prefix=$source_dir=/workspace"
    fi
    cargo build --locked --release --target "$host"
  )
done

first="$work/first/target/$host/release/diffyn"
second="$work/second/target/$host/release/diffyn"
cmp "$first" "$second"
{
  printf 'Source commit: %s\n' "$(git -C "$root" rev-parse HEAD)"
  printf 'Target: %s\n' "$host"
  printf 'SOURCE_DATE_EPOCH: %s\n' "$SOURCE_DATE_EPOCH"
  rustc -vV
  printf 'Binary SHA-256: '
  shasum -a 256 "$first" | cut -d ' ' -f 1
  printf '%s\n' 'Two clean source trees and independent target directories produced identical executable bytes.'
} | tee "$report/repeatable-build.txt"
