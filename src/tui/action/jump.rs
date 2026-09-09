use crate::tui::action::Action;

pub struct Top;
pub struct Bottom;
pub struct NextHunk;
pub struct PrevHunk;

impl Action for Top {
    fn perform(&self, app: &mut crate::tui::App) {
        app.line = 0;
        app.scroll = 0;
    }
}

impl Action for Bottom {
    fn perform(&self, app: &mut crate::tui::App) {
        app.line = app.current_file_line_count().saturating_sub(1);
    }
}

impl Action for NextHunk {
    fn perform(&self, app: &mut crate::tui::App) {
        if let Some(entry) = app.model.get(app.selected_file) {
            let mut first_line = 0;

            for (index, hunk) in entry.hunks().enumerate() {
                if app.hunk_is_hidden(index) {
                    continue;
                }

                let line_count = hunk.critical();
                if line_count == 0 {
                    continue;
                }

                if first_line > app.line {
                    app.line = first_line;
                    return;
                }

                first_line += line_count;
            }
        }
    }
}

impl Action for PrevHunk {
    fn perform(&self, app: &mut crate::tui::App) {
        if let Some(entry) = app.model.get(app.selected_file) {
            let mut first_line = 0;
            let mut previous_hunk = None;

            for (index, hunk) in entry.hunks().enumerate() {
                if app.hunk_is_hidden(index) {
                    continue;
                }

                let line_count = hunk.critical();
                if line_count == 0 {
                    continue;
                }

                if app.line <= first_line || app.line < first_line + line_count {
                    break;
                }

                previous_hunk = Some(first_line);
                first_line += line_count;
            }

            if let Some(line) = previous_hunk {
                app.line = line;
            }
        }
    }
}
