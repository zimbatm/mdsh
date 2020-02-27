# `$ mdsh` - a markdown shell pre-processor

[![Build Status](https://travis-ci.com/zimbatm/mdsh.svg?branch=master)](https://travis-ci.com/zimbatm/mdsh) [![crates.io](https://img.shields.io/crates/v/mdsh.svg)](https://crates.io/crates/mdsh)

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

## Usage

`$ mdsh --help`
```
mdsh 0.4.0
zimbatm <zimbatm@zimbatm.com>
Markdown shell pre-processor. Never let your READMEs and tutorials get out of sync again.

Exits non-zero if a sub-command failed.

USAGE:
    mdsh [FLAGS] [OPTIONS]

FLAGS:
        --clean      
            Remove all generated blocks.

        --frozen     
            Fail if the output is different from the input. Useful for CI.
            
            Using `--frozen`, you can guarantee that developers update documentation when they make a change. Just add
            `mdsh --frozen` as a check to your continuous integration setup.
    -h, --help       
            Prints help information

    -V, --version    
            Prints version information


OPTIONS:
    -i, --input <input>          
            Path to the markdown file. `-` for stdin. [default: ./README.md]

    -o, --output <output>        
            Path to the output file, `-` for stdout [defaults to updating the input file in-place].

        --work_dir <work_dir>    
            Directory to execute the scripts under [defaults to the input file’s directory].

```

## Syntax Extensions

### Inline Shell Code

Syntax regexp:
```regexp
^`[$>] ([^`]+)`\s*$
```

Inline Shell Code are normal `inline code` that:

* start at the beginning of a line
* include either `$` or `>` at the beginning of their content
* contain a shell command

When those are enountered, the command is executed by `mdsh` and output as
either a fenced code block (`$`) or markdown code (`>`).

* `$` runs the command and outputs a code block
* `>` runs the command and outputs markdown

Examples:

~~~
`$ seq 4 | sort -r`
```
4
3
2
1
```
~~~

~~~
`> echo 'I *can* include markdown. <code>Hehe</code>.'`
<!-- BEGIN mdsh -->
I *can* include markdown. <code>Hehe</code>.
<!-- END mdsh -->
~~~

### Variables

Syntax regexp:
```regexp
^`! ([\w_]+)=([^`]+)`\s*$
```

Variables allow you to set new variables in the environment and reachable by
the next blocks that are being executed.

The value part is being evaluated by bash and can thus spawn sub-shells.

Examples:

`! user=bob`

Now the $user environment variable is available:

`$ echo hello $user`
```
hello bob
```

Now capitalize the user

`! USER=$(echo $user | tr '[[:lower:]]' '[[:upper:]]')`

`$ echo hello $USER`
```
hello BOB
```

### Link Includes

Syntax regexp:
```regexp
^\[[$>] ([^\]]+)]\([^\)]+\)\s*$
```

Link Includes work similarily to code blocks but with the link syntax.

* `$` loads the file and embeds it as a code block
* `>` loads the file and embeds it as markdown

Examples:

~~~
[$ code.rb](samples/code.rb) as ruby
```ruby
require "pp"

pp ({ foo: 3 })
```
~~~

~~~
[> example.md](samples/example.md)
<!-- BEGIN mdsh -->
*this is part of the example.md file*
<!-- END mdsh -->
~~~

### ANSI escapes

ANSI escape sequences are filtered from command outputs:

`$ ls --color | sort | grep -v target`
```
samples
src
ci.sh
Cargo.lock
Cargo.toml
CHANGELOG.md
_config.yml
flake.lock
flake.nix
LICENSE
README.md
shell.nix
```

### Commented-out commands

Sometimes it's useful not to render the command that is being shown. All the
commands support being hidden inside of a HTML comment like so:

~~~
<!-- `$ echo example` -->
```
example
```
~~~

### Fenced code type

If you want GitHub to highlight the outputted code fences, it's possible to
postfix the line with `as <type>`. For example:

~~~
`$ echo '{ key: "value" }'` as json
```json
{ key: "value" }
```
~~~

## Installation

The best way to install `mdsh` is with the rust tool cargo.

```bash
cargo install mdsh
```

If you are lucky enough to be a nix user:

```bash
nix-env -f https://github.com/NixOS/nixpkgs/archive/master.tar.gz -iA mdsh
```

### Pre-commit hook

This project can also be installed as a [pre-commit](https://pre-commit.com/)
hook.

Add to your project's `.pre-commit-config.yaml`:

```yaml
-   repo: https://github.com/zimbatm/mdsh.git
    rev: master
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
use through a [GitHub issue](https://github.com/zimbatm/mdsh/issues).

### Contributing

You are invited to contribute new features, fixes or updates, large or small;
we are always thrilled to receive pull requests, and do our best to process
them as fast as we can.

## License

[> LICENSE](LICENSE)
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
