use std::path::Path;

use ratatui::{
    layout::Rect,
    style::Style,
    text::{Span, Text},
    widgets::{StatefulWidget, Widget},
};
use tracing::debug;

use crate::{
    model::{Delta, HunkLine, LineStatus},
    syntax::Syntax,
    tui::Theme,
};

mod window;

#[cfg(test)]
mod unit;

pub struct Diff<'a> {
    pub path: Option<&'a Path>,
    pub selected_entry: usize,

    pub delta: &'a Delta,
    pub hidden_hunks: &'a [usize],

    pub syntax: &'a Syntax,
    pub theme: &'a Theme,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DiffState {
    pub line: usize,
    pub scroll: usize,
    pub center_line: bool,
}

impl Diff<'_> {
    fn effective_bounds(&self, area: Rect) -> Rect {
        let mut len = 0u16;

        let Some(entry) = self.delta.get(self.selected_entry) else {
            return area;
        };

        for hunk in entry.hunks() {
            for line in hunk.lines() {
                len = len.max(line.content().chars().count() as u16);
            }
        }
        let pad = area.width.saturating_sub(len) / 2;

        ratatui::prelude::Rect {
            x: area.x + pad,
            width: area.width - pad,
            ..area
        }
    }

    fn rebalance_state(&self, area: Rect, state: &mut DiffState) {
        let Some(entry) = self.delta.get(self.selected_entry) else {
            return;
        };

        let mut critical_line = 0;

        let mut selected = None::<usize>;
        'check: for hunk in entry.hunks() {
            for (visual_line, line) in hunk.lines().enumerate() {
                if matches!(line.status(), LineStatus::Add | LineStatus::Remove) {
                    if critical_line == state.line {
                        selected = Some(visual_line);
                        break 'check;
                    }

                    critical_line += 1;
                }
            }
        }

        if let Some(selected) = selected {
            let visible_height = area.height as usize;
            if state.center_line {
                state.scroll = selected.saturating_sub(visible_height / 2);
            } else if selected < state.scroll {
                state.scroll = selected;
            } else if selected >= state.scroll.saturating_add(visible_height) {
                state.scroll = selected.saturating_sub(visible_height.saturating_sub(1));
            }
        }

        state.center_line = false;
    }

    fn style_for(&self, line: &HunkLine, selected: bool) -> Style {
        let mut style = match line.status() {
            LineStatus::Add => Style::default().bg(self.theme.added_bg),
            LineStatus::Remove => Style::default().bg(self.theme.removed_bg),
            LineStatus::Context => Style::default(),
            LineStatus::Binary => Style::default().bg(self.theme.binary_bg),
        };

        if selected {
            if let Some(color) = self.theme.selected_bg {
                style = style.bg(color);
            }

            let selected_fg = match line.status() {
                LineStatus::Add => self.theme.selected_added_fg,
                LineStatus::Remove => self.theme.selected_removed_fg,
                _ => None,
            };

            if let Some(color) = selected_fg {
                style = style.fg(color);
            }

            style = style.add_modifier(self.theme.selected_modifier);
        }

        style
    }
}

impl<'a> StatefulWidget for Diff<'a> {
    type State = DiffState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        debug!("rendering diff");
        let Some(entry) = self.delta.get(self.selected_entry) else {
            return;
        };

        let path_height = u16::from(self.path.is_some());
        let diff_area = Rect {
            y: area.y.saturating_add(path_height),
            height: area.height.saturating_sub(path_height),
            ..area
        };
        if diff_area.height == 0 {
            return;
        }

        let render_area = self.effective_bounds(diff_area);
        if let Some(path) = self.path {
            Span::styled(path.display().to_string(), self.theme.hunk_header).render(
                Rect {
                    y: area.y,
                    height: 1,
                    ..render_area
                },
                buf,
            );
        }

        self.rebalance_state(diff_area, state);

        let header_style = self.theme.hunk_header;

        let mut text: Text = Text::default();

        {
            let mut window = window::Window::new(state.scroll, diff_area.height as usize);
            let mut j = 0usize;
            let mut crit = 0;

            'draw: for (i, hunk) in entry.hunks().enumerate() {
                if self.hidden_hunks.contains(&i) {
                    continue;
                }

                if window.visible() {
                    text.push_line(Span::styled(hunk.header().to_string(), header_style));
                    j += 1;
                    window.inc();
                }

                for line in hunk.lines() {
                    let style =
                        self.style_for(&line, line.status().is_critical() && crit == state.line);

                    buf.set_style(
                        ratatui::prelude::Rect {
                            y: diff_area.y + j as u16,
                            height: 1,
                            ..diff_area
                        },
                        style,
                    );

                    if window.visible() {
                        text.push_line(line.highlight());
                        // if let Some(syntax) = syntax.as_mut() {
                        //     text.push_line(line.highlight());
                        // } else {
                        //     let content = line.content().to_string();
                        //     text.push_line(Span::styled(content, style));
                        // }
                        j += 1;
                    }

                    window.inc();

                    if window.fused() {
                        break 'draw;
                    }

                    if line.status().is_critical() {
                        crit += 1;
                    }
                }
            }

            text.render(render_area, buf);
        }
    }
}
