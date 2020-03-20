use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::Arc;

use log::{debug, error, trace};
use waithandle::WaitHandleListener;

use crate::builds::{Build, BuildStatus};
use crate::engine::{EngineEvent, EngineState, EngineThreadMessage};
use crate::providers::observers::*;
use crate::DuckResult;

pub struct Context {
    pub state: Arc<EngineState>,
    pub listener: WaitHandleListener,
    pub engine_receiver: Receiver<EngineThreadMessage>,
    pub accumulator_receiver: Receiver<EngineEvent>,
    pub observers: Vec<Box<dyn Observer>>,
    pub observer_status: HashMap<String, BuildStatus>,
    pub status: BuildStatus,
}

pub enum AggregateResult {
    Success,
    Stopped,
    Disconnected,
    TransientError,
}

pub fn aggregate(context: &mut Context) -> AggregateResult {
    if let Err(e) = check_for_updated_configuration(context) {
        error!("{}", e);
        return AggregateResult::TransientError;
    }

    loop {
        let received = context.accumulator_receiver.try_recv();
        let command = match received {
            Result::Ok(c) => c,
            Result::Err(e) => match e {
                std::sync::mpsc::TryRecvError::Empty => break,
                std::sync::mpsc::TryRecvError::Disconnected => {
                    error!("Observer have been disconnected! {}", e);
                    return AggregateResult::Disconnected;
                }
            },
        };

        match command {
            EngineEvent::BuildUpdated(build) => {
                build_updated(context, build);
            }
            EngineEvent::AbsoluteBuildStatusChanged(build) => {
                // Send the BuildUpdated event to all observers.
                propagate_to_observers(&context.observers, &mut || {
                    Observation::BuildUpdated(&build)
                });
                // Send the BuildStatusChanged event to all observers.
                propagate_to_observers(&context.observers, &mut || {
                    Observation::BuildStatusChanged(&build)
                });
            }
            EngineEvent::ShuttingDown => {
                // Send the ShuttingDown event to all observers.
                propagate_to_observers(&context.observers, &mut || Observation::ShuttingDown);
                return AggregateResult::Stopped;
            }
        }
    }

    AggregateResult::Success
}

fn check_for_updated_configuration(context: &mut Context) -> DuckResult<()> {
    if let Some(config) = super::try_get_updated_configuration(&context.engine_receiver) {
        trace!("Applying new configuration...");
        match crate::providers::create_observers(&config) {
            Ok(collectors) => {
                context.observers.clear();
                for collector in collectors {
                    debug!("Loaded collector: {:?}", collector.info().id);
                    context.observers.push(collector);
                }
            }
            Err(err) => {
                return Err(format_err!(
                    "An error occured while loading observers: {}",
                    err
                ));
            }
        }
    }
    Ok(())
}

fn build_updated(context: &mut Context, build: Box<Build>) {
    // Did the build status change?
    let status = context.state.builds.current_status();
    let overall_status_changed = if context.status != status {
        context.status = status;
        true
    } else {
        false
    };

    // Did the overall build status change for any observers?
    for observer in context.observers.iter() {
        // Only interested in specific collectors?
        if let Some(collectors) = &observer.info().collectors {
            let previous_status = context
                .observer_status
                .entry(observer.info().id.clone())
                .or_insert(BuildStatus::Unknown);
            let current_status = context
                .state
                .builds
                .current_status_for_collectors(collectors);

            if current_status.is_absolute() && *previous_status != current_status {
                trace!(
                    "Collector status changed for observer '{}' ({:?})",
                    observer.info().id,
                    current_status
                );
                // Status changed so send this to the observer.
                propagate_to_observer(
                    &observer,
                    Observation::DuckStatusChanged(current_status.clone()),
                );
                *previous_status = current_status;
            }
        } else {
            // Not interested in specific collectors.
            // So did the overall build status change?
            if overall_status_changed {
                trace!(
                    "Overall status changed for observer '{}' ({:?})",
                    observer.info().id,
                    context.status
                );
                // Notify the observer.
                propagate_to_observer(
                    &observer,
                    Observation::DuckStatusChanged(context.status.clone()),
                );
            }
        }
    }

    // Send the BuildUpdated event to all observers.
    propagate_to_observers(&context.observers, &mut || {
        Observation::BuildUpdated(&build)
    });
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
