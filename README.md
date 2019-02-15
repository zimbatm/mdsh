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
    mdsh [FLAGS] [INPUT]

FLAGS:
        --clean      Only clean the file from blocks
    -h, --help       Prints help information
    -V, --version    Prints version information

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
Fri Feb 15 17:57:50 CET 2019
```

NOTE: the block removal algorithm doesn't support output that contains the
end-of-include marker.

`> nix-info --markdown`

 - system: `"x86_64-linux"`
 - host os: `Linux 4.20.7, NixOS, 19.03.git.0309b923a25M (Koi)`
 - multi-user?: `yes`
 - sandbox: `yes`
 - version: `nix-env (Nix) 2.2`
 - channels(root): `""`
 - channels(zimbatm): `""`
 - nixpkgs: `/home/zimbatm/go/src/github.com/nixos/nixpkgs-zimbatm`

<!-- > nix-info --markdown -->

### Includes

Includes work similarily to code blocks but with the link syntax.

* `$` loads the file and embeds it as a code block
* `>` loads the file and embeds it as markdown

Examples:

[$ code.rb](code.rb)

[> example.md](example.md)

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

[$ LICENSE.md](LICENSE.md)
