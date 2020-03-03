use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use log::{debug, error, info, warn};
use waithandle::{EventWaitHandle, WaitHandle};

use crate::config::{Configuration, ConfigurationLoader};
use crate::utils::NaiveMessageBus;
use crate::DuckResult;

///////////////////////////////////////////////////////////
// Engine handle

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

///////////////////////////////////////////////////////////
// Messages

#[derive(Clone)]
enum EngineThreadMessage {
    CollectorStarted,
    AggregatorStarted,
    ConfigurationUpdated(Configuration),
}

///////////////////////////////////////////////////////////
// Engine

pub struct Engine {}
impl Engine {
    pub fn new() -> DuckResult<Self> {
        Ok(Engine {})
    }

    pub fn run(&self, loader: impl ConfigurationLoader + 'static) -> DuckResult<EngineHandle> {
        let handle = Arc::new(EventWaitHandle::new());
        let bus = Arc::new(NaiveMessageBus::<EngineThreadMessage>::new());

        debug!("Starting configuration watcher...");
        let watcher = std::thread::spawn({
            let handle = handle.clone();
            let bus = bus.clone();
            move || -> DuckResult<()> { watch_configuration(handle, bus, loader) }
        });

        debug!("Starting collector thread...");
        let collector = std::thread::spawn({
            let handle = handle.clone();
            let bus = bus.clone();
            move || -> DuckResult<()> { run_collecting(handle, bus) }
        });

        debug!("Starting aggregator thread...");
        let aggregator = std::thread::spawn({
            let handle = handle.clone();
            let bus = bus.clone();
            move || -> DuckResult<()> { run_aggregation(handle, bus) }
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

fn watch_configuration(
    handle: Arc<dyn WaitHandle>,
    bus: Arc<NaiveMessageBus<EngineThreadMessage>>,
    loader: impl ConfigurationLoader,
) -> DuckResult<()> {
    let message_receiver = bus.subscribe();
    let mut configuration_exist = true;
    let mut configuration_loaded = false;
    let mut configuration_could_not_be_checked = false;
    let mut configuration_has_error = false;

    // Wait for collector and observer to start.
    let mut collector_started = false;
    let mut observer_started = false;
    while !collector_started || !observer_started {
        match message_receiver.recv() {
            Ok(message) => {
                match message {
                    EngineThreadMessage::CollectorStarted => {
                        debug!("Collector thread has been started.");
                        collector_started = true;
                    }
                    EngineThreadMessage::AggregatorStarted => {
                        debug!("Aggregator thread has been started.");
                        observer_started = true;
                    }
                    _ => {}
                }
            },
            Err(err) => {
                error!("An error occured while waiting for threads to start: {}", err);
                break;
            },
        }
    }

    // TODO: Refactor this
    loop {
        if loader.exist() {
            configuration_exist = true;
            match loader.has_changed() {
                Ok(has_changed) => {
                    if has_changed {
                        configuration_could_not_be_checked = false;
                        match loader.load() {
                            Ok(config) => {
                                // Notify subscribers about the new configuration file.
                                if configuration_loaded {
                                    info!("Configuration was updated.");
                                } else {
                                    info!("Configuration loaded.");
                                    configuration_loaded = true;
                                }
                                bus.send(EngineThreadMessage::ConfigurationUpdated(config))?;
                                configuration_has_error = false;
                            }
                            Err(err) => {
                                if !configuration_has_error {
                                    error!("Could not load configuration file: {}", err);
                                }
                                configuration_has_error = true
                            }
                        };
                    }
                }
                Err(err) => {
                    if !configuration_could_not_be_checked {
                        error!("Could not check configuration file: {}", err)
                    }
                    configuration_could_not_be_checked = true;
                }
            };
        } else {
            if configuration_exist {
                warn!("Configuration file could not be found.");
                configuration_exist = false;
            }
        }

        // Wait for a little while.
        if handle.wait(Duration::from_secs(5))? {
            break;
        }
    }
    Ok(())
}

///////////////////////////////////////////////////////////
// Collecting

fn run_collecting(
    handle: Arc<dyn WaitHandle>,
    bus: Arc<NaiveMessageBus<EngineThreadMessage>>,
) -> DuckResult<()> {
    let message_receiver = bus.subscribe();
    let mut configuration: Option<Configuration> = None;
    bus.send(EngineThreadMessage::CollectorStarted)?;
    loop {
        // Have the configuration been updated?
        if let Ok(message) = message_receiver.try_recv() {
            match message {
                EngineThreadMessage::ConfigurationUpdated(config) => {
                    if configuration.is_some() {
                        // TODO: Reload
                    }
                    configuration = Some(config);
                }
                _ => {}
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

fn run_aggregation(
    handle: Arc<dyn WaitHandle>,
    bus: Arc<NaiveMessageBus<EngineThreadMessage>>,
) -> DuckResult<()> {
    let message_receiver = bus.subscribe();
    let mut configuration: Option<Configuration> = None;
    bus.send(EngineThreadMessage::AggregatorStarted)?;
    loop {
        if let Ok(message) = message_receiver.try_recv() {
            match message {
                EngineThreadMessage::ConfigurationUpdated(config) => {
                    if configuration.is_some() {
                        // TODO: Reload
                    }
                    configuration = Some(config);
                }
                _ => {}
            }
        }

        if handle.wait(Duration::from_secs(2))? {
            break;
        }
    }
    Ok(())
}
