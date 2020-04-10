{
  description = "mdsh - a markdown shell pre-processor";
  edition = 201909;
  outputs = { self, nixpkgs }: {
    devShell.x86_64-linux = import ./shell.nix {
      pkgs = nixpkgs.legacyPackages.x86_64-linux;
    };
  };
}
