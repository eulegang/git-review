use std::{fmt, path::PathBuf, str::FromStr};

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
    #[arg(long, short = 's', conflicts_with_all = ["default_branch", "rev"])]
    staged: bool,

    /// Diff against the repository's default branch.
    #[arg(long, short = 'd', conflicts_with = "rev")]
    default_branch: bool,

    /// Optional Git revision or range to diff, e.g. HEAD~1 or main..feature.
    #[arg(value_name = "REV_OR_RANGE")]
    rev: Option<Revision>,

    /// Path to a Lua configuration file.
    #[arg(long, value_name = "PATH", conflicts_with = "no_config")]
    config: Option<PathBuf>,

    /// Disable loading the Lua configuration file.
    #[arg(long)]
    no_config: bool,
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
    DefaultBranch,
    Revision(Revision),
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }

    pub fn config_path(&self) -> Option<PathBuf> {
        if self.no_config {
            return None;
        }

        if let Some(path) = &self.config {
            return Some(path.clone());
        }

        default_config_path()
    }

    pub fn diff_mode(&self) -> Result<DiffMode> {
        match (self.staged, self.default_branch, self.rev.clone()) {
            (false, false, None) => Ok(DiffMode::WorkingTree),
            (true, false, None) => Ok(DiffMode::Staged),
            (false, true, None) => Ok(DiffMode::DefaultBranch),
            (false, false, Some(rev)) => Ok(DiffMode::Revision(rev)),
            (true, _, Some(_)) => bail!("--staged cannot be combined with a revision or range"),
            (_, true, Some(_)) => {
                bail!("--default-branch cannot be combined with a revision or range")
            }
            (true, true, None) => bail!("--staged cannot be combined with --default-branch"),
        }
    }
}

fn default_config_path() -> Option<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(config_home) = std::env::var_os("XDG_CONFIG_HOME") {
        candidates.push(
            PathBuf::from(config_home)
                .join("git-review")
                .join("init.lua"),
        );
    }

    if let Some(home) = dirs::home_dir() {
        candidates.push(home.join(".config").join("git-review").join("init.lua"));
    }

    if let Some(config_dir) = dirs::config_dir() {
        candidates.push(config_dir.join("git-review").join("init.lua"));
    }

    candidates.into_iter().find(|path| path.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_working_tree_diff() {
        let mode = Cli::parse_from(["git-review"]).diff_mode().unwrap();

        assert_eq!(mode, DiffMode::WorkingTree);
    }

    #[test]
    fn supports_staged_diff() {
        let mode = Cli::parse_from(["git-review", "--staged"])
            .diff_mode()
            .unwrap();

        assert_eq!(mode, DiffMode::Staged);
    }

    #[test]
    fn supports_default_branch_diff() {
        let mode = Cli::parse_from(["git-review", "--default-branch"])
            .diff_mode()
            .unwrap();

        assert_eq!(mode, DiffMode::DefaultBranch);
    }

    #[test]
    fn supports_revision_diff() {
        let mode = Cli::parse_from(["git-review", "main..feature"])
            .diff_mode()
            .unwrap();

        assert_eq!(
            mode,
            DiffMode::Revision(Revision::Range("main".to_owned(), "feature".to_owned()))
        );
    }

    #[test]
    fn cli_rejects_conflicting_diff_modes() {
        assert!(Cli::try_parse_from(["git-review", "--staged", "HEAD~1"]).is_err());
        assert!(Cli::try_parse_from(["git-review", "--default-branch", "HEAD~1"]).is_err());
        assert!(Cli::try_parse_from(["git-review", "--staged", "--default-branch"]).is_err());
    }

    #[test]
    fn supports_config_path() {
        let cli = Cli::parse_from(["git-review", "--config", "custom.lua"]);

        assert_eq!(cli.config_path(), Some(PathBuf::from("custom.lua")));
        assert!(!cli.no_config());
    }

    #[test]
    fn rejects_conflicting_config_flags() {
        assert!(
            Cli::try_parse_from(["git-review", "--config", "custom.lua", "--no-config"]).is_err()
        );
    }

    #[test]
    fn rejects_staged_with_revision() {
        let error = Cli::try_parse_from(["git-review", "--staged", "HEAD~1"]).unwrap_err();

        assert!(error.to_string().contains("--staged"));
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
