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

use self::state::{BuildUpdateResult, EngineState};

use log::{debug, error, info};
use waithandle::{EventWaitHandle, WaitHandle};

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
            state: Arc::new(EngineState::new()),
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
            move || -> DuckResult<()> {
                return run_observers(state, observers, receiver);
            }
        });

        debug!("Starting collector thread...");
        let collector_thread = std::thread::spawn({
            let handle = handle.clone();
            let config = self.config.clone();
            let state = self.state.clone();
            move || -> DuckResult<()> {
                return run_collectors(handle, state, config, collectors, sender);
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

fn run_collectors(
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

    while !handle.check().unwrap() {
        for collector in collectors.iter() {
            if handle.check().unwrap() {
                break;
            }

            let mut build_hashes = std::collections::HashSet::<u64>::new();
            if let Err(e) = collector.collect(handle.clone(), &mut |build: Build| {
                build_hashes.insert(build.id);
                match state.builds.update(&build) {
                    BuildUpdateResult::Added | BuildUpdateResult::BuildUpdated => {
                        // The build was updated
                        match sender.send(EngineEvent::BuildUpdated(Box::new(build))) {
                            Result::Ok(_) => (),
                            Result::Err(e) => error!("Failed to send build update event. {}", e),
                        }
                    }
                    BuildUpdateResult::BuildStatusChanged => {
                        // The build's status was changed (success->failed or failed->success)
                        match sender.send(EngineEvent::BuildStatusChanged(Box::new(build))) {
                            Result::Ok(_) => (),
                            Result::Err(e) => {
                                error!("Failed to send canonical build update event. {}", e)
                            }
                        }
                    }
                    _ => {}
                };
            }) {
                // Log the error but continue as normal since
                // we don't want to retain the builds that we could
                // not collect information about
                error!(
                    "An error occured while collecting builds from '{}': {}",
                    collector.info().id,
                    e
                );
            };

            // Retain builds that were updated
            state.builds.retain_builds(&collector.info(), build_hashes);
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

fn run_observers(
    state: Arc<EngineState>,
    observers: Vec<Box<dyn Observer>>,
    receiver: Receiver<EngineEvent>,
) -> DuckResult<()> {
    let mut stopped = false;
    let mut observer_status = HashMap::<&str, BuildStatus>::new();
    let mut overall_status = BuildStatus::Unknown;

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

    while !stopped {
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
        match command {
            EngineEvent::BuildUpdated(build) => {
                // Did the build status change?
                let status = state.builds.current_status();
                let overall_status_changed = if overall_status != status {
                    overall_status = status;
                    true
                } else {
                    false
                };

                // Did the overall build status change for any observers?
                for observer in &observers {
                    // Only interested in specific collectors?
                    if let Some(collectors) = &observer.info().collectors {
                        let previous_status = observer_status
                            .entry(&observer.info().id)
                            .or_insert(BuildStatus::Unknown);
                        let current_status = state.builds.current_status_for_collectors(collectors);
                        if *previous_status != current_status {
                            // Status changed so send this to the observer.
                            propagate_to_observer(
                                observer,
                                Observation::DuckStatusChanged(current_status.clone()),
                            );
                            *previous_status = current_status;
                        }
                    } else {
                        // Not interested in specific collectors.
                        // So did the overall build status change?
                        if overall_status_changed {
                            // Notify the observer.
                            propagate_to_observer(
                                observer,
                                Observation::DuckStatusChanged(overall_status.clone()),
                            );
                        }
                    }
                }

                // Send the BuildUpdated event to observers.
                propagate_to_observers(&observers, &mut || Observation::BuildUpdated(&build));
            }
            EngineEvent::BuildStatusChanged(build) => {
                // Send the BuildUpdated event to observers.
                propagate_to_observers(&observers, &mut || Observation::BuildUpdated(&build));
                // Send the BuildStatusChanged event to observers.
                propagate_to_observers(&observers, &mut || Observation::BuildStatusChanged(&build));
            }
            EngineEvent::ShuttingDown => {
                // Send the ShuttingDown event to observers.
                propagate_to_observers(&observers, &mut || Observation::ShuttingDown);
                stopped = true;
            }
        }
    }

    Ok(())
}

fn propagate_to_observers<'a>(
    observers: &[Box<dyn Observer>],
    observation: &mut dyn Fn() -> Observation<'a>,
) {
    // Iterate through all observers.
    for observer in observers.iter() {
        let observation = observation();

        // Is the origin of the observation a collector?
        if let ObservationOrigin::Collector(collector) = observation.get_origin() {
            if let Some(collectors) = &observer.info().collectors {
                if !collectors.contains(collector) {
                    // The observer is not interested in the origin.
                    return;
                }
            }
        }

        propagate_to_observer(observer, observation);
    }
}

#[allow(clippy::borrowed_box)]
fn propagate_to_observer(observer: &Box<dyn Observer>, observation: Observation) {
    match observer.observe(observation) {
        Result::Ok(_) => (),
        Result::Err(e) => {
            error!("An error occured when sending observation. {}", e);
        }
    };
}
