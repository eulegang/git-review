use crate::{
    filter::{Filter, FilterType},
    tui::action::{Action, Mode},
};

pub struct CenterLine;

impl Action for CenterLine {
    fn perform(&self, app: &mut crate::tui::App) {
        app.center_line = true;
    }
}

pub struct OpenTextInput;

impl Action for OpenTextInput {
    fn perform(&self, app: &mut crate::tui::App) {
        app.input_text = app.file_filter.repr().to_string();
        app.invert = app.file_filter.inverted();
        app.mode = Mode::TextInput;
    }
}

pub struct CloseTextInput;

impl Action for CloseTextInput {
    fn perform(&self, app: &mut crate::tui::App) {
        app.mode = Mode::Diff;
    }
}

pub struct ConfirmTextInput;

impl Action for ConfirmTextInput {
    fn perform(&self, app: &mut crate::tui::App) {
        if let Ok(f) = Filter::try_from((app.file_filter_type, app.input_text.clone(), app.invert))
        {
            app.invert = f.inverted();
            app.file_filter = f;

            app.selected_file = app.selected_file.min(app.model.len().saturating_sub(1));
            app.selector_file = app.selector_file.min(app.model.len().saturating_sub(1));
            app.mode = Mode::Diff;
        }
    }
}

pub struct InputChar(pub char);

impl Action for InputChar {
    fn perform(&self, app: &mut crate::tui::App) {
        app.input_text.push(self.0);
    }
}

pub struct BackspaceInput;

impl Action for BackspaceInput {
    fn perform(&self, app: &mut crate::tui::App) {
        app.input_text.pop();
    }
}

pub struct CycleInput;

impl Action for CycleInput {
    fn perform(&self, app: &mut crate::tui::App) {
        app.file_filter_type = match app.file_filter_type {
            FilterType::Glob => FilterType::Regex,
            FilterType::Regex => FilterType::Fuzzy,
            FilterType::Fuzzy => FilterType::Glob,
        }
    }
}

pub struct InvertInput;

impl Action for InvertInput {
    fn perform(&self, app: &mut crate::tui::App) {
        app.invert = !app.invert;
    }
}
