use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

use crate::{filter::FilterType, tui::Theme};

pub struct TextInput<'a> {
    pub text: &'a str,
    pub filter_type: &'a FilterType,
    pub invert: bool,
    pub theme: &'a Theme,
}

impl Widget for TextInput<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        tracing::trace!("rendering text input");
        let area = input_rect(area);
        Clear.render(area, buf);

        let mode = match self.filter_type {
            FilterType::Glob => "glob",
            FilterType::Regex => "regex",
            FilterType::Fuzzy => "fuzzy",
        };

        let inv = match self.invert {
            true => " inv",
            false => "",
        };

        let input = Paragraph::new(self.text).block(
            Block::default()
                .title(format!("Input — {mode}{inv}"))
                .borders(Borders::ALL)
                .border_style(self.theme.selector_highlight),
        );

        input.render(area, buf);
    }
}

pub(crate) fn input_rect(area: Rect) -> Rect {
    let width_percent = 70;
    let height = 3;
    // Sit immediately above the file selector so its first matches stay visible.
    let vertical_margin = (area.height * 15 / 100).saturating_sub(height);
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(vertical_margin),
            Constraint::Length(height.min(area.height)),
            Constraint::Min(0),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_percent) / 2),
            Constraint::Percentage(width_percent),
            Constraint::Percentage((100 - width_percent) / 2),
        ])
        .split(vertical[1])[1]
}
