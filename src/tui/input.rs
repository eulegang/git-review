use std::{
    mem::forget,
    sync::{
        Arc, Condvar, Mutex,
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

struct State {
    should_stop: Arc<AtomicBool>,
    paused: Arc<(Mutex<bool>, Condvar)>,
}

pub struct InputThread {
    state: State,
    handle: Option<JoinHandle<()>>,
}

impl InputThread {
    pub fn spawn() -> Result<InputThread> {
        let should_stop = Arc::new(AtomicBool::new(false));
        let paused = Arc::new((Mutex::new(false), Condvar::new()));

        let thread_state = State {
            should_stop: Arc::clone(&should_stop),
            paused: Arc::clone(&paused),
        };

        let state = State {
            should_stop,
            paused,
        };

        let handle = thread::Builder::new()
            .name("input".to_owned())
            .spawn(move || {
                if let Err(error) = InputThread::input_loop(thread_state) {
                    error!(?error, "input thread failed");
                }
            })
            .context("failed to spawn input thread")?;

        Ok(InputThread {
            state,
            handle: Some(handle),
        })
    }

    fn input_loop(state: State) -> Result<()> {
        while !state.should_stop.load(Ordering::Relaxed) {
            InputThread::check_paused(&state);

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

    fn check_paused(state: &State) {
        let (lock, cond) = &*state.paused;

        let mut paused = lock.lock().unwrap();
        while *paused {
            paused = cond.wait(paused).unwrap();
        }
    }

    pub fn close(mut self) -> eyre::Result<()> {
        self.state.should_stop.store(true, Ordering::Relaxed);

        if let Some(handle) = self.handle.take() {
            if handle.join().is_err() {
                return Err(eyre::eyre!("failed to join input handle"));
            };
        }

        forget(self);

        Ok(())
    }
}

impl Drop for InputThread {
    fn drop(&mut self) {
        self.state.should_stop.store(true, Ordering::Relaxed);
    }
}
