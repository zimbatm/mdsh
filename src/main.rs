use std::{
    fs::File,
    io::{self, prelude::*},
};

use anyhow::Context;

use clap::Parser;
use mdsh::{
    cli::{FileArg, Opt, Parent},
    executor::TheProcessor,
    Cleaner, Processor,
};

fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();
    let clean = opt.clean;
    let frozen = opt.frozen;
    let inputs = opt.inputs;

    if let [_, _, ..] = &inputs[..] {
        opt.output
            .is_none()
            .then_some(())
            .context("--output is not compatible with multiple inputs")?;
        opt.work_dir
            .is_none()
            .then_some(())
            .context("--work-dir is not compatible with multiple inputs")?;
        for input in inputs {
            let work_dir = input
                .clone()
                .parent()
                .context("an input file has no parent directory")?;
            let output = input.clone();
            process_file(&input, &output, &work_dir, clean, frozen)?;
        }
    } else if let [input, ..] = &inputs[..] {
        let output = opt.output.unwrap_or_else(|| input.clone());
        let work_dir: Parent = opt.work_dir.map_or_else(
            || {
                input
                    .clone()
                    .parent()
                    .context("the input file has no parent directory.")
            },
            |buf| Ok(Parent::from_parent_path_buf(buf)),
        )?;
        process_file(input, &output, &work_dir, clean, frozen)?;
    }

    Ok(())
}

fn process_file(
    input: &FileArg,
    output: &FileArg,
    work_dir: &Parent,
    clean: bool,
    frozen: bool,
) -> anyhow::Result<()> {
    let mut input_content = read_file(input)?;
    let had_trailing_newline = input_content.ends_with('\n');
    // The parser requires every line to end with '\n', so ensure the last line is terminated.
    if !had_trailing_newline {
        input_content.push('\n');
    }

    let work_dir = work_dir.as_path_buf().as_os_str();
    match (input, output) {
        (FileArg::File(inf), FileArg::File(outf)) if inf == outf => {
            let mut buffer = Vec::with_capacity(8192);
            if clean {
                Cleaner::new(&mut buffer).process(&input_content, input)?;
            } else {
                TheProcessor::new(work_dir, &mut buffer).process(&input_content, input)?;
            }
            let file_unmodified_check = !frozen || input_content.as_bytes() == buffer;

            // Strip the trailing '\n' we may have added for the parser,
            // preserving the file's original trailing newline convention.
            let output = if had_trailing_newline {
                &buffer[..]
            } else {
                buffer.strip_suffix(b"\n").unwrap_or(&buffer)
            };
            std::fs::write(outf, output)
                .with_context(|| format!("failed to write file {outf:?}"))?;

            file_unmodified_check
                .then_some(())
                .context("File modified")?;
        }
        (_, FileArg::File(outf)) => {
            let mut outf_handle = File::create(outf)
                .with_context(|| format!("failed to open file {outf:?} for writing"))?;
            if clean {
                Cleaner::new(&mut outf_handle).process(&input_content, input)?;
            } else {
                TheProcessor::new(work_dir, &mut outf_handle).process(&input_content, input)?;
            }
        }
        (_, FileArg::StdHandle) => {
            if clean {
                Cleaner::new(&mut io::stdout()).process(&input_content, input)?;
            } else {
                TheProcessor::new(work_dir, &mut io::stdout()).process(&input_content, input)?;
            }
        }
    }
    Ok(())
}

fn read_file(f: &FileArg) -> anyhow::Result<String> {
    let mut buffer = String::with_capacity(8192);

    match f {
        FileArg::StdHandle => {
            let stdin = io::stdin();
            let mut handle = stdin.lock();
            handle
                .read_to_string(&mut buffer)
                .context("failed to read from stdin")?;
        }
        FileArg::File(path_buf) => {
            File::open(path_buf)
                .with_context(|| format!("failed to open file {:?}", path_buf.display()))?
                .read_to_string(&mut buffer)
                .with_context(|| format!("failed to read file {:?}", path_buf.display()))?;
        }
    }

    Ok(buffer)
}

/*
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
*/
