use std::collections::HashSet;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;

use log::{debug, error, trace};
use waithandle::WaitHandleListener;

use crate::builds::Build;
use crate::engine::{EngineEvent, EngineState};
use crate::providers;
use crate::providers::collectors::Collector;
use crate::DuckResult;

use super::state::builds::BuildUpdateResult;
use super::EngineThreadMessage;

pub struct Context {
    listener: WaitHandleListener,
    engine_receiver: Receiver<EngineThreadMessage>,
    state: Arc<EngineState>,
    sender: Sender<EngineEvent>,
    collectors: Vec<Box<dyn Collector>>,
}

impl Context {
    pub fn new(
        handle: WaitHandleListener,
        state: Arc<EngineState>,
        engine_receiver: Receiver<EngineThreadMessage>,
        accumulator_sender: Sender<EngineEvent>,
    ) -> Self {
        Self {
            listener: handle,
            engine_receiver,
            state,
            sender: accumulator_sender,
            collectors: vec![],
        }
    }
}

#[allow(clippy::borrowed_box)]
pub fn accumulate(context: &mut Context) {
    if let Err(e) = check_for_updated_configuration(context) {
        error!("{}", e);
        return;
    }

    for collector in context.collectors.iter() {
        let mut build_hashes = std::collections::HashSet::<u64>::new();
        if let Err(e) = collector.collect(context.listener.clone(), &mut |build: Build| {
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
                BuildUpdateResult::AbsoluteBuildStatusChanged => {
                    // The build's status was changed (success->failed or failed->success)
                    match context
                        .sender
                        .send(EngineEvent::AbsoluteBuildStatusChanged(Box::new(build)))
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
}

pub enum ConfigurationResult {
    Unchanged,
    Updated,
}

pub fn check_for_updated_configuration(context: &mut Context) -> DuckResult<ConfigurationResult> {
    if let Some(config) =
        super::try_get_updated_configuration(&context.listener, &context.engine_receiver)
    {
        trace!("Applying new configuration...");
        match providers::create_collectors(&config) {
            Ok(collectors) => {
                context.collectors.clear();
                for collector in collectors {
                    debug!(
                        "Loaded {} collector: {}",
                        collector.info().provider,
                        collector.info().id
                    );
                    context.collectors.push(collector);
                }

                // Remove state for unloaded collectors.
                let mut collector_ids = HashSet::<String>::new();
                for collector in context.collectors.iter() {
                    collector_ids.insert(collector.info().id.clone());
                }
                context.state.builds.retain(&collector_ids);
                return Ok(ConfigurationResult::Updated);
            }
            Err(err) => {
                return Err(format_err!(
                    "An error occured while loading collectors: {}",
                    err
                ));
            }
        }
    }
    Ok(ConfigurationResult::Unchanged)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builds::{BuildBuilder, BuildStatus};
    use crate::providers::collectors::CollectorInfo;
    use crate::DuckResult;
    use std::sync::mpsc::channel;
    use test_case::test_case;

    pub struct DummyCollector {
        pub build: Build,
        pub info: CollectorInfo,
    }

    impl DummyCollector {
        pub fn new(build: Build) -> Self {
            DummyCollector {
                build,
                info: CollectorInfo {
                    id: "dummy".to_owned(),
                    enabled: true,
                    provider: "GitHub".to_owned(),
                },
            }
        }
    }

    impl Collector for DummyCollector {
        fn info(&self) -> &crate::providers::collectors::CollectorInfo {
            &self.info
        }
        fn collect(
            &self,
            _: WaitHandleListener,
            callback: &mut dyn FnMut(Build),
        ) -> DuckResult<()> {
            callback(self.build.clone());
            return Ok(());
        }
    }

    #[test]
    fn should_send_build_updated_event_if_build_is_new() {
        // Given
        let (sender, receiver) = channel::<EngineEvent>();
        let (_, engine_receiver) = channel::<EngineThreadMessage>();
        let (_, listener) = waithandle::new();

        let mut context = Context {
            listener,
            sender,
            engine_receiver,
            state: Arc::new(EngineState::new()),
            collectors: Vec::new(),
        };

        let new_build = DummyCollector::new(BuildBuilder::dummy().build().unwrap());
        let collector = Box::new(new_build) as Box<dyn Collector>;
        context.collectors.push(collector);

        // When
        accumulate(&mut context);

        // Then
        assert!(receiver.try_recv().unwrap().is_build_updated());
    }

    #[test]
    #[should_panic(expected = "Channel does not have any events")]
    fn should_not_send_build_updated_event_if_build_is_known() {
        // Given
        let (sender, receiver) = channel::<EngineEvent>();
        let (_, engine_receiver) = channel::<EngineThreadMessage>();
        let (_, listener) = waithandle::new();

        let mut context = Context {
            listener,
            sender,
            engine_receiver,
            state: Arc::new(EngineState::new()),
            collectors: Vec::new(),
        };

        let current_build = BuildBuilder::dummy().build().unwrap();
        context.state.builds.update(&current_build);

        let new_build = BuildBuilder::dummy().build().unwrap();
        let collector = Box::new(DummyCollector::new(new_build)) as Box<dyn Collector>;
        context.collectors.push(collector);

        // When
        accumulate(&mut context);

        // Then
        receiver
            .try_recv()
            .expect("Channel does not have any events");
    }

    #[test_case(BuildStatus::Canceled, BuildStatus::Running ; "Cancelled -> Running")]
    #[test_case(BuildStatus::Canceled, BuildStatus::Unknown ; "Cancelled -> Unknown")]
    #[test_case(BuildStatus::Canceled, BuildStatus::Queued ; "Cancelled -> Queued")]
    #[test_case(BuildStatus::Running, BuildStatus::Canceled ; "Running -> Canceled")]
    #[test_case(BuildStatus::Running, BuildStatus::Unknown ; "Running -> Unknown")]
    #[test_case(BuildStatus::Running, BuildStatus::Queued ; "Running -> Queued")]
    #[test_case(BuildStatus::Unknown, BuildStatus::Canceled ; "Unknown -> Canceled")]
    #[test_case(BuildStatus::Unknown, BuildStatus::Queued ; "Unknown -> Queued")]
    #[test_case(BuildStatus::Unknown, BuildStatus::Running ; "Unknown -> Running")]
    #[test_case(BuildStatus::Queued, BuildStatus::Canceled ; "Queued -> Canceled")]
    #[test_case(BuildStatus::Queued, BuildStatus::Unknown ; "Queued -> Unknown")]
    #[test_case(BuildStatus::Queued, BuildStatus::Running ; "Queued -> Running")]
    #[test_case(BuildStatus::Success, BuildStatus::Canceled ; "Success -> Canceled")]
    #[test_case(BuildStatus::Success, BuildStatus::Unknown ; "Success -> Unknown")]
    #[test_case(BuildStatus::Success, BuildStatus::Running ; "Success -> Running")]
    #[test_case(BuildStatus::Success, BuildStatus::Queued ; "Success -> Queued")]
    #[test_case(BuildStatus::Failed, BuildStatus::Canceled ; "Failed -> Canceled")]
    #[test_case(BuildStatus::Failed, BuildStatus::Unknown ; "Failed -> Unknown")]
    #[test_case(BuildStatus::Failed, BuildStatus::Running ; "Failed -> Running")]
    #[test_case(BuildStatus::Failed, BuildStatus::Queued ; "Failed -> Queued")]
    fn should_send_build_updated_event_if_non_absolute_build_status_change(
        from: BuildStatus,
        to: BuildStatus,
    ) {
        // Given
        let (sender, receiver) = channel::<EngineEvent>();
        let (_, engine_receiver) = channel::<EngineThreadMessage>();
        let (_, listener) = waithandle::new();

        let mut context = Context {
            listener,
            sender,
            engine_receiver,
            state: Arc::new(EngineState::new()),
            collectors: Vec::new(),
        };

        let current_build = BuildBuilder::dummy().status(from).build().unwrap();
        context.state.builds.update(&current_build);

        let new_build = BuildBuilder::dummy().status(to).build().unwrap();
        let collector = Box::new(DummyCollector::new(new_build)) as Box<dyn Collector>;
        context.collectors.push(collector);

        // When
        accumulate(&mut context);

        // Then
        assert!(receiver.try_recv().unwrap().is_build_updated());
    }

    #[test_case(BuildStatus::Unknown, BuildStatus::Failed ; "Unknown -> Failed")]
    #[test_case(BuildStatus::Unknown, BuildStatus::Success ; "Unknown -> Success")]
    #[test_case(BuildStatus::Canceled, BuildStatus::Failed ; "Canceled -> Failed")]
    #[test_case(BuildStatus::Canceled, BuildStatus::Success ; "Canceled -> Success")]
    #[test_case(BuildStatus::Queued, BuildStatus::Failed ; "Queued -> Failed")]
    #[test_case(BuildStatus::Queued, BuildStatus::Success ; "Queued -> Success")]
    #[test_case(BuildStatus::Running, BuildStatus::Failed ; "Running -> Failed")]
    #[test_case(BuildStatus::Running, BuildStatus::Success ; "Running -> Success")]
    fn should_send_build_status_changed_event_if_absolute_build_status_change(
        from: BuildStatus,
        to: BuildStatus,
    ) {
        // Given
        let (sender, receiver) = channel::<EngineEvent>();
        let (_, engine_receiver) = channel::<EngineThreadMessage>();
        let (_, listener) = waithandle::new();

        let mut context = Context {
            listener,
            sender,
            engine_receiver,
            state: Arc::new(EngineState::new()),
            collectors: Vec::new(),
        };

        let current_build = BuildBuilder::dummy().status(from).build().unwrap();
        context.state.builds.update(&current_build);

        let new_build = BuildBuilder::dummy().status(to).build().unwrap();
        let collector = Box::new(DummyCollector::new(new_build)) as Box<dyn Collector>;
        context.collectors.push(collector);

        // When
        accumulate(&mut context);

        // Then
        assert!(receiver.try_recv().unwrap().is_build_status_changed());
    }
}
