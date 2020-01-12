
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
