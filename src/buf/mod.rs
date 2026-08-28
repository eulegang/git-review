use std::path::Path;

use git2::{Oid, Repository};
use ratatui::style::Color;

use eyre::Result;

mod color;

#[derive(Default, Clone)]
pub struct Buffer {
    content: Vec<u8>,
    lines: Vec<(usize, usize)>,
    colors: Vec<Vec<(Color, usize, usize)>>,
}

impl Buffer {
    #[cfg(test)]
    pub fn from_test_lines(lines: &[String]) -> Buffer {
        let mut content = Vec::new();
        let mut spans = Vec::new();

        for line in lines {
            let start = content.len();
            content.extend_from_slice(line.as_bytes());
            spans.push((start, line.len()));
            content.push(b'\n');
        }

        Buffer {
            content,
            lines: spans,
            colors: vec![],
        }
    }

    pub fn line(&self, mut line: usize) -> &str {
        line = line.saturating_sub(1);

        let Some((start, len)) = self.lines.get(line) else {
            return "";
        };

        if start + len > self.content.len() {
            return "";
        }

        let content = &self.content[*start..][..*len];

        std::str::from_utf8(content).unwrap_or_default()
    }

    pub fn buf(content: Vec<u8>) -> Result<Buffer> {
        let mut lines = Vec::default();
        let mut last = 0usize;

        for (pos, &ch) in content.iter().enumerate() {
            if ch == b'\n' {
                let len = pos.saturating_sub(last);
                lines.push((last, len));

                last = pos + 1;
            }
        }

        Ok(Buffer {
            content,
            lines,
            colors: vec![],
        })
    }

    pub fn read(path: &Path) -> Result<Buffer> {
        let content = std::fs::read(path)?;
        Buffer::buf(content)
    }

    pub fn load(repo: &Repository, oid: Oid) -> Result<Buffer> {
        if oid.is_zero() {
            return Ok(Buffer::default());
        }

        tracing::trace!(?oid, "finding blob");
        let blob = repo.find_blob(oid)?;
        let content = blob.content();

        Buffer::buf(content.to_vec())
    }

    pub fn take(&mut self) -> Buffer {
        let content = std::mem::take(&mut self.content);
        let lines = std::mem::take(&mut self.lines);
        let colors = std::mem::take(&mut self.colors);

        Buffer {
            content,
            lines,
            colors,
        }
    }
}

impl std::ops::Deref for Buffer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

impl std::fmt::Debug for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut st = f.debug_struct("Buffer");

        st.field("bytes", &self.content.len());
        st.field("lines", &self.lines.len());

        st.finish()
    }
}
