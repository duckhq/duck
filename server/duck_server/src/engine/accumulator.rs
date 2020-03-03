use std::sync::mpsc::Receiver;
use std::sync::Arc;

use log::debug;
use waithandle::WaitHandle;

use super::EngineThreadMessage;

///////////////////////////////////////////////////////////
// Context

pub struct Context {
    _handle: Arc<dyn WaitHandle>,
    receiver: Receiver<EngineThreadMessage>,
}

impl Context {
    pub fn new(handle: Arc<dyn WaitHandle>, receiver: Receiver<EngineThreadMessage>) -> Self {
        Self {
            _handle: handle,
            receiver,
        }
    }
}

///////////////////////////////////////////////////////////
// Accumulator

pub fn accumulate(context: &Context) {
    if let Some(_config) = super::try_get_updated_configuration(&context.receiver) {
        // Reload the configuration
        debug!("New configuration available!");
    }
}
