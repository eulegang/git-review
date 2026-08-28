use super::*;
use std::fmt::Debug;

impl Debug for DiffLoc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.status {
            LineStatus::Add => write!(f, "+{}", self.line),
            LineStatus::Remove => write!(f, "-{}", self.line),
            LineStatus::Context => write!(f, " {}", self.line),
            LineStatus::Binary => write!(f, "binary"),
        }
    }
}
