#[macro_use]
extern crate lazy_static;

use std::fs::File;
use std::io::prelude::*;
use std::io::{self, ErrorKind, Write};
use std::process::{Command, Output, Stdio};

use difference::Changeset;
use mdsh::cli::{FileArg, Opt, Parent};
use regex::{Captures, Regex};
use clap::Parser;

fn run_command(command: &str, work_dir: &Parent) -> Output {
    let mut cli = Command::new("bash");
    cli.arg("-c")
        .arg(format!("set -euo pipefail && {command}"))
        .stdin(Stdio::null()) // don't read from stdin
        .current_dir(work_dir.as_path_buf())
        .output()
        .expect(
            format!(
                "fatal: failed to execute command `{:?}` in {}",
                cli,
                work_dir.as_path_buf().display()
            )
            .as_str(),
        )
}

fn die<A>(msg: String) -> A {
    std::io::stderr()
        .write_all(format!("fatal: {}\n", msg).as_bytes())
        .unwrap();
    std::process::exit(1)
}

fn read_file(f: &FileArg) -> String {
    let mut buffer = String::new();

    match f {
        FileArg::StdHandle => {
            let stdin = io::stdin();
            let mut handle = stdin.lock();
            handle
                .read_to_string(&mut buffer)
                .unwrap_or_else(|err| die(format!("failed to read from stdin: {}", err)));
        }
        FileArg::File(path_buf) => {
            File::open(path_buf)
                .and_then(|mut file| file.read_to_string(&mut buffer))
                .unwrap_or_else(|err| {
                    die(format!(
                        "failed to read from {}: {}",
                        path_buf.display(),
                        err
                    ))
                });
        }
    }

    buffer
}

fn write_file(f: &FileArg, contents: String) {
    match f {
        FileArg::StdHandle => {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            write!(handle, "{}", contents)
                .unwrap_or_else(|err| die(format!("failed to write to stdout: {}", err)));
        }
        FileArg::File(path_buf) => {
            File::create(path_buf)
                .and_then(|mut file| {
                    write!(file, "{}", contents)?;
                    file.sync_all()
                })
                .unwrap_or_else(|err| {
                    die(format!(
                        "failed to write to {}: {}",
                        path_buf.display(),
                        err
                    ))
                });
        }
    }
}

fn trail_nl<T: AsRef<str>>(s: T) -> String {
    let r = s.as_ref();
    if r.ends_with('\n') {
        r.to_string()
    } else {
        format!("{}\n", r)
    }
}

// make sure that the string starts and ends with new lines
fn wrap_nl(s: String) -> String {
    if s.starts_with('\n') {
        trail_nl(s)
    } else if s.ends_with('\n') {
        format!("\n{}", s)
    } else {
        format!("\n{}\n", s)
    }
}

// remove all ANSI escape characters
fn filter_ansi(s: String) -> String {
    RE_ANSI_FILTER.replace_all(&s, "").to_string()
}

/// Link text block include of form `[$ description](./filename)`
static RE_FENCE_LINK_STR: &str = r"\[\$ [^\]]+\]\((?P<link>[^\)]+)\)";
/// Link markdown block include of form `[> description](./filename)`
static RE_MD_LINK_STR: &str = r"\[> [^\]]+\]\((?P<link>[^\)]+)\)";
/// Command text block include of form `\`$ command\``
static RE_FENCE_COMMAND_STR: &str = r"`\$ (?P<command1>[^`]+)`";
/// Command text block include of form
/// ~~~
/// ```$
/// command
/// ```
/// ~~~
static RE_MULTILINE_FENCE_COMMAND_STR: &str =
    r"```\$( as (?P<fence_type2>\w+))?\n(?P<command2>[^`]+)\n```";
/// Command markdown block include of form `\`> command\``
static RE_MD_COMMAND_STR: &str = r"`> (?P<command1>[^`]+)`";
/// Command markdown block include of form
/// ~~~
/// ```>
/// command
/// ```
/// ~~~
static RE_MULTILINE_MD_COMMAND_STR: &str = r"```>\n(?P<command2>[^`]+)\n```";
/// Command to set a variable
static RE_VAR_COMMAND_STR: &str = r"`! (?P<key>[\w_]+)=(?P<raw_value>[^`]+)`";
/// Delimiter block for marking automatically inserted text
static RE_FENCE_BLOCK_STR: &str = r"^```.+?^```";
/// Delimiter block for marking automatically inserted markdown
static RE_MD_BLOCK_STR: &str = r"^<!-- BEGIN mdsh -->.+?^<!-- END mdsh -->";

/// HTML comment wrappers
static RE_COMMENT_BEGIN_STR: &str = r"(?:<!-- +)?";
static RE_COMMENT_END_STR: &str = r"(?: +-->)?";

/// Fenced code type specifier
static RE_FENCE_TYPE_STR: &str = r"(?: as (?P<fence_type1>\w+))?";

lazy_static! {
    /// Match a whole text block (`$` command or link and then delimiter block)
    static ref RE_MATCH_FENCE_BLOCK_STR: String = format!(
        r"(?sm)(^{}(?:({}|{}){}|{}){} *$)\n+({}|{})",
        RE_COMMENT_BEGIN_STR, RE_FENCE_COMMAND_STR, RE_FENCE_LINK_STR, RE_FENCE_TYPE_STR, RE_MULTILINE_FENCE_COMMAND_STR, RE_COMMENT_END_STR,
        RE_FENCE_BLOCK_STR, RE_MD_BLOCK_STR,
    );
    /// Match a whole markdown block (`>` command or link and then delimiter block)
    static ref RE_MATCH_MD_BLOCK_STR: String = format!(
        r"(?sm)(^{}(?:{}|{}|{}){} *$)\n+({}|{})",
        RE_COMMENT_BEGIN_STR, RE_MD_COMMAND_STR, RE_MD_LINK_STR, RE_MULTILINE_MD_COMMAND_STR, RE_COMMENT_END_STR,
        RE_MD_BLOCK_STR, RE_FENCE_BLOCK_STR,
    );

    static ref RE_MATCH_ANY_COMMAND_STR: String = format!(r"(?sm)^{}(`[^`\n]+`{}|```(\$( as (?P<fence_type2>\w+))?|>)\n[^`]+\n```){} *$", RE_COMMENT_BEGIN_STR, RE_FENCE_TYPE_STR, RE_COMMENT_END_STR);
    /// Match `RE_FENCE_COMMAND_STR`
    static ref RE_MATCH_FENCE_COMMAND_STR: String = format!(r"(?sm)^({}{}|{})$", RE_FENCE_COMMAND_STR, RE_FENCE_TYPE_STR, RE_MULTILINE_FENCE_COMMAND_STR);
    /// Match `RE_MD_COMMAND_STR`
    static ref RE_MATCH_MD_COMMAND_STR: String = format!(r"(?sm)^({}|{})$", RE_MD_COMMAND_STR, RE_MULTILINE_MD_COMMAND_STR);
    /// Match `RE_VAR_COMMAND_STR`
    static ref RE_MATCH_VAR_COMMAND_STR: String = format!(r"(?sm)^{}$", RE_VAR_COMMAND_STR);

    /// Match `RE_FENCE_LINK_STR`
    static ref RE_MATCH_FENCE_LINK_STR: String = format!(r"(?sm)^{}{}{}{} *$", RE_COMMENT_BEGIN_STR, RE_FENCE_LINK_STR, RE_FENCE_TYPE_STR, RE_COMMENT_END_STR);
    /// Match `RE_MD_LINK_STR`
    static ref RE_MATCH_MD_LINK_STR: String = format!(r"(?sm)^{}{}{} *$", RE_COMMENT_BEGIN_STR, RE_MD_LINK_STR, RE_COMMENT_END_STR);


    static ref RE_MATCH_ANY_COMMAND: Regex = Regex::new(&RE_MATCH_ANY_COMMAND_STR).unwrap();
    static ref RE_MATCH_CODE_BLOCK: Regex = Regex::new(&RE_MATCH_FENCE_BLOCK_STR).unwrap();
    static ref RE_MATCH_MD_BLOCK: Regex = Regex::new(&RE_MATCH_MD_BLOCK_STR).unwrap();
    static ref RE_MATCH_FENCE_COMMAND: Regex = Regex::new(&RE_MATCH_FENCE_COMMAND_STR).unwrap();
    static ref RE_MATCH_MD_COMMAND: Regex = Regex::new(&RE_MATCH_MD_COMMAND_STR).unwrap();
    static ref RE_MATCH_VAR_COMMAND: Regex = Regex::new(&RE_MATCH_VAR_COMMAND_STR).unwrap();
    static ref RE_MATCH_FENCE_LINK: Regex = Regex::new(&RE_MATCH_FENCE_LINK_STR).unwrap();
    static ref RE_MATCH_MD_LINK: Regex = Regex::new(&RE_MATCH_MD_LINK_STR).unwrap();

    /// ANSI characters filter
    /// https://superuser.com/questions/380772/removing-ansi-color-codes-from-text-stream
    static ref RE_ANSI_FILTER: Regex = Regex::new(r"\x1b\[[0-9;]*[mGKH]").unwrap();
}

struct FailingCommand {
    output: Output,
    command: String,
    command_char: char,
    is_multiline: bool,
}

fn main() -> std::io::Result<()> {
    let opt = Opt::parse();
    let clean = opt.clean;
    let frozen = opt.frozen;
    let inputs = opt.inputs;

    if inputs.len() == 0 {
        // Nothing to do
        return Ok(());
    } else if inputs.len() == 1 {
        let input = inputs.first().unwrap();
        let output = opt.output.unwrap_or_else(|| input.clone());
        let work_dir: Parent = opt.work_dir.map_or_else(
            || {
                input
                    .clone()
                    .parent()
                    .expect("fatal: your input file has no parent directory.")
            },
            |buf| Parent::from_parent_path_buf(buf),
        );
        process_file(&input, &output, &work_dir, clean, frozen)?;
    } else {
        if opt.output.is_some() {
            return Err(std::io::Error::new(
                ErrorKind::Other,
                "--output is not compatible with multiple inputs",
            ));
        }
        if opt.work_dir.is_some() {
            return Err(std::io::Error::new(
                ErrorKind::Other,
                "--work-dir is not compatible with multiple inputs",
            ));
        }
        for input in inputs {
            let work_dir = input
                .clone()
                .parent()
                .expect("fatal: your input file has no parent directory.");
            let output = input.clone();
            process_file(&input, &output, &work_dir, clean, frozen)?;
        }
    }

    Ok(())
}

fn process_file(
    input: &FileArg,
    output: &FileArg,
    work_dir: &Parent,
    clean: bool,
    frozen: bool,
) -> std::io::Result<()> {
    let original_contents = read_file(&input);
    let mut contents = original_contents.clone();

    eprintln!(
        "Using input={:?} output={:?} work_dir={:?} clean={:?} frozen={:?}",
        &input, output, work_dir, clean, frozen
    );

    /// Remove all outputs of blocks
    fn clean_blocks(file: &mut String, block_regex: &Regex) {
        *file = block_regex
            .replace_all(file, |caps: &Captures| {
                // the 1 group is our command,
                // the 2nd is the block, which we ignore and thus erase
                caps[1].to_string()
            })
            .into_owned()
    }

    clean_blocks(&mut contents, &RE_MATCH_CODE_BLOCK);
    clean_blocks(&mut contents, &RE_MATCH_MD_BLOCK);

    // Write the contents and return if --clean is passed
    if clean {
        write_file(&output, contents.to_string());
        return Ok(());
    }

    // Return either the captures fence type with a whitespace in front,
    // or an empty string.
    // That way if the fence type doesn't apply, nothing is being added.
    fn get_fence_type(caps: &Captures) -> String {
        if let Some(name) = &caps.name("fence_type1") {
            format!("{}", name.as_str())
        } else if let Some(name) = &caps.name("fence_type2") {
            format!("{}", name.as_str())
        } else {
            format!("")
        }
    }

    let mut failures = Vec::new();

    // Run all commands and fill their blocks.
    let fill_commands =
        |data: &mut String, command_regex: &Regex| -> Result<(), Vec<FailingCommand>> {
            *data = command_regex
                .replace_all(data, |caps: &Captures| {
                    let original_line = &caps[0];
                    let command_line = &caps[1];
                    let fence_type = get_fence_type(caps);
                    eprintln!("{}", command_line);
                    // eprintln!("command_line: {}", command_line);
                    // eprintln!("fence_type: {}", fence_type);

                    if let Some(caps) = RE_MATCH_FENCE_COMMAND.captures(command_line) {
                        let command1 = caps.name("command1").map_or("", |m| m.as_str());
                        let command: &str = if command1 != "" {
                            command1
                        } else {
                            &caps["command2"]
                        };
                        // eprintln!("command: {}", command);
                        let start_delimiter = "```";
                        let end_delimiter = "```";
                        let command_char = '$';

                        // TODO: now match on any of the known commands

                        let is_multiline = command.lines().count() > 1;
                        let result = run_command(command, &work_dir);
                        if result.status.success() {
                            let stdout = String::from_utf8_lossy(&result.stdout);
                            // remove ANSI escape sequences
                            let stdout = filter_ansi(stdout.to_string());
                            // we can leave the output block if stdout was empty
                            if stdout.trim().is_empty() {
                                format!("{}", trail_nl(&original_line))
                            } else {
                                format!(
                                    "{}\n{}{}{}{}",
                                    trail_nl(&original_line),
                                    start_delimiter,
                                    fence_type,
                                    wrap_nl(stdout.to_string()),
                                    end_delimiter
                                )
                            }
                        } else {
                            failures.push(FailingCommand {
                                output: result,
                                command: command.to_string(),
                                command_char: command_char,
                                is_multiline: is_multiline,
                            });
                            // re-insert what was there before
                            original_line.to_string()
                        }
                    } else if let Some(caps) = RE_MATCH_MD_COMMAND.captures(command_line) {
                        let command1 = caps.name("command1").map_or("", |m| m.as_str());
                        let command = if command1 != "" {
                            command1
                        } else {
                            &caps["command2"]
                        };
                        // eprintln!("command: {}", command);
                        let start_delimiter = "<!-- BEGIN mdsh -->";
                        let end_delimiter = "<!-- END mdsh -->";
                        let command_char = '>';

                        let result = run_command(command, &work_dir);
                        if result.status.success() {
                            let stdout = String::from_utf8_lossy(&result.stdout);
                            // remove ANSI escape sequences
                            let stdout = filter_ansi(stdout.to_string());
                            // we can leave the output block if STDOUT was empty
                            if stdout.trim().is_empty() {
                                format!("{}", trail_nl(&original_line))
                            } else {
                                format!(
                                    "{}\n{}{}{}{}",
                                    trail_nl(&original_line),
                                    start_delimiter,
                                    fence_type,
                                    wrap_nl(stdout.to_string()),
                                    end_delimiter
                                )
                            }
                        } else {
                            failures.push(FailingCommand {
                                output: result,
                                command: command.to_string(),
                                command_char: command_char,
                                is_multiline: false,
                            });
                            // re-insert what was there before
                            original_line.to_string()
                        }
                    } else if let Some(caps) = RE_MATCH_VAR_COMMAND.captures(command_line) {
                        let key = &caps["key"];
                        let raw_value = &caps["raw_value"];
                        // eprintln!("key: {}", key);
                        // eprintln!("raw_value: {}", raw_value);
                        let command = format!("echo {}", raw_value.trim());
                        let result = run_command(&command, &work_dir);
                        if result.status.success() {
                            let stdout = String::from_utf8_lossy(&result.stdout);
                            // remove ANSI escape sequences
                            let stdout = filter_ansi(stdout.to_string());
                            // set the environment variable
                            std::env::set_var(key, stdout.trim());
                        } else {
                            failures.push(FailingCommand {
                                output: result,
                                command: command.to_string(),
                                command_char: '!',
                                is_multiline: false,
                            });
                        };

                        // re-insert what was there before
                        original_line.to_string()
                    } else {
                        panic!("WTF, not supported")
                    }
                })
                .into_owned();
            if failures.is_empty() {
                Ok(())
            } else {
                Err(failures)
            }
        };

    fn print_failures(fs: Vec<FailingCommand>) {
        eprintln!("\nERROR: some commands failed:\n");
        for f in fs {
            let stderr = match String::from_utf8_lossy(&f.output.stderr)
                .into_owned()
                .as_str()
            {
                "" => String::from(""),
                s => String::from("\nIts stderr was:\n") + s.trim_end(),
            };
            let command_string_ = format!("{} ", f.command_char);
            let (delimiter, newline, command_string) = if f.is_multiline {
                ("```", "\n", "")
            } else {
                ("`", "", command_string_.as_str())
            };
            eprintln!(
                "{}{}{}{}{}{}\nfailed with {}.{}\n",
                delimiter,
                newline,
                command_string,
                f.command,
                newline,
                delimiter,
                f.output.status,
                stderr
            );
        }
    }

    fill_commands(&mut contents, &RE_MATCH_ANY_COMMAND).or_else(
        |failures| -> std::io::Result<()> {
            print_failures(failures);
            std::process::exit(1);
        },
    )?;

    /// Run all link includes and fill their blocks
    fn fill_includes(
        file: &mut String,
        link_regex: &Regex,
        link_char: char,
        start_delimiter: &str,
        end_delimiter: &str,
    ) {
        *file = link_regex
            .replace_all(file, |caps: &Captures| {
                let link = &caps["link"];
                let fence_type = get_fence_type(caps);

                eprintln!("[{} {}]", link_char, link);

                let result = read_file(&FileArg::from_str_unsafe(link));

                format!(
                    "{}\n{}{}{}{}",
                    trail_nl(&caps[0]),
                    start_delimiter,
                    fence_type,
                    wrap_nl(result.to_owned()),
                    end_delimiter
                )
            })
            .into_owned()
    }

    fill_includes(&mut contents, &RE_MATCH_FENCE_LINK, '$', "```", "```");
    fill_includes(
        &mut contents,
        &RE_MATCH_MD_LINK,
        '>',
        "<!-- BEGIN mdsh -->",
        "<!-- END mdsh -->",
    );

    // If there is no change, these is nothing left to do.
    if original_contents == contents {
        return Ok(());
    }

    // Let the user know where things have changed
    let changeset = Changeset::new(&original_contents, &contents, "\n");
    eprintln!("{}", changeset);

    // If there are changes and the file is frozen, abort
    if frozen {
        return Err(std::io::Error::new(
            ErrorKind::Other,
            "--frozen: output is not the same",
        ));
    }

    // Write the file
    write_file(&output, contents.to_string());

    return Ok(());
}
