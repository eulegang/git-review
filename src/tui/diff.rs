use ratatui::{
    style::{Color, Modifier, Style},
    text::{Span, Text},
    widgets::{StatefulWidget, Widget},
};

use crate::model::{Hunk, LineStatus};

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

#[cfg(test)]
mod tests {
    use ratatui::style::{Color, Modifier};

    use super::*;
    use crate::tui::render_stateful;

    #[test]
    fn renders_line_content_and_status_backgrounds() {
        let widget = Diff {
            hunks: &[Hunk {
                lines: vec![
                    (LineStatus::Context, " context").into(),
                    (LineStatus::Add, "+added").into(),
                    (LineStatus::Remove, "-removed").into(),
                    (LineStatus::Binary, "binary").into(),
                ],
            }],
        };

        let buf = render_stateful(widget, 99);

        assert_eq!(buf[(0, 0)].symbol(), " ");
        assert_eq!(buf[(1, 0)].symbol(), "c");
        assert_eq!(buf[(0, 0)].bg, Color::Reset);

        assert_eq!(buf[(0, 1)].symbol(), "+");
        assert_eq!(buf[(0, 1)].bg, Color::Green);

        assert_eq!(buf[(0, 2)].symbol(), "-");
        assert_eq!(buf[(0, 2)].bg, Color::Red);

        assert_eq!(buf[(0, 3)].symbol(), "b");
        assert_eq!(buf[(0, 3)].bg, Color::Gray);
    }

    #[test]
    fn highlights_selected_added_or_removed_line_only() {
        let widget = Diff {
            hunks: &[Hunk {
                lines: vec![
                    (LineStatus::Context, " context").into(),
                    (LineStatus::Add, "+added").into(),
                    (LineStatus::Binary, "binary").into(),
                    (LineStatus::Remove, "-removed").into(),
                ],
            }],
        };

        let buf = render_stateful(widget, 1);

        assert_eq!(buf[(0, 1)].bg, Color::Green);
        assert!(!buf[(0, 1)].modifier.contains(Modifier::BOLD));
        assert!(!buf[(0, 1)].modifier.contains(Modifier::REVERSED));

        assert_eq!(buf[(0, 2)].bg, Color::Gray);
        assert!(!buf[(0, 2)].modifier.contains(Modifier::BOLD));
        assert!(!buf[(0, 2)].modifier.contains(Modifier::REVERSED));

        assert_eq!(buf[(0, 3)].bg, Color::Red);
        assert!(buf[(0, 3)].modifier.contains(Modifier::BOLD));
        assert!(buf[(0, 3)].modifier.contains(Modifier::REVERSED));
    }

    #[test]
    fn continues_rendering_across_hunks() {
        let widget = Diff {
            hunks: &[
                Hunk {
                    lines: vec![(LineStatus::Add, "+first").into()],
                },
                Hunk {
                    lines: vec![(LineStatus::Remove, "-second").into()],
                },
            ],
        };

        let buf = render_stateful(widget, 0);

        assert_eq!(buf[(0, 0)].symbol(), "+");
        assert_eq!(buf[(0, 0)].bg, Color::Green);
        assert_eq!(buf[(0, 1)].symbol(), "-");
        assert_eq!(buf[(0, 1)].bg, Color::Red);
    }
}
