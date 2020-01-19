use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread::JoinHandle;

pub mod state;

use crate::builds::{Build, BuildStatus};
use crate::collectors;
use crate::config::Configuration;
use crate::engine::state::{BuildUpdateResult, EngineState};
use crate::observers;
use crate::observers::{Observation, ObservationOrigin, Observer};
use crate::utils::DuckResult;

use log::{error, info};
use waithandle::{EventWaitHandle, WaitHandle};

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

pub fn run(config: &Configuration, state: &Arc<EngineState>) -> EngineHandle {
    let handle = Arc::new(EventWaitHandle::new());

    let (sender, receiver) = channel::<EngineEvent>();

    info!("Starting observers...");
    let observer_thread = std::thread::spawn({
        let config = config.clone();
        let state = state.clone();
        move || -> DuckResult<()> {
            return run_observers(config, state, receiver);
        }
    });

    info!("Starting collectors...");
    let collector_thread = std::thread::spawn({
        let handle = handle.clone();
        let config = config.clone();
        let state = state.clone();
        move || -> DuckResult<()> {
            return run_collectors(handle, state, config, sender);
        }
    });

    info!("Engine started.");
    EngineHandle {
        wait_handle: handle,
        collector_thread,
        observer_thread,
    }
}

fn run_collectors(
    handle: Arc<EventWaitHandle>,
    state: Arc<EngineState>,
    config: Configuration,
    sender: Sender<EngineEvent>,
) -> DuckResult<()> {
    // Create the collectors from the configuration
    let collectors = collectors::create_collectors(&config);

    // Get the update interval and clamp it to at least 15 seconds.
    let mut interval: u64 = 15;
    if let Some(i) = config.interval {
        if interval > 15 {
            interval = u64::from(i.0);
        }
    }

    while !handle.check().unwrap() {
        for collector in collectors.iter() {
            if handle.check().unwrap() {
                break;
            }

            let mut build_hashes = std::collections::HashSet::<u64>::new();
            if let Err(e) = collector.collect(handle.clone(), &mut |build: crate::builds::Build| {
                build_hashes.insert(build.id);
                match state.builds.update(&build) {
                    BuildUpdateResult::Added | BuildUpdateResult::BuildUpdated => {
                        info!("New build: {}", build.definition_id);
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
    config: Configuration,
    state: Arc<EngineState>,
    receiver: Receiver<EngineEvent>,
) -> DuckResult<()> {
    let observers = observers::create_observers(&config);

    let mut stopped = false;
    let mut observer_status = HashMap::<&str, BuildStatus>::new();
    let mut overall_status = BuildStatus::Unknown;

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
                propagate_to_observers(&observers, &mut || {
                    Observation::BuildUpdated(build.clone())
                });
            }
            EngineEvent::BuildStatusChanged(build) => {
                // Send the BuildUpdated event to observers.
                propagate_to_observers(&observers, &mut || {
                    Observation::BuildUpdated(build.clone())
                });
                // Send the BuildStatusChanged event to observers.
                propagate_to_observers(&observers, &mut || {
                    Observation::BuildStatusChanged(build.clone())
                });
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

fn propagate_to_observers(
    observers: &[Box<dyn Observer>],
    observation: &mut dyn Fn() -> Observation,
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
