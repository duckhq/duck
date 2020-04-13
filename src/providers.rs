pub mod collectors;
pub mod observers;

use log::debug;

use crate::config::{CollectorConfiguration, Configuration, ObserverConfiguration};
use crate::DuckResult;

use self::collectors::*;
use self::observers::*;

///////////////////////////////////////////////////////////
// Collectors

pub fn create_collectors(config: &Configuration) -> DuckResult<Vec<Box<dyn Collector>>> {
    let mut collectors = Vec::<Box<dyn Collector>>::new();
    for config in config.collectors.iter() {
        if config.is_enabled() {
            let loader = get_collector_loader(&config);
            collectors.push(loader.load()?);
        } else {
            debug!("Collector '{}' has been disabled.", config.get_id());
        }
    }
    Ok(collectors)
}

fn get_collector_loader(config: &CollectorConfiguration) -> Box<&dyn CollectorLoader> {
    match config {
        CollectorConfiguration::TeamCity(config) => Box::new(config),
        CollectorConfiguration::Azure(config) => Box::new(config),
        CollectorConfiguration::GitHub(config) => Box::new(config),
        CollectorConfiguration::OctopusDeploy(config) => Box::new(config),
        CollectorConfiguration::AppVeyor(config) => Box::new(config),
        CollectorConfiguration::Duck(config) => Box::new(config),
        CollectorConfiguration::Debugger(config) => Box::new(config),
    }
}

///////////////////////////////////////////////////////////
// Observers

pub fn create_observers(config: &Configuration) -> DuckResult<Vec<Box<dyn Observer>>> {
    let mut result = Vec::<Box<dyn Observer>>::new();
    if let Some(observers) = &config.observers {
        for config in observers.iter() {
            if config.is_enabled() {
                let loader = get_observer_loader(&config);
                match loader.load() {
                    Ok(observer) => {
                        result.push(observer);
                    }
                    Err(e) => {
                        return Err(format_err!(
                            "An error occured when loading observer '{}'. {}",
                            config.get_id(),
                            e
                        ))
                    }
                }
            } else {
                debug!("Observer '{}' has been disabled.", config.get_id());
            }
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
