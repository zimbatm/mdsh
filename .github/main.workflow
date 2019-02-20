workflow "build" {
  on = "push"
}

action "nix-build" {
  uses = "zimbatm/nix-action@master"
  args = "nix-shell --run ./.travis.sh"
}
