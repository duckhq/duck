use waithandle::WaitHandleListener;

use crate::builds::Build;
use crate::DuckResult;

mod appveyor;
mod azure;
mod debugger;
mod duck;
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
        handle: WaitHandleListener,
        callback: &mut dyn FnMut(Build),
    ) -> DuckResult<()>;
}

pub struct CollectorInfo {
    pub id: String,
    pub enabled: bool,
    pub provider: String,
}
