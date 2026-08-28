use std::path::Path;

use eyre::bail;
use git2::{DiffDelta, DiffLine, Oid};
use tracing::{debug, error, trace};

use super::*;

impl Delta {
    pub fn load(repo: &Repository, mode: &DiffMode) -> Result<Self> {
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

        Delta::calc(repo, diff)
    }

    fn calc<'a>(repo: &'a Repository, diff: Diff<'a>) -> Result<Self> {
        let mut collector = PrintCollector::new();

        diff.print(git2::DiffFormat::Patch, |delta, _hunk, line| {
            if let Err(err) = collector.collect(repo, delta, line) {
                error!(?err, "failed to collect diff");
            }

            true
        })
        .context("failed to render diff")?;

        let entries = collector.finalize();

        Ok(Delta { entries })
    }
}

struct PrintCollector {
    entries: Vec<Entry>,
    hunk: IHunk,
    hunks: Vec<IHunk>,
    name: Option<PathBuf>,
    new: Buffer,
    old: Buffer,
}

impl PrintCollector {
    fn new() -> Self {
        let entries = Vec::default();
        let hunks = Vec::default();
        let new = Buffer::default();
        let old = Buffer::default();
        let name = None::<PathBuf>;
        let hunk = IHunk::default();

        PrintCollector {
            entries,
            hunks,
            hunk,
            new,
            old,
            name,
        }
    }

    fn collect(&mut self, repo: &Repository, delta: DiffDelta, line: DiffLine) -> Result<()> {
        match line.origin_value() {
            git2::DiffLineType::Context
            | git2::DiffLineType::Addition
            | git2::DiffLineType::Deletion
            | git2::DiffLineType::Binary => {
                let status = match line.origin_value() {
                    git2::DiffLineType::Context => LineStatus::Context,
                    git2::DiffLineType::Addition => LineStatus::Add,
                    git2::DiffLineType::Deletion => LineStatus::Remove,
                    git2::DiffLineType::Binary => LineStatus::Binary,

                    git2::DiffLineType::ContextEOFNL
                    | git2::DiffLineType::AddEOFNL
                    | git2::DiffLineType::DeleteEOFNL
                    | git2::DiffLineType::FileHeader
                    | git2::DiffLineType::HunkHeader => unreachable!(),
                };

                let line = line.new_lineno().or(line.old_lineno()).unwrap_or_else(|| {
                    error!(line.type = ?line.origin_value(), "missing lineno");

                    0
                }) as usize;

                self.hunk.lines.push(DiffLoc { status, line })
            }

            git2::DiffLineType::FileHeader => {
                if let Some(name) = &self.name {
                    if !self.hunk.lines.is_empty() {
                        self.hunks.push(std::mem::take(&mut self.hunk));
                    }

                    self.entries.push(Entry {
                        hunks: std::mem::take(&mut self.hunks),
                        path: name.clone(),
                        old: self.old.take(),
                        new: self.new.take(),
                    });
                }

                self.name = delta.new_file().path().map(ToOwned::to_owned);

                debug!(id = ?delta.old_file().id(), path = ?delta.old_file().path(), "loading old version");

                if let Ok(buf) = Buffer::load(repo, delta.old_file().id()) {
                    self.old = buf;
                } else if let Some(path) = delta.old_file().path() {
                    self.old = Buffer::read(path)?;
                } else {
                    bail!("failed to load old file");
                }

                debug!(id = ?delta.new_file().id(), path = ?delta.new_file().path(), "loading new version");

                if let Ok(buf) = Buffer::load(repo, delta.new_file().id()) {
                    self.new = buf;
                } else if let Some(path) = delta.new_file().path() {
                    self.new = Buffer::read(path)?;
                } else {
                    bail!("failed to load new file");
                }
            }

            git2::DiffLineType::HunkHeader => {
                if !self.hunk.lines.is_empty() {
                    self.hunks.push(std::mem::take(&mut self.hunk));
                }

                self.hunk.header = std::str::from_utf8(line.content())
                    .unwrap_or_default()
                    .to_string();
                self.hunk.lines.clear();
            }

            git2::DiffLineType::ContextEOFNL
            | git2::DiffLineType::AddEOFNL
            | git2::DiffLineType::DeleteEOFNL => {}
        }

        Ok(())
    }

    fn finalize(mut self) -> Vec<Entry> {
        if (!self.hunk.lines.is_empty() || !self.hunks.is_empty())
            && let Some(name) = &self.name
        {
            if !self.hunk.lines.is_empty() {
                self.hunks.push(std::mem::take(&mut self.hunk));
            }

            self.entries.push(Entry {
                hunks: std::mem::take(&mut self.hunks),
                path: name.clone(),
                old: self.old,
                new: self.new,
            })
        }

        self.entries
    }
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
