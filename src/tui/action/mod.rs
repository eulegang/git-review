use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use eyre::Result;

use crate::tui::App;

pub mod builder;
mod conv;

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
    TextInput,
}

#[derive(Debug, Clone)]
pub struct KeyBindings {
    diff: HashMap<KeyPattern, Intent>,
    file_selector: HashMap<KeyPattern, Intent>,
    text_input: HashMap<KeyPattern, Intent>,
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
    OpenTextInput,
    CloseTextInput,
    ConfirmTextInput,
    InputChar(char),
    CycleInputMode,
    InvertInputMode,
    BackspaceInput,
}

impl KeyBindings {
    pub fn action_for_key(&self, mode: Mode, event: KeyEvent) -> Result<Intent> {
        let keymap = match mode {
            Mode::Diff => &self.diff,
            Mode::FileSelector => &self.file_selector,
            Mode::TextInput => &self.text_input,
        };

        if let Some(pattern) = KeyPattern::from_event(event)
            && let Some(intent) = keymap.get(&pattern)
        {
            return Ok(*intent);
        }

        if mode == Mode::TextInput
            && let KeyCode::Char(ch) = event.code
            && !event
                .modifiers
                .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
        {
            return Ok(Intent::InputChar(ch));
        }

        Err(eyre::eyre!("invalid keycode"))
    }
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

            Intent::OpenTextInput => misc::OpenTextInput.perform(self),
            Intent::CloseTextInput => misc::CloseTextInput.perform(self),
            Intent::ConfirmTextInput => misc::ConfirmTextInput.perform(self),
            Intent::InputChar(ch) => misc::InputChar(ch).perform(self),
            Intent::CycleInputMode => misc::CycleInput.perform(self),
            Intent::InvertInputMode => misc::InvertInput.perform(self),
            Intent::BackspaceInput => misc::BackspaceInput.perform(self),
        }
    }
}

impl Mode {
    pub fn action_for_mouse(self, event: MouseEvent) -> Result<Intent> {
        match self {
            Mode::Diff => diff_mouse_action(event),
            Mode::FileSelector => file_selector_mouse_action(event),
            Mode::TextInput => Err(eyre::eyre!("invalid mouse action")),
        }
    }
}

fn diff_mouse_action(event: MouseEvent) -> Result<Intent> {
    match event.kind {
        MouseEventKind::ScrollDown => Ok(Intent::ScrollDown(3)),
        MouseEventKind::ScrollUp => Ok(Intent::ScrollUp(3)),
        _ => Err(eyre::eyre!("invalid mouse action")),
    }
}

fn file_selector_mouse_action(event: MouseEvent) -> Result<Intent> {
    match event.kind {
        MouseEventKind::ScrollDown => Ok(Intent::SelectNextFile),
        MouseEventKind::ScrollUp => Ok(Intent::SelectPreviousFile),
        _ => Err(eyre::eyre!("invalid mouse action")),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct KeyPattern {
    code: KeyCodePattern,
    modifiers: u8,
}

#[cfg(test)]
impl crate::TestFixture for KeyBindings {
    fn fixture() -> Self {
        let mut builder = builder::Builder::default();
        let _ = builder.load_defaults();
        builder.build()
    }
}

impl KeyPattern {
    fn new(mut code: KeyCodePattern, mut modifiers: KeyModifiers) -> Self {
        if let KeyCodePattern::Char(ch) = &mut code
            && modifiers.contains(KeyModifiers::SHIFT)
        {
            if ch.is_ascii_lowercase() {
                *ch = ch.to_ascii_uppercase();
            }
            modifiers.remove(KeyModifiers::SHIFT);
        }

        if matches!(code, KeyCodePattern::BackTab) {
            modifiers.remove(KeyModifiers::SHIFT);
        }

        Self {
            code,
            modifiers: modifiers.bits(),
        }
    }

    fn from_event(event: KeyEvent) -> Option<Self> {
        let code = match event.code {
            KeyCode::Backspace => KeyCodePattern::Backspace,
            KeyCode::Enter => KeyCodePattern::Enter,
            KeyCode::Left => KeyCodePattern::Left,
            KeyCode::Right => KeyCodePattern::Right,
            KeyCode::Up => KeyCodePattern::Up,
            KeyCode::Down => KeyCodePattern::Down,
            KeyCode::Home => KeyCodePattern::Home,
            KeyCode::End => KeyCodePattern::End,
            KeyCode::PageUp => KeyCodePattern::PageUp,
            KeyCode::PageDown => KeyCodePattern::PageDown,
            KeyCode::Tab => KeyCodePattern::Tab,
            KeyCode::BackTab => KeyCodePattern::BackTab,
            KeyCode::Delete => KeyCodePattern::Delete,
            KeyCode::Insert => KeyCodePattern::Insert,
            KeyCode::F(index) => KeyCodePattern::F(index),
            KeyCode::Char(ch) => KeyCodePattern::Char(ch),
            KeyCode::Null => KeyCodePattern::Null,
            KeyCode::Esc => KeyCodePattern::Esc,
            _ => return None,
        };

        Some(Self::new(code, event.modifiers))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum KeyCodePattern {
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Tab,
    BackTab,
    Delete,
    Insert,
    F(u8),
    Char(char),
    Null,
    Esc,
}
