with import <nixpkgs> {};
buildEnv {
  name = "mdsh-env";
  paths = [
    cargo
    cargo-edit
    rustc
  ];
}
