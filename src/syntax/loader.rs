use std::{
    ops::Deref,
    path::{Path, PathBuf},
};

use libloading::Library;
use tree_sitter::Language;
use tree_sitter_highlight::HighlightConfiguration;

type LangFunc = unsafe extern "C" fn() -> *const tree_sitter::ffi::TSLanguage;

pub struct SynExt {
    name: String,
    language: Language,

    highlight: String,
    injections: String,
    locals: String,

    #[allow(dead_code)]
    library: Library,
}

#[derive(Debug)]
pub struct Loader {
    paths: Vec<PathBuf>,
}

impl Loader {
    pub fn load(&self, lang: &str) -> eyre::Result<SynExt> {
        let (language, library) = self.load_language(lang)?;

        let (highlight, injections, locals) = self.load_queries(lang);

        Ok(SynExt {
            name: lang.to_string(),
            language,
            highlight: highlight.unwrap_or_default(),
            injections: injections.unwrap_or_default(),
            locals: locals.unwrap_or_default(),
            library,
        })
    }

    fn load_language(&self, lang: &str) -> eyre::Result<(Language, Library)> {
        tracing::debug!(?lang, dirs = ?self.paths, "trying to load tree-sitter extension");
        for dir in &self.paths {
            for ext in ["so", "dylib", "dll"] {
                let name = format!("{lang}.{ext}");
                let path = dir.join(name);

                tracing::trace!(?lang, ?path, "checking candidate");

                if !path.exists() {
                    continue;
                }

                tracing::debug!(?lang, ?path, "found tree-sitter extension");

                unsafe {
                    let library = Library::new(path)?;

                    let sym = library.get::<LangFunc>(format!("tree_sitter_{lang}"))?;

                    let raw = sym();

                    let language = Language::from_raw(raw);

                    return Ok((language, library));
                }
            }
        }

        Err(eyre::eyre!("failed to load language extension"))
    }

    fn load_queries(&self, lang: &str) -> (Option<String>, Option<String>, Option<String>) {
        tracing::debug!(?lang, dirs = ?self.paths, "trying to load tree-sitter queries");

        for dir in &self.paths {
            let base = dir.join(lang);

            if base.is_dir() {
                let highlight = read_content(&base.join("highlights.scm"));
                let injections = read_content(&base.join("injections.scm"));
                let locals = read_content(&base.join("locals.scm"));

                tracing::debug!(?lang, highlight = ?highlight.is_some(), injections = ?injections.is_some(), locals = ?locals.is_some(), "loaded queries");

                return (highlight, injections, locals);
            }
        }

        tracing::debug!(?lang, "failed to load tree-sitter queries");

        return (None, None, None);

        fn read_content(path: &Path) -> Option<String> {
            if path.is_file() {
                std::fs::read_to_string(path).ok()
            } else {
                None
            }
        }
    }
}

impl Default for Loader {
    fn default() -> Self {
        let Ok(path) = std::env::var("GIT_REVIEW_TREESITTER_PATH") else {
            return Self { paths: vec![] };
        };

        let paths = path.split(":").map(|e| PathBuf::from(e)).collect();

        Loader { paths }
    }
}

impl AsRef<Language> for SynExt {
    fn as_ref(&self) -> &Language {
        &self.language
    }
}

impl Deref for SynExt {
    type Target = Language;

    fn deref(&self) -> &Self::Target {
        &self.language
    }
}

impl SynExt {
    pub fn to_config(&self) -> Option<HighlightConfiguration> {
        match HighlightConfiguration::new(
            self.language.clone(),
            self.name.as_str(),
            &self.highlight,
            &self.injections,
            &self.locals,
        ) {
            Ok(config) => Some(config),
            Err(err) => {
                tracing::error!(?err, "failed to load treesitter highlight config");
                None
            }
        }
    }
}

#[test]
fn load_rust() {
    let loader = Loader::default();
    assert!(loader.load_language("rust").is_ok());
    let (highlights, injections, locals) = loader.load_queries("rust");

    assert!(highlights.is_some());
    assert!(injections.is_some());
    assert!(locals.is_some());
}
