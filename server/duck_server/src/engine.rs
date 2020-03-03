use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::sync::Barrier;
use std::thread::JoinHandle;
use std::time::Duration;

use log::{debug, info, trace};
use waithandle::{EventWaitHandle, WaitHandle};

use crate::config::{Configuration, ConfigurationLoader};
use crate::utils::NaiveMessageBus;
use crate::DuckResult;

mod accumulator;
mod aggregator;
mod watcher;

///////////////////////////////////////////////////////////
// Engine handle

pub struct EngineHandle {
    wait_handle: Arc<EventWaitHandle>,
    watcher: JoinHandle<DuckResult<()>>,
    accumulator: JoinHandle<DuckResult<()>>,
    aggregator: JoinHandle<DuckResult<()>>,
}

impl EngineHandle {
    pub fn stop(self) -> DuckResult<()> {
        info!("Stopping engine...");
        self.wait_handle.signal()?;
        self.watcher.join().unwrap()?;
        self.accumulator.join().unwrap()?;
        self.aggregator.join().unwrap()?;
        Ok(())
    }
}

///////////////////////////////////////////////////////////
// Messages

#[derive(Clone)]
pub enum EngineThreadMessage {
    ConfigurationUpdated(Configuration),
}

///////////////////////////////////////////////////////////
// Engine

pub fn run(loader: impl ConfigurationLoader + 'static) -> DuckResult<EngineHandle> {
    let stopping = Arc::new(EventWaitHandle::new());
    let bus = Arc::new(NaiveMessageBus::<EngineThreadMessage>::new());
    let barrier = Arc::new(Barrier::new(3));

    // Configuration watcher thread
    debug!("Starting configuration watcher thread...");
    let watcher = std::thread::spawn({
        let barrier = barrier.clone();
        let stopping = stopping.clone();
        let bus = bus.clone();
        move || -> DuckResult<()> { watch_configuration(barrier, stopping, bus, loader) }
    });

    // Accumulator thread
    debug!("Starting accumulator thread...");
    let accumulator = std::thread::spawn({
        let barrier = barrier.clone();
        let stopping = stopping.clone();
        let bus = bus.clone();
        move || -> DuckResult<()> { run_accumulator(barrier, stopping, bus) }
    });

    // Aggregator thread
    debug!("Starting aggregator thread...");
    let aggregator = std::thread::spawn({
        let barrier = barrier.clone();
        let stopping = stopping.clone();
        let bus = bus.clone();
        move || -> DuckResult<()> { run_aggregator(barrier, stopping, bus) }
    });

    Ok(EngineHandle {
        wait_handle: stopping,
        accumulator,
        aggregator,
        watcher,
    })
}

fn watch_configuration(
    barrier: Arc<Barrier>,
    stopping: Arc<dyn WaitHandle>,
    bus: Arc<NaiveMessageBus<EngineThreadMessage>>,
    loader: impl ConfigurationLoader,
) -> DuckResult<()> {
    // Signal other threads that we've started
    barrier.wait();
    debug!("Configuration watcher thread started.");

    let mut context = watcher::Context::new();
    loop {
        // Check if the configuration have changed
        if let Some(config) = watcher::has_changed(&mut context, &loader) {
            trace!("Sending configuration updated message.");
            bus.send(EngineThreadMessage::ConfigurationUpdated(config))?;
        }

        // Time to bail?
        if stopping.wait(Duration::from_secs(5))? {
            break;
        }
    }

    Ok(())
}

fn run_accumulator(
    barrier: Arc<Barrier>,
    stopping: Arc<dyn WaitHandle>,
    bus: Arc<NaiveMessageBus<EngineThreadMessage>>,
) -> DuckResult<()> {
    // Subscribe to engine messages
    let receiver = bus.subscribe();

    // Wait for other threads to start
    barrier.wait();
    debug!("Accumulator thread started.");

    let context = accumulator::Context::new(stopping.clone(), receiver);
    loop {
        accumulator::accumulate(&context);
        if stopping.wait(Duration::from_secs(1))? {
            break;
        }
    }

    Ok(())
}

fn run_aggregator(
    barrier: Arc<Barrier>,
    stopping: Arc<dyn WaitHandle>,
    bus: Arc<NaiveMessageBus<EngineThreadMessage>>,
) -> DuckResult<()> {
    // Subscribe to engine messages
    let receiver = bus.subscribe();

    // Wait for other threads to start
    barrier.wait();
    debug!("Aggregator thread started.");

    let context = aggregator::Context::new(stopping.clone(), receiver);
    loop {
        aggregator::aggregate(&context);
        if stopping.wait(Duration::from_secs(1))? {
            break;
        }
    }

    Ok(())
}

///////////////////////////////////////////////////////////
// Utilities

fn try_get_updated_configuration(
    receiver: &Receiver<EngineThreadMessage>,
) -> Option<Configuration> {
    loop {
        match receiver.try_recv() {
            Ok(message) => match message {
                EngineThreadMessage::ConfigurationUpdated(config) => {
                    return Some(config);
                }
            },
            Err(_) => break,
        }
    }
    None
}
