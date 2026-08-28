use std::{collections::HashMap, path::Path};

#[derive(Debug)]
pub struct Matcher {
    map: HashMap<String, String>,
}

impl Matcher {
    pub fn matches(&self, path: &Path) -> Option<&str> {
        let Some(ext) = path.extension() else {
            return None;
        };

        let Some(ext) = ext.to_str() else { return None };

        self.map.get(ext).map(|l| l.as_str())
    }
}

impl Default for Matcher {
    fn default() -> Self {
        let mut map = HashMap::default();

        map.insert("rs".to_string(), "rust".to_string());
        map.insert("ts".to_string(), "typescript".to_string());

        Self { map }
    }
}
