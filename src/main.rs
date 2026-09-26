mod cli;
mod config;
mod eventing;
mod filter;
mod logging;
mod model;
mod syntax;
mod tui;

mod buf;

use cli::Cli;
use eyre::{Context, Result};
use git2::Repository;

use crate::{config::Configuration, model::Delta, tui::App};

#[cfg(test)]
trait TestFixture {
    fn fixture() -> Self;
}

fn main() -> Result<()> {
    logging::init()?;
    tracing::info!("tracing initialized");

    let cli = Cli::parse_args();
    let repo = Repository::discover(".").context("not inside a Git repository")?;
    let git_config = repo.config().context("Loading git config")?;

    let mut conf = Configuration::default();
    conf.load_git(&git_config);

    if let Some(path) = cli.config_path() {
        conf.load_lua(path.as_path());
    }

    let (syntax, keybindings, theme) = conf.build();

    let mode = cli.diff_mode()?;
    tracing::info!("running with diff mode {mode:?}");
    let model = Delta::load(&repo, &mode)?;

    tracing::debug!("loaded model {:#?}", model);

    let mut app = App::new(model, syntax, theme, keybindings);

    app.run()
}
