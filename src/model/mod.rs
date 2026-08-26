use std::path::PathBuf;

use eyre::{Context, Result, eyre};
use git2::{BranchType, Diff, DiffOptions, Oid, Repository, Tree};

use crate::cli::{DiffMode, Revision};

#[derive(Debug)]
pub struct Model {
    pub entries: Vec<Entry>,
}

#[derive(Debug)]
pub struct Entry {
    pub path: PathBuf,
    #[allow(dead_code)]
    pub old: Oid,
    #[allow(dead_code)]
    pub new: Oid,
    pub hunks: Vec<Hunk>,
}

#[derive(Default, Debug)]
pub struct Hunk {
    pub lines: Vec<Line>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LineStatus {
    Add,
    Remove,
    Context,
    Binary,
}

#[derive(Debug)]
pub struct Line {
    pub status: LineStatus,
    pub content: String,
}

impl From<(LineStatus, &str)> for Line {
    fn from((status, content): (LineStatus, &str)) -> Self {
        Self {
            status,
            content: content.to_string(),
        }
    }
}

impl Model {
    pub fn load(mode: &DiffMode) -> Result<Self> {
        let repo = Repository::discover(".").context("not inside a Git repository")?;
        let mut options = DiffOptions::new();
        options
            .show_untracked_content(true)
            .recurse_untracked_dirs(true);

        let diff = match mode {
            DiffMode::WorkingTree => repo
                .diff_index_to_workdir(None, Some(&mut options))
                .context("failed to diff index against working tree")?,
            DiffMode::Staged => {
                let head_tree = repo.head().ok().and_then(|head| head.peel_to_tree().ok());
                repo.diff_tree_to_index(head_tree.as_ref(), None, Some(&mut options))
                    .context("failed to diff HEAD against index")?
            }
            DiffMode::DefaultBranch => {
                let default_branch = detect_default_branch(&repo)?;
                let rev = Revision::Commitish(default_branch);
                diff_revision(&repo, &rev, &mut options)?
            }
            DiffMode::Revision(rev) => diff_revision(&repo, rev, &mut options)?,
        };

        Model::try_from(diff)
    }
}

impl<'a> TryFrom<Diff<'a>> for Model {
    type Error = eyre::Report;

    fn try_from(diff: Diff<'a>) -> std::prelude::v1::Result<Self, Self::Error> {
        let mut name = Option::<PathBuf>::None;
        let mut new = Oid::zero();
        let mut old = Oid::zero();

        let mut hunk = Hunk::default();

        let mut entries = vec![];
        let mut hunks = vec![];

        diff.print(git2::DiffFormat::Patch, |delta, _hunk, line| {
            if let Ok(content) = std::str::from_utf8(line.content()) {
                match line.origin_value() {
                    git2::DiffLineType::Context => hunk.add(LineStatus::Context, content),
                    git2::DiffLineType::Addition => hunk.add(LineStatus::Add, content),
                    git2::DiffLineType::Deletion => hunk.add(LineStatus::Remove, content),
                    git2::DiffLineType::Binary => hunk.add(LineStatus::Binary, content),

                    git2::DiffLineType::FileHeader => {
                        if let Some(name) = &name {
                            if !hunk.is_empty() {
                                hunks.push(std::mem::take(&mut hunk));
                            }

                            entries.push(Entry {
                                hunks: std::mem::take(&mut hunks),
                                path: name.clone(),
                                old,
                                new,
                            });
                        }

                        name = delta.new_file().path().map(ToOwned::to_owned);
                        new = delta.new_file().id();
                        old = delta.old_file().id();
                    }
                    git2::DiffLineType::HunkHeader => {
                        if !hunk.is_empty() {
                            hunks.push(std::mem::take(&mut hunk));
                        }
                    }
                    git2::DiffLineType::ContextEOFNL
                    | git2::DiffLineType::AddEOFNL
                    | git2::DiffLineType::DeleteEOFNL => (),
                }
            }

            true
        })
        .context("failed to render diff")?;

        if (!hunk.is_empty() || !hunks.is_empty())
            && let Some(name) = &name
        {
            if !hunk.is_empty() {
                hunks.push(std::mem::take(&mut hunk));
            }

            entries.push(Entry {
                hunks: std::mem::take(&mut hunks),
                path: name.clone(),
                old,
                new,
            })
        }

        Ok(Model { entries })
    }
}

impl Hunk {
    pub fn add(&mut self, status: LineStatus, line: &str) {
        let content = line.to_string();
        self.lines.push(Line { status, content })
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn critical(&self) -> usize {
        self.lines
            .iter()
            .filter(|line| matches!(line.status, LineStatus::Add | LineStatus::Remove))
            .count()
    }
}

fn diff_revision<'repo>(
    repo: &'repo Repository,
    rev: &Revision,
    options: &mut DiffOptions,
) -> Result<git2::Diff<'repo>> {
    if let Some((base, head)) = rev.range() {
        let base_tree = rev_to_tree(repo, base)?;
        let head_tree = rev_to_tree(repo, head)?;

        return repo
            .diff_tree_to_tree(Some(&base_tree), Some(&head_tree), Some(options))
            .with_context(|| format!("failed to diff range {rev}"));
    }

    let rev = rev
        .commitish()
        .expect("revision should be a commitish or range");
    let tree = rev_to_tree(repo, rev)?;
    repo.diff_tree_to_workdir_with_index(Some(&tree), Some(options))
        .with_context(|| format!("failed to diff revision {rev} against working tree"))
}

fn rev_to_tree<'repo>(repo: &'repo Repository, rev: &str) -> Result<Tree<'repo>> {
    repo.revparse_single(rev)
        .with_context(|| format!("invalid revision or range endpoint: {rev}"))?
        .peel_to_tree()
        .with_context(|| format!("revision does not resolve to a tree: {rev}"))
}

/// Detect the repository's default branch as a revision string suitable for revparse.
///
/// The default branch is detected for the remote tracked by the current branch, so a branch
/// tracking `upstream/my-feature` compares against `upstream`'s default branch rather than always
/// assuming `origin`.
pub fn detect_default_branch(repo: &Repository) -> Result<String> {
    let remote_name = current_branch_remote(repo).unwrap_or_else(|| "origin".to_owned());

    if let Some(default_branch) = detect_remote_default_branch(repo, &remote_name)? {
        return Ok(default_branch);
    }

    if remote_name != "origin" {
        if let Some(default_branch) = detect_remote_default_branch(repo, "origin")? {
            return Ok(default_branch);
        }
    }

    for branch in ["main", "master"] {
        if repo.find_branch(branch, BranchType::Local).is_ok() {
            return Ok(branch.to_owned());
        }
    }

    Err(eyre!("could not detect default branch"))
}

fn current_branch_remote(repo: &Repository) -> Option<String> {
    let head = repo.head().ok()?;
    if !head.is_branch() {
        return None;
    }

    let branch_name = head.shorthand()?;
    let branch = repo.find_branch(branch_name, BranchType::Local).ok()?;

    if let Ok(upstream) = branch.upstream() {
        if let Ok(Some(upstream_name)) = upstream.name() {
            if let Some(remote_name) = remote_from_upstream_branch(upstream_name) {
                return Some(remote_name.to_owned());
            }
        }
    }

    let config_key = format!("branch.{branch_name}.remote");
    repo.config()
        .ok()?
        .get_string(&config_key)
        .ok()
        .filter(|remote| remote != "." && !remote.is_empty())
}

fn remote_from_upstream_branch(upstream_branch: &str) -> Option<&str> {
    upstream_branch
        .split_once('/')
        .map(|(remote_name, _)| remote_name)
        .filter(|remote_name| !remote_name.is_empty())
}

fn detect_remote_default_branch(repo: &Repository, remote_name: &str) -> Result<Option<String>> {
    let remote_head = format!("refs/remotes/{remote_name}/HEAD");
    if let Ok(head) = repo.find_reference(&remote_head) {
        if let Some(target) = head.symbolic_target() {
            return normalize_default_branch(target, remote_name).map(Some);
        }
    }

    if let Ok(remote) = repo.find_remote(remote_name) {
        if let Ok(default_branch) = remote.default_branch() {
            let branch = std::str::from_utf8(default_branch.as_ref())
                .context("default branch name is not valid UTF-8")?;
            return normalize_default_branch(branch, remote_name).map(Some);
        }
    }

    Ok(None)
}

fn normalize_default_branch(reference: &str, remote_name: &str) -> Result<String> {
    if let Some(branch) = reference.strip_prefix("refs/remotes/") {
        return Ok(branch.to_owned());
    }

    if let Some(branch) = reference.strip_prefix("refs/heads/") {
        return Ok(format!("{remote_name}/{branch}"));
    }

    if reference.is_empty() {
        return Err(eyre!("default branch ref is empty"));
    }

    Ok(reference.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_cached_remote_head() {
        assert_eq!(
            normalize_default_branch("refs/remotes/origin/main", "origin").unwrap(),
            "origin/main"
        );
    }

    #[test]
    fn normalizes_remote_default_branch() {
        assert_eq!(
            normalize_default_branch("refs/heads/main", "origin").unwrap(),
            "origin/main"
        );
    }

    #[test]
    fn extracts_remote_from_upstream_branch() {
        assert_eq!(
            remote_from_upstream_branch("upstream/feature"),
            Some("upstream")
        );
        assert_eq!(remote_from_upstream_branch("origin/main"), Some("origin"));
        assert_eq!(remote_from_upstream_branch("main"), None);
    }
}
