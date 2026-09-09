use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use crossterm::event::{self, Event, KeyEventKind};
use eyre::{Context, Result};
use tracing::error;

use crate::eventing;

const POLL_INTERVAL: Duration = Duration::from_millis(100);

pub struct InputThread {
    should_stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl InputThread {
    pub fn spawn() -> Result<InputThread> {
        let should_stop = Arc::new(AtomicBool::new(false));
        let thread_should_stop = Arc::clone(&should_stop);

        let handle = thread::Builder::new()
            .name("input".to_owned())
            .spawn(move || {
                if let Err(error) = InputThread::input_loop(thread_should_stop) {
                    error!(?error, "input thread failed");
                }
            })
            .context("failed to spawn input thread")?;

        Ok(InputThread {
            should_stop,
            handle: Some(handle),
        })
    }

    fn input_loop(should_stop: Arc<AtomicBool>) -> Result<()> {
        while !should_stop.load(Ordering::Relaxed) {
            if !event::poll(POLL_INTERVAL).context("failed to poll terminal events")? {
                continue;
            }

            match event::read().context("failed to read terminal event")? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    eventing::send_key(key).context("failed to send key event")?;
                }
                Event::Mouse(mouse) => {
                    eventing::send_mouse(mouse).context("failed to send mouse event")?;
                }
                Event::Resize(_, _) => {
                    eventing::send_redraw().context("failed to send redraw event")?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub fn close(mut self) -> eyre::Result<()> {
        self.should_stop.store(true, Ordering::Relaxed);

        if let Some(handle) = self.handle.take() {
            if handle.join().is_err() {
                return Err(eyre::eyre!("failed to join input handle"));
            };
        }

        Ok(())
    }
}

impl Drop for InputThread {
    fn drop(&mut self) {
        self.should_stop.store(true, Ordering::Relaxed);
    }
}
