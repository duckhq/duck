use std::sync::Arc;
use waithandle::EventWaitHandle;

use log::info;

use crate::builds::{Build, BuildProvider};
use crate::collectors::azure::AzureDevOpsCollector;
use crate::collectors::teamcity::TeamCityCollector;
use crate::config::{CollectorConfiguration, Configuration};
use crate::utils::DuckResult;

pub mod azure;
pub mod teamcity;

pub trait Collector {
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

/// Creates collectors from a configuration.
pub fn create_collectors(config: &Configuration) -> Vec<Box<dyn Collector>> {
    let mut result = Vec::<Box<dyn Collector>>::new();
    for collector_config in config.collectors.iter() {
        match collector_config {
            CollectorConfiguration::TeamCity(c) => result.push(Box::new(TeamCityCollector::new(c))),
            CollectorConfiguration::Azure(c) => result.push(Box::new(AzureDevOpsCollector::new(c))),
        };
    }

    // Only keep enabled collectors.
    result.retain(|o| o.info().enabled);

    for collector in result.iter() {
        let collector_info = collector.info();
        info!(
            "Added {:?} collector '{}'.",
            collector_info.provider, collector_info.id
        );
    }

    result
}
