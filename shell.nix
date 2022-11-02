{ pkgs ? import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/nixos-22.05.tar.gz") {} }:
pkgs.mkShell {
  buildInputs = [
    pkgs.gitAndTools.git-extras
    pkgs.gitAndTools.pre-commit
    pkgs.rustfmt
    pkgs.cargo
    pkgs.cargo.passthru.rustc
  ];

  shellHook = ''
    export PATH=$PWD/target/debug:$PATH
  '';
}
