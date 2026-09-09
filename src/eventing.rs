use std::sync::{
    Mutex, OnceLock,
    mpsc::{self, Receiver, Sender},
};

use crossterm::event::{KeyEvent, MouseEventKind};

static EVENT_SENDER: OnceLock<Mutex<Option<Sender<AppEvent>>>> = OnceLock::new();

/// Events distributed through the application's global event channel.
#[derive(Debug, Clone)]
pub enum AppEvent {
    Key(KeyEvent),
    Mouse(MouseEventKind),
    Redraw,
}

/// An installed global event channel.
///
/// Dropping this value removes the global sender.
pub struct EventSystem {
    receiver: Receiver<AppEvent>,
}

impl EventSystem {
    pub fn receiver(&self) -> &Receiver<AppEvent> {
        &self.receiver
    }
}

impl Drop for EventSystem {
    fn drop(&mut self) {
        if let Some(sender) = EVENT_SENDER.get() {
            *sender.lock().expect("event sender lock poisoned") = None;
        }
    }
}

/// Install a fresh global event channel and return its receiver.
pub fn install() -> EventSystem {
    let (sender, receiver) = mpsc::channel();
    let global_sender = EVENT_SENDER.get_or_init(|| Mutex::new(None));
    *global_sender.lock().expect("event sender lock poisoned") = Some(sender);

    EventSystem { receiver }
}

/// Get a clone of the currently installed global event sender.
pub fn sender() -> Option<Sender<AppEvent>> {
    EVENT_SENDER
        .get()
        .and_then(|sender| sender.lock().expect("event sender lock poisoned").clone())
}

/// Send a key event through the global event channel.
pub fn send_key(key: KeyEvent) -> Result<(), mpsc::SendError<AppEvent>> {
    send(AppEvent::Key(key))
}

/// Send a mouse event through the global event channel.
pub fn send_mouse(kind: MouseEventKind) -> Result<(), mpsc::SendError<AppEvent>> {
    send(AppEvent::Mouse(kind))
}

/// Request that the UI redraw through the global event channel.
pub fn send_redraw() -> Result<(), mpsc::SendError<AppEvent>> {
    send(AppEvent::Redraw)
}

pub fn send(event: AppEvent) -> Result<(), mpsc::SendError<AppEvent>> {
    if let Some(sender) = sender() {
        sender.send(event)
    } else {
        Ok(())
    }
}
