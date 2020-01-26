use std::collections::HashSet;

use crate::builds::{Build, BuildStatus};
use crate::config::{Configuration, ObserverConfiguration, Validate};
use crate::utils::http::ReqwestClient;
use crate::utils::DuckResult;

use self::hue::HueObserver;
use self::mattermost::MattermostObserver;
use self::slack::SlackObserver;

use super::DuckProvider;

mod hue;
mod mattermost;
mod slack;

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

pub struct HueProvider {}
impl<'a> DuckProvider<'a> for HueProvider {
    fn get_observers(&self, config: &Configuration) -> DuckResult<Vec<Box<dyn Observer>>> {
        let mut result = Vec::<Box<dyn Observer>>::new();
        if let Some(observers) = &config.observers {
            for item in observers.iter() {
                if let ObserverConfiguration::Hue(c) = item {
                    c.validate()?;
                    result.push(Box::new(HueObserver::<ReqwestClient>::new(&c)));
                }
            }
        }
        Ok(result)
    }
}

pub struct MattermostProvider {}
impl<'a> DuckProvider<'a> for MattermostProvider {
    fn get_observers(&self, config: &Configuration) -> DuckResult<Vec<Box<dyn Observer>>> {
        let mut result = Vec::<Box<dyn Observer>>::new();
        if let Some(observers) = &config.observers {
            for item in observers.iter() {
                if let ObserverConfiguration::Mattermost(c) = item {
                    c.validate()?;
                    result.push(Box::new(MattermostObserver::<ReqwestClient>::new(&c)));
                }
            }
        }
        Ok(result)
    }
}

pub struct SlackProvider {}
impl<'a> DuckProvider<'a> for SlackProvider {
    fn get_observers(&self, config: &Configuration) -> DuckResult<Vec<Box<dyn Observer>>> {
        let mut result = Vec::<Box<dyn Observer>>::new();
        if let Some(observers) = &config.observers {
            for item in observers.iter() {
                if let ObserverConfiguration::Slack(c) = item {
                    c.validate()?;
                    result.push(Box::new(SlackObserver::<ReqwestClient>::new(&c)));
                }
            }
        }
        Ok(result)
    }
}
