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
    pub fn lines(&self) -> impl std::iter::Iterator<Item = HunkLine> {
        self.diff_locs.iter().map(|&diff_loc| HunkLine {
            entry: &self.entry,
            diff_loc,
        })
    }
}
