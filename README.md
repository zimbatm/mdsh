# $ mdsh # markdown shell pre-processor

[![Build Status](https://travis-ci.com/zimbatm/mdsh.svg?branch=master)](https://travis-ci.com/zimbatm/mdsh) [![crates.io](https://img.shields.io/crates/v/mdsh.svg)](https://crates.io/crates/mdsh)

the mdsh project describes a Markdown language extension that can be used to
automate some common tasks. The goal is to keep the syntax compatible while
allowing a pre-processor (`mdsh`) to be run against the file.

Quite often I find myself needing to embed a snippet of code or markdown from
a different file. But GitHub doesn't allow loading other files, even when
selecting a format that supports it (like AsciiDoc).

Another quite common use-case is to embed the output of a command as a fenced
code block or markdown content. For example the project is a CLI and the
`--help` output could be displayed in the README.md.

Both of these cases are supported by extending the existing syntax and running
`mdsh` against the file.

## Usage

`$ mdsh --help`
```
mdsh 0.1.3
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
            Path to the markdown file. `-` for stdin. [default: README.md]

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
[$ code.rb](code.rb)
```
require "pp"

pp ({ foo: 3 })
```
~~~

~~~
[> example.md](example.md)
<!-- BEGIN mdsh -->
*this is part of the example.md file*
<!-- END mdsh -->
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

## Known issues

The tool currently lacks in precision as it doesn't parse the Markdown file,
it just looks for the desired blocks by regexp. It means that in some cases it
might misintepret some of the commands. Most existing Markdown parsers are
used to generate HTML in the end and are thus not position-preserving. Eg:
pulldown-cmark

The block removal algorithm doesn't support output that contains triple
backtick or `<!-- END mdsh -->`.

## Related projects

* http://chriswarbo.net/essays/activecode/ is the closest to this project. It
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