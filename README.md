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
cargo run
```

Show all options:

```bash
cargo run -- --help
```

List color sets:

```bash
cargo run -- --list
```

Custom text and color set:

```bash
cargo run -- --string "HELLO" --colorset 2077
```

Control background scroll speed (0 = off, 10 = fastest):

```bash
cargo run -- --scroll-speed 5
```

## Release build

```bash
cargo build --release
./target/release/matrix
```

## Development

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
```

## License

Unlicense (public domain). See `LICENSE`.
