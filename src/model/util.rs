use super::*;

impl Hunk<'_> {
    pub fn header(&self) -> &str {
        self.header
    }

    pub fn critical(&self) -> usize {
        self.diff_locs
            .iter()
            .filter(|line| matches!(line.status, LineStatus::Add | LineStatus::Remove))
            .count()
    }
}

impl LineStatus {
    pub fn is_critical(&self) -> bool {
        matches!(self, LineStatus::Remove | LineStatus::Add)
    }
}

impl<'a> HunkLine<'a> {
    pub fn status(&self) -> LineStatus {
        self.diff_loc.status
    }

    pub fn content(&self) -> &'a str {
        match self.diff_loc.status {
            LineStatus::Add => self.entry.new.line(self.diff_loc.line),
            LineStatus::Remove => self.entry.old.line(self.diff_loc.line),
            LineStatus::Context => self.entry.new.line(self.diff_loc.line),
            LineStatus::Binary => "",
        }
    }

    pub fn highlight(&self) -> ratatui::text::Line<'static> {
        match self.diff_loc.status {
            LineStatus::Add => self.entry.new.highlight(self.diff_loc.line),
            LineStatus::Remove => self.entry.old.highlight(self.diff_loc.line),
            LineStatus::Context => self.entry.new.highlight(self.diff_loc.line),
            LineStatus::Binary => "".into(),
        }
    }
}
