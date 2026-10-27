#!/usr/bin/env bash
# Exercise the Rust server, browser scripts and repository automation separately.
set -euo pipefail

root=$(git rev-parse --show-toplevel)
cd "$root"
report="$root/target/verification"
mkdir -p "$report"

export CARGO_TARGET_DIR="$root/target/coverage-rust"
cargo llvm-cov show-env --sh > "$report/coverage-env.sh"
source "$report/coverage-env.sh"
if [[ -f "$CARGO_TARGET_DIR/CACHEDIR.TAG" ]]; then
  cargo llvm-cov clean --workspace
fi
cargo test --locked
cargo build --locked
DIFFYN_BINARY="$CARGO_TARGET_DIR/debug/diffyn" python3 tests/stdio_protocol.py
cargo llvm-cov report --json --output-path "$report/rust.json" \
  --fail-under-lines 80 --fail-under-regions 80
cargo llvm-cov report --lcov --output-path "$report/rust.lcov"
cargo llvm-cov report --summary-only
python3 scripts/check-rust-coverage.py "$report/rust.json" > "$report/rust-production.json"

export COVERAGE_FILE="$report/python.coverage"
python3 -m coverage run --source=scripts -m unittest discover -s tests -p 'test_*.py'
python3 -m coverage json -o "$report/python.json"
python3 -m coverage report --show-missing --precision=2 --fail-under=80

c8="$root/scripts/docs/node_modules/.bin/c8"
coverage_options=(--all --include='docs/*.js' --include='scripts/**/*.mjs' --include='scripts/**/*.cjs' \
  --exclude='**/*.test.mjs' --reports-dir="$report/javascript" \
  --temp-directory="$report/javascript/raw" --reporter=json-summary --reporter=lcov --reporter=text)
"$c8" "${coverage_options[@]}" node --test scripts/docs/*.test.mjs
"$c8" "${coverage_options[@]}" --clean=false --check-coverage --statements=80 \
  node scripts/docs/check-docs.mjs
