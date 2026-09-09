use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use crossterm::event::{self, Event, KeyEventKind, MouseEventKind};
use eyre::{Context, Result};
use tracing::error;

use super::action::{Action, Mode};
use crate::eventing;

const POLL_INTERVAL: Duration = Duration::from_millis(100);

pub struct InputThread {
    should_stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Drop for InputThread {
    fn drop(&mut self) {
        self.should_stop.store(true, Ordering::Relaxed);

        if let Some(handle) = self.handle.take() {
            if handle.join().is_err() {
                error!("input thread panicked");
            }
        }
    }
}

pub fn spawn() -> Result<InputThread> {
    let should_stop = Arc::new(AtomicBool::new(false));
    let thread_should_stop = Arc::clone(&should_stop);

    let handle = thread::Builder::new()
        .name("input".to_owned())
        .spawn(move || {
            if let Err(error) = poll_input(thread_should_stop) {
                error!(?error, "input thread failed");
            }
        })
        .context("failed to spawn input thread")?;

    Ok(InputThread {
        should_stop,
        handle: Some(handle),
    })
}

pub fn action_for_mouse(mode: Mode, kind: MouseEventKind) -> Option<Action> {
    match (mode, kind) {
        (Mode::Diff, MouseEventKind::ScrollDown) => Some(Action::ScrollDown(3)),
        (Mode::Diff, MouseEventKind::ScrollUp) => Some(Action::ScrollUp(3)),
        (Mode::FileSelector, MouseEventKind::ScrollDown) => Some(Action::SelectNextFile),
        (Mode::FileSelector, MouseEventKind::ScrollUp) => Some(Action::SelectPreviousFile),
        _ => None,
    }
}

fn poll_input(should_stop: Arc<AtomicBool>) -> Result<()> {
    while !should_stop.load(Ordering::Relaxed) {
        if !event::poll(POLL_INTERVAL).context("failed to poll terminal events")? {
            continue;
        }

        match event::read().context("failed to read terminal event")? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                eventing::send_key(key).context("failed to send key event")?;
            }
            Event::Mouse(mouse) => {
                eventing::send_mouse(mouse.kind).context("failed to send mouse event")?;
            }
            Event::Resize(_, _) => {
                eventing::send_redraw().context("failed to send redraw event")?;
            }
            _ => {}
        }
    }

    Ok(())
}
