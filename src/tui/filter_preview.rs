use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Widget},
};

use crate::{filter::Filter, model::Delta};

/// Read-only preview of which files the draft input filter accepts.
pub struct FilterPreview<'a> {
    pub delta: &'a Delta,
    pub filter: Option<&'a Filter>,
}

impl Widget for FilterPreview<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(15),
                Constraint::Percentage(70),
                Constraint::Percentage(15),
            ])
            .split(area);
        let area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(15),
                Constraint::Percentage(70),
                Constraint::Percentage(15),
            ])
            .split(vertical[1])[1];
        Clear.render(area, buf);

        let files: Vec<ListItem> = self
            .delta
            .entries()
            .map(|entry| {
                let item = ListItem::new(entry.path.display().to_string());
                if self
                    .filter
                    .is_some_and(|filter| filter.accepts(&entry.path))
                {
                    item
                } else {
                    item.style(Style::new().fg(Color::DarkGray))
                }
            })
            .collect();
        let title = if self.filter.is_some() {
            "Files — Filter preview"
        } else {
            "Files — Invalid filter"
        };
        List::new(files)
            .block(Block::default().title(title).borders(Borders::ALL))
            .render(area, buf);
    }
}
