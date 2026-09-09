use crate::tui::action::Action;

pub struct ScrollUp(pub u16);
pub struct ScrollDown(pub u16);

impl Action for ScrollUp {
    fn perform(&self, app: &mut crate::tui::App) {
        app.line = app.line.saturating_sub(self.0 as usize);
    }
}

impl Action for ScrollDown {
    fn perform(&self, app: &mut crate::tui::App) {
        app.line += self.0 as usize;
        app.line = app
            .line
            .min(app.current_file_line_count().saturating_sub(1))
    }
}
