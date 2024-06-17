{
  description = "mdsh - a markdown shell pre-processor";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    systems.url = "github:nix-systems/default";
  };
  outputs =
    {
      self,
      nixpkgs,
      systems,
    }:
    let
      inherit (nixpkgs) lib;
      fs = lib.fileset;
      eachSystem = f: lib.genAttrs (import systems) (system: f nixpkgs.legacyPackages.${system});
    in
    {

      packages = eachSystem (pkgs: {
        default = pkgs.rustPlatform.buildRustPackage rec {
          pname = "mdsh";
          version = "0.8.1";

          src = fs.toSource {
            root = ./.;
            fileset = fs.unions [
              ./Cargo.toml
              ./Cargo.lock
              ./src
            ];
          };

          cargoSha256 = "sha256-Barf/CRt5LYtIxUigBZNwiJwVmmEjCKm2lbp+ww2sBs=";

          meta = with lib; {
            description = "Markdown shell pre-processor";
            homepage = "https://github.com/zimbatm/mdsh";
            license = with licenses; [ mit ];
            maintainers = with maintainers; [ zimbatm ];
            mainProgram = "mdsh";
          };
        };
      });
    };
}
