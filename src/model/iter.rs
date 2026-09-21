use super::*;

impl Delta {
    pub fn entries(&self) -> impl std::iter::Iterator<Item = &Entry> {
        self.entries.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn get(&self, pos: usize) -> Option<&Entry> {
        self.entries.get(pos)
    }

    pub fn get_mut(&mut self, pos: usize) -> Option<&mut Entry> {
        self.entries.get_mut(pos)
    }
}

impl Entry {
    pub fn hunks<'a>(&'a self) -> impl std::iter::Iterator<Item = Hunk<'a>> {
        self.hunks.iter().map(|ihunk| Hunk {
            header: ihunk.header.as_str(),
            entry: self,
            diff_locs: &ihunk.lines,
        })
    }
}

impl Hunk<'_> {
    pub fn lines(&self) -> impl std::iter::Iterator<Item = HunkLine<'_>> {
        self.diff_locs.iter().map(|&diff_loc| HunkLine {
            entry: &self.entry,
            diff_loc,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::buf::Buffer;

    use super::*;

    #[test]
    fn file_filter_does_not_limit_entries() {
        let delta = Delta {
            entries: vec![entry("src/main.rs"), entry("README.md")],
        };

        assert_eq!(delta.len(), 2);
        assert_eq!(
            delta.get(1).map(|entry| entry.path.as_path()),
            Some(std::path::Path::new("README.md"))
        );
    }

    fn entry(path: &str) -> Entry {
        Entry {
            path: PathBuf::from(path),
            old: Buffer::default(),
            new: Buffer::default(),
            hunks: vec![],
        }
    }
}
