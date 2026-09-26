use std::sync::{Arc, Mutex};

use mlua::{Error, FromLua, UserData, Value};
use ratatui::style::Color;

use crate::config::parse_color;

use super::State;

pub struct RegState {
    state: Arc<Mutex<State>>,
}

impl UserData for RegState {}

pub struct Api;

pub struct TreesitterApi {}

impl From<&Arc<Mutex<State>>> for RegState {
    fn from(value: &Arc<Mutex<State>>) -> Self {
        let state = Arc::clone(&value);

        Self { state }
    }
}

impl UserData for Api {
    fn add_fields<F: mlua::prelude::LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("treesitter", |_, _| Ok(TreesitterApi {}));
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
    let mut paths = Vec::new();

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
        state.syntax.add_path(path);
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
