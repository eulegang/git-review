use std::{collections::HashMap, path::Path};

use tree_sitter_highlight::{HighlightConfiguration, Highlighter};

use crate::model::Entry;

pub mod builder;
mod loader;
mod matcher;
pub mod theme;

#[derive(Default)]
pub struct Syntax {
    loader: loader::Loader,
    matcher: matcher::Matcher,
    theme: theme::SyntaxTheme,
    cache: HashMap<String, loader::SynExt>,
}

#[cfg(test)]
impl crate::TestFixture for Syntax {
    fn fixture() -> Self {
        use dirs::home_dir;

        let mut builder = builder::Builder::default();

        let mut path = home_dir().unwrap();
        path.push(".config");
        path.push("git-review");

        builder.add_path(path);
        builder.add_lang("rs".to_string(), "rust".to_string());

        let color = ratatui::style::Color::Black;

        for name in [
            "constructor",
            "constant",
            "float",
            "number",
            "attribute",
            "error",
            "exception",
            "funtion",
            "include",
            "label",
            "operator",
            "parameter",
            "punctuation.deliminator",
            "punctuation.bracket",
            "punctuation.special",
            "symbol",
            "type",
            "tag",
            "text",
            "variable",
            "boolean",
            "constant",
            "comment",
            "conditional",
            "function",
            "method",
            "function",
            "namespace",
            "field",
            "property",
            "keyword",
            "repeat",
            "string",
            "character",
        ] {
            builder.add_color(name.to_string(), color);
        }

        builder.build()
    }
}

impl Syntax {
    pub fn highlight_entry(&mut self, entry: &mut Entry) {
        if entry.old.is_colored() && entry.new.is_colored() {
            return;
        }

        let Some(config) = self.find(&entry.path) else {
            return;
        };

        let mut highlighter = Highlighter::new();

        if !entry.old.is_colored()
            && let Err(err) = entry.old.color(&mut highlighter, &config, &self.theme)
        {
            tracing::error!(?err, "failed to highlight old file");
        }

        if !entry.new.is_colored()
            && let Err(err) = entry.new.color(&mut highlighter, &config, &self.theme)
        {
            tracing::error!(?err, "failed to highlight new file");
        }
    }

    fn find(&mut self, path: &Path) -> Option<HighlightConfiguration> {
        let lang = self.matcher.matches(path)?;

        if self.cache.contains_key(lang) {
            let mut config = self.cache.get(lang).and_then(loader::SynExt::to_config)?;

            self.theme.config(&mut config);
            Some(config)
        } else {
            let ext = match self.loader.load(lang) {
                Ok(lang) => lang,
                Err(err) => {
                    tracing::error!(?err, "failed to load language {lang}");
                    return None;
                }
            };

            self.cache.insert(lang.to_string(), ext);
            let mut config = self.cache.get(lang).and_then(loader::SynExt::to_config)?;
            self.theme.config(&mut config);

            Some(config)
        }
    }
}

impl std::fmt::Debug for Syntax {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Syntax")
            .field("loader", &self.loader)
            .field("matcher", &self.matcher)
            .field("cache", &"...")
            .finish()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::{TestFixture, buf::Buffer, model::Entry};
    use ratatui::text::Line;
    use std::path::PathBuf;

    #[test]
    fn basic_color() -> eyre::Result<()> {
        let mut syntax = Syntax::fixture();
        let mut delta = mock_delta(b"fn main() {\n  println!(\"hello world\");\n}\n".to_vec())?;

        syntax.highlight_entry(&mut delta.entries[0]);

        let buf = delta.entries[0].new.clone();

        assert_eq!(
            line_parts(buf.highlight(1)),
            vec!["fn", " ", "main", "(", ")", " ", "{"]
        );

        assert_eq!(
            line_parts(buf.highlight(2)),
            vec!["  ", "println", "!", "(", "\"hello world\"", ")", ";"]
        );

        assert_eq!(line_parts(buf.highlight(3)), vec!["", "}"]);

        return Ok(());

        fn line_parts(line: Line) -> Vec<String> {
            line.spans
                .iter()
                .map(|s| s.content.to_string())
                .collect::<Vec<_>>()
        }
    }

    #[cfg(test)]
    fn mock_delta(buf: Vec<u8>) -> eyre::Result<crate::model::Delta> {
        let new = Buffer::buf(buf)?;

        let entry = Entry {
            path: PathBuf::from("src/main.rs"),
            old: Buffer::default(),
            new,
            hunks: vec![],
        };

        Ok(crate::model::Delta {
            entries: vec![entry],
        })
    }
}
