{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  name = "hallo_welt_rust_env";

  buildInputs = [
    pkgs.rustup
    pkgs.rustfmt
    pkgs.zed
    pkgs.codex
    pkgs.gcc
    pkgs.stdenv.cc
    
  ];

  shellHook = ''
    echo "Rust-Entwicklungsumgebung mit ZED bereit."
  '';
}
