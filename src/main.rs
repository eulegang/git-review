mod cli;
mod logging;
mod model;
mod syntax;
mod tui;

mod buf;

use cli::Cli;
use eyre::{Context, Result};
use git2::Repository;
use tracing::{debug, info};

use crate::{
    model::Delta,
    syntax::Syntax,
    tui::{App, Theme},
};

fn main() -> Result<()> {
    logging::init()?;
    info!("tracing initialized");

    let cli = Cli::parse_args();
    let repo = Repository::discover(".").context("not inside a Git repository")?;
    let config = repo.config().context("Loading git config")?;

    let mut syntax = Syntax::new(&config);
    let theme = Theme::load(&config)?;
    let mode = cli.diff_mode()?;
    let mut model = Delta::load(&repo, &mode)?;

    syntax.highlight(&mut model);

    debug!("loaded model {:#?}", model);

    let workdir = repo.workdir().map(ToOwned::to_owned);
    let mut app = App::new(model, theme, workdir, syntax);

    app.run()
}
