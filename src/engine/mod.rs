use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread::JoinHandle;

use crate::builds::{Build, BuildStatus};
use crate::config::Configuration;
use crate::providers::collectors::*;
use crate::providers::observers::*;
use crate::providers::*;
use crate::utils::DuckResult;

use self::state::EngineState;

use log::{debug, error, info};
use waithandle::{EventWaitHandle, WaitHandle};

mod collecting;
mod observing;
pub mod state;

pub struct Engine<'a> {
    config: &'a Configuration,
    state: Arc<EngineState>,
    providers: DuckProviderCollection<'a>,
}

pub struct EngineHandle {
    wait_handle: Arc<EventWaitHandle>,
    collector_thread: JoinHandle<DuckResult<()>>,
    observer_thread: JoinHandle<DuckResult<()>>,
}

impl EngineHandle {
    pub fn stop(self) -> DuckResult<()> {
        self.wait_handle.signal()?;
        self.collector_thread.join().unwrap()?;
        self.observer_thread.join().unwrap()?;
        Ok(())
    }
}

pub enum EngineEvent {
    /// The build was updated.
    BuildUpdated(Box<Build>),
    /// Build status changed from Success->Failure or vice versa.
    BuildStatusChanged(Box<Build>),
    /// Duck is shutting down.
    ShuttingDown,
}

impl<'a> Engine<'a> {
    pub fn new(config: &'a Configuration) -> DuckResult<Self> {
        Ok(Engine {
            config,
            state: Arc::new(EngineState::new(config)),
            providers: DuckProviderCollection::new(),
        })
    }

    pub fn get_state(&self) -> Arc<EngineState> {
        self.state.clone()
    }

    pub fn run(&self) -> DuckResult<EngineHandle> {
        let handle = Arc::new(EventWaitHandle::new());
        let (sender, receiver) = channel::<EngineEvent>();

        // Create all collectors.
        let collectors = self.providers.get_collectors(self.config)?;
        let observers = self.providers.get_observers(self.config)?;

        debug!("Starting observer thread...");
        let observer_thread = std::thread::spawn({
            let state = self.state.clone();
            move || -> DuckResult<()> { run_observer_thread(state, observers, receiver) }
        });

        debug!("Starting collector thread...");
        let collector_thread = std::thread::spawn({
            let handle = handle.clone();
            let config = self.config.clone();
            let state = self.state.clone();
            move || -> DuckResult<()> {
                run_collector_thread(handle, state, config, collectors, sender)
            }
        });

        info!("Engine started.");
        Ok(EngineHandle {
            wait_handle: handle,
            collector_thread,
            observer_thread,
        })
    }
}

fn run_collector_thread(
    handle: Arc<EventWaitHandle>,
    state: Arc<EngineState>,
    config: Configuration,
    collectors: Vec<Box<dyn Collector>>,
    sender: Sender<EngineEvent>,
) -> DuckResult<()> {
    let interval: u64 = config.get_interval();

    for collector in collectors.iter() {
        info!("Added collector '{}'.", collector.info().id);
    }

    let context = collecting::Context {
        handle: handle.clone(),
        state: state.clone(),
        sender: sender.clone(),
    };

    while !handle.check().unwrap() {
        for collector in collectors.iter() {
            if handle.check().unwrap() {
                break;
            }
            collecting::process(&context, collector);
        }

        // Wait for a little while
        if handle
            .wait(std::time::Duration::from_secs(interval))
            .unwrap()
        {
            info!("We've been instructed to stop.");
            break;
        }
    }

    match sender.send(EngineEvent::ShuttingDown) {
        Result::Ok(_) => (),
        Result::Err(e) => error!("Failed to send shut down event. {}", e),
    }

    Ok(())
}

fn run_observer_thread(
    state: Arc<EngineState>,
    observers: Vec<Box<dyn Observer>>,
    receiver: Receiver<EngineEvent>,
) -> DuckResult<()> {
    for observer in observers.iter() {
        info!("Added observer '{}'.", observer.info().id);
        if let Some(collectors) = &observer.info().collectors {
            for collector in collectors {
                info!(
                    "Observer '{}' is interested in collector '{}'",
                    observer.info().id,
                    collector
                );
            }
        };
    }

    let mut context = observing::Context {
        observers: &observers,
        state: state.clone(),
        observer_status: HashMap::<&str, BuildStatus>::new(),
        status: BuildStatus::Unknown,
    };

    loop {
        let result = receiver.recv();
        if result.is_err() {
            match result {
                Result::Ok(_) => (),
                Result::Err(e) => {
                    info!("Observer have been disconnected! {}", e);
                    break;
                }
            }
        }

        let command = result.unwrap();
        if observing::process(&mut context, command) {
            break;
        }
    }

    Ok(())
}
