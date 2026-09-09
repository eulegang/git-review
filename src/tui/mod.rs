use std::{collections::BTreeSet, io};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use diff::{Diff, DiffState};
use eyre::{Context, Result};
use file_selector::FileSelector;
use ratatui::{Terminal, backend::CrosstermBackend, layout::Alignment, widgets::Paragraph};

use action::Mode;

use crate::{
    eventing::{self, AppEvent},
    model::Delta,
};

mod action;
mod diff;
mod file_selector;
mod input;
pub mod theme;

pub use theme::Theme;

#[derive(Debug)]
pub struct App {
    model: Delta,
    selected_file: usize,
    selector_file: usize,
    mode: Mode,
    line: usize,
    scroll: usize,
    hidden_hunks: Vec<BTreeSet<usize>>,
    center_line: bool,
    should_quit: bool,
    theme: Theme,
}

impl App {
    pub fn new(model: Delta, theme: Theme) -> Self {
        let len = model.len();

        Self {
            model,
            selected_file: 0,
            selector_file: 0,
            mode: Mode::Diff,
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
            tracing::debug!(?app_event, "Processing event");
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
            theme: &app.theme,
        };
        frame.render_stateful_widget(selector, area, &mut app.selector_file);
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
