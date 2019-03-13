let
  sources = import ./nix/sources.nix;
  pkgs = import sources.nixpkgs {
    config = {};
    overlays = [];
  };

  clippy = pkgs.rustPlatform.buildRustPackage rec {
    name = "clippy-${version}";
    version = "0.0.212";

    src = pkgs.fetchFromGitHub {
      owner = "rust-lang";
      repo = "rust-clippy";
      rev = "v0.0.212";
      sha256 = "0zk8lds5ry9x8ww3qvz137pzi431qv0gh4z88cyfxiwpy6wmxpm2";
    };

    cargoUpdateHook = ''
      cp ${./xxx/Cargo.lock} Cargo.lock
    '';

    cargoSha256 = "0rvck5ahg7s51fdlr2ch698cwnyc6qp84zhfgs3wkszj9r5j430k";

    buildInputs = with pkgs; [ pkg-config ];
  };
in
  pkgs.buildEnv {
    name = "mdsh-env";
    paths = with pkgs; [
      cargo
      cargo-edit
      clippy
      rustc
      rustfmt
    ];
  }
