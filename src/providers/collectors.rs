use std::sync::Arc;

use waithandle::EventWaitHandle;

use crate::builds::{Build, BuildProvider};
use crate::DuckResult;

mod appveyor;
mod azure;
mod github;
mod octopus;
mod teamcity;

pub trait CollectorLoader {
    fn load(&self) -> DuckResult<Box<dyn Collector>>;
}

pub trait Collector: Send {
    fn info(&self) -> &CollectorInfo;
    fn collect(
        &self,
        handle: Arc<EventWaitHandle>,
        callback: &mut dyn FnMut(Build),
    ) -> DuckResult<()>;
}

pub struct CollectorInfo {
    pub id: String,
    pub enabled: bool,
    pub provider: BuildProvider,
}
