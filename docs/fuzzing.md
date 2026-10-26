# Fuzzing the communication tools

**Fuzzing generates varied inputs to find crashes and broken invariants.**
Diffyn uses the official [cargo-fuzz](https://rust-fuzz.github.io/book/cargo-fuzz.html)
tool with libFuzzer and AddressSanitizer.

## What is checked

- **Text preservation:** formatting must retain every non-whitespace character and the complete original.
- **Checklist integrity:** extracted and deferred steps must come from the source in order, with the original retained.
- **Clarity spans:** finding offsets must identify the exact phrase on valid UTF-8 boundaries.
- **Input limits:** blank text, oversized text and invalid preferences must fail safely.
- **Request parsing:** malformed JSON, unknown fields, invalid enum values and nested data exercise typed deserialization.

Both harnesses call the same Rust library used by the MCP server. Synthetic
seeds include Welsh, emoji, quantities, conditions, completed items, Markdown,
idioms and size boundaries.

## Run a bounded session

**Use Linux or macOS with Rust installed.** The pinned nightly is for fuzzing;
the server retains its stable Rust requirement.

```sh
rustup toolchain install nightly-2026-10-25 --profile minimal
cargo +stable install cargo-fuzz --version 0.13.2 --locked
cargo +nightly-2026-10-25 fetch --manifest-path fuzz/Cargo.toml --locked
cargo +stable test --manifest-path fuzz/Cargo.toml --locked --lib
mkdir -p fuzz/corpus/tools_text fuzz/corpus/tools_json
cp fuzz/seeds/tools_text/* fuzz/corpus/tools_text/
cp fuzz/seeds/tools_json/* fuzz/corpus/tools_json/
CARGO_NET_OFFLINE=true cargo +nightly-2026-10-25 fuzz run tools_text -- -max_total_time=60 -max_len=65536 -timeout=5 -rss_limit_mb=1024 -dict=fuzz/dictionary.txt
CARGO_NET_OFFLINE=true cargo +nightly-2026-10-25 fuzz run tools_json -- -max_total_time=60 -max_len=65536 -timeout=5 -rss_limit_mb=1024 -dict=fuzz/dictionary.txt
```

- **Pull requests and pushes:** each target runs for approximately 45 seconds after compilation.
- **Weekly runs:** each target runs for approximately five minutes.
- **Bounds:** individual inputs have a five-second timeout and the fuzzer has a 1 GiB memory limit.
- **Reproducibility:** tool versions, the dated nightly and harness dependencies are pinned.
- **Updates:** review pins together, regenerate the fuzz lockfile and replay seeds before accepting an update.

The workflow fetches the committed dependency lock first, runs fuzzing offline,
and rejects lockfile changes. Harness dependencies live in a separate workspace
and are not server runtime dependencies.

## Investigate a failure

**Keep the smallest reproducible synthetic input.** A failing workflow retains
crash inputs for seven days. Local artifacts are ignored by Git.

```sh
cargo +nightly-2026-10-25 fuzz run tools_text fuzz/artifacts/tools_text/CRASH_FILE
```

Replace `CRASH_FILE` with the reported artifact name, or use `tools_json` for a
JSON failure. Confirm the cause, add a focused regression test and keep a safe
seed when it improves future coverage.

- **Do not add real conversations**, health information or credentials to corpora or artifacts.
- **Use private reporting** for vulnerabilities, following the [security policy](../.github/SECURITY.md).
- **Keep limits clear:** passing fuzz runs does not prove semantic correctness, accessibility outcomes or absence of vulnerabilities.

These targets exercise the domain tools and their request types. The existing
process tests cover MCP framing, lifecycle, malformed messages and shutdown;
the fuzz targets do not claim transport coverage.
