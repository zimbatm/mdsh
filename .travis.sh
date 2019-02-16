#!/usr/bin/env bash
#
# Travis CI specific build script
#
set -euo pipefail

# load profile
profile=$(nix-build --no-out-link profile.nix)
export PATH=$profile/bin:$PATH

cargo build
cargo test
