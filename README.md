# Matrix Rain

Retro-futuristic Matrix digital rain animation for the terminal. It renders a centered 3x5 text block with a border, while a colorful rain effect flows around it.

## Features
- Color palettes with a `--list` option
- Centered 3x5 text (built-in font, no external `figlet` dependency)
- Adjustable background scroll speed
- Quit with `q`, `Esc`, or `Ctrl+C`

## Usage

Build and run:

```bash
nix-shell
cargo run
```

Note: `nix-shell` provides the C toolchain (`cc`) and Rust tooling needed for dependencies to compile.

Show all options:

```bash
nix-shell
cargo run -- --help
```

List color sets:

```bash
nix-shell
cargo run -- --list
```

Custom text and color set:

```bash
nix-shell
cargo run -- --string "HELLO" --colorset 2077
```

Control background scroll speed (0 = off, 10 = fastest):

```bash
nix-shell
cargo run -- --scroll-speed 5
```

## Release build

```bash
nix-shell
cargo build --release
./target/release/matrix
```

## Development

```bash
nix-shell
cargo fmt
cargo clippy -- -D warnings
cargo test
```

## License

Unlicense (public domain). See `LICENSE`.
