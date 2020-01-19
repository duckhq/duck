use std::collections::HashSet;

use log::info;

use crate::builds::{Build, BuildStatus};
use crate::config::{Configuration, ObserverConfiguration};
use crate::observers::hue::HueObserver;
use crate::observers::mattermost::MattermostObserver;
use crate::observers::slack::SlackObserver;
use crate::utils::DuckResult;

pub mod hue;
pub mod mattermost;
pub mod slack;

pub trait Observer {
    fn info(&self) -> &ObserverInfo;
    fn observe(&self, observation: Observation) -> DuckResult<()>;
}

pub struct ObserverInfo {
    pub id: String,
    pub enabled: bool,
    pub collectors: Option<HashSet<String>>,
}

pub enum Observation {
    DuckStatusChanged(BuildStatus),
    BuildUpdated(Box<Build>),
    BuildStatusChanged(Box<Build>),
    ShuttingDown,
}

pub enum ObservationOrigin<'a> {
    System,
    Collector(&'a str),
}

impl Observation {
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

pub fn create_observers(config: &Configuration) -> Vec<Box<dyn Observer>> {
    let mut result = Vec::<Box<dyn Observer>>::new();
    if config.observers.is_some() {
        for observer_config in config.observers.as_ref().unwrap().iter() {
            match observer_config {
                ObserverConfiguration::Hue(c) => result.push(Box::new(HueObserver::new(c))),
                ObserverConfiguration::Slack(c) => result.push(Box::new(SlackObserver::new(c))),
                ObserverConfiguration::Mattermost(c) => {
                    result.push(Box::new(MattermostObserver::new(c)))
                }
            };
        }
    }

    // Only keep enabled observers.
    result.retain(|o| o.info().enabled);

    for observer in result.iter() {
        info!("Added observer '{}'.", observer.info().id);
        if let Some(collectors) = &observer.info().collectors {
            for collector in collectors {
                info!(
                    "Observer '{}' is interested in collector '{}'",
                    observer.info().id,
                    collector
                );
            }
        };
    }

    return result;
}
