#!/usr/bin/env nix-shell
#!nix-shell -i bash
#
# Travis CI specific build script
set -euo pipefail

## Functions ##

run() {
  echo >&2
  echo "$ $*" >&2
  "$@"
}

## Main ##

mkdir -p "${TMPDIR}"

# build mdsh
run cargo build --verbose

# run after build, pre-commit needs mdsh
run pre-commit run --all-files

# run the tests
run cargo test --verbose
