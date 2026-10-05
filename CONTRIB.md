# Contributing to `mdbook-wavedrom`

This document explains how to propose a change (pull request) and, most
importantly, how to verify that the repository still behaves correctly after an
update. `mdbook-wavedrom` is a preprocessor for [mdbook][] that turns fenced
` ```wavedrom ` code blocks into `<script type="WaveDrom">` elements and installs
the companion JavaScript assets. It is a fork of [mdbook-mermaid][] and is kept
deliberately close to it (mostly a `mermaid` → `wavedrom` search/replace).

[mdbook]: https://github.com/rust-lang-nursery/mdBook
[mdbook-mermaid]: https://github.com/badboy/mdbook-mermaid

## 1. Repository layout

| Path                          | Purpose                                                        |
| ----------------------------- | ------------------------------------------------------------- |
| `src/lib.rs`                  | Core preprocessor logic (`Wavedrom`, `add_wavedrom`) + unit tests |
| `src/bin/mdbook-wavedrom.rs`  | CLI binary: `install`, `supports`, and preprocessing modes    |
| `src/bin/assets/`             | Bundled `wavedrom.min.js` and `wavedrom-init.js` (embedded at build time) |
| `tests/it/`                   | Integration tests for the `install` command + `*.toml` fixtures and expected `*.toml.output` |
| `xtask/`                      | Helper crate to download/refresh the bundled `wavedrom.min.js` |
| `.github/workflows/`          | CI (build/test) and release (`deploy.yml`) pipelines          |
| `Cargo.toml`                  | Crate metadata, dependencies, and the `[workspace]` (`.` + `xtask`) |

## 2. Prerequisites

- A Rust toolchain. 

## 3. Making a change (pull-request workflow)

1. Fork / branch from `main`:
   ```sh
   git checkout -b my-change
   ```
2. Make the smallest change that solves the problem. Keep `src/lib.rs` logically
   in sync with upstream `mdbook-mermaid` — if you change behavior, prefer
   mirroring the upstream approach so future merges stay trivial.
3. Update `CHANGELOG.md` with a short entry describing the change.
4. Run the full verification suite in section 5. **All checks must pass.**
5. Commit and open a PR against `main` with a clear description and the test
   output.

## 4. Updating the bundled WaveDrom JavaScript

The `wavedrom.min.js` asset is refreshed with the `xtask` helper, which downloads
the given release, prepends the license header, stages it, and commits:

```sh
cargo run -p xtask -- <wavedrom-version>   # e.g. 3.5.0
```

After updating the asset, re-run the full verification suite (section 5), since
the integration tests copy and check these files.

## 5. Verifying the repository after an update

Always run these from the repository root with the toolchain and proxy active.

### 5.1 Format and lint

```sh
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
```

### 5.2 Build

```sh
cargo build --workspace
```

### 5.3 Test (unit + integration)

```sh
cargo test
```

Expected result — everything green:

- **Unit tests** (`src/lib.rs`): validate the Markdown → `<script type="WaveDrom">`
  transformation, HTML escaping, CRLF handling, tables/HTML/list passthrough.
- **Integration tests** (`tests/it/`): run the real `install` command against the
  `empty` / `full` / `some` / `missing-js` `book.toml` fixtures, assert the
  rewritten `book.toml` matches the `*.toml.output` fixture, and assert that
  `wavedrom.min.js` and `wavedrom-init.js` were copied.

> The integration tests download dev-dependencies (`assert_cmd`, `tempfile`,
> `pretty_assertions`) on first run.

If you intentionally changed the generated output, update the matching fixture:
the input `tests/it/<name>.toml` and/or the expected `tests/it/<name>.toml.output`.

### 5.4 Manual end-to-end check (recommended for behavior changes)

Confirm the tool works on a real book:

```sh
# Build the binary
cargo build

# Point to it and install into a scratch book
BIN=$PWD/target/debug/mdbook-wavedrom
mkdir -p /tmp/wd-book/src
printf '[book]\ntitle = "t"\n' > /tmp/wd-book/book.toml
printf '# Hello\n\n```wavedrom\n{signal: [{name: "clk", wave: "p..."}]}\n```\n' > /tmp/wd-book/src/chapter_1.md
printf '# Summary\n\n- [Chapter 1](./chapter_1.md)\n' > /tmp/wd-book/src/SUMMARY.md

$BIN install /tmp/wd-book
```

Then check that:

- `/tmp/wd-book/book.toml` now contains the `[preprocessor.wavedrom]` section and
  `additional-js` entries for `wavedrom.min.js` / `wavedrom-init.js`.
- `wavedrom.min.js` and `wavedrom-init.js` exist in `/tmp/wd-book`.
- Building the book with `mdbook` (if available) produces HTML in which each
  diagram is rendered as `<script type="WaveDrom">…</script>`.

You can also exercise the preprocessor protocol directly:

```sh
$BIN supports html ; echo "exit=$?"   # must exit 0 (html is supported)
$BIN supports latex ; echo "exit=$?"  # must exit 1 (unsupported renderer)
```

## 6. Pre-PR checklist

- [ ] `cargo fmt --all --check` passes.
- [ ] `cargo clippy --all-targets -- -D warnings` passes.
- [ ] `cargo build --workspace` succeeds.
- [ ] `cargo test` is fully green (unit + integration).
- [ ] Crate name and asset names are consistent across `Cargo.toml`, the binary,
      CI (`deploy.yml`), `README.md`, and the test fixtures.
- [ ] `CHANGELOG.md` updated.
- [ ] Manual install/build check done for behavior changes (section 5.4).
