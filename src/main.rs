mod cli;
mod logging;
mod model;
mod tui;

use cli::Cli;
use eyre::{Context, Result};
use git2::Repository;
use tracing::{debug, info};

use crate::{
    model::Model,
    tui::{App, Theme},
};

fn main() -> Result<()> {
    logging::init()?;
    info!("tracing initialized");

    let cli = Cli::parse_args();
    let repo = Repository::discover(".").context("not inside a Git repository")?;
    let theme = Theme::load(&repo)?;
    let mode = cli.diff_mode()?;
    let model = Model::load(&repo, &mode)?;

    debug!(?model, "loaded model");

    let workdir = repo.workdir().map(ToOwned::to_owned);
    let mut app = App::new(&model, theme, workdir);

    app.run()
}
