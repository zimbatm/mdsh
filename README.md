# mdsh - Markdown shell pre-processor

**STATUS: WIP**

mdsh is a markdown pre-processor that works in place. There is no need to
maintain a separate file with the template.

## Usage

`$ cargo run -- --help`
```
mdsh 0.1.0
zimbatm <zimbatm@zimbatm.com>
Markdown shell pre-processor

USAGE:
    mdsh [FLAGS] [OPTIONS] [INPUT]

FLAGS:
        --clean      Only clean the file from blocks
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -o, --output <OUTPUT>    Path to the output file, defaults to the input value

ARGS:
    <INPUT>    Path to the markdown file [default: README.md]
```

## Extensions

### Shell code

Inline `code span` that are alone on one line (only enclosed by whitespaces)
And start with one of the following character with a whitespace.

Those are interpreted. The output will depend on the
character. The output will be attached below the line.

TODO: make it clear that it's `<char><whitespace><command>`
TODO: how to select the code block syntax?

* `$` runs the command and outputs a code block
* `>` runs the command and outputs markdown

Examples:

NOTE: the block removal algorithm doesn't support output that contains the
triple backtick.

`$ date`
```
Sat Feb 16 12:45:45 CET 2019
```

NOTE: the block removal algorithm doesn't support output that contains the
end-of-include marker.

`> nix-info --markdown`
<!-- BEGIN mdsh -->
 - system: `"x86_64-linux"`
 - host os: `Linux 4.20.7, NixOS, 19.03.git.0309b923a25M (Koi)`
 - multi-user?: `yes`
 - sandbox: `yes`
 - version: `nix-env (Nix) 2.2`
 - channels(root): `""`
 - channels(zimbatm): `""`
 - nixpkgs: `/home/zimbatm/go/src/github.com/nixos/nixpkgs-zimbatm`

<!-- END mdsh -->

### Includes

Includes work similarily to code blocks but with the link syntax.

* `$` loads the file and embeds it as a code block
* `>` loads the file and embeds it as markdown

Examples:

[$ code.rb](code.rb)
```
require "pp"

pp ({ foo: 3 })
```

[> example.md](example.md)
<!-- BEGIN mdsh -->
*this is part of the example.md file*
<!-- END mdsh -->

## Known issues

The tool currently lacks in precision as it doesn't parse the Markdown file.
It means that in some cases it might misintepret some of the commands.

Most existing Markdown parsers are used to generate HTML in the end and are
thus not position-preserving. Eg: pulldown-cmark

## User Feedback

### Issues

If you have any problems with or questions about this project, please contact
use through a [GitHub issue](https://github.com/zimbatm/mdsh/issues).

### Contributing

You are invited to contribute new features, fixes or updates, large or small;
we are always thrilled to receive pull requests, and do our brest ot process
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
