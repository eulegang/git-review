use ratatui::style::{Color, Modifier};

use super::*;
use crate::{
    model::{Delta, LineStatus},
    syntax::Syntax,
    tui::render_stateful,
};

fn centered_x(widest_line: &str) -> u16 {
    ((167 - widest_line.chars().count()) / 2) as u16
}

fn config() -> git2::Config {
    let path = std::env::temp_dir().join(format!(
        "git-review-diff-test-{}-{}.config",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_file(&path);

    git2::Config::open(&path).expect("open test config")
}

fn syntax() -> Syntax {
    Syntax::new(&config())
}

fn widget<'a>(
    delta: &'a Delta,
    hidden_hunks: &'a [usize],
    theme: &'a Theme,
    syntax: &'a Syntax,
) -> Diff<'a> {
    Diff {
        path: delta.get(0).map(|entry| entry.path.as_path()),
        selected_entry: 0,
        delta,
        hidden_hunks,
        syntax,
        theme,
    }
}

#[test]
fn renders_hunk_headers() {
    let theme = Theme::default();
    let syntax = syntax();
    let delta = Delta::from_test_hunks(vec![(
        "@@ -1 +1 @@",
        vec![(LineStatus::Add, "+added".to_string())],
    )]);

    let buf = render_stateful(
        widget(&delta, &[], &theme, &syntax),
        DiffState {
            line: 0,
            scroll: 0,
            center_line: false,
        },
    );

    let x = centered_x("+added");

    assert_eq!(buf[(x, 0)].symbol(), "t");
    assert_eq!(buf[(x + 1, 0)].symbol(), "e");
    assert_eq!(buf[(x, 1)].symbol(), "@");
    assert_eq!(buf[(x, 1)].fg, Color::Cyan);
    assert!(buf[(x, 1)].modifier.contains(Modifier::BOLD));
    assert_eq!(buf[(x, 2)].symbol(), "+");
    assert_eq!(buf[(0, 2)].bg, Color::Green);
}

#[test]
fn continues_rendering_across_hunks() {
    let theme = Theme::default();
    let syntax = syntax();
    let delta = Delta::from_test_hunks(vec![
        ("@@ -1 +1 @@", vec![(LineStatus::Add, "+first".to_string())]),
        (
            "@@ -2 +2 @@",
            vec![(LineStatus::Remove, "-second".to_string())],
        ),
    ]);

    let buf = render_stateful(
        widget(&delta, &[], &theme, &syntax),
        DiffState {
            line: 0,
            scroll: 0,
            center_line: false,
        },
    );

    let x = centered_x("-second");

    assert_eq!(buf[(x, 2)].symbol(), "+");
    assert_eq!(buf[(0, 2)].bg, Color::Green);
    assert_eq!(buf[(166, 2)].bg, Color::Green);
    assert_eq!(buf[(x, 4)].symbol(), "-");
    assert_eq!(buf[(0, 4)].bg, Color::Red);
    assert_eq!(buf[(166, 4)].bg, Color::Red);
}

#[test]
fn skips_hidden_hunks() {
    let theme = Theme::default();
    let syntax = syntax();
    let hidden = [0];
    let delta = Delta::from_test_hunks(vec![
        (
            "@@ -1 +1 @@",
            vec![(LineStatus::Add, "+hidden".to_string())],
        ),
        (
            "@@ -2 +2 @@",
            vec![(LineStatus::Remove, "-visible".to_string())],
        ),
    ]);

    let buf = render_stateful(
        widget(&delta, &hidden, &theme, &syntax),
        DiffState {
            line: 0,
            scroll: 0,
            center_line: false,
        },
    );

    let x = centered_x("-visible");

    assert_eq!(buf[(x, 1)].symbol(), "@");
    assert_eq!(buf[(x, 2)].symbol(), "-");
    assert_eq!(buf[(0, 2)].bg, Color::Red);
    assert_eq!(buf[(x, 3)].symbol(), " ");
}

#[test]
fn renders_line_content_and_status_backgrounds() {
    let theme = Theme::default();
    let syntax = syntax();
    let delta = Delta::from_test_hunks(vec![(
        "@@ -1 +1 @@",
        vec![
            (LineStatus::Context, " context".to_string()),
            (LineStatus::Add, "+added".to_string()),
            (LineStatus::Remove, "-removed".to_string()),
            (LineStatus::Binary, "binary".to_string()),
        ],
    )]);

    let buf = render_stateful(
        widget(&delta, &[], &theme, &syntax),
        DiffState {
            line: 99,
            scroll: 0,
            center_line: false,
        },
    );

    let x = centered_x("-removed");

    assert_eq!(buf[(x, 2)].symbol(), " ");
    assert_eq!(buf[(x + 1, 2)].symbol(), "c");
    assert_eq!(buf[(x, 2)].bg, Color::Reset);

    assert_eq!(buf[(x, 3)].symbol(), "+");
    assert_eq!(buf[(0, 3)].bg, Color::Green);
    assert_eq!(buf[(166, 3)].bg, Color::Green);

    assert_eq!(buf[(x, 4)].symbol(), "-");
    assert_eq!(buf[(0, 4)].bg, Color::Red);
    assert_eq!(buf[(166, 4)].bg, Color::Red);

    assert_eq!(buf[(x, 5)].symbol(), " ");
    assert_eq!(buf[(0, 5)].bg, Color::Gray);
    assert_eq!(buf[(166, 5)].bg, Color::Gray);
}

#[test]
fn highlights_selected_added_or_removed_line_only() {
    let mut theme = Theme::default();
    theme.selected_removed_fg = Some(Color::Yellow);
    let syntax = syntax();
    let delta = Delta::from_test_hunks(vec![(
        "@@ -1 +1 @@",
        vec![
            (LineStatus::Context, " context".to_string()),
            (LineStatus::Add, "+added".to_string()),
            (LineStatus::Binary, "binary".to_string()),
            (LineStatus::Remove, "-removed".to_string()),
        ],
    )]);

    let buf = render_stateful(
        widget(&delta, &[], &theme, &syntax),
        DiffState {
            line: 1,
            scroll: 0,
            center_line: false,
        },
    );

    let x = centered_x("-removed");

    assert_eq!(buf[(x, 3)].bg, Color::Green);
    assert!(!buf[(x, 3)].modifier.contains(Modifier::BOLD));
    assert!(!buf[(x, 3)].modifier.contains(Modifier::REVERSED));

    assert_eq!(buf[(x, 4)].bg, Color::Gray);
    assert!(!buf[(x, 4)].modifier.contains(Modifier::BOLD));
    assert!(!buf[(x, 4)].modifier.contains(Modifier::REVERSED));

    assert_eq!(buf[(x, 5)].bg, Color::Red);
    assert_eq!(buf[(x, 5)].fg, Color::Yellow);
    assert!(buf[(x, 5)].modifier.contains(Modifier::BOLD));
}

#[test]
fn applies_selected_added_foreground() {
    let mut theme = Theme::default();
    theme.selected_added_fg = Some(Color::Blue);
    theme.selected_bg = Some(Color::White);
    let syntax = syntax();
    let delta = Delta::from_test_hunks(vec![(
        "@@ -1 +1 @@",
        vec![(LineStatus::Add, "+added".to_string())],
    )]);

    let buf = render_stateful(
        widget(&delta, &[], &theme, &syntax),
        DiffState {
            line: 0,
            scroll: 0,
            center_line: false,
        },
    );

    let x = centered_x("+added");

    assert_eq!(buf[(x, 2)].fg, Color::Blue);
    assert_eq!(buf[(x, 2)].bg, Color::White);
    assert!(buf[(x, 2)].modifier.contains(Modifier::BOLD));
}

#[test]
fn syntax_highlights_rust_diff_lines() {
    let theme = Theme::default();
    let mut config = config();
    config
        .set_str("git-review.tree-sitter.keyword", "magenta")
        .expect("set syntax color");
    let mut syntax = Syntax::new(&config);
    let mut delta = Delta::from_test_hunks(vec![(
        "@@ -1 +1 @@",
        vec![(LineStatus::Add, "fn main() {}".to_string())],
    )]);
    syntax.highlight(&mut delta);

    let buf = render_stateful(
        widget(&delta, &[], &theme, &syntax),
        DiffState {
            line: 99,
            scroll: 0,
            center_line: false,
        },
    );

    let x = centered_x("fn main() {}");

    assert_eq!(buf[(x, 2)].symbol(), "f");
    assert_eq!(buf[(x, 2)].fg, Color::Magenta);
}

#[test]
fn scrolls_to_keep_selected_line_visible() {
    let theme = Theme::default();
    let syntax = syntax();
    let lines = (0..50)
        .map(|i| (LineStatus::Add, format!("+line {i}")))
        .collect();
    let delta = Delta::from_test_hunks(vec![("@@ -1 +1 @@", lines)]);

    let state = DiffState {
        line: 49,
        scroll: 0,
        center_line: false,
    };
    let buf = render_stateful(widget(&delta, &[], &theme, &syntax), state);

    let x = centered_x("+line 10");

    assert_eq!(buf[(x, 1)].symbol(), "+");
    assert_eq!(buf[(x + 6, 1)].symbol(), "1");
    assert_eq!(buf[(x + 7, 1)].symbol(), "3");
    assert!(buf[(x, 37)].modifier.contains(Modifier::BOLD));
}

#[test]
fn centers_selected_line_even_at_file_end() {
    let theme = Theme::default();
    let syntax = syntax();
    let lines = (0..50)
        .map(|i| (LineStatus::Add, format!("+line {i}")))
        .collect();
    let delta = Delta::from_test_hunks(vec![("@@ -1 +1 @@", lines)]);

    let state = DiffState {
        line: 49,
        scroll: 0,
        center_line: true,
    };
    let buf = render_stateful(widget(&delta, &[], &theme, &syntax), state);

    let x = centered_x("+line 10");

    assert_eq!(buf[(x, 19)].symbol(), "+");
    assert_eq!(buf[(x + 6, 19)].symbol(), "4");
    assert_eq!(buf[(x + 7, 19)].symbol(), "9");
    assert!(buf[(x, 19)].modifier.contains(Modifier::BOLD));
}
