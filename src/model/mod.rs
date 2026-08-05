use std::path::PathBuf;

use eyre::{Context, Result};
use git2::{Diff, DiffOptions, Oid, Repository, Tree};

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

impl Model {
    pub fn load(mode: &DiffMode) -> Result<Self> {
        let repo = Repository::discover(".").context("not inside a Git repository")?;
        let mut options = DiffOptions::new();

        let diff = match mode {
            DiffMode::WorkingTree => repo
                .diff_index_to_workdir(None, Some(&mut options))
                .context("failed to diff index against working tree")?,
            DiffMode::Staged => {
                let head_tree = repo.head().ok().and_then(|head| head.peel_to_tree().ok());
                repo.diff_tree_to_index(head_tree.as_ref(), None, Some(&mut options))
                    .context("failed to diff HEAD against index")?
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

        if !hunk.is_empty()
            && let Some(name) = &name
        {
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
    fn add(&mut self, status: LineStatus, line: &str) {
        let content = line.to_string();
        self.lines.push(Line { status, content })
    }

    fn is_empty(&self) -> bool {
        self.lines.is_empty()
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
