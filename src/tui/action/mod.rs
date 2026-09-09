use crate::tui::App;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};

mod file;
mod hunk;
mod jump;
mod misc;
mod quit;
mod scroll;
mod selector;

const PAGE_SCROLL_LINES: u16 = 20;

/// The set of keybindings currently active in the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Diff,
    FileSelector,
}

pub trait Action {
    fn perform(&self, app: &mut App);
}

/// an action to be taken
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intent {
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
    CloseFileSelector,
    SelectNextFile,
    SelectPreviousFile,
    SelectFirstFile,
    SelectLastFile,
    ConfirmFileSelection,
}

impl App {
    pub fn execute(&mut self, intent: Intent) {
        match intent {
            Intent::Quit => quit::Quit.perform(self),

            Intent::ScrollDown(amount) => scroll::ScrollDown(amount).perform(self),
            Intent::ScrollUp(amount) => scroll::ScrollUp(amount).perform(self),

            Intent::JumpToTop => jump::Top.perform(self),
            Intent::JumpToBottom => jump::Bottom.perform(self),
            Intent::JumpToNextHunk => jump::NextHunk.perform(self),
            Intent::JumpToPreviousHunk => jump::PrevHunk.perform(self),

            Intent::HideCurrentHunk => hunk::HideHunk.perform(self),
            Intent::ShowHiddenHunks => hunk::UnhideHunks.perform(self),
            Intent::NextFile => file::NextFile.perform(self),
            Intent::PreviousFile => file::PrevFile.perform(self),

            Intent::CenterSelectedLine => misc::CenterLine.perform(self),

            Intent::OpenFileSelector => selector::Open.perform(self),
            Intent::CloseFileSelector => selector::Close.perform(self),
            Intent::SelectNextFile => selector::NextFile.perform(self),
            Intent::SelectPreviousFile => selector::PrevFile.perform(self),
            Intent::SelectFirstFile => selector::FirstFile.perform(self),
            Intent::SelectLastFile => selector::LastFile.perform(self),
            Intent::ConfirmFileSelection => selector::Confirm.perform(self),
        }
    }
}

impl Mode {
    pub fn action_for_key(self, event: KeyEvent) -> eyre::Result<Intent> {
        match self {
            Mode::Diff => diff_action(event),
            Mode::FileSelector => file_selector_action(event),
        }
    }

    pub fn action_for_mouse(self, event: MouseEvent) -> eyre::Result<Intent> {
        match self {
            Mode::Diff => diff_mouse_action(event),
            Mode::FileSelector => file_selector_mouse_action(event),
        }
    }
}
fn diff_action(event: KeyEvent) -> eyre::Result<Intent> {
    match event.code {
        KeyCode::Char('q') => Ok(Intent::Quit),
        KeyCode::Char('j') | KeyCode::Down => Ok(Intent::ScrollDown(1)),
        KeyCode::Char('k') | KeyCode::Up => Ok(Intent::ScrollUp(1)),
        KeyCode::PageDown => Ok(Intent::ScrollDown(PAGE_SCROLL_LINES)),
        KeyCode::PageUp => Ok(Intent::ScrollUp(PAGE_SCROLL_LINES)),
        KeyCode::Char('d') => Ok(Intent::JumpToNextHunk),
        KeyCode::Char('u') => Ok(Intent::JumpToPreviousHunk),
        KeyCode::Char('z') => Ok(Intent::CenterSelectedLine),
        KeyCode::Char('g') | KeyCode::Home => Ok(Intent::JumpToTop),
        KeyCode::Char('G') | KeyCode::End => Ok(Intent::JumpToBottom),
        KeyCode::Char('h') => Ok(Intent::HideCurrentHunk),
        KeyCode::Char('H') => Ok(Intent::ShowHiddenHunks),
        KeyCode::Char('n') | KeyCode::Tab => Ok(Intent::NextFile),
        KeyCode::Char('p') | KeyCode::BackTab => Ok(Intent::PreviousFile),
        KeyCode::Char('f') => Ok(Intent::OpenFileSelector),
        _ => Err(eyre::eyre!("invalid keycode")),
    }
}

fn diff_mouse_action(event: MouseEvent) -> eyre::Result<Intent> {
    match event.kind {
        MouseEventKind::ScrollDown => Ok(Intent::ScrollDown(3)),
        MouseEventKind::ScrollUp => Ok(Intent::ScrollUp(3)),
        _ => Err(eyre::eyre!("invalid mouse action")),
    }
}

fn file_selector_action(event: KeyEvent) -> eyre::Result<Intent> {
    match event.code {
        KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('f') => Ok(Intent::CloseFileSelector),
        KeyCode::Char('j')
        | KeyCode::Down
        | KeyCode::PageDown
        | KeyCode::Char('d')
        | KeyCode::Char('n')
        | KeyCode::Tab => Ok(Intent::SelectNextFile),
        KeyCode::Char('k')
        | KeyCode::Up
        | KeyCode::PageUp
        | KeyCode::Char('u')
        | KeyCode::Char('p')
        | KeyCode::BackTab => Ok(Intent::SelectPreviousFile),
        KeyCode::Char('g') | KeyCode::Home => Ok(Intent::SelectFirstFile),
        KeyCode::Char('G') | KeyCode::End => Ok(Intent::SelectLastFile),
        KeyCode::Enter => Ok(Intent::ConfirmFileSelection),
        _ => Err(eyre::eyre!("invalid keycode")),
    }
}

fn file_selector_mouse_action(event: MouseEvent) -> eyre::Result<Intent> {
    match event.kind {
        MouseEventKind::ScrollDown => Ok(Intent::SelectNextFile),
        MouseEventKind::ScrollUp => Ok(Intent::SelectPreviousFile),
        _ => Err(eyre::eyre!("invalid mouse action")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_does_not_quit_diff_mode() {
        assert!(
            Mode::Diff
                .action_for_key(KeyEvent::from(KeyCode::Esc))
                .is_err()
        );
    }

    #[test]
    fn q_still_quits_diff_mode() {
        assert_eq!(
            Mode::Diff
                .action_for_key(KeyEvent::from(KeyCode::Char('q')))
                .unwrap(),
            Intent::Quit
        );
    }

    #[test]
    fn escape_closes_file_selector() {
        assert_eq!(
            Mode::FileSelector
                .action_for_key(KeyEvent::from(KeyCode::Esc))
                .unwrap(),
            Intent::CloseFileSelector
        );
    }
}
