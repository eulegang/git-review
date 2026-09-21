use std::{collections::BTreeSet, io};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use diff::{Diff, DiffState};
use eyre::{Context, Result};
use file_selector::FileSelector;
use filter_preview::FilterPreview;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Position},
    widgets::{Paragraph, Widget},
};
use text_input::TextInput;

use action::Mode;

use crate::{
    eventing::{self, AppEvent},
    filter::{Filter, FilterType},
    model::{Delta, Entry},
    syntax::Syntax,
};

mod action;
mod diff;
mod file_selector;
mod filter_preview;
mod input;
mod text_input;
pub mod theme;

pub use theme::Theme;

#[derive(Debug)]
pub struct App {
    model: Delta,
    syntax: Syntax,
    selected_file: usize,
    selector_file: usize,
    mode: Mode,
    input_text: String,
    file_filter_type: FilterType,
    file_filter: Filter,
    invert: bool,
    line: usize,
    scroll: usize,
    hidden_hunks: Vec<BTreeSet<usize>>,
    center_line: bool,
    should_quit: bool,
    theme: Theme,
}

impl App {
    pub fn new(model: Delta, syntax: Syntax, theme: Theme) -> Self {
        let len = model.len();

        Self {
            model,
            syntax,
            selected_file: 0,
            selector_file: 0,
            mode: Mode::Diff,
            input_text: String::new(),
            file_filter_type: FilterType::default(),
            file_filter: Filter::default(),
            invert: false,
            line: 0,
            scroll: 0,
            hidden_hunks: vec![BTreeSet::new(); len],
            center_line: false,
            should_quit: false,
            theme,
        }
    }

    fn jump_to_selected_file(&mut self) {
        self.line = 0;
        self.scroll = 0;
    }

    fn entry_matches_file_filter(&self, entry: &Entry) -> bool {
        self.file_filter.accepts(&entry.path)
    }

    fn highlight_selected_file(&mut self) {
        if let Some(entry) = self.model.get_mut(self.selected_file) {
            self.syntax.highlight_entry(entry);
        }
    }

    fn current_file_line_count(&self) -> usize {
        self.model
            .get(self.selected_file)
            .map(|e| {
                e.hunks()
                    .enumerate()
                    .filter(|(index, _)| !self.hunk_is_hidden(*index))
                    .map(|(_, h)| h.critical())
                    .sum::<usize>()
            })
            .unwrap_or_default()
    }

    fn hunk_is_hidden(&self, hunk: usize) -> bool {
        self.hidden_hunks
            .get(self.selected_file)
            .is_some_and(|hidden| hidden.contains(&hunk))
    }

    fn current_hunk(&self) -> Option<usize> {
        let entry = self.model.get(self.selected_file)?;
        let mut first_line = 0;

        for (index, hunk) in entry.hunks().enumerate() {
            if self.hunk_is_hidden(index) {
                continue;
            }

            let line_count = hunk.critical();
            if line_count == 0 {
                continue;
            }

            if self.line < first_line + line_count {
                return Some(index);
            }

            first_line += line_count;
        }

        None
    }

    pub fn run(&mut self) -> Result<()> {
        enable_raw_mode().context("failed to enable raw terminal mode")?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
            .context("failed to enter alternate screen")?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).context("failed to initialize terminal")?;

        let result = self.main_loop(&mut terminal);

        disable_raw_mode().context("failed to disable raw terminal mode")?;
        execute!(
            terminal.backend_mut(),
            DisableMouseCapture,
            LeaveAlternateScreen
        )
        .context("failed to leave alternate screen")?;
        terminal.show_cursor().context("failed to show cursor")?;

        result
    }

    fn main_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        let events = eventing::install();
        let input = input::InputThread::spawn()?;

        terminal.draw(|frame| render(frame, self))?;

        while !self.should_quit {
            let app_event = events.receiver().recv().context("event channel closed")?;
            tracing::trace!(?app_event, "Processing event");
            let needs_redraw = match app_event {
                AppEvent::Key(key) => {
                    if let Ok(action) = self.mode.action_for_key(key) {
                        self.execute(action);
                        true
                    } else {
                        false
                    }
                }
                AppEvent::Mouse(mouse) => {
                    if let Ok(action) = self.mode.action_for_mouse(mouse) {
                        self.execute(action);
                        true
                    } else {
                        false
                    }
                }
                AppEvent::Redraw => true,
            };

            if needs_redraw {
                terminal.draw(|frame| render(frame, self))?;
            }
        }

        input.close()?;

        Ok(())
    }
}

fn render(frame: &mut ratatui::Frame<'_>, app: &mut App) {
    let area = frame.area();

    if app.model.len() == 0 {
        let warning = Paragraph::new("No changes to review")
            .alignment(Alignment::Center)
            .style(app.theme.warning);
        frame.render_widget(warning, area);
        return;
    }

    app.highlight_selected_file();

    let hidden_hunks = app
        .hidden_hunks
        .get(app.selected_file)
        .map(|hidden| hidden.iter().copied().collect::<Vec<_>>())
        .unwrap_or_default();

    let current_entry = app.model.get(app.selected_file);
    let diff = Diff {
        path: current_entry.map(|entry| entry.path.as_path()),
        delta: &app.model,
        selected_entry: app.selected_file,

        hidden_hunks: &hidden_hunks,
        theme: &app.theme,
    };

    let mut state = DiffState {
        line: app.line,
        scroll: app.scroll,
        center_line: app.center_line,
    };
    frame.render_stateful_widget(diff, area, &mut state);
    app.line = state.line;
    app.scroll = state.scroll;
    app.center_line = state.center_line;

    if app.mode == Mode::FileSelector {
        let selector = FileSelector {
            delta: &app.model,
            file_filter: &app.file_filter,
            theme: &app.theme,
        };
        frame.render_stateful_widget(selector, area, &mut app.selector_file);
    }

    if app.mode == Mode::TextInput {
        // Preview the draft filter without changing the committed filter or selection.
        let preview = Filter::try_from((app.file_filter_type, app.input_text.clone(), app.invert));
        let preview = FilterPreview {
            delta: &app.model,
            filter: preview.as_ref().ok(),
        };
        frame.render_widget(preview, area);

        let input = TextInput {
            text: &app.input_text,
            filter_type: &app.file_filter_type,
            invert: app.invert,
            theme: &app.theme,
        };
        input.render(area, frame.buffer_mut());

        let input_area = text_input::input_rect(area);
        let cursor_x = input_area.x
            + 1
            + app
                .input_text
                .chars()
                .count()
                .min(input_area.width.saturating_sub(2) as usize) as u16;
        frame.set_cursor_position(Position::new(cursor_x, input_area.y + 1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use action::Intent;
    use ratatui::{backend::TestBackend, style::Color};

    #[test]
    fn input_previews_filter_without_committing_it() {
        let mut model = Delta::from_test_hunks(vec![]);
        model.entries[0].path = "alpha.rs".into();
        let mut other = Delta::from_test_hunks(vec![]).entries.remove(0);
        other.path = "beta.rs".into();
        model.entries.push(other);

        let mut app = App::new(
            model,
            Syntax::new(&git2::Config::new().unwrap()),
            Theme::default(),
        );
        app.execute(Intent::OpenTextInput);
        for ch in "*beta*".chars() {
            app.execute(Intent::InputChar(ch));
        }

        let mut terminal = Terminal::new(TestBackend::new(100, 38)).unwrap();
        terminal.draw(|frame| render(frame, &mut app)).unwrap();
        let buf = terminal.backend().buffer();
        let row = |y| (0..100).map(|x| buf[(x, y)].symbol()).collect::<String>();
        assert!(row(7).contains("alpha.rs"));
        assert!(row(8).contains("beta.rs"));
        let alpha_x = row(7).find("alpha.rs").unwrap() as u16;
        assert_eq!(buf[(alpha_x, 7)].fg, Color::DarkGray);
        assert!(row(3).contains("*beta*"));
        assert_eq!(app.file_filter.repr(), "");
        assert_eq!(app.selected_file, 0);

        app.execute(Intent::CloseTextInput);
        assert_eq!(app.file_filter.repr(), "");
    }

    #[test]
    fn invalid_input_still_renders_preview() {
        let model = Delta::from_test_hunks(vec![]);
        let mut app = App::new(
            model,
            Syntax::new(&git2::Config::new().unwrap()),
            Theme::default(),
        );
        app.execute(Intent::OpenTextInput);
        app.execute(Intent::CycleInputMode);
        app.execute(Intent::InputChar('['));
        let mut terminal = Terminal::new(TestBackend::new(100, 38)).unwrap();
        terminal.draw(|frame| render(frame, &mut app)).unwrap();
        let buf = terminal.backend().buffer();
        let text = (0..100).map(|x| buf[(x, 6)].symbol()).collect::<String>();
        assert!(text.contains("Invalid filter"));
        assert_eq!(app.file_filter.repr(), "");
    }
}

#[cfg(test)]
pub(crate) fn render_stateful<W, S>(widget: W, mut state: S) -> ratatui::buffer::Buffer
where
    W: ratatui::widgets::StatefulWidget<State = S>,
{
    let area = ratatui::layout::Rect::new(0, 0, 167, 38);
    let mut buf = ratatui::buffer::Buffer::empty(area);

    ratatui::widgets::StatefulWidget::render(widget, area, &mut buf, &mut state);

    buf
}
