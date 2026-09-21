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
        if !matches_file_filter(app, app.selector_file) {
            if let Some(index) = next_matching_file(app, app.selector_file)
                .or_else(|| previous_matching_file(app, app.selector_file))
            {
                app.selector_file = index;
            }
        }
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
        if let Some(index) = next_matching_file(app, app.selector_file) {
            app.selector_file = index;
        }
    }
}

impl Action for PrevFile {
    fn perform(&self, app: &mut crate::tui::App) {
        if let Some(index) = previous_matching_file(app, app.selector_file) {
            app.selector_file = index;
        }
    }
}

impl Action for FirstFile {
    fn perform(&self, app: &mut crate::tui::App) {
        if let Some(index) = first_matching_file(app) {
            app.selector_file = index;
        }
    }
}

impl Action for LastFile {
    fn perform(&self, app: &mut crate::tui::App) {
        if let Some(index) = last_matching_file(app) {
            app.selector_file = index;
        }
    }
}

impl Action for Confirm {
    fn perform(&self, app: &mut crate::tui::App) {
        if matches_file_filter(app, app.selector_file) {
            app.selected_file = app.selector_file;
            app.mode = super::Mode::Diff;
            app.jump_to_selected_file();
        }
    }
}

fn matches_file_filter(app: &crate::tui::App, index: usize) -> bool {
    app.model
        .get(index)
        .is_some_and(|entry| app.entry_matches_file_filter(entry))
}

fn first_matching_file(app: &crate::tui::App) -> Option<usize> {
    app.model
        .entries()
        .position(|entry| app.entry_matches_file_filter(entry))
}

fn last_matching_file(app: &crate::tui::App) -> Option<usize> {
    (0..app.model.len())
        .rev()
        .find(|&index| matches_file_filter(app, index))
}

fn next_matching_file(app: &crate::tui::App, current: usize) -> Option<usize> {
    app.model
        .entries()
        .enumerate()
        .skip(current.saturating_add(1))
        .find_map(|(index, entry)| app.entry_matches_file_filter(entry).then_some(index))
}

fn previous_matching_file(app: &crate::tui::App, current: usize) -> Option<usize> {
    (0..current)
        .rev()
        .find(|&index| matches_file_filter(app, index))
}
