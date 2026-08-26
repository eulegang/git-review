use ratatui::style::{Color, Modifier};

use super::*;
use crate::tui::render_stateful;

fn centered_x(widest_line: &str) -> u16 {
    ((167 - widest_line.chars().count()) / 2) as u16
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

    let x = centered_x("-second");

    assert_eq!(buf[(x, 0)].symbol(), "+");
    assert_eq!(buf[(0, 0)].bg, Color::Green);
    assert_eq!(buf[(166, 0)].bg, Color::Green);
    assert_eq!(buf[(x, 1)].symbol(), "-");
    assert_eq!(buf[(0, 1)].bg, Color::Red);
    assert_eq!(buf[(166, 1)].bg, Color::Red);
}

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

    let x = centered_x("-removed");

    assert_eq!(buf[(x, 0)].symbol(), " ");
    assert_eq!(buf[(x + 1, 0)].symbol(), "c");
    assert_eq!(buf[(x, 0)].bg, Color::Reset);

    assert_eq!(buf[(x, 1)].symbol(), "+");
    assert_eq!(buf[(0, 1)].bg, Color::Green);
    assert_eq!(buf[(166, 1)].bg, Color::Green);

    assert_eq!(buf[(x, 2)].symbol(), "-");
    assert_eq!(buf[(0, 2)].bg, Color::Red);
    assert_eq!(buf[(166, 2)].bg, Color::Red);

    assert_eq!(buf[(x, 3)].symbol(), "b");
    assert_eq!(buf[(0, 3)].bg, Color::Gray);
    assert_eq!(buf[(166, 3)].bg, Color::Gray);
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

    let x = centered_x("-removed");

    assert_eq!(buf[(x, 1)].bg, Color::Green);
    assert!(!buf[(x, 1)].modifier.contains(Modifier::BOLD));
    assert!(!buf[(x, 1)].modifier.contains(Modifier::REVERSED));

    assert_eq!(buf[(x, 2)].bg, Color::Gray);
    assert!(!buf[(x, 2)].modifier.contains(Modifier::BOLD));
    assert!(!buf[(x, 2)].modifier.contains(Modifier::REVERSED));

    assert_eq!(buf[(x, 3)].bg, Color::Red);
    assert!(buf[(x, 3)].modifier.contains(Modifier::BOLD));
    assert!(buf[(x, 3)].modifier.contains(Modifier::REVERSED));
}
