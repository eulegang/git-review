use crossterm::event::{KeyCode, KeyEvent};

const PAGE_SCROLL_LINES: u16 = 20;

/// an action to be taken
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    ScrollDown(u16),
    ScrollUp(u16),
    JumpToTop,
    JumpToBottom,
    JumpToNextHunk,
    JumpToPreviousHunk,
    NextFile,
    PreviousFile,
    ToggleFileSelector,
    ConfirmFileSelection,
}

impl TryFrom<KeyEvent> for Action {
    type Error = eyre::Report;

    fn try_from(event: KeyEvent) -> std::prelude::v1::Result<Self, Self::Error> {
        match event.code {
            KeyCode::Char('q') | KeyCode::Esc => Ok(Action::Quit),
            KeyCode::Char('j') | KeyCode::Down => Ok(Action::ScrollDown(1)),
            KeyCode::Char('k') | KeyCode::Up => Ok(Action::ScrollUp(1)),
            KeyCode::PageDown => Ok(Action::ScrollDown(PAGE_SCROLL_LINES)),
            KeyCode::PageUp => Ok(Action::ScrollUp(PAGE_SCROLL_LINES)),
            KeyCode::Char('d') => Ok(Action::JumpToNextHunk),
            KeyCode::Char('u') => Ok(Action::JumpToPreviousHunk),
            KeyCode::Char('g') | KeyCode::Home => Ok(Action::JumpToTop),
            KeyCode::Char('G') | KeyCode::End => Ok(Action::JumpToBottom),
            KeyCode::Char('n') | KeyCode::Tab => Ok(Action::NextFile),
            KeyCode::Char('p') | KeyCode::BackTab => Ok(Action::PreviousFile),
            KeyCode::Char('f') => Ok(Action::ToggleFileSelector),
            KeyCode::Enter => Ok(Action::ConfirmFileSelection),
            _ => Err(eyre::eyre!("invalid keycode")),
        }
    }
}
