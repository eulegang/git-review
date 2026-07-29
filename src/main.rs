mod cli;
mod diff;
mod tui;

use cli::Cli;
use diff::run_git_diff;
use eyre::Result;

fn main() -> Result<()> {
    let mode = Cli::parse_args().diff_mode()?;
    let output = run_git_diff(&mode)?;

    tui::run(output)
}
