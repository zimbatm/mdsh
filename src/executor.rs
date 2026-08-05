use std::{
    collections::BTreeMap as Map,
    ffi::OsStr,
    fs::File,
    io::{Cursor, Read, Write},
    process::{self, Stdio},
};

use anyhow::{Context, Error, Result};

use crate::{MdPiece, BEGIN_MDSH, END_MDSH};

#[derive(Debug)]
/// Actionable container: comment/code/link.
pub struct Action<'a> {
    pub command: Command<'a>,
    pub data_line: Option<&'a str>,
    pub data: Option<&'a str>,
    pub transition: Option<&'a str>,
}

/// Command to execute: get data, act on data.
#[derive(Debug)]
pub struct Command<'a> {
    pub in_type: InType,
    pub out_type: OutType<'a>,
}

/// How to get data: command output, file content, or raw.
#[derive(Debug)]
pub enum InType {
    /// `$ cmd` executes `cmd` and uses data if available as stdin.
    /// `$` executes data.
    /// Use stdout as the result.
    Execute,
    /// `< fname` reads file.
    /// `<` reads list of files and concats them.
    /// Use file contents as the result
    Read,
    /// Use data as the result, only useful for setting env vars
    RawData,
}

/// What to do with the data
#[derive(Debug)]
pub enum OutType<'a> {
    /// `>` results in fence gated inlined markdown
    Markdown,
    /// `!` results are sourced as environment variables
    Environment,
    /// `> foo.yaml`, where lang name is `yaml`, results in code block
    CodeBlock(&'a str),
}

impl<'a, W: Write> crate::Processor<'a> for TheProcessor<'a, W> {
    fn process_piece(&mut self, piece: MdPiece<'a>) -> Result<()> {
        match piece {
            MdPiece::FencedBlock => (),
            MdPiece::Action((source, action)) => {
                self.out.write_all(source.as_bytes())?;
                self.process_action(action)?;
            }
            MdPiece::RawLine(raw_line) => {
                self.out.write_all(raw_line.as_bytes())?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct TheProcessor<'a, W> {
    variables: Map<String, String>,
    workdir: &'a OsStr,
    pub out: W,
}

impl<'a, W: Write> TheProcessor<'a, W> {
    pub fn new(workdir: &'a OsStr, out: W) -> Self {
        Self {
            variables: Default::default(),
            workdir,
            out,
        }
    }

    /// Process parsed [`Action`]
    pub fn process_action(&mut self, action: Action<'a>) -> Result<()> {
        let mut r = self
            .get_data(action.command.in_type, action.data_line, action.data)
            .context("getting data")?;
        self.act_on_data(action.command.out_type, action.transition, &mut r)
    }

    /// Execute or read to get the data
    fn get_data(
        &self,
        in_type: InType,
        data_line: Option<&'a str>,
        data: Option<&'a str>,
    ) -> Result<Box<dyn Read + 'a>> {
        match in_type {
            InType::RawData => Ok(match (data_line, data) {
                (Some(data_line), None) => {
                    Box::new(Cursor::new(format!("{data_line}\n"))) as Box<dyn Read>
                }
                (Some(data_line), Some(data)) => {
                    Box::new(Cursor::new(format!("{data_line}\n")).chain(data.as_bytes()))
                }
                (None, data) => Box::new(data.unwrap_or("").as_bytes()),
            }),
            InType::Read => data_line
                .into_iter()
                .chain(data.map(str::lines).into_iter().flatten())
                .try_fold(Box::new(std::io::empty()) as Box<dyn Read>, |s, x| {
                    eprintln!("< {x}");
                    Ok::<Box<dyn Read>, Error>(Box::new(s.chain(File::open(x)?)))
                }),
            InType::Execute => {
                if let Some(data) = data_line.and(data) {
                    let mut lines: usize = 0;
                    for line in data.lines() {
                        eprintln!("$ {line}");
                        lines += 1;
                        if lines > 4 {
                            eprintln!("$ ...");
                            break;
                        }
                    }
                } else if let Some(data_line) = data_line {
                    eprintln!("$ {data_line}");
                }

                let oneliner = data_line.map(|command| format!("set -euo pipefail && {command}"));

                let mut cmd = process::Command::new("bash");

                if let Some(command) = oneliner {
                    cmd.args(["-c", &command]);
                }

                let mut child = cmd
                    .envs(&self.variables)
                    .stdin(data.map_or_else(Stdio::null, |_| Stdio::piped()))
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::inherit())
                    .current_dir(self.workdir)
                    .spawn()?;

                if let Some(data) = data {
                    let mut stdin = child
                        .stdin
                        .take()
                        .context("child process didn't provide stdin pipe")?;
                    stdin
                        .write_all(data.as_bytes())
                        .context("writing to command's stdin")?;
                    stdin.flush()?;
                }
                Ok(Box::new(Child(child)) as Box<dyn Read>)
            }
        }
    }

    /// Takes data and acts on it
    fn act_on_data<R: Read>(
        &mut self,
        out_type: OutType<'a>,
        transition: Option<&str>,
        data: &mut R,
    ) -> Result<()> {
        match out_type {
            OutType::Markdown => produce_fenced_block(data, &mut self.out, transition),
            OutType::Environment => self.env_var_list(data),
            OutType::CodeBlock(lang_name) => {
                produce_code_block(lang_name, data, &mut self.out, transition)
            }
        }
        .context("acting on data")
    }

    pub fn env_var_list<R: Read>(&mut self, data: &mut R) -> Result<()> {
        use std::borrow::Cow;

        use nom::Finish;

        use crate::parser::{env_var_line, fmt_nom_error};

        let mut input = String::with_capacity(8192);
        data.read_to_string(&mut input)?;
        let input = input.as_str();

        let mut iter = nom::combinator::iterator(input, env_var_line());
        for (k, v) in iter.by_ref().flatten() {
            let val = shellexpand::env_with_context(v, |x| {
                self.variables
                    .get(x)
                    .map_or_else(|| std::env::var(x).map(Cow::from), |x| Ok(Cow::from(x)))
                    .map(Some)
            })
            .context("expanding shell variables")?
            .into();

            eprintln!("! {k}='{val}'");
            self.variables.insert(k.to_owned(), val);
        }
        // TODO: error msg with absolute line numbers
        iter.finish()
            .finish()
            .map_err(fmt_nom_error(input, "`!` env block"))?;
        Ok(())
    }
}

fn produce_fenced_block<R: Read, W: Write>(
    r: &mut R,
    w: &mut W,
    transition: Option<&str>,
) -> Result<()> {
    writeln!(w, "\n{BEGIN_MDSH}")?;
    if let Some(text) = transition {
        writeln!(w, "{text}\n")?;
    }
    std::io::copy(r, w)?;
    writeln!(w, "{END_MDSH}")?;
    Ok(())
}

fn produce_code_block<R: Read, W: Write>(
    lang: &str,
    r: &mut R,
    w: &mut W,
    transition: Option<&str>,
) -> Result<()> {
    produce_fenced_block(
        &mut format!("```{lang}\n")
            .as_bytes()
            .chain(r)
            .chain("```\n".as_bytes()),
        w,
        transition,
    )
}

/// Helper wrapper over [`std::process::Child`] that calls
/// [`std::process::Child::wait`] when [`Read::read`] returns 0.
struct Child(std::process::Child);

impl Read for Child {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        use std::io::Error;
        let n = self
            .0
            .stdout
            .as_mut()
            .ok_or_else(|| Error::other("Child process didn't provide stdout pipe"))?
            .read(buf)?;
        if n == 0 {
            let res = self.0.wait()?;
            if !res.success() {
                return Err(Error::other(format!("Child process terminated with {res}")));
            }
        }
        Ok(n)
    }
}
