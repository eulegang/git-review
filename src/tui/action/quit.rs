use crate::tui::action::Action;

pub struct Quit;

impl Action for Quit {
    fn perform(&self, app: &mut crate::tui::App) {
        app.should_quit = true;
    }
}
