pub mod collectors;
pub mod observers;

use crate::config::Configuration;
use crate::DuckResult;

use crate::config::{CollectorConfiguration, ObserverConfiguration};

use self::collectors::*;
use self::observers::*;

///////////////////////////////////////////////////////////
// Collectors

pub fn create_collectors(config: &Configuration) -> DuckResult<Vec<Box<dyn Collector>>> {
    let mut collectors = Vec::<Box<dyn Collector>>::new();
    for config in config.collectors.iter() {
        let loader = get_collector_loader(&config);
        collectors.push(loader.load()?);
    }
    Ok(collectors)
}

fn get_collector_loader(config: &CollectorConfiguration) -> Box<&dyn CollectorLoader> {
    match config {
        CollectorConfiguration::TeamCity(config) => Box::new(config),
        CollectorConfiguration::Azure(config) => Box::new(config),
        CollectorConfiguration::GitHub(config) => Box::new(config),
        CollectorConfiguration::OctopusDeploy(config) => Box::new(config),
    }
}

///////////////////////////////////////////////////////////
// Observers

pub fn create_observers(config: &Configuration) -> DuckResult<Vec<Box<dyn Observer>>> {
    let mut result = Vec::<Box<dyn Observer>>::new();
    if let Some(observers) = &config.observers {
        for config in observers.iter() {
            let loader = get_observer_loader(&config);
            result.push(loader.load()?);
        }
    }
    Ok(result)
}

fn get_observer_loader(config: &ObserverConfiguration) -> Box<&dyn ObserverLoader> {
    match config {
        ObserverConfiguration::Hue(config) => Box::new(config),
        ObserverConfiguration::Mattermost(config) => Box::new(config),
        ObserverConfiguration::Slack(config) => Box::new(config),
    }
}
