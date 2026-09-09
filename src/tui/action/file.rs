use crate::tui::action::Action;

pub struct NextFile;
pub struct PrevFile;

impl Action for NextFile {
    fn perform(&self, app: &mut crate::tui::App) {
        if app.selected_file + 1 < app.model.len() {
            app.selected_file += 1;
            app.jump_to_selected_file();
        }
    }
}

impl Action for PrevFile {
    fn perform(&self, app: &mut crate::tui::App) {
        if app.selected_file > 0 {
            app.selected_file -= 1;
            app.jump_to_selected_file();
        }
    }
}
