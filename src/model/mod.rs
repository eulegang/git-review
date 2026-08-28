use std::path::PathBuf;

use eyre::{Context, Result, eyre};
use git2::{BranchType, Diff, DiffOptions, Repository, Tree};

use crate::{
    buf::Buffer,
    cli::{DiffMode, Revision},
};

mod debug;
mod iter;
mod load;
mod util;

#[derive(Debug)]
pub struct Delta {
    pub entries: Vec<Entry>,
}

#[derive(Debug)]
pub struct Entry {
    pub path: PathBuf,
    pub old: Buffer,
    pub new: Buffer,
    pub hunks: Vec<IHunk>,
}

#[derive(Debug, Default)]
pub struct IHunk {
    pub header: String,
    pub lines: Vec<DiffLoc>,
}

#[derive(Debug)]
pub struct Hunk<'a> {
    header: &'a str,
    entry: &'a Entry,
    diff_locs: &'a [DiffLoc],
}

#[derive(Debug)]
pub struct HunkLine<'a> {
    entry: &'a Entry,
    diff_loc: DiffLoc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineStatus {
    Add,
    Remove,
    Context,
    Binary,
}

#[derive(Clone, Copy)]
pub struct DiffLoc {
    pub status: LineStatus,
    pub line: usize,
}

#[cfg(test)]
impl Delta {
    pub fn from_test_hunks(hunks: Vec<(&str, Vec<(LineStatus, String)>)>) -> Self {
        let mut old_lines = Vec::new();
        let mut new_lines = Vec::new();
        let mut test_hunks = Vec::new();

        for (header, lines) in hunks {
            let mut diff_locs = Vec::new();

            for (status, content) in lines {
                let line = match status {
                    LineStatus::Add => {
                        new_lines.push(content);
                        new_lines.len()
                    }
                    LineStatus::Remove => {
                        old_lines.push(content);
                        old_lines.len()
                    }
                    LineStatus::Context => {
                        old_lines.push(content.clone());
                        new_lines.push(content);
                        new_lines.len()
                    }
                    LineStatus::Binary => 0,
                };

                diff_locs.push(DiffLoc { status, line });
            }

            test_hunks.push(IHunk {
                header: header.to_owned(),
                lines: diff_locs,
            });
        }

        Delta {
            entries: vec![Entry {
                path: PathBuf::from("test.rs"),
                old: Buffer::from_test_lines(&old_lines),
                new: Buffer::from_test_lines(&new_lines),
                hunks: test_hunks,
            }],
        }
    }
}
