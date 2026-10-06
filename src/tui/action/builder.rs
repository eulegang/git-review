use super::{Intent, KeyBindings, KeyPattern, Mode};
use std::{collections::HashMap, str::FromStr};

#[derive(Default, Clone)]
pub struct Builder {
    diff: HashMap<KeyPattern, Intent>,
    file_selector: HashMap<KeyPattern, Intent>,
    text_input: HashMap<KeyPattern, Intent>,
}

impl Builder {
    pub fn build(self) -> KeyBindings {
        KeyBindings {
            diff: self.diff,
            file_selector: self.file_selector,
            text_input: self.text_input,
        }
    }

    pub fn bind(&mut self, mode: Mode, pattern: &str, intent: Intent) -> eyre::Result<()> {
        let pattern = KeyPattern::from_str(pattern)?;

        let map = match mode {
            Mode::Diff => &mut self.diff,
            Mode::FileSelector => &mut self.file_selector,
            Mode::TextInput => &mut self.text_input,
        };

        map.insert(pattern, intent);

        Ok(())
    }

    #[allow(dead_code)]
    pub fn unbind(&mut self, mode: Mode, pattern: &str) -> eyre::Result<()> {
        let pattern = KeyPattern::from_str(pattern)?;

        let map = match mode {
            Mode::Diff => &mut self.diff,
            Mode::FileSelector => &mut self.file_selector,
            Mode::TextInput => &mut self.text_input,
        };

        map.remove(&pattern);

        Ok(())
    }

    pub fn clear(&mut self, mode: Mode) -> eyre::Result<()> {
        let map = match mode {
            Mode::Diff => &mut self.diff,
            Mode::FileSelector => &mut self.file_selector,
            Mode::TextInput => &mut self.text_input,
        };

        map.clear();

        Ok(())
    }

    pub fn load_defaults(&mut self) -> eyre::Result<()> {
        self.bind(Mode::Diff, "q", Intent::Quit)?;
        self.bind(Mode::Diff, "j", Intent::ScrollDown(1))?;
        self.bind(Mode::Diff, "<down>", Intent::ScrollDown(1))?;
        self.bind(Mode::Diff, "k", Intent::ScrollUp(1))?;
        self.bind(Mode::Diff, "<up>", Intent::ScrollUp(1))?;
        self.bind(
            Mode::Diff,
            "<pagedown>",
            Intent::ScrollDown(super::PAGE_SCROLL_LINES),
        )?;
        self.bind(
            Mode::Diff,
            "<pageup>",
            Intent::ScrollUp(super::PAGE_SCROLL_LINES),
        )?;
        self.bind(Mode::Diff, "d", Intent::JumpToNextHunk)?;
        self.bind(Mode::Diff, "u", Intent::JumpToPreviousHunk)?;
        self.bind(Mode::Diff, "z", Intent::CenterSelectedLine)?;
        self.bind(Mode::Diff, "g", Intent::JumpToTop)?;
        self.bind(Mode::Diff, "<home>", Intent::JumpToTop)?;
        self.bind(Mode::Diff, "G", Intent::JumpToBottom)?;
        self.bind(Mode::Diff, "<end>", Intent::JumpToBottom)?;
        self.bind(Mode::Diff, "h", Intent::HideCurrentHunk)?;
        self.bind(Mode::Diff, "H", Intent::ShowHiddenHunks)?;
        self.bind(Mode::Diff, "n", Intent::NextFile)?;
        self.bind(Mode::Diff, "<tab>", Intent::NextFile)?;
        self.bind(Mode::Diff, "p", Intent::PreviousFile)?;
        self.bind(Mode::Diff, "<backtab>", Intent::PreviousFile)?;
        self.bind(Mode::Diff, "f", Intent::OpenFileSelector)?;
        self.bind(Mode::Diff, "F", Intent::OpenTextInput)?;

        self.bind(Mode::FileSelector, "q", Intent::CloseFileSelector)?;
        self.bind(Mode::FileSelector, "<esc>", Intent::CloseFileSelector)?;
        self.bind(Mode::FileSelector, "f", Intent::CloseFileSelector)?;
        self.bind(Mode::FileSelector, "j", Intent::SelectNextFile)?;
        self.bind(Mode::FileSelector, "<down>", Intent::SelectNextFile)?;
        self.bind(Mode::FileSelector, "<pagedown>", Intent::SelectNextFile)?;
        self.bind(Mode::FileSelector, "d", Intent::SelectNextFile)?;
        self.bind(Mode::FileSelector, "n", Intent::SelectNextFile)?;
        self.bind(Mode::FileSelector, "<tab>", Intent::SelectNextFile)?;
        self.bind(Mode::FileSelector, "k", Intent::SelectPreviousFile)?;
        self.bind(Mode::FileSelector, "<up>", Intent::SelectPreviousFile)?;
        self.bind(Mode::FileSelector, "<pageup>", Intent::SelectPreviousFile)?;
        self.bind(Mode::FileSelector, "u", Intent::SelectPreviousFile)?;
        self.bind(Mode::FileSelector, "p", Intent::SelectPreviousFile)?;
        self.bind(Mode::FileSelector, "<backtab>", Intent::SelectPreviousFile)?;
        self.bind(Mode::FileSelector, "g", Intent::SelectFirstFile)?;
        self.bind(Mode::FileSelector, "<home>", Intent::SelectFirstFile)?;
        self.bind(Mode::FileSelector, "G", Intent::SelectLastFile)?;
        self.bind(Mode::FileSelector, "<end>", Intent::SelectLastFile)?;
        self.bind(Mode::FileSelector, "<enter>", Intent::ConfirmFileSelection)?;

        self.bind(Mode::TextInput, "<esc>", Intent::CloseTextInput)?;
        self.bind(Mode::TextInput, "<enter>", Intent::ConfirmTextInput)?;
        self.bind(Mode::TextInput, "<backspace>", Intent::BackspaceInput)?;
        self.bind(Mode::TextInput, "<backtab>", Intent::InvertInputMode)?;
        self.bind(Mode::TextInput, "<S-tab>", Intent::InvertInputMode)?;
        self.bind(Mode::TextInput, "<tab>", Intent::CycleInputMode)?;

        Ok(())
    }
}
