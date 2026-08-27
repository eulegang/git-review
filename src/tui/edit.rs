use std::{ffi::OsString, os::unix::fs::PermissionsExt, path::Path, process::Command};

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use eyre::{Context, ContextCompat};
use ratatui::{Terminal, backend::CrosstermBackend};

pub type Term = Terminal<CrosstermBackend<std::io::Stdout>>;

pub struct Edit<'a> {
    path: &'a Path,
    workdir: Option<&'a Path>,
}

impl<'a> Edit<'a> {
    pub fn new(path: &'a Path, workdir: Option<&'a Path>) -> Self {
        Edit { path, workdir }
    }

    pub fn run(self, term: &mut Term) -> eyre::Result<()> {
        let cooked = Cooked::enable(term)?;

        let editor = Edit::editor_command()?;
        let mut command = Command::new(editor);
        command.arg(self.path);

        if let Some(workdir) = &self.workdir {
            command.current_dir(workdir);
        }

        let result = command.status().context("failed to launch $EDITOR");

        cooked.disable(term)?;

        let status = result?;
        if !status.success() {
            eyre::bail!("$EDITOR exited with status {status}");
        }

        Ok(())
    }

    fn editor_command() -> eyre::Result<OsString> {
        if let Some(editor) = std::env::var_os("EDITOR")
            && !editor.is_empty()
        {
            return Ok(editor);
        }

        let path = std::env::var_os("PATH").context("$EDITOR is not set and PATH is not set")?;
        for editor in ["vi", "nano"] {
            if std::env::split_paths(&path)
                .any(|directory| Edit::is_executable(&directory.join(editor)))
            {
                return Ok(editor.into());
            }
        }

        eyre::bail!("$EDITOR is not set and neither vi nor nano was found in PATH")
    }

    fn is_executable(path: &std::path::Path) -> bool {
        path.metadata()
            .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
    }
}

struct Cooked;

impl Cooked {
    fn enable(terminal: &mut Term) -> eyre::Result<Self> {
        disable_raw_mode().context("failed to disable raw terminal mode")?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)
            .context("failed to leave alternate screen")?;
        terminal.show_cursor().context("failed to show cursor")?;
        Ok(Cooked)
    }

    fn disable(self, terminal: &mut Term) -> eyre::Result<()> {
        execute!(terminal.backend_mut(), EnterAlternateScreen)
            .context("failed to enter alternate screen")?;
        enable_raw_mode().context("failed to enable raw terminal mode")?;
        terminal.clear().context("failed to clear terminal")?;

        Ok(())
    }
}
