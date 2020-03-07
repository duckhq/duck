use std::sync::mpsc::Receiver;
use std::sync::Arc;

use log::debug;
use waithandle::WaitHandle;

use super::{AccumulatorMessage, EngineThreadMessage};

///////////////////////////////////////////////////////////
// Context

pub struct Context {
    _handle: Arc<dyn WaitHandle>,
    engine_channel: Receiver<EngineThreadMessage>,
    _accumulator_channel: Receiver<AccumulatorMessage>,
}

impl Context {
    pub fn new(
        handle: Arc<dyn WaitHandle>,
        engine_channel: Receiver<EngineThreadMessage>,
        accumulator_channel: Receiver<AccumulatorMessage>,
    ) -> Self {
        Self {
            _handle: handle,
            engine_channel,
            _accumulator_channel: accumulator_channel,
        }
    }
}

///////////////////////////////////////////////////////////
// Accumulator

pub fn aggregate(context: &Context) {
    if let Some(_config) = super::try_get_updated_configuration(&context.engine_channel) {
        debug!("New configuration available!");
    }
}
