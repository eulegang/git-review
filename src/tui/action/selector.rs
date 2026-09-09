use crate::tui::action::Action;

pub struct Open;
pub struct Close;
pub struct NextFile;
pub struct PrevFile;
pub struct FirstFile;
pub struct LastFile;
pub struct Confirm;

impl Action for Open {
    fn perform(&self, app: &mut crate::tui::App) {
        app.selector_file = app.selected_file;
        app.mode = super::Mode::FileSelector;
    }
}

impl Action for Close {
    fn perform(&self, app: &mut crate::tui::App) {
        app.mode = super::Mode::Diff;
    }
}

impl Action for NextFile {
    fn perform(&self, app: &mut crate::tui::App) {
        if app.selector_file + 1 < app.model.len() {
            app.selector_file += 1;
        }
    }
}

impl Action for PrevFile {
    fn perform(&self, app: &mut crate::tui::App) {
        app.selector_file = app.selector_file.saturating_sub(1);
    }
}

impl Action for FirstFile {
    fn perform(&self, app: &mut crate::tui::App) {
        app.selector_file = 0
    }
}

impl Action for LastFile {
    fn perform(&self, app: &mut crate::tui::App) {
        if !app.model.len() == 0 {
            app.selector_file = app.model.len() - 1;
        }
    }
}

impl Action for Confirm {
    fn perform(&self, app: &mut crate::tui::App) {
        app.selected_file = app.selector_file;
        app.mode = super::Mode::Diff;
        app.jump_to_selected_file();
    }
}
