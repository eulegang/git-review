use std::{io, time::Duration};

use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use diff::Diff;
use eyre::{Context, Result};
use ratatui::{Terminal, backend::CrosstermBackend};

use action::Action;

use crate::model::Model;

mod action;
mod diff;

#[derive(Debug)]
pub struct App<'a> {
    model: &'a Model,
    selected_file: usize,
    line: usize,
    scroll: u16,
    should_quit: bool,
}

impl<'a> App<'a> {
    pub fn new(model: &'a Model) -> Self {
        Self {
            model,
            selected_file: 0,
            line: 0,
            scroll: 0,
            should_quit: false,
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

    fn jump_to_selected_file(&mut self) {}

    fn apply(&mut self, action: Action) {
        match action {
            Action::Quit => self.should_quit = true,
            Action::ScrollDown(amount) => self.scroll_down(amount),
            Action::ScrollUp(amount) => self.scroll_up(amount),
            Action::JumpToTop => {
                self.scroll = 0;
                self.selected_file = 0;
            }
            Action::JumpToBottom => {}
            Action::NextFile => self.next_file(),
            Action::PreviousFile => self.previous_file(),
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

    let diff = Diff {
        hunks: app
            .model
            .entries
            .get(app.selected_file)
            .map(|e| e.hunks.as_slice())
            .unwrap_or_default(),
    };

    frame.render_stateful_widget(diff, area, &mut app.line);
}
