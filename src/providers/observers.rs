use std::collections::HashSet;

use crate::builds::{Build, BuildStatus};
use crate::DuckResult;

mod hue;
mod mattermost;
mod slack;

pub trait ObserverLoader {
    fn load(&self) -> DuckResult<Box<dyn Observer>>;
}

pub trait Observer: Send {
    fn info(&self) -> &ObserverInfo;
    fn observe(&self, observation: Observation) -> DuckResult<()>;
}

pub struct ObserverInfo {
    pub id: String,
    pub enabled: bool,
    pub collectors: Option<HashSet<String>>,
}

pub enum Observation<'a> {
    DuckStatusChanged(BuildStatus),
    BuildUpdated(&'a Build),
    BuildStatusChanged(&'a Build),
    ShuttingDown,
}

pub enum ObservationOrigin<'a> {
    System,
    Collector(&'a str),
}

impl<'a> Observation<'a> {
    /// Gets the collector for an observation.
    pub fn get_origin(&self) -> ObservationOrigin {
        match self {
            Observation::DuckStatusChanged(_) => ObservationOrigin::System,
            Observation::BuildUpdated(build) => ObservationOrigin::Collector(&build.collector),
            Observation::BuildStatusChanged(build) => {
                ObservationOrigin::Collector(&build.collector)
            }
            Observation::ShuttingDown => ObservationOrigin::System,
        }
    }
}
