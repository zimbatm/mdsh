# `mdsh` spec.

Each H4 section is converted into a test case by `build.rs` script. Each section
in `spec.clear.md` must correspond to the same section in `spec.processed.md`.
`spec.processed.md` is a version of this file after one `mdsh` pass.
`spec.processed.md` must be idempotent, i.e. any next passes result in the same content.
`mdsh --clean` pass on `spec.processed.md` must result in `spec.clear.md`.

<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->

- [Producing raw markdown](#producing-raw-markdown)
  - [Executing shell commands](#executing-shell-commands)
    - [Executing command in inline code and producing raw markdown](#executing-command-in-inline-code-and-producing-raw-markdown)
    - [Executing command in code blocks with data line and producing raw markdown](#executing-command-in-code-blocks-with-data-line-and-producing-raw-markdown)
    - [Executing command in code blocks and producing raw markdown](#executing-command-in-code-blocks-and-producing-raw-markdown)
    - [Executing command in one-line comments and producing raw markdown](#executing-command-in-one-line-comments-and-producing-raw-markdown)
    - [Executing command in multiline comments with data line and producing raw markdown](#executing-command-in-multiline-comments-with-data-line-and-producing-raw-markdown)
    - [Executing command in multiline comments and producing raw markdown](#executing-command-in-multiline-comments-and-producing-raw-markdown)
    - [Executing command in markdown link and producing raw markdown](#executing-command-in-markdown-link-and-producing-raw-markdown)
    - [Code block with transition producing raw markdown](#code-block-with-transition-producing-raw-markdown)
    - [Inline code with transition as a comment producing raw markdown](#inline-code-with-transition-as-a-comment-producing-raw-markdown)
  - [Reading files contents](#reading-files-contents)
    - [Reading file in inline code and producing raw markdown](#reading-file-in-inline-code-and-producing-raw-markdown)
    - [Reading file in code blocks and producing raw markdown](#reading-file-in-code-blocks-and-producing-raw-markdown)
    - [Reading file in one-line comments and producing raw markdown](#reading-file-in-one-line-comments-and-producing-raw-markdown)
    - [Reading file in multiline comments and producing raw markdown](#reading-file-in-multiline-comments-and-producing-raw-markdown)
    - [Reading file in markdown link and producing raw markdown](#reading-file-in-markdown-link-and-producing-raw-markdown)
  - [Using inlined values](#using-inlined-values)
- [Producing code blocks](#producing-code-blocks)
  - [Executing shell commands](#executing-shell-commands-1)
    - [Executing command in inline code and producing code block](#executing-command-in-inline-code-and-producing-code-block)
    - [Executing command in code blocks with data line and producing code block](#executing-command-in-code-blocks-with-data-line-and-producing-code-block)
    - [Executing command in code blocks and producing code block](#executing-command-in-code-blocks-and-producing-code-block)
    - [Executing command in one-line comments and producing code block](#executing-command-in-one-line-comments-and-producing-code-block)
    - [Executing command in multiline comments with data line and producing code block](#executing-command-in-multiline-comments-with-data-line-and-producing-code-block)
    - [Executing command in multiline comments and producing code block](#executing-command-in-multiline-comments-and-producing-code-block)
    - [Executing command in markdown link and producing code block](#executing-command-in-markdown-link-and-producing-code-block)
    - [Code block with transition producing code block](#code-block-with-transition-producing-code-block)
  - [Reading files contents](#reading-files-contents-1)
    - [Reading file in inline code and producing code block](#reading-file-in-inline-code-and-producing-code-block)
    - [Reading file in code blocks and producing code block](#reading-file-in-code-blocks-and-producing-code-block)
    - [Reading file in one-line comments and producing code block](#reading-file-in-one-line-comments-and-producing-code-block)
    - [Reading file in multiline comments and producing code block](#reading-file-in-multiline-comments-and-producing-code-block)
    - [Reading file in markdown link and producing code block](#reading-file-in-markdown-link-and-producing-code-block)
  - [Using inlined values](#using-inlined-values-1)
    - [Using inlined data in inline code and producing code block](#using-inlined-data-in-inline-code-and-producing-code-block)
    - [Using inlined data in code blocks and producing code block](#using-inlined-data-in-code-blocks-and-producing-code-block)
    - [Using inlined data in one-line comments and producing code block](#using-inlined-data-in-one-line-comments-and-producing-code-block)
    - [Using inlined data in multiline comments and producing code block](#using-inlined-data-in-multiline-comments-and-producing-code-block)
    - [Using inlined data in markdown link and producing code block](#using-inlined-data-in-markdown-link-and-producing-code-block)
- [Sourcing environment variables](#sourcing-environment-variables)
  - [Executing shell commands](#executing-shell-commands-2)
    - [Executing command in inline code and sourcing env variable(s)](#executing-command-in-inline-code-and-sourcing-env-variables)
    - [Executing command in code blocks with data line and sourcing env variable(s)](#executing-command-in-code-blocks-with-data-line-and-sourcing-env-variables)
    - [Executing command in code blocks and sourcing env variable(s)](#executing-command-in-code-blocks-and-sourcing-env-variables)
    - [Executing command in one-line comments and sourcing env variable(s)](#executing-command-in-one-line-comments-and-sourcing-env-variables)
    - [Executing command in multiline comments with data line and sourcing env variable(s)](#executing-command-in-multiline-comments-with-data-line-and-sourcing-env-variables)
    - [Executing command in multiline comments and sourcing env variable(s)](#executing-command-in-multiline-comments-and-sourcing-env-variables)
    - [Executing command in markdown link and sourcing env variable(s)](#executing-command-in-markdown-link-and-sourcing-env-variables)
  - [Reading files contents](#reading-files-contents-2)
    - [Reading file in inline code and sourcing env variable(s)](#reading-file-in-inline-code-and-sourcing-env-variables)
    - [Reading file in code blocks and sourcing env variable(s)](#reading-file-in-code-blocks-and-sourcing-env-variables)
    - [Reading file in one-line comments and sourcing env variable(s)](#reading-file-in-one-line-comments-and-sourcing-env-variables)
    - [Reading file in multiline comments and sourcing env variable(s)](#reading-file-in-multiline-comments-and-sourcing-env-variables)
    - [Reading file in markdown link and sourcing env variable(s)](#reading-file-in-markdown-link-and-sourcing-env-variables)
  - [Using inlined values](#using-inlined-values-2)
    - [Using inlined data in inline code and sourcing env variable(s)](#using-inlined-data-in-inline-code-and-sourcing-env-variables)
    - [Using inlined data in code blocks and sourcing env variable(s)](#using-inlined-data-in-code-blocks-and-sourcing-env-variables)
    - [Using inlined data in one-line comments and sourcing env variable(s)](#using-inlined-data-in-one-line-comments-and-sourcing-env-variables)
    - [Using inlined data in multiline comments and sourcing env variable(s)](#using-inlined-data-in-multiline-comments-and-sourcing-env-variables)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->

## Producing raw markdown

### Executing shell commands

#### Executing command in inline code and producing raw markdown

<!-- Debug data: (Code(Oneline), Markdown, Execute) -->

`> $ echo 'I am *markdown*'`

#### Executing command in code blocks with data line and producing raw markdown

<!-- Debug data: (Code(Multiline(true)), Markdown, Execute) -->

```md > $ sed 's/.*/Hi, \0/'
I am *markdown*
```

#### Executing command in code blocks and producing raw markdown

<!-- Debug data: (Code(Multiline(false)), Markdown, Execute) -->

```sh > $
echo 'I am *markdown*'
```

#### Executing command in one-line comments and producing raw markdown

<!-- Debug data: (Comment(Oneline), Markdown, Execute) -->

<!-- > $ echo 'I am *markdown*' -->

#### Executing command in multiline comments with data line and producing raw markdown

<!-- Debug data: (Comment(Multiline(true)), Markdown, Execute) -->

<!-- > $ sed 's/.*/Hi, \0/'
I am *markdown*
-->

#### Executing command in multiline comments and producing raw markdown

<!-- Debug data: (Comment(Multiline(false)), Markdown, Execute) -->

<!-- > $
echo 'I am *markdown*'
-->

#### Executing command in markdown link and producing raw markdown

<!-- Debug data: (Link, Markdown, Execute) -->

[> $ description](./samples/gen-md.sh)

#### Code block with transition producing raw markdown

```sh > $ :: which outputs:
echo 'hello world'
```

#### Inline code with transition as a comment producing raw markdown

`> $ echo 'hello world'`
<!-- :: which outputs: -->

### Reading files contents

#### Reading file in inline code and producing raw markdown

<!-- Debug data: (Code(Oneline), Markdown, Read) -->

`> < ./samples/example.md`

#### Reading file in code blocks and producing raw markdown

<!-- Debug data: (Code(Multiline(false)), Markdown, Read) -->

```filelist > <
./samples/example.md
```

#### Reading file in one-line comments and producing raw markdown

<!-- Debug data: (Comment(Oneline), Markdown, Read) -->

<!-- > < ./samples/example.md -->

#### Reading file in multiline comments and producing raw markdown

<!-- Debug data: (Comment(Multiline(false)), Markdown, Read) -->

<!-- > <
./samples/example.md
-->

#### Reading file in markdown link and producing raw markdown

<!-- Debug data: (Link, Markdown, Read) -->

[> < description](./samples/example.md)

### Using inlined values

## Producing code blocks

### Executing shell commands

#### Executing command in inline code and producing code block

<!-- Debug data: (Code(Oneline), CodeBlock, Execute) -->

`> yaml $ echo 'foo: true'`

#### Executing command in code blocks with data line and producing code block

<!-- Debug data: (Code(Multiline(true)), CodeBlock, Execute) -->

```yml > yaml $ sed 's/.*/\0 # hmm/'
foo: true
```

#### Executing command in code blocks and producing code block

<!-- Debug data: (Code(Multiline(false)), CodeBlock, Execute) -->

```sh > yaml $
echo 'foo: true'
```

#### Executing command in one-line comments and producing code block

<!-- Debug data: (Comment(Oneline), CodeBlock, Execute) -->

<!-- > yaml $ echo 'foo: true' -->

#### Executing command in multiline comments with data line and producing code block

<!-- Debug data: (Comment(Multiline(true)), CodeBlock, Execute) -->

<!-- > yaml $ sed 's/.*/\0 # hmm/'
foo: true
-->

#### Executing command in multiline comments and producing code block

<!-- Debug data: (Comment(Multiline(false)), CodeBlock, Execute) -->

<!-- > yaml $
echo 'foo: true'
-->

#### Executing command in markdown link and producing code block

<!-- Debug data: (Link, CodeBlock, Execute) -->

[> yaml $ description](./samples/gen-yml.sh)

#### Code block with transition producing code block

```sh > txt $ :: which outputs:
echo 'hello world'
```

### Reading files contents

#### Reading file in inline code and producing code block

<!-- Debug data: (Code(Oneline), CodeBlock, Read) -->

`> yaml < ./samples/example.yml`

#### Reading file in code blocks and producing code block

<!-- Debug data: (Code(Multiline(false)), CodeBlock, Read) -->

```filelist > yaml <
./samples/example.yml
```

#### Reading file in one-line comments and producing code block

<!-- Debug data: (Comment(Oneline), CodeBlock, Read) -->

<!-- > yaml < ./samples/example.yml -->

#### Reading file in multiline comments and producing code block

<!-- Debug data: (Comment(Multiline(false)), CodeBlock, Read) -->

<!-- > yaml <
./samples/example.yml
-->

#### Reading file in markdown link and producing code block

<!-- Debug data: (Link, CodeBlock, Read) -->

[> yaml < description](./samples/example.yml)

### Using inlined values

#### Using inlined data in inline code and producing code block

<!-- Debug data: (Code(Oneline), CodeBlock, Raw) -->

`> yaml foo: true`

#### Using inlined data in code blocks and producing code block

<!-- Debug data: (Code(Multiline(false)), CodeBlock, Raw) -->

```yml > yaml
foo: true
```

#### Using inlined data in one-line comments and producing code block

<!-- Debug data: (Comment(Oneline), CodeBlock, Raw) -->

<!-- > yaml foo: true -->

#### Using inlined data in multiline comments and producing code block

<!-- Debug data: (Comment(Multiline(false)), CodeBlock, Raw) -->

<!-- > yaml
foo: true
-->

#### Using inlined data in markdown link and producing code block

<!-- Debug data: (Link, CodeBlock, Raw) -->

[> yaml description](./samples/example.yml)

## Sourcing environment variables

### Executing shell commands

#### Executing command in inline code and sourcing env variable(s)

<!-- Debug data: (Code(Oneline), EnvVars, Execute) -->

`! $ echo 'foo=bar'`

``> $ echo "\`\$foo\` is $foo"``

#### Executing command in code blocks with data line and sourcing env variable(s)

<!-- Debug data: (Code(Multiline(true)), EnvVars, Execute) -->

```env ! $ sed 's/.*/\0 # hmm/'
foo=bar
```

``> $ echo "\`\$foo\` is $foo"``

#### Executing command in code blocks and sourcing env variable(s)

<!-- Debug data: (Code(Multiline(false)), EnvVars, Execute) -->

```sh ! $
echo 'foo=bar'
```

``> $ echo "\`\$foo\` is $foo"``

#### Executing command in one-line comments and sourcing env variable(s)

<!-- Debug data: (Comment(Oneline), EnvVars, Execute) -->

<!-- ! $ echo 'foo=bar' -->

``> $ echo "\`\$foo\` is $foo"``

#### Executing command in multiline comments with data line and sourcing env variable(s)

<!-- Debug data: (Comment(Multiline(true)), EnvVars, Execute) -->

<!-- ! $ sed 's/.*/\0 # hmm/'
foo=bar
-->

``> $ echo "\`\$foo\` is $foo"``

#### Executing command in multiline comments and sourcing env variable(s)

<!-- Debug data: (Comment(Multiline(false)), EnvVars, Execute) -->

<!-- ! $
echo 'foo=bar'
-->

``> $ echo "\`\$foo\` is $foo"``

#### Executing command in markdown link and sourcing env variable(s)

<!-- Debug data: (Link, EnvVars, Execute) -->

[! $ description](./samples/gen-env.sh)

``> $ echo "\`\$foo\` is $foo"``

### Reading files contents

#### Reading file in inline code and sourcing env variable(s)

<!-- Debug data: (Code(Oneline), EnvVars, Read) -->

`! < ./samples/example.env`

``> $ echo "\`\$foo\` is $foo"``

#### Reading file in code blocks and sourcing env variable(s)

<!-- Debug data: (Code(Multiline(false)), EnvVars, Read) -->

```filelist ! <
./samples/example.env
```

``> $ echo "\`\$foo\` is $foo"``

#### Reading file in one-line comments and sourcing env variable(s)

<!-- Debug data: (Comment(Oneline), EnvVars, Read) -->

<!-- ! < ./samples/example.env -->

``> $ echo "\`\$foo\` is $foo"``

#### Reading file in multiline comments and sourcing env variable(s)

<!-- Debug data: (Comment(Multiline(false)), EnvVars, Read) -->

<!-- ! <
./samples/example.env
-->

``> $ echo "\`\$foo\` is $foo"``

#### Reading file in markdown link and sourcing env variable(s)

<!-- Debug data: (Link, EnvVars, Read) -->

[! < description](./samples/example.env)

``> $ echo "\`\$foo\` is $foo"``

### Using inlined values

#### Using inlined data in inline code and sourcing env variable(s)

<!-- Debug data: (Code(Oneline), EnvVars, Raw) -->

`! foo=bar`

``> $ echo "\`\$foo\` is $foo"``

#### Using inlined data in code blocks and sourcing env variable(s)

<!-- Debug data: (Code(Multiline(false)), EnvVars, Raw) -->

```env !
foo=bar
```

``> $ echo "\`\$foo\` is $foo"``

#### Using inlined data in one-line comments and sourcing env variable(s)

<!-- Debug data: (Comment(Oneline), EnvVars, Raw) -->

<!-- ! foo=bar -->

``> $ echo "\`\$foo\` is $foo"``

#### Using inlined data in multiline comments and sourcing env variable(s)

<!-- Debug data: (Comment(Multiline(false)), EnvVars, Raw) -->

<!-- !
foo=bar
-->

``> $ echo "\`\$foo\` is $foo"``

The end!
