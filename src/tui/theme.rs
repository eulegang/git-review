use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    pub added_bg: Color,
    pub removed_bg: Color,
    pub selected_added_fg: Option<Color>,
    pub selected_removed_fg: Option<Color>,
    pub selected_added_bg: Option<Color>,
    pub selected_removed_bg: Option<Color>,
    pub binary_bg: Color,
    pub selected_modifier: Modifier,
    pub selector_highlight: Style,
    pub hunk_header: Style,
    pub warning: Style,
}

impl Theme {
    pub const fn new() -> Self {
        Self {
            added_bg: Color::Green,
            removed_bg: Color::Red,
            selected_added_fg: None,
            selected_removed_fg: None,
            selected_added_bg: None,
            selected_removed_bg: None,
            binary_bg: Color::Gray,
            selected_modifier: Modifier::BOLD,
            selector_highlight: Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            hunk_header: Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            warning: Style::new().fg(Color::Yellow),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl crate::TestFixture for Theme {
    fn fixture() -> Self {
        Self::new()
    }
}
