use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, StatefulWidget, Widget},
};

use crate::model::Entry;

pub struct FileSelector<'a> {
    pub entries: &'a [Entry],
}

impl StatefulWidget for FileSelector<'_> {
    type State = usize;

    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer, state: &mut Self::State) {
        let area = centered_rect(70, 70, area);
        Clear.render(area, buf);

        let files: Vec<ListItem> = self
            .entries
            .iter()
            .map(|entry| ListItem::new(entry.path.display().to_string()))
            .collect();
        let selector = List::new(files)
            .block(
                Block::default()
                    .title("Files — Enter select, Esc close")
                    .borders(Borders::ALL),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("› ");
        let mut selector_state = ListState::default().with_selected(Some(*state));

        StatefulWidget::render(selector, area, buf, &mut selector_state);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}
