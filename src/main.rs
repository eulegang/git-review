mod cli;
mod model;
mod tui;

use cli::Cli;
use eyre::Result;

use crate::{model::Model, tui::App};

fn main() -> Result<()> {
    let mode = Cli::parse_args().diff_mode()?;
    let model = Model::load(&mode)?;

    if option_env!("DEBUG_MODEL") != None {
        dbg!(&model);

        Ok(())
    } else {
        let mut app = App::new(&model);

        app.run()
    }
}
