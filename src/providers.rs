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
            debug!("Collector '{}' has been disabled", config.get_id());
        }
    }
    Ok(collectors)
}

fn get_collector_loader(config: &CollectorConfiguration) -> &dyn CollectorLoader {
    match config {
        CollectorConfiguration::TeamCity(config) => config,
        CollectorConfiguration::Azure(config) => config,
        CollectorConfiguration::GitHub(config) => config,
        CollectorConfiguration::OctopusDeploy(config) => config,
        CollectorConfiguration::AppVeyor(config) => config,
        CollectorConfiguration::Duck(config) => config,
        CollectorConfiguration::Debugger(config) => config,
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
                debug!("Observer '{}' has been disabled", config.get_id());
            }
        }
    }
    Ok(result)
}

fn get_observer_loader(config: &ObserverConfiguration) -> &dyn ObserverLoader {
    match config {
        ObserverConfiguration::Hue(config) => config,
        ObserverConfiguration::Mattermost(config) => config,
        ObserverConfiguration::Slack(config) => config,
    }
}
