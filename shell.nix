{ pkgs ? import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/nixos-22.05.tar.gz") { } }:
pkgs.mkShell {
  buildInputs = [
    pkgs.cargo
    pkgs.rustc
    pkgs.gitAndTools.git-extras
    pkgs.gitAndTools.pre-commit
    pkgs.libiconv
    pkgs.rustfmt
    pkgs.rust-analyzer
  ];

  shellHook = ''
    export PATH=$PWD/target/debug:$PATH
    export RUST_SRC_PATH="${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
  '';
}
