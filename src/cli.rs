use std::{fmt, str::FromStr};

use clap::Parser;
use eyre::{Result, bail};

#[derive(Debug, Parser)]
#[command(
    name = "git-review",
    version,
    about = "View Git diffs from the terminal"
)]
pub struct Cli {
    /// Show staged changes instead of working tree changes.
    #[arg(long)]
    staged: bool,

    /// Optional Git revision or range to diff, e.g. HEAD~1 or main..feature.
    #[arg(value_name = "REV_OR_RANGE")]
    rev: Option<Revision>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Revision {
    Commitish(String),
    Range(String, String),
}

impl Revision {
    pub fn commitish(&self) -> Option<&str> {
        match self {
            Self::Commitish(rev) => Some(rev),
            Self::Range(..) => None,
        }
    }

    pub fn range(&self) -> Option<(&str, &str)> {
        match self {
            Self::Commitish(_) => None,
            Self::Range(base, head) => Some((base, head)),
        }
    }
}

impl FromStr for Revision {
    type Err = String;

    fn from_str(raw: &str) -> std::result::Result<Self, Self::Err> {
        if raw.contains("...") {
            return Err("triple-dot ranges are not supported yet; use <base>..<head>".to_owned());
        }

        if let Some((base, head)) = raw.split_once("..") {
            if base.is_empty() || head.is_empty() {
                return Err("range must be in the form <base>..<head>".to_owned());
            }

            return Ok(Self::Range(base.to_owned(), head.to_owned()));
        }

        Ok(Self::Commitish(raw.to_owned()))
    }
}

impl fmt::Display for Revision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Commitish(rev) => f.write_str(rev),
            Self::Range(base, head) => write!(f, "{base}..{head}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffMode {
    WorkingTree,
    Staged,
    Revision(Revision),
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }

    pub fn diff_mode(self) -> Result<DiffMode> {
        match (self.staged, self.rev) {
            (false, None) => Ok(DiffMode::WorkingTree),
            (true, None) => Ok(DiffMode::Staged),
            (false, Some(rev)) => Ok(DiffMode::Revision(rev)),
            (true, Some(_)) => bail!("--staged cannot be combined with a revision or range"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_working_tree_diff() {
        let mode = Cli {
            staged: false,
            rev: None,
        }
        .diff_mode()
        .unwrap();

        assert_eq!(mode, DiffMode::WorkingTree);
    }

    #[test]
    fn supports_staged_diff() {
        let mode = Cli {
            staged: true,
            rev: None,
        }
        .diff_mode()
        .unwrap();

        assert_eq!(mode, DiffMode::Staged);
    }

    #[test]
    fn supports_revision_diff() {
        let mode = Cli {
            staged: false,
            rev: Some("main..feature".parse().unwrap()),
        }
        .diff_mode()
        .unwrap();

        assert_eq!(
            mode,
            DiffMode::Revision(Revision::Range("main".to_owned(), "feature".to_owned()))
        );
    }

    #[test]
    fn rejects_staged_with_revision() {
        let error = Cli {
            staged: true,
            rev: Some("HEAD~1".parse().unwrap()),
        }
        .diff_mode()
        .unwrap_err();

        assert!(error.to_string().contains("--staged cannot be combined"));
    }

    #[test]
    fn models_single_revision() {
        assert_eq!(
            "HEAD~1".parse::<Revision>().unwrap(),
            Revision::Commitish("HEAD~1".to_owned())
        );
    }

    #[test]
    fn rejects_invalid_range() {
        assert_eq!(
            "main..".parse::<Revision>().unwrap_err(),
            "range must be in the form <base>..<head>"
        );
    }
}
