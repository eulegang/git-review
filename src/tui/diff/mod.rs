use ratatui::{
    style::{Color, Modifier, Style},
    text::{Span, Text},
    widgets::{StatefulWidget, Widget},
};

use crate::model::{Hunk, LineStatus};

#[cfg(test)]
mod unit;

pub struct Diff<'a> {
    pub hunks: &'a [Hunk],
}

impl<'a> StatefulWidget for Diff<'a> {
    type State = usize;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let mut text: Text = Text::default();

        let mut hunk_id = 0;
        let mut line_id = 0;
        let mut critical_line = 0;
        'base: for i in 0.. {
            if i > area.height {
                break;
            }

            let line = loop {
                let Some(hunk) = self.hunks.get(hunk_id) else {
                    break 'base;
                };

                let Some(line) = hunk.lines.get(line_id) else {
                    hunk_id += 1;
                    line_id = 0;

                    continue;
                };

                break line;
            };

            let mut style = match line.status {
                LineStatus::Add => Style::default().bg(Color::Green),
                LineStatus::Remove => Style::default().bg(Color::Red),
                LineStatus::Context => Style::default(),
                LineStatus::Binary => Style::default().bg(Color::Gray),
            };

            if line.status == LineStatus::Add || line.status == LineStatus::Remove {
                if critical_line == *state {
                    style = style
                        .add_modifier(Modifier::BOLD)
                        .add_modifier(Modifier::REVERSED);
                }

                critical_line += 1;
            }

            text.push_line(Span::styled(&line.content, style));

            line_id += 1;
        }

        text.render(area, buf);
    }
}
