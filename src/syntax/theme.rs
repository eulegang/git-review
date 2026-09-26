use ratatui::style::Color;
use std::collections::HashMap;
use tree_sitter_highlight::HighlightConfiguration;

#[derive(Default)]
pub struct SyntaxTheme {
    names: Vec<String>,
    colors: Vec<Color>,
}

impl SyntaxTheme {
    pub fn new(mapping: HashMap<String, Color>) -> Self {
        let mut names = Vec::new();
        let mut colors = Vec::new();

        for (name, color) in mapping {
            names.push(name);
            colors.push(color);
        }

        SyntaxTheme { names, colors }
    }

    pub fn resolve(&self, code: usize) -> Color {
        self.colors[code]
    }

    pub fn config(&mut self, config: &mut HighlightConfiguration) {
        config.configure(&self.names);
    }
}
