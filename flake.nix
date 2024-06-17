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
      version = (lib.importTOML "${self}/Cargo.toml").package.version;

      inherit (nixpkgs) lib;
      fs = lib.fileset;
      eachSystem = f: lib.genAttrs (import systems) (system: f nixpkgs.legacyPackages.${system});
    in
    {
      devShells = eachSystem (pkgs: {
        default = pkgs.mkShell {
          buildInputs = [
            pkgs.cargo
            pkgs.gitAndTools.git-extras
            pkgs.gitAndTools.pre-commit
            pkgs.libiconv
            pkgs.rust-analyzer
            pkgs.rustc
            pkgs.rustfmt
          ];

          shellHook = ''
            export PATH=$PWD/target/debug:$PATH
            export RUST_SRC_PATH="${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          '';
        };
      });

      packages = eachSystem (pkgs: {
        default = pkgs.rustPlatform.buildRustPackage {
          pname = "mdsh";
          inherit version;

          src = fs.toSource {
            root = ./.;
            fileset = fs.unions [
              ./Cargo.toml
              ./Cargo.lock
              ./src
            ];
          };

          cargoLock.lockFile = ./Cargo.lock;

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
