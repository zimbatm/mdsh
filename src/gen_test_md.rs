//! Helper script to generate `spec.clear.md`

const FILE_HEADER: &str = "# `mdsh` spec.

Each H4 section is converted into a test case by `build.rs` script. Each section
in `spec.clear.md` must correspond to the same section in `spec.processed.md`.
`spec.processed.md` is a version of this file after one `mdsh` pass.
`spec.processed.md` must be idempotent, i.e. any next passes result in the same content.
`mdsh --clean` pass on `spec.processed.md` must result in `spec.clear.md`.";

fn main() {
    println!("{FILE_HEADER}\n\n");
    for (out_cmd, out_cmd_title) in [
        (Markdown, "Producing raw markdown"),
        (CodeBlock, "Producing code blocks"),
        (EnvVars, "Sourcing environment variables"),
    ]
    .iter()
    {
        println!("## {out_cmd_title}\n");
        for (in_cmd, in_cmd_title) in [
            (Execute, "Executing shell commands"),
            (Read, "Reading files contents"),
            (Raw, "Using inlined values"),
        ]
        .iter()
        {
            println!("### {in_cmd_title}\n");
            let container_types = [Oneline, Multiline(true), Multiline(false)];
            for cnt in container_types
                .clone()
                .iter()
                .cloned()
                .map(Code)
                .chain(container_types.map(Comment))
                .chain([Link])
            {
                // Excluding few useless cases
                if in_cmd == &Raw && out_cmd == &Markdown
                    || (cnt.kind() == &Multiline(true) && matches!(in_cmd, Read | Raw))
                    || cnt == Link && (out_cmd, in_cmd) == (&EnvVars, &Raw)
                {
                    continue;
                }

                println!("#### {}\n", test_case_name(&cnt, out_cmd, in_cmd));
                println!("<!-- Debug data: {:?} -->\n", (&cnt, out_cmd, in_cmd));
                println!("{}\n", test_case(&cnt, out_cmd, in_cmd));

                if out_cmd == &EnvVars {
                    println!("``> $ echo \"\\`\\$foo\\` is $foo\"``\n");
                }
            }
        }
    }
    println!("The end!");
}

#[derive(Debug, Eq, PartialEq)]
enum Container {
    Code(ContainerType),
    Comment(ContainerType),
    Link,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ContainerType {
    Oneline,
    Multiline(bool),
}

#[derive(Debug, Eq, PartialEq)]
enum InCmd {
    Execute,
    Read,
    Raw,
}

#[derive(Debug, Eq, PartialEq)]
enum OutCmd {
    Markdown,
    CodeBlock,
    EnvVars,
}

use Container::*;
use ContainerType::*;
use InCmd::*;
use OutCmd::*;

impl Container {
    fn kind(&self) -> &ContainerType {
        match self {
            Code(k) => k,
            Comment(k) => k,
            Link => &Oneline,
        }
    }
}

fn test_case_name(cnt: &Container, out_cmd: &OutCmd, in_cmd: &InCmd) -> String {
    let doing = in_cmd_name(in_cmd);
    let resulting = out_cmd_name(out_cmd);
    match cnt {
        Code(Oneline) => format!("{doing} in inline code and {resulting}"),
        Code(Multiline(true)) => format!("{doing} in code blocks with data line and {resulting}"),
        Code(Multiline(false)) => format!("{doing} in code blocks and {resulting}"),

        Comment(Oneline) => format!("{doing} in one-line comments and {resulting}"),
        Comment(Multiline(true)) => {
            format!("{doing} in multiline comments with data line and {resulting}")
        }
        Comment(Multiline(false)) => format!("{doing} in multiline comments and {resulting}"),
        Link => format!("{doing} in markdown link and {resulting}"),
    }
}

const fn in_cmd_name(in_cmd: &InCmd) -> &'static str {
    match in_cmd {
        Execute => "Executing command",
        Read => "Reading file",
        Raw => "Using inlined data",
    }
}

const fn out_cmd_name(out_cmd: &OutCmd) -> &'static str {
    match out_cmd {
        Markdown => "producing raw markdown",
        CodeBlock => "producing code block",
        EnvVars => "sourcing env variable(s)",
    }
}

fn test_case(cnt: &Container, out_cmd: &OutCmd, in_cmd: &InCmd) -> String {
    let cmd = command(out_cmd, in_cmd);
    let data_line = mk_data_line(cnt.kind(), out_cmd, in_cmd);
    let data = if cnt.kind() == &Multiline(true) {
        data(true, in_cmd, out_cmd)
    } else {
        data(false, in_cmd, out_cmd)
    };
    match cnt {
        Code(Oneline) => format!("`{cmd} {data_line}`"),
        Code(Multiline(true)) => format!(
            "```{} {cmd} {data_line}\n{data}\n```",
            src_lang(out_cmd, in_cmd, true),
        ),
        Code(Multiline(false)) => {
            format!("```{} {cmd}\n{data}\n```", src_lang(out_cmd, in_cmd, false),)
        }

        Comment(Oneline) => format!("<!-- {cmd} {data_line} -->"),
        Comment(Multiline(true)) => format!("<!-- {cmd} {data_line}\n{data}\n-->",),
        Comment(Multiline(false)) => format!("<!-- {cmd}\n{data}\n-->"),

        Link => format!("[{cmd} description]({})", link_file(out_cmd, in_cmd)),
    }
}

const fn link_file(out_cmd: &OutCmd, in_cmd: &InCmd) -> &'static str {
    match (out_cmd, in_cmd) {
        (Markdown, Execute) => "./samples/gen-md.sh",
        (CodeBlock, Execute) => "./samples/gen-yml.sh",
        (EnvVars, Execute) => "./samples/gen-env.sh",
        (_, _) => read_data_line(out_cmd),
    }
}

const fn data(data_line: bool, in_cmd: &InCmd, out_cmd: &OutCmd) -> &'static str {
    match (data_line, in_cmd, out_cmd) {
        (true, Execute, Markdown) => "I am *markdown*",
        (true, Execute, CodeBlock) => "foo: true",
        (true, Execute, EnvVars) => "foo=bar",
        (false, Execute, _) => exec_data_line(&Oneline, out_cmd),
        (_, Read, _) => read_data_line(out_cmd),
        (_, Raw, Markdown) => "I am *markdown*",
        (_, Raw, CodeBlock) => "foo: true",
        (_, Raw, EnvVars) => "foo=bar",
    }
}

const fn exec_data_line(cnt_type: &ContainerType, out_cmd: &OutCmd) -> &'static str {
    match (cnt_type, out_cmd) {
        (Multiline(true), Markdown) => "sed 's/.*/Hi, \\0/'",
        (Multiline(true), CodeBlock | EnvVars) => "sed 's/.*/\\0 # hmm/'",
        (Oneline, CodeBlock) => "echo 'foo: true'",
        (Oneline, Markdown) => "echo 'I am *markdown*'",
        (Oneline, EnvVars) => "echo 'foo=bar'",
        (Multiline(false), _) => "",
    }
}

const fn read_data_line(out_cmd: &OutCmd) -> &'static str {
    match out_cmd {
        Markdown => "./samples/example.md",
        CodeBlock => "./samples/example.yml",
        EnvVars => "./samples/example.env",
    }
}

const fn src_lang(out_cmd: &OutCmd, in_cmd: &InCmd, data_line: bool) -> &'static str {
    match (out_cmd, data_line, in_cmd) {
        (Markdown, true, Execute) => "md",
        (CodeBlock, true, Execute) => "yml",
        (EnvVars, true, Execute) => "env",
        (_, false, Execute) => "sh",
        (_, _, Read) => "filelist",
        (Markdown, _, Raw) => "md",
        (CodeBlock, _, Raw) => "yml",
        (EnvVars, _, Raw) => "env",
    }
}

fn command(out_cmd: &OutCmd, in_cmd: &InCmd) -> String {
    [mk_out_cmd(out_cmd), mk_in_cmd(in_cmd)]
        .join(" ")
        .trim()
        .into()
}

const fn mk_in_cmd(in_cmd: &InCmd) -> &'static str {
    match in_cmd {
        Execute => "$",
        Read => "<",
        Raw => "",
    }
}

fn mk_data_line(cnt_type: &ContainerType, out_cmd: &OutCmd, in_cmd: &InCmd) -> &'static str {
    match in_cmd {
        Execute => exec_data_line(cnt_type, out_cmd),
        Read => read_data_line(out_cmd),
        Raw => data(false, &Raw, out_cmd),
    }
}

const fn mk_out_cmd(x: &OutCmd) -> &'static str {
    match x {
        Markdown => ">",
        CodeBlock => "> yaml",
        EnvVars => "!",
    }
}
