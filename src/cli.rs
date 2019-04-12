//! Command line interface
use std::path::{Path, PathBuf};
use std::str::FromStr;
use structopt::StructOpt;

/// Markdown shell pre-processor.
/// Never let your READMEs and tutorials get out of sync again.
///
/// Exits non-zero if a sub-command failed.
#[derive(Debug, StructOpt)]
#[structopt(name = "mdsh")]
pub struct Opt {
    /// Path to the markdown file. `-` for stdin.
    #[structopt(short = "i", long = "input", default_value = "./README.md")]
    pub input: FileArg,

    /// Path to the output file, `-` for stdout [defaults to updating the input file in-place].
    #[structopt(short = "o", long = "output")]
    pub output: Option<FileArg>,

    /// Directory to execute the scripts under [defaults to the input file’s directory].
    #[structopt(long = "work_dir", parse(from_os_str))]
    pub work_dir: Option<PathBuf>,

    /// Fail if the output is different from the input. Useful for CI.
    ///
    /// Using `--frozen`, you can guarantee that developers update
    /// documentation when they make a change. Just add `mdsh --frozen`
    /// as a check to your continuous integration setup.
    #[structopt(long = "frozen")]
    pub frozen: bool,

    /// Remove all generated blocks.
    #[structopt(long = "clean")]
    pub clean: bool,
}

/// Possible file input (either a file name or `-`)
#[derive(Debug, Clone)]
pub enum FileArg {
    /// equal to - (so stdin or stdout)
    StdHandle,
    File(PathBuf),
}

impl FileArg {
    /// Return the parent, if it is a `StdHandle` use the current directory.
    /// Returns `None` if there is no parent (that is we are `/`).
    pub fn parent(&self) -> Option<Parent> {
        match self {
            FileArg::StdHandle => Some(Parent::current_dir()),
            FileArg::File(buf) => Parent::of(buf),
        }
    }

    /// return a `FileArg::File`, don’t parse
    pub fn from_str_unsafe(s: &str) -> Self {
        FileArg::File(PathBuf::from(s))
    }
}

impl FromStr for FileArg {
    type Err = std::string::ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "-" => Ok(FileArg::StdHandle),
            p => Ok(FileArg::File(PathBuf::from(p))),
        }
    }
}

/// Parent path, gracefully handling relative path inputs
pub struct Parent(PathBuf);

impl Parent {
    /// Create from a `Path`, falling back to the
    /// `current_dir()` if necessary.
    /// Returns `None` if there is no parent (that is we are `/`).
    pub fn of(p: &Path) -> Option<Self> {
        let prnt = p.parent()?;
        if prnt.as_os_str().is_empty() {
            Some(Self::current_dir())
        } else {
            Some(Parent(prnt.to_path_buf()))
        }
    }

    /// Creates a `Parent` that is the current directory.
    /// Asks the operating system for the path.
    pub fn current_dir() -> Self {
        Parent(
            std::env::current_dir().expect(
                "fatal: current working directory not accessible and `--work_dir` not given",
            ),
        )
    }

    /// Convert from a `PathBuf` that is already a parent.
    pub fn from_parent_path_buf(buf: PathBuf) -> Self {
        Parent(buf)
    }

    pub fn as_path_buf(&self) -> &PathBuf {
        &self.0
    }
}
