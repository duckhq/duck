use std::collections::HashMap;
use std::sync::Arc;

use log::error;

use crate::builds::{Build, BuildStatus};
use crate::engine::{EngineEvent, EngineState};
use crate::providers::observers::*;

pub struct Context<'a> {
    pub state: Arc<EngineState>,
    pub observers: &'a Vec<Box<dyn Observer>>,
    pub observer_status: HashMap<&'a str, BuildStatus>,
    pub status: BuildStatus,
}

pub fn process(context: &mut Context, command: EngineEvent) -> bool {
    let mut stopped = false;

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
            stopped = true;
        }
    }

    return stopped;
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
    for observer in context.observers {
        // Only interested in specific collectors?
        if let Some(collectors) = &observer.info().collectors {
            let previous_status = context
                .observer_status
                .entry(&observer.info().id)
                .or_insert(BuildStatus::Unknown);
            let current_status = context
                .state
                .builds
                .current_status_for_collectors(collectors);
            if *previous_status != current_status && *previous_status != BuildStatus::Unknown {
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