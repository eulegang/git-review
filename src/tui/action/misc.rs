use crate::tui::action::Action;

pub struct CenterLine;

impl Action for CenterLine {
    fn perform(&self, app: &mut crate::tui::App) {
        app.center_line = true;
    }
}
