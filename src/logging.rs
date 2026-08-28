use std::fs::{self, OpenOptions};

use eyre::{Context, Result, bail, eyre};
use tracing_subscriber::EnvFilter;

const APP_NAME: &str = "git-review";
const LOG_FILE: &str = "git-review.log";

pub fn init() -> Result<()> {
    let Some(state_home) = dirs::data_dir() else {
        bail!("could not determine state directory for logs")
    };

    let log_dir = state_home.join(APP_NAME);
    fs::create_dir_all(&log_dir).context("failed to create log directory")?;

    let log_file = log_dir.join(LOG_FILE);
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file)
        .expect("Failed to open or create log file");

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(file)
        .with_ansi(true)
        .with_target(true)
        .try_init()
        .map_err(|error| eyre!("failed to initialize tracing subscriber: {error}"))?;

    Ok(())
}
