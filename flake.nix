{
  description = "mdsh - a markdown shell pre-processor";
  outputs = { self, nixpkgs }: {
    devShell.x86_64-linux = import ./shell.nix {
      pkgs = nixpkgs.legacyPackages.x86_64-linux;
    };
  };
}
