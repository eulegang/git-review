use std::{collections::HashMap, path::PathBuf};

use ratatui::style::Color;

use crate::syntax::{loader, matcher, theme};

#[derive(Default, Clone)]
pub struct Builder {
    paths: Vec<PathBuf>,
    langs: HashMap<String, String>,
    theme: HashMap<String, Color>,
}

impl Builder {
    pub fn add_path(&mut self, path: PathBuf) {
        self.paths.push(path);
    }

    pub fn add_lang(&mut self, ext: String, lang: String) {
        self.langs.insert(ext, lang);
    }

    pub fn add_color(&mut self, name: String, color: Color) {
        self.theme.insert(name, color);
    }

    pub fn build(self) -> super::Syntax {
        super::Syntax {
            loader: loader::Loader::new(self.paths),
            cache: HashMap::default(),
            matcher: matcher::Matcher::new(self.langs),
            theme: theme::SyntaxTheme::new(self.theme),
        }
    }
}
