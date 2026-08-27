use crossterm::event::{KeyCode, KeyEvent};

const PAGE_SCROLL_LINES: u16 = 20;

/// The set of keybindings currently active in the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Diff,
    FileSelector,
}

impl Mode {
    pub fn action_for(self, event: KeyEvent) -> eyre::Result<Action> {
        match self {
            Mode::Diff => diff_action(event),
            Mode::FileSelector => file_selector_action(event),
        }
    }
}

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
    CenterSelectedLine,
    HideCurrentHunk,
    ShowHiddenHunks,
    NextFile,
    PreviousFile,
    OpenFileSelector,
    EditFile,
    CloseFileSelector,
    SelectNextFile,
    SelectPreviousFile,
    SelectFirstFile,
    SelectLastFile,
    ConfirmFileSelection,
}

fn diff_action(event: KeyEvent) -> eyre::Result<Action> {
    match event.code {
        KeyCode::Char('q') | KeyCode::Esc => Ok(Action::Quit),
        KeyCode::Char('j') | KeyCode::Down => Ok(Action::ScrollDown(1)),
        KeyCode::Char('k') | KeyCode::Up => Ok(Action::ScrollUp(1)),
        KeyCode::PageDown => Ok(Action::ScrollDown(PAGE_SCROLL_LINES)),
        KeyCode::PageUp => Ok(Action::ScrollUp(PAGE_SCROLL_LINES)),
        KeyCode::Char('d') => Ok(Action::JumpToNextHunk),
        KeyCode::Char('u') => Ok(Action::JumpToPreviousHunk),
        KeyCode::Char('z') => Ok(Action::CenterSelectedLine),
        KeyCode::Char('g') | KeyCode::Home => Ok(Action::JumpToTop),
        KeyCode::Char('G') | KeyCode::End => Ok(Action::JumpToBottom),
        KeyCode::Char('h') => Ok(Action::HideCurrentHunk),
        KeyCode::Char('H') => Ok(Action::ShowHiddenHunks),
        KeyCode::Char('n') | KeyCode::Tab => Ok(Action::NextFile),
        KeyCode::Char('p') | KeyCode::BackTab => Ok(Action::PreviousFile),
        KeyCode::Char('f') => Ok(Action::OpenFileSelector),
        KeyCode::Char('e') => Ok(Action::EditFile),
        _ => Err(eyre::eyre!("invalid keycode")),
    }
}

fn file_selector_action(event: KeyEvent) -> eyre::Result<Action> {
    match event.code {
        KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('f') => Ok(Action::CloseFileSelector),
        KeyCode::Char('j')
        | KeyCode::Down
        | KeyCode::PageDown
        | KeyCode::Char('d')
        | KeyCode::Char('n')
        | KeyCode::Tab => Ok(Action::SelectNextFile),
        KeyCode::Char('k')
        | KeyCode::Up
        | KeyCode::PageUp
        | KeyCode::Char('u')
        | KeyCode::Char('p')
        | KeyCode::BackTab => Ok(Action::SelectPreviousFile),
        KeyCode::Char('g') | KeyCode::Home => Ok(Action::SelectFirstFile),
        KeyCode::Char('G') | KeyCode::End => Ok(Action::SelectLastFile),
        KeyCode::Enter => Ok(Action::ConfirmFileSelection),
        _ => Err(eyre::eyre!("invalid keycode")),
    }
}
