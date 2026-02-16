# Repository Guidelines

## Project Structure & Module Organization
- `src/main.rs` holds the application entry point (single-binary crate).
- `Cargo.toml` defines crate metadata and dependencies.
- `Cargo.lock` pins dependency versions for reproducible builds.
- `shell.nix` provides a Nix dev shell with Rust tooling.
- `target/` is the Cargo build output directory.

## Build, Test, and Development Commands
- `cargo run` — build and run the binary in debug mode.
- `cargo run -- --help` — show CLI flags (color sets, text, list mode).
- `cargo build --release` — produce an optimized release binary.
- `cargo test` — run unit/integration tests (none currently).
- `cargo fmt` — format Rust code using rustfmt.
- `cargo clippy -- -D warnings` — lint and fail on warnings.

## Coding Style & Naming Conventions
- Rust 2024 edition; follow standard rustfmt output.
- Indentation: 4 spaces (rustfmt default).
- Naming: `snake_case` for functions/vars, `CamelCase` for types/traits, `SCREAMING_SNAKE_CASE` for constants.
- Prefer explicit error handling with `anyhow::Result` as used in `main`.

## Testing Guidelines
- No tests are present yet; add unit tests alongside logic when introducing helpers.
- Integration tests should live in `tests/` and use `*_test.rs` filenames.
- Run the full suite with `cargo test` before submitting changes.

## Commit & Pull Request Guidelines
- No Git history is available in this repo, so there is no established commit message convention.
- Use clear, imperative commit messages (e.g., "Add palette list command").
- PRs should include: a short summary, rationale, and testing notes (commands + results).
- If changes affect terminal behavior, add a brief manual test note (e.g., "Ran `cargo run` and verified quit keys").

## Configuration & Runtime Notes
- The program renders a terminal animation; quitting is via `q` or `Ctrl+C`.
- If terminal rendering is off, resize the terminal or rerun `cargo run`.
