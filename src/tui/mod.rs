use std::{io, time::Duration};

use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use eyre::{Context, Result};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use action::Action;

use crate::model::Model;

mod action;

#[derive(Debug, Clone)]
struct FileSection {
    name: String,
    start_line: usize,
}

#[derive(Debug)]
pub struct App<'a> {
    model: &'a Model,
    selected_file: usize,
    scroll: u16,
    should_quit: bool,
}

impl<'a> App<'a> {
    pub fn new(model: &'a Model) -> Self {
        Self {
            model,
            selected_file: 0,
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

    fn jump_to_selected_file(&mut self) {
        todo!("reimplement")
        // if let Some(file) = self.model.entries.get(self.selected_file) {
        //     self.scroll = file.start_line.min(u16::MAX as usize) as u16;
        // }
    }

    fn sync_selected_file_to_scroll(&mut self) {
        todo!("reimplement");
        // let scroll = usize::from(self.scroll);
        // if let Some(index) = self
        //     .model
        //     .entries
        //     .iter()
        //     .rposition(|file| file.start_line <= scroll)
        // {
        //     self.selected_file = index;
        // }
    }

    fn apply(&mut self, action: Action) {
        match action {
            Action::Quit => self.should_quit = true,
            Action::ScrollDown(amount) => self.scroll_down(amount),
            Action::ScrollUp(amount) => self.scroll_up(amount),
            Action::JumpToTop => {
                self.scroll = 0;
                self.selected_file = 0;
            }
            Action::JumpToBottom => {
                todo!("reimplement");
                // self.scroll = self.lines.len().saturating_sub(1).min(u16::MAX as usize) as u16;
                // self.sync_selected_file_to_scroll();
            }
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

    let title = if app.model.entries.is_empty() {
        "Diff - no changes"
    } else {
        "Diff - q quit · j/k scroll · n/p files · g/G top/bottom"
    };
    let diff = Paragraph::new(styled_diff_lines(&[]))
        .block(Block::default().title(title).borders(Borders::ALL))
        .scroll((app.scroll, 0));
    frame.render_widget(diff, chunks[1]);
}

fn find_file_sections(lines: &[String]) -> Vec<FileSection> {
    let mut files = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| {
            line.strip_prefix("diff --git ").map(|rest| FileSection {
                name: rest
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or(rest)
                    .trim_start_matches("b/")
                    .to_owned(),
                start_line: index,
            })
        })
        .collect::<Vec<_>>();

    if files.is_empty() && !lines.is_empty() {
        files.push(FileSection {
            name: "Diff".to_owned(),
            start_line: 0,
        });
    }

    files
}

fn styled_diff_lines(lines: &[String]) -> Vec<Line<'static>> {
    if lines.is_empty() {
        return vec![Line::from(Span::styled(
            "No changes to review.",
            Style::default().fg(Color::DarkGray),
        ))];
    }

    lines.iter().map(|line| styled_diff_line(line)).collect()
}

fn styled_diff_line(line: &str) -> Line<'static> {
    let style = if line.starts_with("diff --git")
        || line.starts_with("index ")
        || line.starts_with("--- ")
        || line.starts_with("+++ ")
    {
        Style::default()
            .fg(Color::Blue)
            .add_modifier(Modifier::BOLD)
    } else if line.starts_with("@@") {
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD)
    } else if line.starts_with('+') {
        Style::default().fg(Color::Green)
    } else if line.starts_with('-') {
        Style::default().fg(Color::Red)
    } else {
        Style::default()
    };

    Line::from(Span::styled(line.to_owned(), style))
}
