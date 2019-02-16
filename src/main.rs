#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate clap;
extern crate regex;

use clap::{App, Arg};
use regex::{Captures, Regex};
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, Write};
// use std::path::Path;
use std::process::{Command, Stdio, Output};

fn run_command(command: &str) -> Output {
    Command::new("bash")
        .arg("-c")
        .arg(command)
        .stdin(Stdio::null()) // don't read from stdin
        .stderr(Stdio::inherit()) // send stderr to stderr
        // .current_dir(work_dir)
        .output()
        .expect("failed to execute command")
}

fn read_file(path: &str) -> Result<String, std::io::Error> {
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

fn write_file(path: &str, contents: String) -> Result<(), std::io::Error> {
    if path == "-" {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        write!(handle, "{}", contents)?;
    } else {
        let mut file = File::create(path)?;
        write!(file, "{}", contents)?;
        file.sync_all()?;
    }

    Ok(())
}

// make sure that the string starts and ends with new lines
fn wrap_nl<'a>(s: String) -> String {
    if s.starts_with("\n") {
        if s.ends_with("\n") {
            s
        } else {
            format!("{}\n", s)
        }
    } else {
        if s.ends_with("\n") {
            format!("\n{}", s)
        } else {
            format!("\n{}\n", s)
        }
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
        r"(?sm)({}|{})[\s\n]+{}",
        RE_FENCE_COMMAND_STR.to_string(),
        RE_FENCE_LINK_STR.to_string(),
        RE_FENCE_BLOCK_STR.to_string()
    );
    static ref RE_MATCH_FENCE_BLOCK: Regex = Regex::new(&RE_MATCH_FENCE_BLOCK_STR).unwrap();

    static ref RE_MATCH_MD_BLOCK_STR: String = format!(
        r"(?sm)({}|{})[\s\n]+{}",
        RE_MD_COMMAND_STR.to_string(),
        RE_MD_LINK_STR.to_string(),
        RE_MD_BLOCK_STR.to_string()
    );
    static ref RE_MATCH_MD_BLOCK: Regex = Regex::new(&RE_MATCH_MD_BLOCK_STR).unwrap();

    static ref RE_MATCH_FENCE_COMMAND_STR: String =
        format!(r"(?sm){}", RE_FENCE_COMMAND_STR.to_string());
    static ref RE_MATCH_FENCE_COMMAND: Regex = Regex::new(&RE_MATCH_FENCE_COMMAND_STR).unwrap();

    static ref RE_MATCH_MD_COMMAND_STR: String =
        format!(r"(?sm){}", RE_MD_COMMAND_STR.to_string());
    static ref RE_MATCH_MD_COMMAND: Regex = Regex::new(&RE_MATCH_MD_COMMAND_STR).unwrap();

    static ref RE_MATCH_FENCE_LINK_STR : String =
        format!(r"(?sm){}", RE_FENCE_LINK_STR.to_string());
    static ref RE_MATCH_FENCE_LINK: Regex = Regex::new(&RE_MATCH_FENCE_LINK_STR).unwrap();

    static ref RE_MATCH_MD_LINK_STR : String =
        format!(r"(?sm){}", RE_MD_LINK_STR.to_string());
    static ref RE_MATCH_MD_LINK: Regex = Regex::new(&RE_MATCH_MD_LINK_STR).unwrap();
}

fn main() -> std::io::Result<()> {
    let matches = App::new("mdsh")
        .about("Markdown shell pre-processor")
        .author("zimbatm <zimbatm@zimbatm.com>")
        .version(crate_version!())
        .arg(
            Arg::with_name("input")
                .value_name("INPUT")
                .help("Path to the markdown file")
                .default_value("README.md")
                .index(1),
        )
        /*
        .arg(
            Arg::with_name("work_dir")
                .long("work_dir")
                .value_name("DIR")
                .help("Directory to execute the scripts under, defaults to the input folder"),
        )
        */
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("OUTPUT")
                .help("Path to the output file, defaults to the input value"),
        )
        .arg(
            Arg::with_name("clean")
                .long("clean")
                .help("Only clean the file from blocks"),
        )
        .get_matches();

    let clean = matches.is_present("clean");
    let input = matches.value_of("input").unwrap();
    /*
    let input_path = Path::new(input).canonicalize();
    let input_dir = if input == "-" {
        Path::new(".")
    } else {
        input_path.unwrap().parent().unwrap()
    };
    */
    let output = matches.value_of("output").unwrap_or(input);
    /*
    let work_dir = match matches.value_of("work_dir") {
        Some(path) => Path::new(path),
        None => input_dir,
    };
    */
    let contents = read_file(input)?;

    eprintln!("Using clean={} input={} output={}", clean, input, output,);

    let contents = RE_MATCH_FENCE_BLOCK.replace_all(&contents, |caps: &Captures| {
        // println!("caps1: {:?}", caps);
        format!("{}", &caps[1])
    });

    let contents = RE_MATCH_MD_BLOCK.replace_all(&contents, |caps: &Captures| {
        // println!("caps2: {:?}", caps);
        format!("{}", &caps[1])
    });

    // Write the contents and return if --clean is passed
    if clean {
        write_file(output, contents.to_string())?;
        return Ok(());
    }

    let contents = RE_MATCH_FENCE_COMMAND.replace_all(&contents, |caps: &Captures| {
        let command = &caps["command"];

        eprintln!("$ {}", command);

        let result = run_command(command);

        // TODO: if there is an error, write to stdout
        let stdout = String::from_utf8(result.stdout).unwrap();

        format!("{}```{}```\n", &caps[0], wrap_nl(stdout))
    });

    let contents = RE_MATCH_MD_COMMAND.replace_all(&contents, |caps: &Captures| {
        let command = &caps["command"];

        eprintln!("> {}", command);

        let result = run_command(command);

        // TODO: if there is an error, write to stdout
        let stdout = String::from_utf8(result.stdout).unwrap();

        format!("{}<!-- BEGIN mdsh -->{}<!-- END mdsh -->\n", &caps[0], wrap_nl(stdout))
    });

    let contents = RE_MATCH_FENCE_LINK.replace_all(&contents, |caps: &Captures| {
        let link = &caps["link"];

        eprintln!("[$ {}]", link);

        let result = read_file(link).unwrap_or(String::from("[mdsh error]: failed to read file"));

        format!("{}```{}```\n", &caps[0], wrap_nl(result))
    });

    let contents = RE_MATCH_MD_LINK.replace_all(&contents, |caps: &Captures| {
        let link = &caps["link"];

        eprintln!("[> {}]", link);

        let result = read_file(link).unwrap_or(String::from("failed to read file"));

        format!("{}<!-- BEGIN mdsh -->{}<!-- END mdsh -->\n", &caps[0], wrap_nl(result))
    });

    write_file(output, contents.to_string())?;

    Ok(())
}
