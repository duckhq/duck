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

#[allow(clippy::borrowed_box)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builds::{BuildBuilder, BuildProvider, BuildStatus};
    use crate::config::Configuration;
    use crate::providers::collectors::CollectorInfo;
    use crate::utils::text::TestVariableProvider;
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
                    provider: BuildProvider::GitHub,
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
            _: Arc<EventWaitHandle>,
            callback: &mut dyn FnMut(Build),
        ) -> DuckResult<()> {
            callback(self.build.clone());
            return Ok(());
        }
    }

    #[test]
    fn should_send_build_updated_event_if_build_is_new() {
        // Given
        let config = Configuration::empty(&TestVariableProvider::new()).unwrap();
        let (sender, receiver) = channel::<EngineEvent>();
        let context = Context {
            handle: Arc::new(EventWaitHandle::new()),
            sender,
            state: Arc::new(EngineState::new(&config)),
        };

        let new_build = DummyCollector::new(BuildBuilder::dummy().build().unwrap());
        let collector = Box::new(new_build) as Box<dyn Collector>;

        // When
        process(&context, &collector);

        // Then
        assert!(receiver.try_recv().unwrap().is_build_updated());
    }

    #[test]
    #[should_panic(expected = "Channel does not have any events")]
    fn should_not_send_build_updated_event_if_build_is_known() {
        // Given
        let config = Configuration::empty(&TestVariableProvider::new()).unwrap();
        let (sender, receiver) = channel::<EngineEvent>();
        let context = Context {
            handle: Arc::new(EventWaitHandle::new()),
            sender,
            state: Arc::new(EngineState::new(&config)),
        };

        let current_build = BuildBuilder::dummy().build().unwrap();
        context.state.builds.update(&current_build);

        let new_build = BuildBuilder::dummy().build().unwrap();
        let collector = Box::new(DummyCollector::new(new_build)) as Box<dyn Collector>;

        // When
        process(&context, &collector);

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
        let config = Configuration::empty(&TestVariableProvider::new()).unwrap();
        let (sender, receiver) = channel::<EngineEvent>();
        let context = Context {
            handle: Arc::new(EventWaitHandle::new()),
            sender,
            state: Arc::new(EngineState::new(&config)),
        };

        let current_build = BuildBuilder::dummy().status(from).build().unwrap();
        context.state.builds.update(&current_build);

        let new_build = BuildBuilder::dummy().status(to).build().unwrap();
        let collector = Box::new(DummyCollector::new(new_build)) as Box<dyn Collector>;

        // When
        process(&context, &collector);

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
        let config = Configuration::empty(&TestVariableProvider::new()).unwrap();
        let (sender, receiver) = channel::<EngineEvent>();
        let context = Context {
            handle: Arc::new(EventWaitHandle::new()),
            sender,
            state: Arc::new(EngineState::new(&config)),
        };

        let current_build = BuildBuilder::dummy().status(from).build().unwrap();
        context.state.builds.update(&current_build);

        let new_build = BuildBuilder::dummy().status(to).build().unwrap();
        let collector = Box::new(DummyCollector::new(new_build)) as Box<dyn Collector>;

        // When
        process(&context, &collector);

        // Then
        assert!(receiver.try_recv().unwrap().is_build_status_changed());
    }
}
