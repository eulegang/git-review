use crate::tui::action::Action;

pub struct HideHunk;
pub struct UnhideHunks;

impl Action for HideHunk {
    fn perform(&self, app: &mut crate::tui::App) {
        if let Some(hunk) = app.current_hunk() {
            if let Some(hidden) = app.hidden_hunks.get_mut(app.selected_file) {
                hidden.insert(hunk);
            }
            let last_line = app.current_file_line_count().saturating_sub(1);
            app.line = app.line.min(last_line);
            app.scroll = app.scroll.min(last_line);
        }
    }
}

impl Action for UnhideHunks {
    fn perform(&self, app: &mut crate::tui::App) {
        if let Some(hidden) = app.hidden_hunks.get_mut(app.selected_file) {
            hidden.clear();
        }
    }
}
