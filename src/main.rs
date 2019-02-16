#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate clap;
extern crate regex;

use clap::{Arg, App};
use regex::{Regex,Captures};
use std::process::{Command, Stdio};
use std::path::Path;
use std::io::prelude::*;
use std::io::{self, Write};
use std::fs::File;

// a cross-platform command executor
fn cmd() -> Command {
    if cfg!(target_os = "windows") {
        let mut cmd = Command::new("cmd");
        cmd.arg("/C");
        cmd
    } else {
        let mut cmd = Command::new("sh");
        cmd.arg("-c");
        cmd
    }
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

lazy_static! {
    static ref RE_MD_BLOCK_STR: String = String::from(r"^<!-- BEGIN mdsh -->.+?^<-- END mdsh -->");
    static ref RE_FENCE_BLOCK_STR: String = String::from(r"^```.+?^```");

    static ref RE_MD_COMMAND_STR: String = String::from(r"^`\> (?P<command>[^`]+)`\s*$");
    static ref RE_FENCE_COMMAND_STR: String = String::from(r"^`\$ (?P<command>[^`]+)`\s*$");

    static ref RE_MATCH_FENCE_BLOCK_STR: String = format!(r"(?sm)({})[\s\n]+{}", RE_FENCE_COMMAND_STR.to_string(), RE_FENCE_BLOCK_STR.to_string());
    static ref RE_MATCH_FENCE_BLOCK: Regex = Regex::new(&RE_MATCH_FENCE_BLOCK_STR).unwrap();

    static ref RE_MATCH_FENCE_COMMAND_STR: String = format!(r"(?sm){}", RE_FENCE_COMMAND_STR.to_string());
    static ref RE_MATCH_FENCE_COMMAND: Regex = Regex::new(&RE_MATCH_FENCE_COMMAND_STR).unwrap();


}



fn main() -> std::io::Result<()> {
    let matches = App::new("mdsh")
                          .about("Markdown shell pre-processor")
                          .author("zimbatm <zimbatm@zimbatm.com>")
                          .version(crate_version!())
                          .arg(Arg::with_name("input")
                               .value_name("INPUT")
                               .help("Path to the markdown file")
                               .default_value("README.md")
                               .index(1))
                          .arg(Arg::with_name("work_dir")
                               .long("work_dir")
                               .value_name("DIR")
                               .help("Directory to execute the scripts under, defaults to the input folder"))
                          .arg(Arg::with_name("output")
                               .short("o")
                               .long("output")
                               .value_name("OUTPUT")
                               .help("Path to the output file, defaults to the input value"))
                          .arg(Arg::with_name("clean")
                               .long("clean")
                               .help("Only clean the file from blocks"))
                          .get_matches();

    let clean = matches.is_present("clean");
    let input = matches.value_of("input").unwrap();
    let output = matches.value_of("output").unwrap_or(input);
    let work_dir = match matches.value_of("work_dir") {
        Some(path) => {
            Path::new(path)
        },
        None => {
            Path::new(input).parent().unwrap()
        },
    };
    let contents = read_file(input)?;

    println!("Using clean={} input={}", clean, input);

    let contents = RE_MATCH_FENCE_BLOCK.replace_all(&contents, |caps: &Captures| {
        //println!("caps: {:?}", caps);
        format!("{}", &caps[1])
    });

    // Write the contents and return if --clean is passed
    if clean {
        write_file(output, contents.to_string())?;
        return Ok(());
    }

    let contents = RE_MATCH_FENCE_COMMAND.replace_all(&contents, |caps: &Captures| {
        let command = &caps["command"];

        println!("Running command: {}", command);

        let result = cmd()
            .stdin(Stdio::null()) // don't read from stdin
            .stderr(Stdio::inherit()) // send stderr to stderr
            .current_dir(work_dir)
            .arg(command)
            .output()
            .expect("failed to execute command");

        let stdout = String::from_utf8(result.stdout).unwrap();

        format!("{}```\n{}```\n", &caps[0], stdout)
    });

    write_file(output, contents.to_string())?;

    Ok(())
}
