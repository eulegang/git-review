use ratatui::{
    style::Style,
    text::{Span, Text},
    widgets::{StatefulWidget, Widget},
};

use crate::{
    model::{Hunk, LineStatus},
    tui::Theme,
};

#[cfg(test)]
mod unit;

pub struct Diff<'a> {
    pub hunks: &'a [Hunk],
    pub theme: &'a Theme,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DiffState {
    pub line: usize,
    pub scroll: usize,
}

impl<'a> StatefulWidget for Diff<'a> {
    type State = DiffState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let widest_line = self
            .hunks
            .iter()
            .flat_map(|hunk| &hunk.lines)
            .map(|line| line.content.chars().count().min(area.width as usize) as u16)
            .max()
            .unwrap_or_default();
        let left_padding = area.width.saturating_sub(widest_line) / 2;
        let render_area = ratatui::prelude::Rect {
            x: area.x + left_padding,
            width: area.width - left_padding,
            ..area
        };

        let lines: Vec<_> = self
            .hunks
            .iter()
            .flat_map(|hunk| hunk.lines.iter())
            .collect();

        if let Some(selected_visual_line) = selected_visual_line(&lines, state.line) {
            let visible_height = area.height as usize;
            if selected_visual_line < state.scroll {
                state.scroll = selected_visual_line;
            } else if selected_visual_line >= state.scroll.saturating_add(visible_height) {
                state.scroll =
                    selected_visual_line.saturating_sub(visible_height.saturating_sub(1));
            }
        }

        let max_scroll = lines.len().saturating_sub(area.height as usize);
        state.scroll = state.scroll.min(max_scroll);

        let mut text: Text = Text::default();
        let mut critical_line = lines
            .iter()
            .take(state.scroll)
            .filter(|line| matches!(line.status, LineStatus::Add | LineStatus::Remove))
            .count();

        for (i, line) in lines
            .iter()
            .skip(state.scroll)
            .take(render_area.height as usize)
            .enumerate()
        {
            let mut style = match line.status {
                LineStatus::Add => Style::default().bg(self.theme.added_bg),
                LineStatus::Remove => Style::default().bg(self.theme.removed_bg),
                LineStatus::Context => Style::default(),
                LineStatus::Binary => Style::default().bg(self.theme.binary_bg),
            };

            if line.status == LineStatus::Add || line.status == LineStatus::Remove {
                if critical_line == state.line {
                    style = style.add_modifier(self.theme.selected_modifier);
                }

                critical_line += 1;
            }

            buf.set_style(
                ratatui::prelude::Rect {
                    y: area.y + i as u16,
                    height: 1,
                    ..area
                },
                style,
            );
            text.push_line(Span::styled(&line.content, style));
        }

        text.render(render_area, buf);
    }
}

fn selected_visual_line(lines: &[&crate::model::Line], selected_line: usize) -> Option<usize> {
    let mut critical_line = 0;

    for (visual_line, line) in lines.iter().enumerate() {
        if matches!(line.status, LineStatus::Add | LineStatus::Remove) {
            if critical_line == selected_line {
                return Some(visual_line);
            }

            critical_line += 1;
        }
    }

    None
}
