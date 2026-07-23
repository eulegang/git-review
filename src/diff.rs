use crate::cli::DiffMode;
use eyre::{Context, Result, bail};
use git2::{DiffFormat, DiffLineType, DiffOptions, Repository, Tree};

pub fn run_git_diff(mode: &DiffMode) -> Result<String> {
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

    diff_to_string(&diff)
}

fn diff_revision<'repo>(
    repo: &'repo Repository,
    rev: &str,
    options: &mut DiffOptions,
) -> Result<git2::Diff<'repo>> {
    if rev.contains("...") {
        bail!("triple-dot ranges are not supported yet; use <base>..<head>");
    }

    if let Some((base, head)) = rev.split_once("..") {
        if base.is_empty() || head.is_empty() {
            bail!("range must be in the form <base>..<head>");
        }

        let base_tree = rev_to_tree(repo, base)?;
        let head_tree = rev_to_tree(repo, head)?;

        return repo
            .diff_tree_to_tree(Some(&base_tree), Some(&head_tree), Some(options))
            .with_context(|| format!("failed to diff range {rev}"));
    }

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

fn diff_to_string(diff: &git2::Diff<'_>) -> Result<String> {
    let mut output = String::new();

    diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
        match line.origin_value() {
            DiffLineType::Context | DiffLineType::Addition | DiffLineType::Deletion => {
                output.push(line.origin());
            }
            DiffLineType::ContextEOFNL | DiffLineType::AddEOFNL | DiffLineType::DeleteEOFNL => {
                output.push('\\');
            }
            DiffLineType::FileHeader | DiffLineType::HunkHeader | DiffLineType::Binary => {}
        }

        output.push_str(&String::from_utf8_lossy(line.content()));
        true
    })
    .context("failed to render diff")?;

    Ok(output)
}
