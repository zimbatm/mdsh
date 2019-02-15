#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate clap;
extern crate regex;

use clap::{Arg, App};
use regex::{Regex,Captures};
use std::process::{Command, Stdio};
use std::io::prelude::*;
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

lazy_static! {
    static ref RE_FENCE: String = String::from(r"^```.+?^```");
    static ref RE_CODE_BLOCK_STR: String = String::from(r"^`\s*\$ (?P<command>[^`]+)`\s*$");

    static ref RE_MATCH_FENCE_STR: String = format!(r"(?sm)({})[\s\n]+{}", RE_CODE_BLOCK_STR.to_string(), RE_FENCE.to_string());
    static ref RE_MATCH_FENCE: Regex = Regex::new(&RE_MATCH_FENCE_STR).unwrap();

    static ref RE_MATCH_BLOCK_STR: String = format!(r"(?sm){}", RE_CODE_BLOCK_STR.to_string());
    static ref RE_MATCH_BLOCK: Regex = Regex::new(&RE_MATCH_BLOCK_STR).unwrap();


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
                          .arg(Arg::with_name("clean")
                               .long("clean")
                               .help("Only clean the file from blocks"))
                          .get_matches();

    let clean = matches.is_present("clean");
    let input = matches.value_of("input").unwrap();
    let mut file = File::open(input)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // FIXME: change directory to the INPUT directory by default
    let current_dir = ".";

    println!("Using clean={} input={}", clean, input);

    let clean_contents = RE_MATCH_FENCE.replace_all(&contents, |caps: &Captures| {
        //println!("caps: {:?}", caps);
        format!("{}", &caps[1])
    });

    // FIXME: write the clean_contents out if --clean is passed

    let new_contents = RE_MATCH_BLOCK.replace_all(&clean_contents, |caps: &Captures| {
        let command = &caps["command"];

        println!("Running command: {}", command);

        let output = cmd()
            .stdin(Stdio::null()) // don't read from stdin
            .stderr(Stdio::inherit()) // send stderr to stderr
            .current_dir(current_dir)
            .arg(command)
            .output()
            .expect("failed to execute command");

        let stdout = String::from_utf8(output.stdout).unwrap();

        format!("{}```\n{}```\n", &caps[0], stdout)
    });

    let mut f = File::create(input)?;
    write!(f, "{}", new_contents)?;
    f.sync_all()?;

    Ok(())
}
