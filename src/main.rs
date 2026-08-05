mod cli;
mod diff;
mod model;
mod tui;

use cli::Cli;
use diff::run_git_diff;
use eyre::Result;

use crate::model::Model;

fn main() -> Result<()> {
    let mode = Cli::parse_args().diff_mode()?;
    let model = Model::load(&mode)?;

    let output = run_git_diff(&mode)?;

    tui::run(output)
}
