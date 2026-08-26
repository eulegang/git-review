use std::{io, time::Duration};

use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use diff::{Diff, DiffState};
use eyre::{Context, Result};
use file_selector::FileSelector;
use ratatui::{Terminal, backend::CrosstermBackend, layout::Alignment, widgets::Paragraph};

use action::Action;

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
    file_selector_open: bool,
    line: usize,
    scroll: usize,
    should_quit: bool,
    theme: Theme,
}

impl<'a> App<'a> {
    pub fn new(model: &'a Model, theme: Theme) -> Self {
        Self {
            model,
            selected_file: 0,
            selector_file: 0,
            file_selector_open: false,
            line: 0,
            scroll: 0,
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
        self.line = self.line.min(
            self.model
                .entries
                .get(self.selected_file)
                .map(|e| e.hunks.iter().map(|h| h.critical()).sum::<usize>())
                .unwrap_or_default()
                .saturating_sub(1),
        )
    }

    fn scroll_up(&mut self, amount: u16) {
        self.line = self.line.saturating_sub(amount as usize);
    }

    fn jump_to_selected_file(&mut self) {
        self.line = 0;
        self.scroll = 0;
    }

    fn current_file_line_count(&self) -> usize {
        self.model
            .entries
            .get(self.selected_file)
            .map(|e| e.hunks.iter().map(|h| h.critical()).sum::<usize>())
            .unwrap_or_default()
    }

    fn apply(&mut self, action: Action) {
        if self.file_selector_open {
            match action {
                Action::Quit | Action::ToggleFileSelector => self.file_selector_open = false,
                Action::ConfirmFileSelection => {
                    self.selected_file = self.selector_file;
                    self.file_selector_open = false;
                    self.jump_to_selected_file();
                }
                Action::ScrollDown(_) | Action::NextFile => self.next_selector_file(),
                Action::ScrollUp(_) | Action::PreviousFile => self.previous_selector_file(),
                Action::JumpToTop => self.selector_file = 0,
                Action::JumpToBottom => {
                    if !self.model.entries.is_empty() {
                        self.selector_file = self.model.entries.len() - 1;
                    }
                }
            }
            return;
        }

        match action {
            Action::Quit => self.should_quit = true,
            Action::ScrollDown(amount) => self.scroll_down(amount),
            Action::ScrollUp(amount) => self.scroll_up(amount),
            Action::JumpToTop => {
                self.line = 0;
                self.scroll = 0;
            }
            Action::JumpToBottom => {
                self.line = self.current_file_line_count().saturating_sub(1);
            }
            Action::NextFile => self.next_file(),
            Action::PreviousFile => self.previous_file(),
            Action::ToggleFileSelector => {
                self.selector_file = self.selected_file;
                self.file_selector_open = true;
            }
            Action::ConfirmFileSelection => {}
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

                if let Ok(action) = Action::try_from(key) {
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

    let diff = Diff {
        hunks: app
            .model
            .entries
            .get(app.selected_file)
            .map(|e| e.hunks.as_slice())
            .unwrap_or_default(),
        theme: &app.theme,
    };

    let mut state = DiffState {
        line: app.line,
        scroll: app.scroll,
    };
    frame.render_stateful_widget(diff, area, &mut state);
    app.line = state.line;
    app.scroll = state.scroll;

    if app.file_selector_open {
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
