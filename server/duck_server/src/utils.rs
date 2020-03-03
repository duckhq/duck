use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;

use crate::DuckResult;

/// A super naive implementation of a message bus
/// where all subscribers get all messages. I need something
/// to synchronize the threads in the engine and the option was
/// to take a dependency on something like crosstream - which
/// is a great thing - but a bit overkill for what I need atm.
pub struct NaiveMessageBus<T: Send + Sync + Clone + 'static> {
    pub channels: Mutex<Vec<Sender<T>>>,
}

impl<T: Send + Sync + Clone + 'static> NaiveMessageBus<T> {
    /// Creates a new message bus
    pub fn new() -> Self {
        NaiveMessageBus::<T> {
            channels: Mutex::new(Vec::new()),
        }
    }

    /// Subsribes to messages from the bus
    pub fn subscribe(&self) -> Receiver<T> {
        let (sender, reciever) = channel::<T>();

        let mut channels = self.channels.lock().unwrap();
        channels.push(sender);

        reciever
    }

    /// Send a message to subscribers
    pub fn send(&self, message: T) -> DuckResult<()> {
        let channels = self.channels.lock().unwrap();
        for sender in channels.iter() {
            sender.send(message.clone())?;
        }
        Ok(())
    }
}
