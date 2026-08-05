use std::{io, time::Duration};

use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use diff::Diff;
use eyre::{Context, Result};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
};

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
        self.scroll = self.scroll.saturating_add(amount);
        self.sync_selected_file_to_scroll();
    }

    fn scroll_up(&mut self, amount: u16) {
        self.scroll = self.scroll.saturating_sub(amount);
        self.sync_selected_file_to_scroll();
    }

    fn jump_to_selected_file(&mut self) {}

    fn sync_selected_file_to_scroll(&mut self) {}

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
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(32), Constraint::Min(20)])
        .split(area);

    let file_items = app
        .model
        .entries
        .iter()
        .map(|file| ListItem::new(file.path.display().to_string()))
        .collect::<Vec<_>>();
    let mut list_state = ListState::default();
    if !app.model.entries.is_empty() {
        list_state.select(Some(app.selected_file));
    }

    let files = List::new(file_items)
        .block(Block::default().title("Files").borders(Borders::ALL))
        .highlight_style(Style::default().fg(Color::Black).bg(Color::Cyan))
        .highlight_symbol("› ");
    frame.render_stateful_widget(files, chunks[0], &mut list_state);

    let diff = Diff {
        hunks: &app.model.entries[app.selected_file].hunks,
    };

    frame.render_stateful_widget(diff, chunks[1], &mut app.line);
}
