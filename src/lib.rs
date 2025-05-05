pub mod cli;
pub mod executor;
mod nom_ext;
pub mod parser;
#[cfg(test)]
mod tests;

use std::io::Write;

use anyhow::{Context, Result};
use nom::Finish;

use crate::parser::fmt_nom_error;

const BEGIN_MDSH: &str = "<!-- BEGIN mdsh -->";
const END_MDSH: &str = "<!-- END mdsh -->";

pub trait Processor<'a> {
    fn process_piece(&mut self, piece: MdPiece<'a>) -> Result<()>;

    fn process(&'a mut self, input: &'a str, input_pipe: &cli::FileArg) -> Result<()> {
        // TODO: consider streaming directly from BufReader or smth,
        // see https://github.com/rust-bakery/nom/issues/1145
        let mut iter = nom::combinator::iterator(input, parser::markdown_piece());

        for piece in iter.by_ref() {
            self.process_piece(piece)
                .context("processing markdown piece")?;
        }

        let (_input, _) = iter
            .finish()
            .finish()
            .map_err(fmt_nom_error(input, &format!("{input_pipe:?}")))
            .context("parsing markdown")?;

        Ok(())
    }
}

pub struct Cleaner<W> {
    pub out: W,
}

impl<W> Cleaner<W> {
    pub fn new(out: W) -> Self {
        Self { out }
    }
}

impl<'a, W: Write> Processor<'a> for Cleaner<W> {
    fn process_piece(&mut self, piece: MdPiece<'a>) -> Result<()> {
        match piece {
            MdPiece::FencedBlock => (),
            MdPiece::Action((source, _action)) => {
                self.out.write_all(source.as_bytes())?;
            }
            MdPiece::RawLine(raw_line) => {
                self.out.write_all(raw_line.as_bytes())?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum MdPiece<'a> {
    FencedBlock,
    Action(parser::ActionWithSource<'a>),
    RawLine(&'a str),
}

#[cfg(test)]
pub(crate) mod test {
    use crate::{cli::FileArg, executor::TheProcessor, Cleaner, Processor};

    pub(crate) fn process(input: &str) -> anyhow::Result<String> {
        let mut buf = Vec::new();
        TheProcessor::new(std::ffi::OsStr::new("."), &mut buf)
            .process(input, &FileArg::StdHandle)?;
        Ok(String::from_utf8(buf)?)
    }

    fn process_clean(input: &str) -> anyhow::Result<String> {
        let mut buf = Vec::new();
        Cleaner::new(&mut buf).process(input, &FileArg::StdHandle)?;
        Ok(String::from_utf8(buf)?)
    }

    macro_rules! assert_process_eq {
        ($i:tt, $o:tt) => {
            const ANSI_R: &str = "\x1b[1;31m";
            const ANSI_G: &str = "\x1b[1;32m";
            const ANSI_B: &str = "\x1b[1;34m";
            const ANSI_0: &str = "\x1b[0m";
            const EOF_M: &str = "\x1b[1;33mEOF\x1b[0m";

            let input = $i;
            println!("{ANSI_B}INPUT ============={ANSI_0}\n{}{EOF_M}", input);
            let result = process(&input)
                .inspect_err(|e| println!("{e}"))
                .expect("processing");
            let expected = $o;

            if result != expected {
                println!("{ANSI_R}RESULT ============{ANSI_0}\n{}{EOF_M}", result);
                println!("{ANSI_G}EXPECTED =========={ANSI_0}\n{}{EOF_M}", expected);
            }
            assert_eq!(result, expected, "unexpected processing result");

            let result2 = process(&result)
                .inspect_err(|e| println!("{e}"))
                .expect("second processing");
            if result != result2 {
                println!("{ANSI_R}RESULT 2 =========={ANSI_0}\n{}{EOF_M}", result2);
                println!("{ANSI_G}EXPECTED =========={ANSI_0}\n{}{EOF_M}", expected);
            }

            assert_eq!(result, result2, "processing is not idempotent");
        };
    }
    pub(crate) use assert_process_eq;

    #[test]
    fn test_whole_file() {
        let file_in = String::from_utf8(std::fs::read("spec.clear.md").unwrap()).unwrap();
        let file_out = String::from_utf8(std::fs::read("spec.processed.md").unwrap()).unwrap();
        let result = process(&file_in)
            .inspect_err(|e| println!("{e}"))
            .expect("processing");
        if result != file_out {
            std::fs::write("/tmp/tmp.md", &result).unwrap();
        }

        assert_eq!(
            result, file_out,
            "unexpected result, see `diff /tmp/tmp.md spec.clear.md`"
        );
    }

    #[test]
    fn test_whole_file_idempotency() {
        let file_in = String::from_utf8(std::fs::read("spec.processed.md").unwrap()).unwrap();
        let result = process(&file_in)
            .inspect_err(|e| println!("{e}"))
            .expect("processing");
        if result != file_in {
            std::fs::write("/tmp/tmp2.md", &result).unwrap();
        }

        assert_eq!(
            result, file_in,
            "unexpected result, see `diff /tmp/tmp2.md spec.processed.md`"
        );
    }

    #[test]
    fn test_whole_file_cleaner() {
        let file_in = String::from_utf8(std::fs::read("spec.processed.md").unwrap()).unwrap();
        let file_out = String::from_utf8(std::fs::read("spec.clear.md").unwrap()).unwrap();
        let result = process_clean(&file_in)
            .inspect_err(|e| println!("{e}"))
            .expect("processing");

        if result != file_out {
            std::fs::write("/tmp/tmp3.md", &result).unwrap();
        }

        assert_eq!(
            result, file_out,
            "unexpected result, see `diff /tmp/tmp3.md spec.clear.md`"
        );
    }
}
