# Release Checklist

This repo uses plain `git` for releases. Adjust version numbers as needed.

## 1) Update version
- `Cargo.toml`
- `CHANGELOG.md`

## 2) Build and verify
```bash
nix-shell
cargo fmt
cargo clippy -- -D warnings
cargo test
cargo build --release
```

## 3) Package artifacts
```bash
mkdir -p release
cp target/release/matrix release/matrix-vX.Y.Z-x86_64-linux
sha256sum release/matrix-vX.Y.Z-x86_64-linux > release/SHA256SUMS
```

## 4) Commit and tag
```bash
git add Cargo.toml CHANGELOG.md README.md release/SHA256SUMS release/matrix-vX.Y.Z-x86_64-linux
git commit -m "Release vX.Y.Z"
git tag -a vX.Y.Z -m "Release vX.Y.Z"
```

## 5) Push
```bash
git push
git push --tags
```

## 6) Create GitHub release (manual)
- Create a new release on GitHub with tag `vX.Y.Z`.
- Attach the artifacts in `release/`.
