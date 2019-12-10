let
  pkgs = import ./nix/nixpkgs.nix;
in
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
