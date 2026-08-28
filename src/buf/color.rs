use ratatui::{
    style::Style,
    text::{Line, Span},
};
use tree_sitter_highlight::{Highlight, HighlightConfiguration, HighlightEvent, Highlighter};

use crate::syntax::theme::SyntaxTheme;

use super::*;

impl Buffer {
    pub fn highlight(&self, index: usize) -> Line<'static> {
        let index = index.saturating_sub(1);

        let Some((start, len)) = self.lines.get(index) else {
            return Line::default();
        };

        if start + len > self.content.len() {
            return Line::default();
        }

        let content = &self.content[*start..][..*len];
        let Ok(content) = std::str::from_utf8(content) else {
            return Line::default();
        };

        let Some(colors) = self.colors.get(index) else {
            return Line::from(content.to_string());
        };

        let mut line: Line<'static> = Line::default();

        for (color, offset, offlen) in colors {
            let offset = *offset;
            let offlen = *offlen;
            let color = *color;
            tracing::trace!(?color, offset = ?offset, offlen = ?offlen, ?len, line = ?content, start = ?start, "coloring line");

            let cap = len.saturating_sub(offset);

            if offset > content.len() {
                break;
            }

            line.push_span(Span::styled(
                content[offset..][..(offlen).min(cap)].to_string(),
                Style::default().fg(color),
            ));
        }

        line
    }

    pub fn color(
        &mut self,
        highlighter: &mut Highlighter,
        hiconfig: &HighlightConfiguration,
        theme: &SyntaxTheme,
    ) -> Result<()> {
        let iter = highlighter.highlight(&hiconfig, &self.content, None, |_lang| None)?;
        let mut colors = Vec::<Color>::new();
        colors.push(Color::default());

        self.colors.push(vec![]);
        for event in iter {
            let event = event?;

            match event {
                HighlightEvent::Source { start, end } => {
                    let split = self.content[start..end].contains(&b'\n');
                    tracing::trace!(?start, ?end, ?split, content = ?std::str::from_utf8(&self.content[start..end]), "found tree-sitter span");

                    let (line, _) = self.lines[self.colors.len() - 1];
                    let s = start - line;
                    let e = end - start;

                    if e == 0 {
                        continue;
                    }

                    if !self.content[start..end].contains(&b'\n') {
                        self.colors
                            .last_mut()
                            .unwrap()
                            .push((colors[colors.len() - 1], s, e));
                    } else {
                        let mut iter = self.content[start..end].split(|b| *b == b'\n');

                        if let Some(line) = iter.next()
                            && !line.is_empty()
                        {
                            self.colors.last_mut().unwrap().push((
                                colors[colors.len() - 1],
                                s,
                                line.len(),
                            ));
                        }

                        while let Some(line) = iter.next() {
                            self.colors
                                .push(vec![(colors[colors.len() - 1], 0, line.len())]);
                        }
                    }
                }

                HighlightEvent::HighlightStart(Highlight(code)) => colors.push(theme.resolve(code)),
                HighlightEvent::HighlightEnd => {
                    colors.pop();
                }
            }
        }

        tracing::debug!(color = ?self.colors, "loaded colors");

        Ok(())
    }
}
