use std::sync::mpsc::Sender;
use std::sync::Arc;

use log::error;
use waithandle::EventWaitHandle;

use crate::builds::Build;
use crate::engine::{EngineEvent, EngineState};
use crate::providers::collectors::Collector;

use super::state::builds::BuildUpdateResult;

pub struct Context {
    pub handle: Arc<EventWaitHandle>,
    pub state: Arc<EngineState>,
    pub sender: Sender<EngineEvent>,
}

pub fn process(context: &Context, collector: &Box<dyn Collector>) {
    let mut build_hashes = std::collections::HashSet::<u64>::new();
    if let Err(e) = collector.collect(context.handle.clone(), &mut |build: Build| {
        build_hashes.insert(build.id);
        match context.state.builds.update(&build) {
            BuildUpdateResult::Added | BuildUpdateResult::BuildUpdated => {
                // The build was updated
                match context
                    .sender
                    .send(EngineEvent::BuildUpdated(Box::new(build)))
                {
                    Result::Ok(_) => (),
                    Result::Err(e) => error!("Failed to send build update event. {}", e),
                }
            }
            BuildUpdateResult::BuildStatusChanged => {
                // The build's status was changed (success->failed or failed->success)
                match context
                    .sender
                    .send(EngineEvent::BuildStatusChanged(Box::new(build)))
                {
                    Result::Ok(_) => (),
                    Result::Err(e) => error!("Failed to send build status event. {}", e),
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
    context
        .state
        .builds
        .retain_builds(&collector.info(), build_hashes);
}
