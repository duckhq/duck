use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use log::info;
use waithandle::{EventWaitHandle, WaitHandle};

use crate::DuckResult;
use crate::config::ConfigurationLoader;
use crate::utils::NaiveMessageBus;

pub struct EngineHandle {
    wait_handle: Arc<EventWaitHandle>,
    watcher: JoinHandle<DuckResult<()>>,
    collector: JoinHandle<DuckResult<()>>,
    aggregator: JoinHandle<DuckResult<()>>,
}

impl EngineHandle {
    pub fn stop(self) -> DuckResult<()> {
        info!("Stopping engine...");
        self.wait_handle.signal()?;
        self.watcher.join().unwrap()?;
        self.collector.join().unwrap()?;
        self.aggregator.join().unwrap()?;
        Ok(())
    }
}

#[derive(Clone)]
pub enum EngineThreadMessage {
    CollectorStarted,
    AggregatorStarted,
}

pub struct Engine<'a> {
    _config: Box<&'a dyn ConfigurationLoader>,
}

impl<'a> Engine<'a> {
    pub fn new(config: &'a impl ConfigurationLoader) -> DuckResult<Self> {
        Ok(Engine {
            _config: Box::new(config),
        })
    }

    pub fn run(&self) -> DuckResult<EngineHandle> {
        info!("Starting engine...");
        let handle = Arc::new(EventWaitHandle::new());
        let bus = NaiveMessageBus::<EngineThreadMessage>::new();

        info!("Starting configuration watcher...");
        let watcher = std::thread::spawn({
            let handle = handle.clone();
            let foo = bus.clone();
            move || -> DuckResult<()> { watch_configuration(handle, foo) }
        });

        info!("Starting collector thread...");
        let collector = std::thread::spawn({
            let handle = handle.clone();
            let foo = bus.clone();
            move || -> DuckResult<()> { run_collectors(handle, foo) }
        });

        info!("Starting aggregator thread...");
        let aggregator = std::thread::spawn({
            let handle = handle.clone();
            let foo = bus.clone();
            move || -> DuckResult<()> { run_aggregation(handle, foo) }
        });

        Ok(EngineHandle {
            wait_handle: handle,
            collector,
            aggregator,
            watcher,
        })
    }
}

///////////////////////////////////////////////////////////
// Configuration watcher

fn watch_configuration(handle: Arc<dyn WaitHandle>, bus: NaiveMessageBus<EngineThreadMessage>) -> DuckResult<()> {
    let message_receiver = bus.subscribe();

    // Wait for collector and observer to start.
    let mut collector_started = false;
    let mut observer_started = false;
    while !collector_started || !observer_started {
        if let Ok(message) = message_receiver.try_recv() {
            match message {
                EngineThreadMessage::CollectorStarted => {
                    info!("Collector was started!");
                    collector_started = true;
                },
                EngineThreadMessage::AggregatorStarted => {
                    info!("Aggregator was started!");
                    observer_started = true;
                }
            }
        }
    }

    loop {
        // TODO: Check if the configuration have been updated.

        // Wait for a little while.
        if handle.wait(Duration::from_secs(5))? {
            break;
        }
    }
    Ok(())
}

///////////////////////////////////////////////////////////
// Collecting

fn run_collectors(handle: Arc<dyn WaitHandle>, bus: NaiveMessageBus<EngineThreadMessage>) -> DuckResult<()> {
    let message_receiver = bus.subscribe();
    bus.send(EngineThreadMessage::CollectorStarted)?;
    loop {
        // Have the configuration been updated?
        if let Ok(message) = message_receiver.try_recv() {
            match message {
                _ => { },
            }
        }

        if handle.wait(Duration::from_secs(1))? {
            break;
        }
    }
    Ok(())
}

///////////////////////////////////////////////////////////
// Aggregation

fn run_aggregation(handle: Arc<dyn WaitHandle>, bus: NaiveMessageBus<EngineThreadMessage>) -> DuckResult<()> {
    let message_receiver = bus.subscribe();
    bus.send(EngineThreadMessage::AggregatorStarted)?;
    loop {
        if let Ok(message) = message_receiver.try_recv() {
            match message {
                _ => { },
            }
        }

        if handle.wait(Duration::from_secs(2))? {
            break;
        }
    }
    Ok(())
}
