use color_eyre::eyre::OptionExt;
use crossterm::event::Event as CrosstermEvent;
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;

/// Representation of all possible events.
#[derive(Clone, Debug)]
pub enum Event {
    /// Crossterm events.
    ///
    /// These events are emitted by the terminal.
    Crossterm(CrosstermEvent),
    /// Application events.
    ///
    /// Use this event to emit custom events that are specific to your application.
    App(AppEvent),
    /// Request a redraw of the UI.
    Render,
}

/// Application events.
///
/// You can extend this enum with your own custom events.
#[derive(Clone, Debug)]
pub enum AppEvent {
    /// Move to the next item.
    NextItem,
    /// Move to the previous item.
    PrevItem,
    /// Jump to the first item.
    FirstItem,
    /// Jump to the last item.
    LastItem,
    /// Deselect the current item.
    Deselect,
    /// Quit the application.
    Quit,
    /// A target was selected.
    TargetSelected,
    /// Refresh the list of warpgate targets.
    RefreshTargets,
    /// Recalculate the filtered targets (e.g. after a fetch completes).
    RecalculateTargets,
    /// Check for application updates in the background.
    CheckForUpdate,
    /// An update is available with the given version string.
    UpdateAvailable(String),
    /// The user wants to trigger the update process.
    TriggerUpdate,
}

/// Terminal event handler.
#[derive(Debug)]
pub struct EventHandler {
    /// Event sender channel.
    pub sender: mpsc::UnboundedSender<Event>,
    /// Event receiver channel.
    receiver: mpsc::UnboundedReceiver<Event>,
}

impl EventHandler {
    /// Constructs a new instance of [`EventHandler`] and spawns a new thread to handle events.
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let actor = EventTask::new(sender.clone());
        tokio::spawn(async { actor.run().await });
        tracing::debug!("Event handler initialized");
        Self { sender, receiver }
    }

    /// Receives an event from the sender.
    ///
    /// This function blocks until an event is received.
    ///
    /// # Errors
    ///
    /// This function returns an error if the sender channel is disconnected. This can happen if an
    /// error occurs in the event thread. In practice, this should not happen unless there is a
    /// problem with the underlying terminal.
    pub async fn next(&mut self) -> color_eyre::Result<Event> {
        self.receiver
            .recv()
            .await
            .ok_or_eyre("Failed to receive event")
    }

    /// Queue an app event to be sent to the event receiver.
    ///
    /// This is useful for sending events to the event handler which will be processed by the next
    /// iteration of the application's event loop.
    pub fn send(&mut self, app_event: AppEvent) {
        // Ignore the result as the receiver cannot be dropped while this struct still has a
        // reference to it
        let _ = self.sender.send(Event::App(app_event));
    }
}

/// A thread that handles reading crossterm events and emitting tick events on a regular schedule.
struct EventTask {
    /// Event sender channel.
    sender: mpsc::UnboundedSender<Event>,
}

impl EventTask {
    fn new(sender: mpsc::UnboundedSender<Event>) -> Self {
        Self { sender }
    }

    /// Runs the event thread.
    ///
    /// This function waits for crossterm events and forwards them to the application.
    async fn run(self) -> color_eyre::Result<()> {
        let mut reader = crossterm::event::EventStream::new();
        loop {
            let crossterm_event = reader.next().fuse();
            tokio::select! {
              _ = self.sender.closed() => {
                break;
              }
              Some(Ok(evt)) = crossterm_event => {
                self.send(Event::Crossterm(evt));
              }
            };
        }
        Ok(())
    }

    /// Sends an event to the receiver.
    fn send(&self, event: Event) {
        // Ignores the result because shutting down the app drops the receiver, which causes the send
        // operation to fail. This is expected behavior and should not panic.
        let _ = self.sender.send(event);
    }
}
