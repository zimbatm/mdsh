# `$ mdsh` - a markdown shell pre-processor

[![Build Status](https://github.com/zimbatm/mdsh/actions/workflows/ci.yaml/badge.svg)](https://github.com/zimbatm/mdsh/actions/workflows/ci.yaml?branch=master) [![crates.io](https://img.shields.io/crates/v/mdsh.svg)](https://crates.io/crates/mdsh)

The mdsh project describes a Markdown language extension that can be used to
automate some common tasks in README.md files. Quite often I find myself
needing to embed a snippet of code or markdown from a different file. Or I
want to show the output of a command. In both cases this can be done manually,
but what all you had to do was run `mdsh` and have the file updated
automatically?

So the goal of this tool is first to extend the syntax of Markdown in a
natural way. Something that you might type. And if the `mdsh` tool is run, the
related blocks get updated in place. Most other tools would produce a new file
but we really want a sort of idempotent operation here.

In the end this gives a tool that is a bit akin to literate programming or
jupyer notebooks but for shell commands. It adds a bit of verbosity to the
file and in exchange it allows to automate the refresh of those outputs.

See the source code of [./spec.clear.md](./spec.clear.md) and
[./spec.processed.md](./spec.processed.md) for **everything** that `mdsh` can.

## Usage

Run `mdsh --help`

<!-- $ cargo run -- --help -->

```
Markdown shell pre-processor. Never let your READMEs and tutorials get out of sync again.

Exits non-zero if a sub-command failed.

Usage: mdsh [OPTIONS]

Options:
  -i, --inputs <INPUTS>
          Path to the markdown files. `-` for stdin

          [default: ./README.md]

  -o, --output <OUTPUT>
          Path to the output file, `-` for stdout [defaults to updating the input file in-place]

  -w, --work-dir <WORK_DIR>
          Directory to execute the scripts under [defaults to the input file’s directory]

      --frozen
          Fail if the output is different from the input. Useful for CI.

          Using `--frozen`, you can guarantee that developers update documentation when they make a change. Just add `mdsh --frozen` as a check to your continuous integration setup.

      --clean
          Remove all generated blocks

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## `mdsh` command

The mdsh "Command" consists of these parts:

```
[langname] <out_cmd> <in_cmd> [data_line]
[data]
```

`in_cmd` defines how and where to source data, it can be one of three:
- `<` — read file as is. The filepath is sourced from `data_line`, if `data` is available, it is read per line for filenames and each file is concatenated to previos one.
- `$` — command execution. If the `data_line` is available, then it is executed as shell command. If the `data` is available it is passed to the command as via stdin (Closes https://github.com/zimbatm/mdsh/issues/57). If only `data` is available but not `data_line`, then the `data` is executed as shell script.
- "empty command" aka "use data as is", concatenating `data_line` and `data`. In practice this is useful only for env variables setting

`out_cmd` defines what to do with the data from `in_cmd`, it can be one of three:
- `> lang` — produce code block with `lang` (similarly to current `as lang` statements).
- `>` — produce raw markdown output fenced by comment-tags
- `!` — expand data to shell variables

with these 3 * 3 commands you get 9 combinations, for example:

- `> < include.md` — read file and produce raw markdown
- `> py < script.py` — read script.py and produce code block with language `py`
- `> yml $ ./script.py foo $bar` — execute `script.py foo $bar` in shell and produce `yml` code block
- `>$ ./gen-md.py` — execute `gen-md.py` and produce raw markdown
- `! foo=$bar` — use `foo=$bar` as "raw data" and expand env variables that can be used in the next shell executions
- `!< .env` — read `.env` and eval shell vars
- `!$ ./gen-vars.py` — execute gen-vars and treat output as the list of shell variable assignments

So it can do quite a lot of things and the underlying model is pretty simple, and even allows to do some useless things, like `> hello` — would produce an empty code block with `hello` language.

## Containers

Commands can be put into containers, here's all of them:

### Inline code blocks

Must start from new line and end with newline.  `langname` is skipped, parsing starts right from `out_cmd`, `data` is absent.

```md
`>$ echo hi`
```

### Code blocks
````
```[langname] <out_cmd> <in_cmd> [data_line]
[data]
```
````

Source environment variables:
````md
```env !
foo=$bar
```
````

Execute script and produce yaml block (you can even put shebang at the top and use other than bash scripting languages.
````md
```sh > yaml $
echo 'foo: true'
```
````

Run `data_line` as oneline command and pass code block to it via stdin, producing raw markdown.
````md
```> $ sed 's/.*/Hi, \0/'
Bobby
```
````

### Oneline comments

Similar to inline code blocks but hidden:

```md
`<!-- >< LICENSE.md -->` — includes LICENSE.md
```

### Multiline comment blocks

Behaves similarly to code blocks, but `langname` is not needed

````md
<!-- > yml $
echo 'hi: true'
-->
````

### Links

These slightly deviate from the rest of containers:

```md
[<out_cmd> <in_cmd> whatever here is ignored](<data_line>)
```

## Transition text

You can insert text between an `mdsh` invocation and its output by appending `:: text` to the command. 
For example:

````md
```sh > sh $ :: which outputs:
echo 'hello world'
```
````

produces:

````md
<!-- BEGIN mdsh -->
which outputs:

```sh
hello world
```
<!-- END mdsh --
````

When rendered, the command code block, the transition text, and the output block support a natural reading flow.

You may also add the transition text as a comment following an `mdsh` invocation.
For example:

````md
`> $ echo hi`
<!-- :: outputs -->
````

produces:

````md
<!-- BEGIN mdsh -->
outputs

hi
<!-- END mdsh -->
````

## Installation

The best way to install `mdsh` is with the rust tool cargo.

```bash
cargo install mdsh
```

If you are lucky enough to be a nix user:

```bash
nix-env -f https://github.com/NixOS/nixpkgs/archive/master.tar.gz -iA mdsh
```

If you are a nix + flakes user:

```bash
nix profile install github:zimbatm/mdsh
```

## Running without installation

If you are a nix + flakes user:

```bash
nix run github:zimbatm/mdsh -- --help
```

### Pre-commit hook

This project can also be installed as a [pre-commit](https://pre-commit.com/)
hook.

Add to your project's `.pre-commit-config.yaml`:

```yaml
-   repo: https://github.com/zimbatm/mdsh.git
    rev: main
    hooks:
    -   id: mdsh
```

Make sure to have rust available in your environment.

Then run `pre-commit install-hooks`

## Known issues

The tool currently lacks in precision as it doesn't parse the Markdown file,
it just looks for the desired blocks by regexp. It means that in some cases it
might misintepret some of the commands. Most existing Markdown parsers are
used to generate HTML in the end and are thus not position-preserving. Eg:
pulldown-cmark

The block removal algorithm doesn't support output that contains triple
backtick or `<!-- END mdsh -->`.

## Related projects

* <http://chriswarbo.net/essays/activecode/> is the closest to this project. It
  has some interesting Pandoc filters that capture code blocks into outputs.
  The transformation is not in-place like `mdsh`.
* [Enola.dev's ExecMD](https://docs.enola.dev/use/execmd) is another similar tool.
* [Literate Programming](https://en.wikipedia.org/wiki/Literate_programming)
  is the practice of interspesing executable code into documents. There are
  many language-specific implementations out there. `mdsh` is a bit like a
  bash literate programming language.
* [Jupyter Notebooks](https://jupyter.org/) is a whole other universe of
  documentation and code. It's great but stores the notebooks as JSON files. A
  special viewer program is required to render them to HTML or text.

## User Feedback

### Issues

If you have any problems with or questions about this project, please contact
us through a [GitHub issue](https://github.com/zimbatm/mdsh/issues).

### Contributing

You are invited to contribute new features, fixes or updates, large or small;
we are always thrilled to receive pull requests, and do our best to process
them as fast as we can.

## License

[>< LICENSE](LICENSE)

<!-- BEGIN mdsh -->
MIT License

Copyright (c) 2019 zimbatm and contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
<!-- END mdsh -->