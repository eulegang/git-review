# AGENTS.md

Guidance for coding agents working in this repository.

## Project overview

- Rust 2024 binary crate: `gr-git-review`.
- Installed binary name: `git-review` (`src/main.rs`).
- Purpose: terminal UI for reviewing Git diffs.
- Main dependencies: `git2`, `ratatui`, `crossterm`, `clap`, `eyre`, `tracing`, `tree-sitter-highlight`.

## Repository layout

- `src/main.rs` wires CLI parsing, repository discovery, config/theme/syntax loading, model loading, and TUI startup.
- `src/cli.rs` contains command-line parsing and diff-mode selection.
- `src/model/` loads and represents Git diff data.
- `src/tui/` contains application state, input handling, rendering, widgets, and themes.
- `src/syntax/` contains syntax highlighting support.
- `src/filter/` contains file filtering logic.
- `src/buf/` contains buffer/color helpers.
- `src/eventing.rs` and `src/logging.rs` provide terminal event and tracing setup.

## Build and test commands

Run these from the repository root:

- `cargo fmt` — format Rust code.
- `cargo clippy --all-targets --all-features` — lint all targets.
- `cargo test` — run the full test suite.
- `cargo run -- [ARGS]` — run the TUI locally.

Before handing off Rust code changes, prefer running at least:

```sh
cargo fmt
cargo clippy
cargo test
```

## Coding guidelines

- Keep changes focused and idiomatic Rust.
- Prefer small, testable functions over adding logic directly to rendering or event loops.
- Use `eyre::Result`/context for fallible application-level operations, consistent with existing code.
- Keep TUI state transitions explicit and covered by unit tests where practical.
- Preserve terminal cleanup behavior when editing TUI/event code.
- Avoid committing generated build output such as `target/`.

## Testing notes

- Unit tests live inline in module files under `#[cfg(test)]`.
- Add or update tests when changing CLI parsing, model loading, filtering, theme parsing, syntax highlighting, or TUI navigation/rendering behavior.
