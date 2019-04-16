#!/usr/bin/env bash
#
# Travis CI specific build script
#
set -euo pipefail

# load profile
profile=$(nix-build --no-out-link profile.nix)
export PATH=$profile/bin:$PATH

# first, check formatting
# everything after -- is passed to rustfmt
cargo fmt -- --check

cargo build
cargo test
