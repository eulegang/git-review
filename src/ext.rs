use std::path::PathBuf;

pub trait ExpandHome {
    fn expand_home(&self) -> PathBuf;
}

impl ExpandHome for PathBuf {
    fn expand_home(&self) -> PathBuf {
        if let Ok(post) = self.strip_prefix("~/") {
            if let Some(mut base) = dirs::home_dir() {
                base.push(post);
                return base;
            }
        }

        return self.clone();
    }
}
