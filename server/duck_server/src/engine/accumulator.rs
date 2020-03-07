use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;

use log::{debug, error, trace};
use waithandle::WaitHandle;

use super::{AccumulatorMessage, EngineThreadMessage};
use crate::providers;
use crate::providers::Collector;

///////////////////////////////////////////////////////////
// Context

pub struct Context {
    handle: Arc<dyn WaitHandle>,
    engine_receiver: Receiver<EngineThreadMessage>,
    accumulator_sender: Sender<AccumulatorMessage>,
    collectors: Vec<Box<dyn Collector>>,
}

impl Context {
    pub fn new(
        handle: Arc<dyn WaitHandle>,
        engine_receiver: Receiver<EngineThreadMessage>,
        accumulator_sender: Sender<AccumulatorMessage>,
    ) -> Self {
        Self {
            handle,
            engine_receiver,
            accumulator_sender,
            collectors: vec![],
        }
    }
}

///////////////////////////////////////////////////////////
// Accumulator

pub fn accumulate(context: &mut Context) {
    if !check_for_updated_configuration(context) {
        return;
    }

    for collector in context.collectors.iter() {
        let result = collector.collect(context.handle.clone());
        if result.len() > 0 {
            trace!(
                "Sending {} items from {} collector...",
                result.len(),
                collector.kind()
            );
            if let Err(err) = context
                .accumulator_sender
                .send(AccumulatorMessage::NewBuilds(result))
            {
                error!("Could not send new messages to aggregator: {}", err);
            }
        }
    }
}

fn check_for_updated_configuration(context: &mut Context) -> bool {
    if let Some(config) = super::try_get_updated_configuration(&context.engine_receiver) {
        // Reload the configuration
        debug!("New configuration available!");

        // Create collectors from the new configuration.
        match providers::create_collectors(&config) {
            Ok(collectors) => {
                context.collectors.clear();
                for collector in collectors {
                    debug!("Loaded {} collector: {}", collector.kind(), collector.id());
                    context.collectors.push(collector);
                }
            }
            Err(err) => {
                error!("An error occured while loading collectors: {}", err);
                return false;
            }
        }
    }
    true
}

///////////////////////////////////////////////////////////
// Tests

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builds::*;
    use std::sync::mpsc::channel;
    use waithandle::EventWaitHandle;

    struct DummyCollector {
        builds: Vec<Build>,
    }
    impl DummyCollector {
        pub fn new(builds: Vec<Build>) -> Self {
            Self { builds }
        }
    }
    impl Collector for DummyCollector {
        fn id(&self) -> &str {
            "dummy"
        }
        fn kind(&self) -> &str {
            "Dummy"
        }
        fn enabled(&self) -> bool {
            true
        }
        fn collect(&self, _handle: Arc<dyn WaitHandle>) -> Vec<Build> {
            self.builds.clone()
        }
    }

    #[test]
    pub fn should_send_message_when_new_builds_are_encountered() {
        // Given
        let waithandle = EventWaitHandle::new();
        let (_, engine_receiver) = channel::<EngineThreadMessage>();
        let (builder_sender, build_receiver) = channel::<AccumulatorMessage>();
        let mut context = Context::new(Arc::new(waithandle), engine_receiver, builder_sender);
        
        context.collectors.push(Box::new(DummyCollector::new(vec![
            BuildBuilder::dummy().build().unwrap(),
            BuildBuilder::dummy().build().unwrap(),
        ])));

        // When
        accumulate(&mut context);

        // Then
        match build_receiver.try_recv().expect("No builds in queue") {
            AccumulatorMessage::NewBuilds(builds) => {
                assert_eq!(2, builds.len());
            }
        }
    }
}
