use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Barrier};
use std::{thread::JoinHandle, time::Duration};

use log::{debug, error, info, trace};
use waithandle::{WaitHandleListener, WaitHandleSignaler};

use crate::builds::{Build, BuildStatus};
use crate::config::{Configuration, ConfigurationLoader};
use crate::utils::NaiveMessageBus;
use crate::DuckResult;

use self::state::EngineState;

mod accumulator;
mod aggregator;
mod watcher;

pub mod state;

///////////////////////////////////////////////////////////
// Engine handle

pub struct EngineHandle {
    signaler: WaitHandleSignaler,
    watcher: JoinHandle<DuckResult<()>>,
    accumulator: JoinHandle<DuckResult<()>>,
    aggregator: JoinHandle<DuckResult<()>>,
}

impl EngineHandle {
    pub fn stop(self) -> DuckResult<()> {
        info!("Shutting down engine...");
        self.signaler.signal()?;
        self.watcher.join().unwrap()?;
        trace!("The configuration watcher stopped");
        self.accumulator.join().unwrap()?;
        trace!("The accumulator stopped");
        self.aggregator.join().unwrap()?;
        trace!("The aggregator stopped");
        Ok(())
    }
}

///////////////////////////////////////////////////////////
// Events

#[derive(Clone)]
pub enum EngineThreadMessage {
    ConfigurationUpdated(Configuration),
}

pub enum EngineEvent {
    /// The build was updated.
    BuildUpdated(Box<Build>),
    /// Absolute status for a build changed from Success->Failure or vice versa.
    AbsoluteBuildStatusChanged(Box<Build>),
    /// Duck is shutting down.
    ShuttingDown,
}

#[cfg(test)]
impl EngineEvent {
    fn is_build_updated(&self) -> bool {
        match self {
            EngineEvent::BuildUpdated(_) => true,
            _ => false,
        }
    }
    fn is_build_status_changed(&self) -> bool {
        match self {
            EngineEvent::AbsoluteBuildStatusChanged(_) => true,
            _ => false,
        }
    }
}

///////////////////////////////////////////////////////////
// Engine

pub struct Engine {
    state: Arc<EngineState>,
}

impl Engine {
    pub fn new() -> DuckResult<Self> {
        Ok(Engine {
            state: Arc::new(EngineState::new()),
        })
    }

    pub fn get_state(&self) -> Arc<EngineState> {
        self.state.clone()
    }

    pub fn run(&self, loader: impl ConfigurationLoader + 'static) -> DuckResult<EngineHandle> {
        let barrier = Arc::new(Barrier::new(3));
        let bus = Arc::new(NaiveMessageBus::<EngineThreadMessage>::new());
        let (sender, receiver) = channel::<EngineEvent>();
        let (signaler, listener) = waithandle::new();

        // Configuration watcher thread
        debug!("Starting configuration watcher thread...");
        let watcher = std::thread::spawn({
            let listener = listener.clone();
            let state = self.state.clone();
            let barrier = barrier.clone();
            let bus = bus.clone();
            move || -> DuckResult<()> { watch_configuration(barrier, listener, state, bus, loader) }
        });

        debug!("Starting aggregator thread...");
        let aggregator = std::thread::spawn({
            let listener = listener.clone();
            let state = self.state.clone();
            let barrier = barrier.clone();
            let bus = bus.clone();
            move || -> DuckResult<()> { run_aggregator(barrier, listener, state, receiver, bus) }
        });

        debug!("Starting accumulator thread...");
        let accumulator = std::thread::spawn({
            let state = self.state.clone();
            move || -> DuckResult<()> { run_accumulator(barrier, listener, state, sender, bus) }
        });

        debug!("Engine started");
        Ok(EngineHandle {
            signaler,
            watcher,
            accumulator,
            aggregator,
        })
    }
}

///////////////////////////////////////////////////////////
// Watcher

fn watch_configuration(
    barrier: Arc<Barrier>,
    stopping: WaitHandleListener,
    state: Arc<EngineState>,
    bus: Arc<NaiveMessageBus<EngineThreadMessage>>,
    loader: impl ConfigurationLoader,
) -> DuckResult<()> {
    // Signal other threads that we've started
    barrier.wait();
    debug!("Configuration watcher thread started");

    let environment = crate::utils::text::EnvironmentVariableProvider::new();
    let mut context = watcher::Context::new(environment);
    let mut loaded = false;
    loop {
        // Check if the configuration have changed
        if let Some(config) = watcher::try_load(&mut context, &loader) {
            if loaded {
                info!("Reloaded Duck configuration");
            }
            loaded = true;
            state.refresh(&config);
            trace!("Sending configuration updated message");
            bus.send(EngineThreadMessage::ConfigurationUpdated(config))?;
        }

        // Time to bail?
        if stopping.wait(Duration::from_secs(5))? {
            debug!("The configuration watcher was instructed to stop");
            break;
        }
    }

    Ok(())
}

///////////////////////////////////////////////////////////
// Accumulator

fn run_accumulator(
    barrier: Arc<Barrier>,
    handle: WaitHandleListener,
    state: Arc<EngineState>,
    sender: Sender<EngineEvent>,
    bus: Arc<NaiveMessageBus<EngineThreadMessage>>,
) -> DuckResult<()> {
    // Subscribe to engine messages
    let receiver = bus.subscribe();

    barrier.wait();
    debug!("Accumulator thread started");

    let mut context = accumulator::Context::new(handle.clone(), state, receiver, sender.clone());

    trace!("Waiting for a configuration to be loaded");
    loop {
        match accumulator::check_for_updated_configuration(&mut context) {
            Err(_) => (),
            Ok(result) => match result {
                accumulator::ConfigurationResult::Unchanged => (),
                accumulator::ConfigurationResult::Updated => break,
            },
        };
        if handle.wait(Duration::from_millis(500)).unwrap() {
            break;
        }
    }

    while !handle.check().unwrap() {
        accumulator::accumulate(&mut context);

        // Wait for a little while
        if handle.wait(std::time::Duration::from_secs(15)).unwrap() {
            debug!("The accumulator was instructed to stop");
            break;
        }
    }

    debug!("Sending shutdown message");
    match sender.send(EngineEvent::ShuttingDown) {
        Result::Ok(_) => (),
        Result::Err(e) => error!("Failed to send shut down event. {}", e),
    }

    Ok(())
}

///////////////////////////////////////////////////////////
// Aggregator

fn run_aggregator(
    barrier: Arc<Barrier>,
    listener: WaitHandleListener,
    state: Arc<EngineState>,
    accumulator_receiver: Receiver<EngineEvent>,
    bus: Arc<NaiveMessageBus<EngineThreadMessage>>,
) -> DuckResult<()> {
    // Subscribe to engine messages
    let engine_receiver = bus.subscribe();

    // Wait for other threads to start
    barrier.wait();
    debug!("Aggregator thread started");

    let mut context = aggregator::Context {
        observers: Vec::new(),
        listener,
        engine_receiver,
        accumulator_receiver,
        state,
        observer_status: HashMap::<String, BuildStatus>::new(),
        status: BuildStatus::Unknown,
    };

    loop {
        match aggregator::aggregate(&mut context) {
            aggregator::AggregateResult::Stopped => {
                debug!("The aggregator was instructed to stop");
                break;
            }
            aggregator::AggregateResult::Disconnected => break,
            _ => {}
        }
    }

    Ok(())
}

///////////////////////////////////////////////////////////
// Utilities

fn try_get_updated_configuration(
    handle: &WaitHandleListener,
    receiver: &Receiver<EngineThreadMessage>,
) -> Option<Configuration> {
    let mut result: Option<Configuration> = None;
    while let Ok(message) = receiver.try_recv() {
        match message {
            EngineThreadMessage::ConfigurationUpdated(config) => {
                result = Some(config);
            }
        }
        if handle.wait(Duration::from_millis(500)).unwrap() {
            return None;
        }
    }
    result
}
