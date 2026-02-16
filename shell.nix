{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  name = "hallo_welt_rust_env";

  buildInputs = [
    pkgs.rustup
    pkgs.zed
    pkgs.codex
    
  ];

  shellHook = ''
    echo "Rust-Entwicklungsumgebung mit ZED bereit."
  '';
}
