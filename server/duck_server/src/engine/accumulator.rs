use std::sync::mpsc::Receiver;
use std::sync::Arc;

use log::{debug, error};
use waithandle::WaitHandle;

use super::EngineThreadMessage;
use crate::providers;
use crate::providers::Collector;

///////////////////////////////////////////////////////////
// Context

pub struct Context {
    _handle: Arc<dyn WaitHandle>,
    receiver: Receiver<EngineThreadMessage>,
    collectors: Vec<Box<dyn Collector>>,
}

impl Context {
    pub fn new(handle: Arc<dyn WaitHandle>, receiver: Receiver<EngineThreadMessage>) -> Self {
        Self {
            _handle: handle,
            receiver,
            collectors: vec![],
        }
    }
}

///////////////////////////////////////////////////////////
// Accumulator

pub fn accumulate(context: &mut Context) {
    if let Some(config) = super::try_get_updated_configuration(&context.receiver) {
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
                return;
            }
        }
    }
}
