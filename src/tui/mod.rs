use std::{collections::BTreeSet, io, time::Duration};

use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use diff::{Diff, DiffState};
use eyre::{Context, Result};
use file_selector::FileSelector;
use ratatui::{Terminal, backend::CrosstermBackend, layout::Alignment, widgets::Paragraph};

use action::{Action, Mode};

use crate::model::Model;

mod action;
mod diff;
mod file_selector;
mod theme;

pub use theme::Theme;

#[derive(Debug)]
pub struct App<'a> {
    model: &'a Model,
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

impl<'a> App<'a> {
    pub fn new(model: &'a Model, theme: Theme) -> Self {
        Self {
            model,
            selected_file: 0,
            selector_file: 0,
            mode: Mode::Diff,
            line: 0,
            scroll: 0,
            hidden_hunks: vec![BTreeSet::new(); model.entries.len()],
            center_line: false,
            should_quit: false,
            theme,
        }
    }

    fn next_file(&mut self) {
        if self.selected_file + 1 < self.model.entries.len() {
            self.selected_file += 1;
            self.jump_to_selected_file();
        }
    }

    fn previous_file(&mut self) {
        if self.selected_file > 0 {
            self.selected_file -= 1;
            self.jump_to_selected_file();
        }
    }

    fn next_selector_file(&mut self) {
        if self.selector_file + 1 < self.model.entries.len() {
            self.selector_file += 1;
        }
    }

    fn previous_selector_file(&mut self) {
        self.selector_file = self.selector_file.saturating_sub(1);
    }

    fn scroll_down(&mut self, amount: u16) {
        self.line += amount as usize;
        self.line = self
            .line
            .min(self.current_file_line_count().saturating_sub(1))
    }

    fn scroll_up(&mut self, amount: u16) {
        self.line = self.line.saturating_sub(amount as usize);
    }

    fn jump_to_next_hunk(&mut self) {
        if let Some(entry) = self.model.entries.get(self.selected_file) {
            let mut first_line = 0;

            for (index, hunk) in entry.hunks.iter().enumerate() {
                if self.hunk_is_hidden(index) {
                    continue;
                }

                let line_count = hunk.critical();
                if line_count == 0 {
                    continue;
                }

                if first_line > self.line {
                    self.line = first_line;
                    return;
                }

                first_line += line_count;
            }
        }
    }

    fn jump_to_previous_hunk(&mut self) {
        if let Some(entry) = self.model.entries.get(self.selected_file) {
            let mut first_line = 0;
            let mut previous_hunk = None;

            for (index, hunk) in entry.hunks.iter().enumerate() {
                if self.hunk_is_hidden(index) {
                    continue;
                }

                let line_count = hunk.critical();
                if line_count == 0 {
                    continue;
                }

                if self.line <= first_line || self.line < first_line + line_count {
                    break;
                }

                previous_hunk = Some(first_line);
                first_line += line_count;
            }

            if let Some(line) = previous_hunk {
                self.line = line;
            }
        }
    }

    fn jump_to_selected_file(&mut self) {
        self.line = 0;
        self.scroll = 0;
    }

    fn current_file_line_count(&self) -> usize {
        self.model
            .entries
            .get(self.selected_file)
            .map(|e| {
                e.hunks
                    .iter()
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
        let entry = self.model.entries.get(self.selected_file)?;
        let mut first_line = 0;

        for (index, hunk) in entry.hunks.iter().enumerate() {
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

    fn hide_current_hunk(&mut self) {
        if let Some(hunk) = self.current_hunk() {
            if let Some(hidden) = self.hidden_hunks.get_mut(self.selected_file) {
                hidden.insert(hunk);
            }
            let last_line = self.current_file_line_count().saturating_sub(1);
            self.line = self.line.min(last_line);
            self.scroll = self.scroll.min(last_line);
        }
    }

    fn show_hidden_hunks(&mut self) {
        if let Some(hidden) = self.hidden_hunks.get_mut(self.selected_file) {
            hidden.clear();
        }
    }

    fn apply(&mut self, action: Action) {
        match action {
            Action::Quit => self.should_quit = true,
            Action::ScrollDown(amount) => self.scroll_down(amount),
            Action::ScrollUp(amount) => self.scroll_up(amount),
            Action::JumpToNextHunk => self.jump_to_next_hunk(),
            Action::JumpToPreviousHunk => self.jump_to_previous_hunk(),
            Action::CenterSelectedLine => self.center_line = true,
            Action::HideCurrentHunk => self.hide_current_hunk(),
            Action::ShowHiddenHunks => self.show_hidden_hunks(),
            Action::JumpToTop => {
                self.line = 0;
                self.scroll = 0;
            }
            Action::JumpToBottom => {
                self.line = self.current_file_line_count().saturating_sub(1);
            }
            Action::NextFile => self.next_file(),
            Action::PreviousFile => self.previous_file(),
            Action::OpenFileSelector => {
                self.selector_file = self.selected_file;
                self.mode = Mode::FileSelector;
            }
            Action::CloseFileSelector => self.mode = Mode::Diff,
            Action::SelectNextFile => self.next_selector_file(),
            Action::SelectPreviousFile => self.previous_selector_file(),
            Action::SelectFirstFile => self.selector_file = 0,
            Action::SelectLastFile => {
                if !self.model.entries.is_empty() {
                    self.selector_file = self.model.entries.len() - 1;
                }
            }
            Action::ConfirmFileSelection => {
                self.selected_file = self.selector_file;
                self.mode = Mode::Diff;
                self.jump_to_selected_file();
            }
        }
    }

    pub fn run(&mut self) -> Result<()> {
        enable_raw_mode().context("failed to enable raw terminal mode")?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen).context("failed to enter alternate screen")?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).context("failed to initialize terminal")?;

        let result = self.main_loop(&mut terminal);

        disable_raw_mode().context("failed to disable raw terminal mode")?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)
            .context("failed to leave alternate screen")?;
        terminal.show_cursor().context("failed to show cursor")?;

        result
    }

    fn main_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| render(frame, self))?;

            if event::poll(Duration::from_millis(100)).context("failed to poll terminal events")? {
                let Event::Key(key) = event::read().context("failed to read terminal event")?
                else {
                    continue;
                };

                if key.kind != KeyEventKind::Press {
                    continue;
                }

                if let Ok(action) = self.mode.action_for(key) {
                    self.apply(action);
                }
            }
        }

        Ok(())
    }
}

fn render(frame: &mut ratatui::Frame<'_>, app: &mut App) {
    let area = frame.area();

    if app.model.entries.is_empty() {
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

    let diff = Diff {
        hunks: app
            .model
            .entries
            .get(app.selected_file)
            .map(|e| e.hunks.as_slice())
            .unwrap_or_default(),
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
            entries: &app.model.entries,
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
