use std::path::Path;

#[derive(Default, Debug, Clone, Copy)]
pub enum FilterType {
    #[default]
    Glob,
    Regex,
    Fuzzy,
}

pub struct Filter {
    pattern: String,
    engine: Engine,
    inverted: bool,
}

impl Default for Filter {
    fn default() -> Self {
        Self {
            pattern: Default::default(),
            engine: Default::default(),
            inverted: false,
        }
    }
}

#[derive(Default)]
enum Engine {
    #[default]
    None,
    Glob(globset::GlobSet),
    Regex(regex::Regex),
    Fuzzy(frizbee::Matcher),
}

impl TryFrom<(FilterType, String)> for Filter {
    type Error = eyre::Report;

    fn try_from((ftype, pattern): (FilterType, String)) -> Result<Self, Self::Error> {
        Filter::try_from((ftype, pattern, false))
    }
}

impl TryFrom<(FilterType, String, bool)> for Filter {
    type Error = eyre::Report;

    fn try_from(
        (ftype, pattern, inverted): (FilterType, String, bool),
    ) -> Result<Self, Self::Error> {
        if pattern.is_empty() {
            return Ok(Filter {
                pattern,
                engine: Engine::None,
                inverted: false,
            });
        }

        let engine = match ftype {
            FilterType::Glob => {
                let mut builder = globset::GlobSetBuilder::new();
                builder.add(globset::Glob::new(&pattern)?);

                Engine::Glob(builder.build()?)
            }

            FilterType::Regex => {
                let re = regex::Regex::new(&pattern)?;

                Engine::Regex(re)
            }

            FilterType::Fuzzy => {
                let matcher = frizbee::Matcher::from_query(&pattern, &frizbee::Config::default());

                Engine::Fuzzy(matcher)
            }
        };

        Ok(Filter {
            pattern,
            engine,
            inverted,
        })
    }
}

impl Filter {
    pub fn accepts(&self, path: &Path) -> bool {
        let repr = path.display().to_string();

        let accepted = match &self.engine {
            Engine::None => true,
            Engine::Glob(glob_set) => glob_set.is_match(path),
            Engine::Regex(regex) => regex.is_match(&repr),
            Engine::Fuzzy(matcher) => matcher.clone().match_one(&repr, 0).is_some(),
        };

        accepted ^ self.inverted
    }

    pub fn repr(&self) -> &str {
        &self.pattern
    }

    pub fn inverted(&self) -> bool {
        self.inverted
    }
}

impl std::fmt::Debug for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pattern = &self.pattern;
        let prefix = if self.inverted { "!" } else { "" };
        match self.engine {
            Engine::None => write!(f, "None"),
            Engine::Glob(_) => write!(f, "{prefix}Glob({pattern})"),
            Engine::Regex(_) => write!(f, "{prefix}Regex({pattern})"),
            Engine::Fuzzy(_) => write!(f, "{prefix}Fuzzy({pattern})"),
        }
    }
}
