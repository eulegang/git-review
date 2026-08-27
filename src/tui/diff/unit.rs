use ratatui::style::{Color, Modifier};

use super::*;
use crate::tui::render_stateful;

fn centered_x(widest_line: &str) -> u16 {
    ((167 - widest_line.chars().count()) / 2) as u16
}

#[test]
fn renders_hunk_headers() {
    let theme = Theme::default();
    let widget = Diff {
        hunks: &[Hunk {
            lines: vec![
                (LineStatus::HunkHeader, "@@ -1 +1 @@").into(),
                (LineStatus::Add, "+added").into(),
            ],
        }],
        hidden_hunks: &[],
        theme: &theme,
    };

    let buf = render_stateful(
        widget,
        DiffState {
            line: 0,
            scroll: 0,
            center_line: false,
        },
    );

    let x = centered_x("@@ -1 +1 @@");

    assert_eq!(buf[(x, 0)].symbol(), "@");
    assert_eq!(buf[(x, 0)].fg, Color::Cyan);
    assert!(buf[(x, 0)].modifier.contains(Modifier::BOLD));
    assert_eq!(buf[(x, 1)].symbol(), "+");
    assert_eq!(buf[(0, 1)].bg, Color::Green);
}

#[test]
fn continues_rendering_across_hunks() {
    let theme = Theme::default();
    let widget = Diff {
        hunks: &[
            Hunk {
                lines: vec![(LineStatus::Add, "+first").into()],
            },
            Hunk {
                lines: vec![(LineStatus::Remove, "-second").into()],
            },
        ],
        hidden_hunks: &[],
        theme: &theme,
    };

    let buf = render_stateful(
        widget,
        DiffState {
            line: 0,
            scroll: 0,
            center_line: false,
        },
    );

    let x = centered_x("-second");

    assert_eq!(buf[(x, 0)].symbol(), "+");
    assert_eq!(buf[(0, 0)].bg, Color::Green);
    assert_eq!(buf[(166, 0)].bg, Color::Green);
    assert_eq!(buf[(x, 1)].symbol(), "-");
    assert_eq!(buf[(0, 1)].bg, Color::Red);
    assert_eq!(buf[(166, 1)].bg, Color::Red);
}

#[test]
fn skips_hidden_hunks() {
    let theme = Theme::default();
    let widget = Diff {
        hunks: &[
            Hunk {
                lines: vec![(LineStatus::Add, "+hidden").into()],
            },
            Hunk {
                lines: vec![(LineStatus::Remove, "-visible").into()],
            },
        ],
        hidden_hunks: &[0],
        theme: &theme,
    };

    let buf = render_stateful(
        widget,
        DiffState {
            line: 0,
            scroll: 0,
            center_line: false,
        },
    );

    let x = centered_x("-visible");

    assert_eq!(buf[(x, 0)].symbol(), "-");
    assert_eq!(buf[(x + 1, 0)].symbol(), "v");
    assert_eq!(buf[(0, 0)].bg, Color::Red);
    assert_eq!(buf[(x, 1)].symbol(), " ");
}

#[test]
fn renders_line_content_and_status_backgrounds() {
    let theme = Theme::default();
    let widget = Diff {
        hunks: &[Hunk {
            lines: vec![
                (LineStatus::Context, " context").into(),
                (LineStatus::Add, "+added").into(),
                (LineStatus::Remove, "-removed").into(),
                (LineStatus::Binary, "binary").into(),
            ],
        }],
        hidden_hunks: &[],
        theme: &theme,
    };

    let buf = render_stateful(
        widget,
        DiffState {
            line: 99,
            scroll: 0,
            center_line: false,
        },
    );

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
    let mut theme = Theme::default();
    theme.selected_removed_fg = Some(Color::Yellow);
    let widget = Diff {
        hunks: &[Hunk {
            lines: vec![
                (LineStatus::Context, " context").into(),
                (LineStatus::Add, "+added").into(),
                (LineStatus::Binary, "binary").into(),
                (LineStatus::Remove, "-removed").into(),
            ],
        }],
        hidden_hunks: &[],
        theme: &theme,
    };

    let buf = render_stateful(
        widget,
        DiffState {
            line: 1,
            scroll: 0,
            center_line: false,
        },
    );

    let x = centered_x("-removed");

    assert_eq!(buf[(x, 1)].bg, Color::Green);
    assert!(!buf[(x, 1)].modifier.contains(Modifier::BOLD));
    assert!(!buf[(x, 1)].modifier.contains(Modifier::REVERSED));

    assert_eq!(buf[(x, 2)].bg, Color::Gray);
    assert!(!buf[(x, 2)].modifier.contains(Modifier::BOLD));
    assert!(!buf[(x, 2)].modifier.contains(Modifier::REVERSED));

    assert_eq!(buf[(x, 3)].bg, Color::Red);
    assert_eq!(buf[(x, 3)].fg, Color::Yellow);
    assert!(buf[(x, 3)].modifier.contains(Modifier::BOLD));
    assert!(buf[(x, 3)].modifier.contains(Modifier::REVERSED));
}

#[test]
fn applies_selected_added_foreground() {
    let mut theme = Theme::default();
    theme.selected_added_fg = Some(Color::Blue);
    theme.selected_bg = Some(Color::White);
    let widget = Diff {
        hunks: &[Hunk {
            lines: vec![(LineStatus::Add, "+added").into()],
        }],
        hidden_hunks: &[],
        theme: &theme,
    };

    let buf = render_stateful(
        widget,
        DiffState {
            line: 0,
            scroll: 0,
            center_line: false,
        },
    );

    let x = centered_x("+added");

    assert_eq!(buf[(x, 0)].fg, Color::Blue);
    assert_eq!(buf[(x, 0)].bg, Color::White);
    assert!(buf[(x, 0)].modifier.contains(Modifier::BOLD));
}

#[test]
fn scrolls_to_keep_selected_line_visible() {
    let theme = Theme::default();
    let lines = (0..50)
        .map(|i| crate::model::Line {
            status: LineStatus::Add,
            content: format!("+line {i}"),
        })
        .collect();
    let widget = Diff {
        hunks: &[Hunk { lines }],
        hidden_hunks: &[],
        theme: &theme,
    };

    let state = DiffState {
        line: 49,
        scroll: 0,
        center_line: false,
    };
    let buf = render_stateful(widget, state);

    let x = centered_x("+line 10");

    assert_eq!(buf[(x, 0)].symbol(), "+");
    assert_eq!(buf[(x + 6, 0)].symbol(), "1");
    assert_eq!(buf[(x + 7, 0)].symbol(), "2");
    assert!(buf[(x, 37)].modifier.contains(Modifier::BOLD));
}

#[test]
fn centers_selected_line_even_at_file_end() {
    let theme = Theme::default();
    let lines = (0..50)
        .map(|i| crate::model::Line {
            status: LineStatus::Add,
            content: format!("+line {i}"),
        })
        .collect();
    let widget = Diff {
        hunks: &[Hunk { lines }],
        hidden_hunks: &[],
        theme: &theme,
    };

    let state = DiffState {
        line: 49,
        scroll: 0,
        center_line: true,
    };
    let buf = render_stateful(widget, state);

    let x = centered_x("+line 10");

    assert_eq!(buf[(x, 19)].symbol(), "+");
    assert_eq!(buf[(x + 6, 19)].symbol(), "4");
    assert_eq!(buf[(x + 7, 19)].symbol(), "9");
    assert!(buf[(x, 19)].modifier.contains(Modifier::BOLD));
}
