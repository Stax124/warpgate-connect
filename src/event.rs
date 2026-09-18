use color_eyre::eyre::OptionExt;
use crossterm::event::Event as CrosstermEvent;
use futures::StreamExt;
use tokio::sync::mpsc;

use crate::app::ConnectionType;
use crate::warpgate::target::WarpgateTarget;

#[derive(Debug)]
pub enum Event {
    Crossterm(CrosstermEvent),
    App(AppEvent),
}

#[derive(Debug)]
pub enum AppEvent {
    Quit,
    TargetSelected,
    ConnectionTypeSelected(ConnectionType),
    RefreshTargets,
    TargetsFetched(color_eyre::Result<Vec<WarpgateTarget>>),
    CheckForUpdate,
    UpdateAvailable(String),
    TriggerUpdate,
}

#[derive(Debug)]
pub struct EventHandler {
    pub sender: mpsc::UnboundedSender<Event>,
    receiver: mpsc::UnboundedReceiver<Event>,
}

impl EventHandler {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        tokio::spawn(forward_crossterm_events(sender.clone()));
        tracing::debug!("Event handler initialized");
        Self { sender, receiver }
    }

    /// Blocks until an event is received.
    ///
    /// # Errors
    ///
    /// Returns an error if the sender channel is disconnected, which in practice means the event
    /// task died.
    pub async fn next(&mut self) -> color_eyre::Result<Event> {
        self.receiver
            .recv()
            .await
            .ok_or_eyre("Failed to receive event")
    }

    pub fn send(&mut self, app_event: AppEvent) {
        // Ignore the result as the receiver cannot be dropped while this struct still has a
        // reference to it
        let _ = self.sender.send(Event::App(app_event));
    }
}

/// Forwards crossterm events to the application until the receiver is dropped.
async fn forward_crossterm_events(sender: mpsc::UnboundedSender<Event>) {
    let mut reader = crossterm::event::EventStream::new();
    loop {
        tokio::select! {
          _ = sender.closed() => {
            break;
          }
          Some(Ok(event)) = reader.next() => {
            // Ignores the result because shutting down the app drops the receiver, which causes
            // the send operation to fail. This is expected behavior and should not panic.
            let _ = sender.send(Event::Crossterm(event));
          }
        };
    }
}
