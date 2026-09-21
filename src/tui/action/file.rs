use crate::tui::action::Action;

pub struct NextFile;
pub struct PrevFile;

impl Action for NextFile {
    fn perform(&self, app: &mut crate::tui::App) {
        if let Some(index) = (app.selected_file.saturating_add(1)..app.model.len()).find(|&index| {
            app.model
                .get(index)
                .is_some_and(|entry| app.entry_matches_file_filter(entry))
        }) {
            app.selected_file = index;
            app.jump_to_selected_file();
        }
    }
}

impl Action for PrevFile {
    fn perform(&self, app: &mut crate::tui::App) {
        if let Some(index) = (0..app.selected_file).rev().find(|&index| {
            app.model
                .get(index)
                .is_some_and(|entry| app.entry_matches_file_filter(entry))
        }) {
            app.selected_file = index;
            app.jump_to_selected_file();
        }
    }
}
