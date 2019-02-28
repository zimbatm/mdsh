#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate structopt;

use regex::{Captures, Regex};
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use structopt::StructOpt;

fn run_command(command: &str, work_dir: &Path) -> Output {
    Command::new("bash")
        .arg("-c")
        .arg(command)
        .stdin(Stdio::null()) // don't read from stdin
        .stderr(Stdio::inherit()) // send stderr to stderr
        .current_dir(work_dir)
        .output()
        .expect("failed to execute command")
}

fn read_file<T: AsRef<str>>(p: T) -> Result<String, std::io::Error> {
    let path = p.as_ref();
    let mut buffer = String::new();

    if path == "-" {
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        handle.read_to_string(&mut buffer)?;
    } else {
        let mut file = File::open(path)?;
        file.read_to_string(&mut buffer)?;
    }

    Ok(buffer)
}

fn write_file<T: AsRef<str>>(path: T, contents: String) -> Result<(), std::io::Error> {
    let p = path.as_ref();
    if p == "-" {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        write!(handle, "{}", contents)?;
    } else {
        let mut file = File::create(p)?;
        write!(file, "{}", contents)?;
        file.sync_all()?;
    }

    Ok(())
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

lazy_static! {
    static ref RE_FENCE_LINK_STR: String = String::from(r"^\[\$ (?P<link>[^\]]+)\]\([^\)]+\)\s*$");
    static ref RE_MD_LINK_STR: String = String::from(r"^\[> (?P<link>[^\]]+)\]\([^\)]+\)\s*$");
    static ref RE_FENCE_COMMAND_STR: String = String::from(r"^`\$ (?P<command>[^`]+)`\s*$");
    static ref RE_MD_COMMAND_STR: String = String::from(r"^`> (?P<command>[^`]+)`\s*$");
    static ref RE_MD_BLOCK_STR: String = String::from(r"^<!-- BEGIN mdsh -->.+?^<!-- END mdsh -->");
    static ref RE_FENCE_BLOCK_STR: String = String::from(r"^```.+?^```");
    static ref RE_MATCH_FENCE_BLOCK_STR: String = format!(
        r"(?sm)({}|{})[\s\n]+({}|{})",
        RE_FENCE_COMMAND_STR.to_string(),
        RE_FENCE_LINK_STR.to_string(),
        RE_FENCE_BLOCK_STR.to_string(),
        RE_MD_BLOCK_STR.to_string(),
    );
    static ref RE_MATCH_FENCE_BLOCK: Regex = Regex::new(&RE_MATCH_FENCE_BLOCK_STR).unwrap();
    static ref RE_MATCH_MD_BLOCK_STR: String = format!(
        r"(?sm)({}|{})[\s\n]+({}|{})",
        RE_MD_COMMAND_STR.to_string(),
        RE_MD_LINK_STR.to_string(),
        RE_MD_BLOCK_STR.to_string(),
        RE_FENCE_BLOCK_STR.to_string(),
    );
    static ref RE_MATCH_MD_BLOCK: Regex = Regex::new(&RE_MATCH_MD_BLOCK_STR).unwrap();
    static ref RE_MATCH_FENCE_COMMAND_STR: String =
        format!(r"(?sm){}", RE_FENCE_COMMAND_STR.to_string());
    static ref RE_MATCH_FENCE_COMMAND: Regex = Regex::new(&RE_MATCH_FENCE_COMMAND_STR).unwrap();
    static ref RE_MATCH_MD_COMMAND_STR: String = format!(r"(?sm){}", RE_MD_COMMAND_STR.to_string());
    static ref RE_MATCH_MD_COMMAND: Regex = Regex::new(&RE_MATCH_MD_COMMAND_STR).unwrap();
    static ref RE_MATCH_FENCE_LINK_STR: String = format!(r"(?sm){}", RE_FENCE_LINK_STR.to_string());
    static ref RE_MATCH_FENCE_LINK: Regex = Regex::new(&RE_MATCH_FENCE_LINK_STR).unwrap();
    static ref RE_MATCH_MD_LINK_STR: String = format!(r"(?sm){}", RE_MD_LINK_STR.to_string());
    static ref RE_MATCH_MD_LINK: Regex = Regex::new(&RE_MATCH_MD_LINK_STR).unwrap();
}

#[derive(Debug, StructOpt)]
#[structopt(name = "mdsh", about = "markdown shell pre-processor")]
struct Opt {
    /// Path to the markdown file
    #[structopt(short = "i", long = "input", default_value = "README.md")]
    input: String,
    /// Path to the output file, defaults to the input value
    #[structopt(short = "o", long = "output")]
    output: Option<String>,
    /// Directory to execute the scripts under, defaults to the input folder
    #[structopt(long = "work_dir", parse(from_os_str))]
    work_dir: Option<PathBuf>,

    /// Only clean the file from blocks
    #[structopt(long = "clean")]
    clean: bool,
}

fn main() -> std::io::Result<()> {
    let opt = Opt::from_args();
    let clean = opt.clean;
    let input = opt.input;
    let output = opt.output.unwrap_or_else(|| input.clone());
    let work_dir = match opt.work_dir {
        Some(path) => path,
        None => {
            let path = match Path::new(&input).parent() {
                Some(path) => {
                    if path == Path::new("") {
                        Path::new(".")
                    } else {
                        path
                    }
                }
                // FIXME: crash here
                None => Path::new("."),
            };
            path.to_path_buf()
        }
    };
    let contents = read_file(&input)?;

    eprintln!("Using clean={} input={} output={}", clean, &input, output,);

    let contents = RE_MATCH_FENCE_BLOCK.replace_all(&contents, |caps: &Captures| {
        // println!("caps1: {:?}", caps);
        caps[1].to_string()
    });

    let contents = RE_MATCH_MD_BLOCK.replace_all(&contents, |caps: &Captures| {
        // println!("caps2: {:?}", caps);
        caps[1].to_string()
    });

    // Write the contents and return if --clean is passed
    if clean {
        write_file(output, contents.to_string())?;
        return Ok(());
    }

    let contents = RE_MATCH_FENCE_COMMAND.replace_all(&contents, |caps: &Captures| {
        let command = &caps["command"];

        eprintln!("$ {}", command);

        let result = run_command(command, &work_dir);

        // TODO: if there is an error, write to stdout
        let stdout = String::from_utf8(result.stdout).unwrap();

        format!("{}```{}```", trail_nl(&caps[0]), wrap_nl(stdout))
    });

    let contents = RE_MATCH_MD_COMMAND.replace_all(&contents, |caps: &Captures| {
        let command = &caps["command"];

        eprintln!("> {}", command);

        let result = run_command(command, &work_dir);

        // TODO: if there is an error, write to stdout
        let stdout = String::from_utf8(result.stdout).unwrap();

        format!(
            "{}<!-- BEGIN mdsh -->{}<!-- END mdsh -->",
            trail_nl(&caps[0]),
            wrap_nl(stdout)
        )
    });

    let contents = RE_MATCH_FENCE_LINK.replace_all(&contents, |caps: &Captures| {
        let link = &caps["link"];

        eprintln!("[$ {}]", link);

        let result =
            read_file(link).unwrap_or_else(|_| String::from("[mdsh error]: failed to read file"));

        format!("{}```{}```", trail_nl(&caps[0]), wrap_nl(result))
    });

    let contents = RE_MATCH_MD_LINK.replace_all(&contents, |caps: &Captures| {
        let link = &caps["link"];

        eprintln!("[> {}]", link);

        let result = read_file(link).unwrap_or_else(|_| String::from("failed to read file"));

        format!(
            "{}<!-- BEGIN mdsh -->{}<!-- END mdsh -->",
            trail_nl(&caps[0]),
            wrap_nl(result)
        )
    });

    write_file(output, contents.to_string())?;

    Ok(())
}
