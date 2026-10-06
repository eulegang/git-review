use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use mlua::{Error, FromLua, UserData, Value};
use ratatui::style::Color;

use crate::{
    config::parse_color,
    ext::ExpandHome,
    tui::action::{Intent, Mode},
};

use super::State;

pub struct RegState {
    state: Arc<Mutex<State>>,
}

impl UserData for RegState {}

pub struct Api;

pub struct TreesitterApi;
pub struct DiffKeys;
pub struct SelectorKeys;
pub struct FilterKeys;

impl From<&Arc<Mutex<State>>> for RegState {
    fn from(value: &Arc<Mutex<State>>) -> Self {
        let state = Arc::clone(&value);

        Self { state }
    }
}

impl UserData for Api {
    fn add_fields<F: mlua::prelude::LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("treesitter", |_, _| Ok(TreesitterApi));
        fields.add_field_method_get("diff", |_, _| Ok(DiffKeys));
        fields.add_field_method_get("selector", |_, _| Ok(SelectorKeys));
        fields.add_field_method_get("filter", |_, _| Ok(FilterKeys));
    }

    fn add_methods<M: mlua::prelude::LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("theme", theme);
    }

    fn register(registry: &mut mlua::prelude::LuaUserDataRegistry<Self>) {
        Self::add_fields(registry);
        Self::add_methods(registry);
    }
}

fn theme(lua: &mlua::Lua, value: Value) -> mlua::Result<()> {
    tracing::info!("calling theme");

    let appdata = lua.app_data_ref::<RegState>().unwrap();
    let mut state = appdata.state.lock().unwrap();

    if let Some(table) = value.as_table() {
        if let Ok(Some(added_bg)) = table.get::<Option<LuaColor>>("added_bg") {
            state.theme.added_bg = added_bg.0;
        }

        if let Ok(Some(removed_bg)) = table.get::<Option<LuaColor>>("removed_bg") {
            state.theme.removed_bg = removed_bg.0;
        }

        if let Ok(Some(selected_added_bg)) = table.get::<Option<LuaColor>>("selected_added_bg") {
            state.theme.selected_added_bg = Some(selected_added_bg.0);
        }

        if let Ok(Some(selected_removed_bg)) = table.get::<Option<LuaColor>>("selected_removed_bg")
        {
            state.theme.selected_removed_bg = Some(selected_removed_bg.0);
        }

        if let Ok(Some(selected_added_fg)) = table.get::<Option<LuaColor>>("selected_added_fg") {
            state.theme.selected_added_fg = Some(selected_added_fg.0);
        }

        if let Ok(Some(selected_removed_fg)) = table.get::<Option<LuaColor>>("selected_removed_fg")
        {
            state.theme.selected_removed_fg = Some(selected_removed_fg.0);
        }

        if let Ok(Some(binary_bg)) = table.get::<Option<LuaColor>>("binary_bg") {
            state.theme.binary_bg = binary_bg.0;
        }

        if let Ok(Some(warning_fg)) = table.get::<Option<LuaColor>>("warning_fg") {
            state.theme.warning = state.theme.warning.fg(warning_fg.0);
        }

        if let Ok(Some(hunk_header_fg)) = table.get::<Option<LuaColor>>("hunk_header_fg") {
            state.theme.hunk_header = state.theme.hunk_header.fg(hunk_header_fg.0);
        }
    }

    Ok(())
}

pub struct LuaColor(Color);
impl FromLua for LuaColor {
    fn from_lua(value: Value, _lua: &mlua::prelude::Lua) -> mlua::prelude::LuaResult<Self> {
        if let Some(raw) = value.as_string() {
            match parse_color(raw.to_str()?.as_ref()) {
                Ok(color) => Ok(LuaColor(color)),
                Err(err) => Err(Error::RuntimeError(format!(
                    "\"{raw:?}\" is not a color: {err}"
                ))),
            }
        } else {
            return Err(Error::RuntimeError("value is not a color".to_string()));
        }
    }
}

impl UserData for TreesitterApi {
    fn add_methods<M: mlua::prelude::LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("path", path);
        methods.add_function("extensions", extensions);
        methods.add_function("highlights", highlights);
    }

    fn add_fields<F: mlua::prelude::LuaUserDataFields<Self>>(_: &mut F) {}

    fn register(registry: &mut mlua::prelude::LuaUserDataRegistry<Self>) {
        Self::add_fields(registry);
        Self::add_methods(registry);
    }
}

fn path(lua: &mlua::Lua, value: Value) -> mlua::Result<()> {
    let mut paths = Vec::<PathBuf>::new();

    let Some(table) = value.as_table() else {
        return Ok(());
    };

    for elem in table.sequence_values() {
        let elem = elem?;

        paths.push(elem);
    }

    let appdata = lua.app_data_ref::<RegState>().unwrap();
    let mut state = appdata.state.lock().unwrap();
    for path in paths {
        state.syntax.add_path(path.expand_home());
    }

    Ok(())
}

fn extensions(lua: &mlua::Lua, value: Value) -> mlua::Result<()> {
    let appdata = lua.app_data_ref::<RegState>().unwrap();
    let mut state = appdata.state.lock().unwrap();

    let Some(table) = value.as_table() else {
        return Ok(());
    };

    for pair in table.pairs() {
        let (ext, lang) = pair?;
        state.syntax.add_lang(ext, lang);
    }

    Ok(())
}

fn highlights(lua: &mlua::Lua, value: Value) -> mlua::Result<()> {
    let appdata = lua.app_data_ref::<RegState>().unwrap();
    let mut state = appdata.state.lock().unwrap();

    let Some(table) = value.as_table() else {
        return Ok(());
    };

    for pair in table.pairs() {
        let (ext, LuaColor(color)) = pair?;
        state.syntax.add_color(ext, color);
    }

    Ok(())
}

impl UserData for DiffKeys {
    fn add_methods<M: mlua::prelude::LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("bind", diff_bind);
        methods.add_function("unbind", diff_unbind);
        methods.add_function("clear", diff_clear);
    }
}

fn diff_bind(lua: &mlua::Lua, value: Value) -> mlua::Result<()> {
    let appdata = lua.app_data_ref::<RegState>().unwrap();
    let mut state = appdata.state.lock().unwrap();

    let Some(table) = value.as_table() else {
        return Ok(());
    };

    for pair in table.pairs::<String, String>() {
        let (key, action) = pair?;

        let intent = match Intent::parse(Mode::Diff, &action) {
            Ok(intent) => intent,
            Err(err) => {
                tracing::error!(?err, "failed to parse action");
                continue;
            }
        };

        if let Err(err) = state.keybindings.bind(Mode::Diff, &key, intent) {
            tracing::error!(?err, "failed to unbind pattern");
        }
    }

    Ok(())
}

fn diff_unbind(lua: &mlua::Lua, value: Value) -> mlua::Result<()> {
    let appdata = lua.app_data_ref::<RegState>().unwrap();
    let mut state = appdata.state.lock().unwrap();

    if let Some(pattern) = value.as_str() {
        if let Err(err) = state.keybindings.unbind(Mode::Diff, &pattern) {
            tracing::error!(?err, "failed to unbind pattern");
        }

        Ok(())
    } else if let Some(table) = value.as_table() {
        for entry in table.sequence_values::<String>() {
            let pattern = entry?;

            if let Err(err) = state.keybindings.unbind(Mode::Diff, &pattern) {
                tracing::error!(?err, "failed to unbind pattern");
            }
        }
        Ok(())
    } else {
        return Err(mlua::Error::BadArgument {
            to: Some("unbind".to_string()),
            pos: 0,
            name: Some("pattern".to_string()),
            cause: Arc::new(mlua::Error::RuntimeError("".to_string())),
        });
    }
}

fn diff_clear(lua: &mlua::Lua, _value: Value) -> mlua::Result<()> {
    let appdata = lua.app_data_ref::<RegState>().unwrap();
    let mut state = appdata.state.lock().unwrap();

    let _ = state.keybindings.clear(Mode::Diff);

    Ok(())
}

impl UserData for SelectorKeys {
    fn add_methods<M: mlua::prelude::LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("bind", selector_bind);
        methods.add_function("unbind", selector_unbind);
        methods.add_function("clear", selector_clear);
    }
}

fn selector_bind(lua: &mlua::Lua, value: Value) -> mlua::Result<()> {
    let appdata = lua.app_data_ref::<RegState>().unwrap();
    let mut state = appdata.state.lock().unwrap();

    let Some(table) = value.as_table() else {
        return Ok(());
    };

    for pair in table.pairs::<String, String>() {
        let (key, action) = pair?;

        let intent = match Intent::parse(Mode::Diff, &action) {
            Ok(intent) => intent,
            Err(err) => {
                tracing::error!(?err, "failed to parse action");
                continue;
            }
        };

        if let Err(err) = state.keybindings.bind(Mode::FileSelector, &key, intent) {
            tracing::error!(?err, "failed to unbind pattern");
        }
    }

    Ok(())
}

fn selector_unbind(lua: &mlua::Lua, value: Value) -> mlua::Result<()> {
    let appdata = lua.app_data_ref::<RegState>().unwrap();
    let mut state = appdata.state.lock().unwrap();

    if let Some(pattern) = value.as_str() {
        if let Err(err) = state.keybindings.unbind(Mode::FileSelector, &pattern) {
            tracing::error!(?err, "failed to unbind pattern");
        }

        Ok(())
    } else if let Some(table) = value.as_table() {
        for entry in table.sequence_values::<String>() {
            let pattern = entry?;

            if let Err(err) = state.keybindings.unbind(Mode::FileSelector, &pattern) {
                tracing::error!(?err, "failed to unbind pattern");
            }
        }
        Ok(())
    } else {
        return Err(mlua::Error::BadArgument {
            to: Some("unbind".to_string()),
            pos: 0,
            name: Some("pattern".to_string()),
            cause: Arc::new(mlua::Error::RuntimeError("".to_string())),
        });
    }
}

fn selector_clear(lua: &mlua::Lua, _value: Value) -> mlua::Result<()> {
    let appdata = lua.app_data_ref::<RegState>().unwrap();
    let mut state = appdata.state.lock().unwrap();

    let _ = state.keybindings.clear(Mode::FileSelector);

    Ok(())
}

impl UserData for FilterKeys {
    fn add_methods<M: mlua::prelude::LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("bind", filter_bind);
        methods.add_function("unbind", filter_unbind);
        methods.add_function("clear", filter_clear);
    }
}

fn filter_bind(lua: &mlua::Lua, value: Value) -> mlua::Result<()> {
    let appdata = lua.app_data_ref::<RegState>().unwrap();
    let mut state = appdata.state.lock().unwrap();

    let Some(table) = value.as_table() else {
        return Ok(());
    };

    for pair in table.pairs::<String, String>() {
        let (key, action) = pair?;

        let intent = match Intent::parse(Mode::TextInput, &action) {
            Ok(intent) => intent,
            Err(err) => {
                tracing::error!(?err, "failed to parse action");
                continue;
            }
        };

        if let Err(err) = state.keybindings.bind(Mode::TextInput, &key, intent) {
            tracing::error!(?err, "failed to unbind pattern");
        }
    }

    Ok(())
}

fn filter_unbind(lua: &mlua::Lua, value: Value) -> mlua::Result<()> {
    let appdata = lua.app_data_ref::<RegState>().unwrap();
    let mut state = appdata.state.lock().unwrap();

    if let Some(pattern) = value.as_str() {
        if let Err(err) = state.keybindings.unbind(Mode::TextInput, &pattern) {
            tracing::error!(?err, "failed to unbind pattern");
        }

        Ok(())
    } else if let Some(table) = value.as_table() {
        for entry in table.sequence_values::<String>() {
            let pattern = entry?;

            if let Err(err) = state.keybindings.unbind(Mode::TextInput, &pattern) {
                tracing::error!(?err, "failed to unbind pattern");
            }
        }
        Ok(())
    } else {
        return Err(mlua::Error::BadArgument {
            to: Some("unbind".to_string()),
            pos: 0,
            name: Some("pattern".to_string()),
            cause: Arc::new(mlua::Error::RuntimeError("".to_string())),
        });
    }
}

fn filter_clear(lua: &mlua::Lua, _value: Value) -> mlua::Result<()> {
    let appdata = lua.app_data_ref::<RegState>().unwrap();
    let mut state = appdata.state.lock().unwrap();

    let _ = state.keybindings.clear(Mode::TextInput);

    Ok(())
}
