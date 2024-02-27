{
  description = "mdsh - a markdown shell pre-processor";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = inputs: inputs.flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = inputs.nixpkgs.legacyPackages.${system};
      inherit (pkgs) lib;
      fs = lib.fileset;
    in
    {
      devShell = import ./shell.nix { inherit pkgs; };
      packages.default = pkgs.rustPlatform.buildRustPackage rec {
        pname = "mdsh";
        version = "0.8.0";

        src = fs.toSource {
          root = ./.;
          fileset =
            fs.unions [
              ./Cargo.toml
              ./Cargo.lock
              ./README.md
              ./src
            ];
        };

        cargoSha256 = "sha256-irbEkKJyLmQry/dBY92kAsfpyHx1a7XumLYWiLU99mY=";

        meta = with lib; {
          description = "Markdown shell pre-processor";
          homepage = "https://github.com/zimbatm/mdsh";
          license = with licenses; [ mit ];
          maintainers = with maintainers; [ zimbatm ];
          mainProgram = "mdsh";
        };
      };
    });
}
