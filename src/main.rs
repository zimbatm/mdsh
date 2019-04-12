extern crate diff;
extern crate mdsh;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate structopt;

use mdsh::cli::{FileArg, Opt, Parent};
use regex::{Captures, Regex};
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, ErrorKind, Write};
use std::process::{Command, Output, Stdio};
use structopt::StructOpt;

fn run_command(command: &str, work_dir: &Parent) -> Output {
    let mut cli = Command::new("bash");
    cli.arg("-c")
        .arg(command)
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
    std::io::stderr().write_all(format!("fatal: {}\n", msg).as_bytes()).unwrap();
    std::process::exit(1)
}

fn read_file(f: &FileArg) -> String {
    let mut buffer = String::new();

    match f {
        FileArg::StdHandle => {
            let stdin = io::stdin();
            let mut handle = stdin.lock();
            handle.read_to_string(&mut buffer)
                .unwrap_or_else(|err| die(format!("failed to read from stdin: {}", err)));
        }
        FileArg::File(path_buf) => {
            File::open(path_buf)
                .and_then(|mut file| file.read_to_string(&mut buffer))
                .unwrap_or_else(|err| die(format!("failed to read from {}: {}", path_buf.display(), err)));
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
                .unwrap_or_else(|err| die(format!("failed to write to {}: {}", path_buf.display(), err)));
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

/// Link text block include of form `[$ description](./filename)`
static RE_FENCE_LINK_STR: &str = r"^\[\$ (?P<link>[^\]]+)\]\([^\)]+\)\s*$";
/// Link markdown block include of form `[> description](./filename)`
static RE_MD_LINK_STR: &str = r"^\[> (?P<link>[^\]]+)\]\([^\)]+\)\s*$";
/// Command text block include of form `\`$ command\``
static RE_FENCE_COMMAND_STR: &str = r"^`\$ (?P<command>[^`]+)`\s*$";
/// Command markdown block include of form `\`> command\``
static RE_MD_COMMAND_STR: &str = r"^`> (?P<command>[^`]+)`\s*$";
/// Delimiter block for marking automatically inserted text
static RE_FENCE_BLOCK_STR: &str = r"^```.+?^```";
/// Delimiter block for marking automatically inserted markdown
static RE_MD_BLOCK_STR: &str = r"^<!-- BEGIN mdsh -->.+?^<!-- END mdsh -->";

lazy_static! {
    /// Match a whole text block (`$` command or link and then delimiter block)
    static ref RE_MATCH_FENCE_BLOCK_STR: String = format!(
        r"(?sm)({}|{})[\s\n]+({}|{})",
        RE_FENCE_COMMAND_STR, RE_FENCE_LINK_STR, RE_FENCE_BLOCK_STR, RE_MD_BLOCK_STR,
    );
    /// Match a whole markdown block (`>` command or link and then delimiter block)
    static ref RE_MATCH_MD_BLOCK_STR: String = format!(
        r"(?sm)({}|{})[\s\n]+({}|{})",
        RE_MD_COMMAND_STR, RE_MD_LINK_STR, RE_MD_BLOCK_STR, RE_FENCE_BLOCK_STR,
    );

    /// Match `RE_FENCE_COMMAND_STR`
    static ref RE_MATCH_FENCE_COMMAND_STR: String = format!(r"(?sm){}", RE_FENCE_COMMAND_STR);
    /// Match `RE_MD_COMMAND_STR`
    static ref RE_MATCH_MD_COMMAND_STR: String = format!(r"(?sm){}", RE_MD_COMMAND_STR);
    /// Match `RE_FENCE_LINK_STR`
    static ref RE_MATCH_FENCE_LINK_STR: String = format!(r"(?sm){}", RE_FENCE_LINK_STR);
    /// Match `RE_MD_LINK_STR`
    static ref RE_MATCH_MD_LINK_STR: String = format!(r"(?sm){}", RE_MD_LINK_STR);


    static ref RE_MATCH_FENCE_BLOCK: Regex = Regex::new(&RE_MATCH_FENCE_BLOCK_STR).unwrap();
    static ref RE_MATCH_MD_BLOCK: Regex = Regex::new(&RE_MATCH_MD_BLOCK_STR).unwrap();
    static ref RE_MATCH_FENCE_COMMAND: Regex = Regex::new(&RE_MATCH_FENCE_COMMAND_STR).unwrap();
    static ref RE_MATCH_MD_COMMAND: Regex = Regex::new(&RE_MATCH_MD_COMMAND_STR).unwrap();
    static ref RE_MATCH_FENCE_LINK: Regex = Regex::new(&RE_MATCH_FENCE_LINK_STR).unwrap();
    static ref RE_MATCH_MD_LINK: Regex = Regex::new(&RE_MATCH_MD_LINK_STR).unwrap();
}

struct FailingCommand {
    output: Output,
    command: String,
    command_char: char,
}

fn main() -> std::io::Result<()> {
    let opt = Opt::from_args();
    let clean = opt.clean;
    let frozen = opt.frozen;
    let input = opt.input;
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
    let original_contents = read_file(&input);
    let mut contents = original_contents.clone();

    eprintln!(
        "Using clean={:?} input={:?} output={:?}",
        clean, &input, output,
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

    clean_blocks(&mut contents, &RE_MATCH_FENCE_BLOCK);
    clean_blocks(&mut contents, &RE_MATCH_MD_BLOCK);

    // Write the contents and return if --clean is passed
    if clean {
        write_file(&output, contents.to_string());
        return Ok(());
    }

    // Run all commands and fill their blocks.
    // If some commands return a non-zero exit code,
    // returns a `Vec<FailingCommand>` of all commands that failed.
    let fill_commands = |file: &mut String,
                         command_regex: &Regex,
                         command_char: char,
                         start_delimiter: &str,
                         end_delimiter: &str|
     -> Result<(), Vec<FailingCommand>> {
        let mut failures = Vec::new();

        *file = command_regex
            .replace_all(file, |caps: &Captures| {
                let command = &caps["command"];

                eprintln!("{} {}", command_char, command);

                let result = run_command(command, &work_dir);

                if result.status.success() {
                    let stdout = String::from_utf8_lossy(&result.stdout);
                    format!(
                        "{}{}{}{}",
                        trail_nl(&caps[0]),
                        start_delimiter,
                        wrap_nl(stdout.to_string()),
                        end_delimiter
                    )
                } else {
                    failures.push(FailingCommand {
                        output: result,
                        command: command.to_string(),
                        command_char: command_char,
                    });
                    // re-insert what was there before
                    caps[0].to_string()
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
            eprintln!(
                "`{} {}` failed with status {}.{}\n",
                f.command_char, f.command, f.output.status, stderr
            );
        }
    }

    fill_commands(&mut contents, &RE_MATCH_FENCE_COMMAND, '$', "```", "```")
        .and(fill_commands(
            &mut contents,
            &RE_MATCH_MD_COMMAND,
            '>',
            "<!-- BEGIN mdsh -->",
            "<!-- END mdsh -->",
        ))
        .or_else(|failures| -> std::io::Result<()> {
            print_failures(failures);
            std::process::exit(1);
        })?;

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

                eprintln!("[{} {}]", link_char, link);

                let result = read_file(&FileArg::from_str_unsafe(link));

                format!(
                    "{}{}{}{}",
                    trail_nl(&caps[0]),
                    start_delimiter,
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

    // Special path if the file is frozen
    if frozen {
        if original_contents == contents {
            return Ok(());
        }

        eprintln!("Found differences in output:");
        let mut line = 0;
        for diff in diff::lines(&original_contents, &contents) {
            match diff {
                diff::Result::Left(l) => {
                    line += 1;
                    eprintln!("{}- {}", line, l)
                }
                diff::Result::Both(_, _) => {
                    // nothing changed, just increase the line number
                    line += 1
                }
                diff::Result::Right(l) => eprintln!("{}+ {}", line, l),
            };
        }

        return Err(std::io::Error::new(
            ErrorKind::Other,
            "--frozen: input is not the same",
        ));
    }

    write_file(&output, contents.to_string());

    Ok(())
}
