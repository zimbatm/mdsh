
0.9.2 / 2025-03-17
==================

  * chore: update `--help` output to fix test (#81)
  * fix: add `--version` back (#80)
  * chore: fmt

0.9.1 / 2025-03-14
==================

  * fix(README): update mdsh output
  * fix(deps): update rust crate clap to v4.5.32 (#79)
  * fix(deps): update rust crate lazy_static to v1.5.0 (#68)
  * fix(deps): update rust crate regex to v1.11.1 (#71)
  * chore(deps): update actions/checkout digest to 11bd719 (#73)
  * chore(deps): update cachix/install-nix-action action to v31 (#77)
  * cli: port from structopt to clap/derive (#75)
  * chore(deps): update cachix/install-nix-action action to v30 (#72)
  * chore(deps): update cachix/install-nix-action action to v29 (#70)
  * fix(deps): update rust crate regex to v1.10.6 (#69)
  * chore(nix): read deps from Cargo.lock directly

0.9.0 / 2024-06-17
==================

  * chore(deps): update actions/checkout digest to 692973e (#64)
  * chore(nix): fix shell invocation
  * chore(deps): cargo update
  * chore(deps): flake update
  * chore(flake): replace flake-utils with systems
  * chore(deps): update cachix/install-nix-action action to v27 (#66)
  * Merge pull request #63 from deemp/main
  * fix: mdsh derivation - don't depend on readme - update hash - bump patch version
  * chore: bump version
  * fix: improve messages about failing commands
  * fix: set RUST_SRC_PATH
  * chore(deps): update cachix/install-nix-action action to v26 (#60)

0.8.0 / 2024-02-27
==================

  * FEAT: support multiline commands (#59)
  * FEAT: add Nix package and describe usage with flakes (#59)
  * FIX: print newline after command
  * CHORE: update readme
  * CHORE: switch default branch to main
  * CHORE: update deps

0.7.0 / 2023-02-03
==================

  * FEAT: add support for multiple inputs (#33)
  * FIX: add libiconv as a dev dependency
  * FIX: avoid writing if no change
  * README: make the run reproducible
  * CHORE: fix CI on macOS
  * CHORE: fix warning
  * CHORE: Bump regex from 1.4.3 to 1.5.5 (#31)

0.6.0 / 2021-02-26
==================

  * CHANGE: handle empty lines between command and result
  * bump dependencies

0.5.0 / 2020-05-08
==================

  * NEW: add variables support (#27)

0.4.0 / 2020-01-12
==================

  * NEW: Codefence type (#26)

0.3.0 / 2019-10-19
==================

  * CHANGE: use the RHS of the link as a source.
    Eg: `$ [before.rb](after.rb)` now loads `after.rb` instead of `before.rb`

0.2.0 / 2019-10-08
==================

  * FEAT: add support for commented-out commands
  * FIX: fix line collapsing

0.1.5 / 2019-08-24
==================

  * FEAT: add pre-commit hooks
  * improve diff output for --frozen

0.1.4 / 2019-08-01
==================

  * FEAT: implement --frozen option (#13)
  * FEAT: filter out ANSI escape characters (#22)
  * FEAT: better error messages on read/write errors (#18)
  * DOC: improved documentation overall

0.1.3 / 2019-02-18
==================

  * FEAT: allow switching between outputs
  * FEAT: add support for work_dir. Fixes #5
  * README: add installation instructions
  * README: clarify the syntax
  * README: Fix typos (#3)

0.1.2 / 2019-02-17
==================

  * pin nixpkgs
  * README: improve the docs

0.1.1 / 2019-02-16
==================

  * README: add badges
  * cargo fmt
  * Cargo.toml: add metadata

0.1.0 / 2019-02-16
==================

  * add linking support
  * support stdin and stdout
  * basic implementation
