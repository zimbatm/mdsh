let
  sources = import ./nix/sources.nix;
  pkgs = import sources.nixpkgs {
    config = {};
    overlays = [];
  };
in
  pkgs.buildEnv {
    name = "mdsh-env";
    paths = with pkgs; [
      cargo
      cargo-edit
      rustc
      rustfmt
    ];
  }
