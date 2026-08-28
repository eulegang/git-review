use std::{collections::HashMap, path::Path};

use tracing::error;
use tree_sitter_highlight::{HighlightConfiguration, Highlighter};

use crate::{model::Delta, syntax::theme::SyntaxTheme};

mod loader;
mod matcher;
pub mod theme;

pub struct Syntax {
    loader: loader::Loader,
    matcher: matcher::Matcher,
    theme: theme::SyntaxTheme,
    cache: HashMap<String, loader::SynExt>,
}

impl Syntax {
    pub fn new(config: &git2::Config) -> Syntax {
        let loader = loader::Loader::default();
        let matcher = matcher::Matcher::default();
        let cache = HashMap::default();
        let theme = SyntaxTheme::new(config);

        Syntax {
            loader,
            matcher,
            theme,
            cache,
        }
    }

    pub fn highlight(&mut self, delta: &mut Delta) {
        let mut highlighter = Highlighter::new();

        for entry in &mut delta.entries {
            let Some(config) = self.find(&entry.path) else {
                continue;
            };

            if let Err(err) = entry.old.color(&mut highlighter, &config, &self.theme) {
                tracing::error!(?err, "failed to highlight old file");
            }

            if let Err(err) = entry.new.color(&mut highlighter, &config, &self.theme) {
                tracing::error!(?err, "failed to highlight old file");
            }
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
                    error!(?err, "failed to load language {lang}");
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
    use std::path::PathBuf;

    use git2::Repository;
    use ratatui::text::Line;

    use crate::{buf::Buffer, model::Entry};

    use super::*;

    #[test]
    fn basic_color() -> eyre::Result<()> {
        tracing_subscriber::fmt()
            .pretty()
            .with_max_level(tracing::Level::TRACE)
            .with_level(true)
            .init();

        let repo = Repository::discover(".")?;
        let config = repo.config()?;

        let mut syntax = Syntax::new(&config);
        let mut delta = mock_delta(b"fn main() {\n  println!(\"hello world\");\n}\n".to_vec())?;

        syntax.highlight(&mut delta);

        let buf = delta.entries[0].new.clone();

        assert_eq!(
            line_parts(buf.highlight(1)),
            vec!["fn", " ", "main", "(", ")", " ", "{"]
        );

        assert_eq!(
            line_parts(buf.highlight(2)),
            vec!["  ", "println", "!", "(", "\"hello world\"", ")", ";"]
        );

        assert_eq!(line_parts(buf.highlight(3)), vec!["}"]);

        return Ok(());

        fn line_parts(line: Line) -> Vec<String> {
            line.spans
                .iter()
                .map(|s| s.content.to_string())
                .collect::<Vec<_>>()
        }
    }

    fn mock_delta(buf: Vec<u8>) -> eyre::Result<Delta> {
        let new = Buffer::buf(buf)?;

        let entry = Entry {
            path: PathBuf::from("src/main.rs"),
            old: Buffer::default(),
            new,
            hunks: vec![],
        };

        Ok(Delta {
            entries: vec![entry],
        })
    }
}
