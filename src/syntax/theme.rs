use ratatui::style::Color;
use tree_sitter_highlight::HighlightConfiguration;

use crate::tui::theme::parse_color;

pub struct SyntaxTheme {
    names: Vec<String>,
    colors: Vec<Color>,
}

impl SyntaxTheme {
    pub fn new(config: &git2::Config) -> SyntaxTheme {
        let mut names = Vec::new();
        let mut colors = Vec::new();

        let Ok(entries) = config.entries(Some("git-review.tree-sitter.*")) else {
            return SyntaxTheme { names, colors };
        };

        let _ = entries.for_each(|e| {
            let (Some(name), Some(value)) = (e.name(), e.value()) else {
                return;
            };

            let name = name
                .strip_prefix("git-review.tree-sitter.")
                .unwrap_or(name)
                .replace("-", ".");

            let color = match parse_color(&value) {
                Ok(color) => color,
                Err(err) => {
                    tracing::error!(?err, "failed to parse treesitter color");

                    return;
                }
            };

            names.push(name);
            colors.push(color);
        });

        tracing::debug!(?names, ?colors, "loaded syntax theme");

        SyntaxTheme { names, colors }
    }

    pub fn resolve(&self, code: usize) -> Color {
        self.colors[code]
    }

    pub fn config(&mut self, config: &mut HighlightConfiguration) {
        config.configure(&self.names);
    }
}
