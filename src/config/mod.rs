use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use eyre::{Context, bail};
use mlua::Lua;
use ratatui::style::Color;

use crate::{
    syntax::Syntax,
    tui::{KeyBindings, Theme},
};

pub mod api;

type SynBuilder = crate::syntax::builder::Builder;
type KeyBuilder = crate::tui::action::builder::Builder;

#[derive(Default)]
struct State {
    pub syntax: SynBuilder,
    pub keybindings: KeyBuilder,
    pub theme: Theme,
}

pub struct Configuration {
    state: Arc<Mutex<State>>,
}

impl Configuration {
    pub fn load_git(&mut self, config: &git2::Config) {
        let mut state = self.state.lock().unwrap();

        if let Ok(entries) = config.entries(Some("git-review.tree-sitter.*")) {
            let _ = entries.for_each(|e| {
                let (Some(name), Some(value)) = (e.name(), e.value()) else {
                    return;
                };

                let name = name
                    .strip_prefix("git-review.tree-sitter.")
                    .unwrap_or(name)
                    .replace("-", ".");

                let color = match parse_color(value) {
                    Ok(color) => color,
                    Err(err) => {
                        tracing::error!(?err, ?name, ?value, "failed to parse treesitter color");
                        return;
                    }
                };

                state.syntax.add_color(name, color);
            });
        }
    }

    pub fn load_lua(&mut self, path: &Path) {
        let source = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(err) => {
                tracing::error!(?err, "failed to load config source");
                return;
            }
        };

        let lua = Lua::new();

        let chunk_name = path.display().to_string();
        let chunk = lua.load(&source).set_name(&chunk_name);
        let api = api::Api;
        let state = api::RegState::from(&self.state);
        lua.set_app_data(state);

        if let Err(err) = lua.globals().set("review", api) {
            tracing::error!(?err, "failed to configure execution environment");
        }

        if let Err(err) = chunk.exec() {
            tracing::error!(?err, "failed to execute configuration");
        }
    }

    pub fn build(&self) -> (Syntax, KeyBindings, Theme) {
        let state = self.state.lock().unwrap();

        let syntax = state.syntax.clone().build();
        let keybindings = state.keybindings.clone().build();
        let theme = state.theme.clone();

        tracing::debug!("loaded syntax {syntax:#?}");
        tracing::debug!("loaded keybindings {keybindings:#?}");
        tracing::debug!("loaded theme {theme:#?}");

        (syntax, keybindings, theme)
    }
}

impl Default for Configuration {
    fn default() -> Self {
        let state: Arc<Mutex<State>> = Default::default();
        let arc = Arc::clone(&state);
        let mut s = arc.lock().unwrap();
        let _ = s.keybindings.load_defaults();

        Self { state }
    }
}

pub fn parse_color(raw: &str) -> eyre::Result<Color> {
    let color = raw.trim().to_ascii_lowercase();

    if color == "reset" || color == "default" {
        return Ok(Color::Reset);
    }

    if let Some(hex) = color.strip_prefix('#') {
        if hex.len() != 6 {
            bail!("hex colors must be in #rrggbb format");
        }

        let red = u8::from_str_radix(&hex[0..2], 16).context("invalid red channel")?;
        let green = u8::from_str_radix(&hex[2..4], 16).context("invalid green channel")?;
        let blue = u8::from_str_radix(&hex[4..6], 16).context("invalid blue channel")?;

        return Ok(Color::Rgb(red, green, blue));
    }

    if let Ok(index) = color.parse::<u8>() {
        return Ok(Color::Indexed(index));
    }

    match color.as_str() {
        "black" => Ok(Color::Black),
        "red" => Ok(Color::Red),
        "green" => Ok(Color::Green),
        "yellow" => Ok(Color::Yellow),
        "blue" => Ok(Color::Blue),
        "magenta" => Ok(Color::Magenta),
        "cyan" => Ok(Color::Cyan),
        "gray" | "grey" => Ok(Color::Gray),
        "dark-gray" | "dark-grey" => Ok(Color::DarkGray),
        "light-red" => Ok(Color::LightRed),
        "light-green" => Ok(Color::LightGreen),
        "light-yellow" => Ok(Color::LightYellow),
        "light-blue" => Ok(Color::LightBlue),
        "light-magenta" => Ok(Color::LightMagenta),
        "light-cyan" => Ok(Color::LightCyan),
        "white" => Ok(Color::White),
        _ => bail!("unknown color {raw:?}"),
    }
}
