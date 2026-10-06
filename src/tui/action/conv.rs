use crossterm::event::KeyModifiers;

use crate::tui::action::Mode;

use super::{Intent, KeyCodePattern, KeyPattern};

impl std::str::FromStr for KeyPattern {
    type Err = eyre::Report;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let raw = raw.trim();

        if raw.starts_with('<') && raw.ends_with('>') {
            return parse_angle_key(&raw[1..raw.len() - 1]);
        }

        let mut chars = raw.chars();
        let Some(ch) = chars.next() else {
            return Err(eyre::eyre!("keybinding cannot be empty"));
        };

        if chars.next().is_some() {
            return Err(eyre::eyre!(
                "keybinding {raw:?} must be a single character or <named-key>"
            ));
        }

        Ok(KeyPattern::new(
            KeyCodePattern::Char(ch),
            KeyModifiers::empty(),
        ))
    }
}

impl Intent {
    pub fn parse(mode: Mode, s: &str) -> eyre::Result<Self> {
        match mode {
            Mode::Diff => match s {
                "quit" => return Ok(Intent::Quit),
                "top" => return Ok(Intent::JumpToTop),
                "bottom" => return Ok(Intent::JumpToBottom),
                "next" => return Ok(Intent::NextFile),
                "prev" => return Ok(Intent::PreviousFile),
                "next_hunk" => return Ok(Intent::JumpToBottom),
                "prev_hunk" => return Ok(Intent::JumpToPreviousHunk),
                "selector" => return Ok(Intent::OpenFileSelector),
                "filter" => return Ok(Intent::OpenTextInput),

                _ => (),
            },
            Mode::FileSelector => match s {
                "quit" => return Ok(Intent::CloseFileSelector),
                "next" => return Ok(Intent::SelectNextFile),
                "prev" => return Ok(Intent::SelectPreviousFile),
                "top" => return Ok(Intent::SelectFirstFile),
                "bottom" => return Ok(Intent::SelectLastFile),

                _ => (),
            },
            Mode::TextInput => match s {
                "quit" => return Ok(Intent::CloseTextInput),
                "next" => return Ok(Intent::CycleInputMode),
                "invert" => return Ok(Intent::InvertInputMode),

                _ => (),
            },
        };

        Err(eyre::eyre!("failed to parse intent: {s}"))
    }
}

fn parse_angle_key(raw: &str) -> eyre::Result<KeyPattern> {
    let parts = raw.split('-').collect::<Vec<_>>();
    let mut modifiers = KeyModifiers::empty();
    let mut index = 0;

    while index + 1 < parts.len() {
        if let Some(modifier) = parse_modifier(parts[index]) {
            modifiers.insert(modifier);
            index += 1;
        } else {
            break;
        }
    }

    let key = parts[index..].join("-");
    if key.is_empty() {
        return Err(eyre::eyre!("missing key name in <{raw}>"));
    }

    let code = parse_key_code(&key)?;
    Ok(KeyPattern::new(code, modifiers))
}

fn parse_modifier(raw: &str) -> Option<KeyModifiers> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "c" | "ctrl" | "control" => Some(KeyModifiers::CONTROL),
        "a" | "alt" | "m" | "meta" => Some(KeyModifiers::ALT),
        "s" | "shift" => Some(KeyModifiers::SHIFT),
        _ => None,
    }
}

fn parse_key_code(raw: &str) -> eyre::Result<KeyCodePattern> {
    let normalized = raw.trim().to_ascii_lowercase().replace('_', "-");

    match normalized.as_str() {
        "backspace" | "bs" => Ok(KeyCodePattern::Backspace),
        "enter" | "return" | "cr" => Ok(KeyCodePattern::Enter),
        "left" => Ok(KeyCodePattern::Left),
        "right" => Ok(KeyCodePattern::Right),
        "up" => Ok(KeyCodePattern::Up),
        "down" => Ok(KeyCodePattern::Down),
        "home" => Ok(KeyCodePattern::Home),
        "end" => Ok(KeyCodePattern::End),
        "pageup" | "page-up" | "pgup" => Ok(KeyCodePattern::PageUp),
        "pagedown" | "page-down" | "pgdown" => Ok(KeyCodePattern::PageDown),
        "tab" => Ok(KeyCodePattern::Tab),
        "backtab" | "back-tab" | "shift-tab" | "s-tab" => Ok(KeyCodePattern::BackTab),
        "delete" | "del" => Ok(KeyCodePattern::Delete),
        "insert" | "ins" => Ok(KeyCodePattern::Insert),
        "null" => Ok(KeyCodePattern::Null),
        "esc" | "escape" => Ok(KeyCodePattern::Esc),
        "space" => Ok(KeyCodePattern::Char(' ')),
        key if key.starts_with('f') && key.len() > 1 => {
            let index = key[1..].parse::<u8>()?;
            Ok(KeyCodePattern::F(index))
        }
        key => {
            let mut chars = key.chars();
            let Some(ch) = chars.next() else {
                return Err(eyre::eyre!("missing key name"));
            };

            if chars.next().is_some() {
                return Err(eyre::eyre!("unknown key name <{raw}>"));
            }

            Ok(KeyCodePattern::Char(ch))
        }
    }
}
