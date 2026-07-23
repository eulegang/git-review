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
    rev: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffMode {
    WorkingTree,
    Staged,
    Revision(String),
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
            rev: Some("main..feature".to_owned()),
        }
        .diff_mode()
        .unwrap();

        assert_eq!(mode, DiffMode::Revision("main..feature".to_owned()));
    }

    #[test]
    fn rejects_staged_with_revision() {
        let error = Cli {
            staged: true,
            rev: Some("HEAD~1".to_owned()),
        }
        .diff_mode()
        .unwrap_err();

        assert!(error.to_string().contains("--staged cannot be combined"));
    }
}
