use std::sync::Arc;

use waithandle::EventWaitHandle;

use crate::builds::{Build, BuildProvider};
use crate::config::{CollectorConfiguration, Configuration, Validate};
use crate::utils::DuckResult;

use self::azure::AzureDevOpsCollector;
use self::octopus::OctopusDeployCollector;
use self::teamcity::TeamCityCollector;

use super::DuckProvider;

mod azure;
mod octopus;
mod teamcity;

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

pub struct AzureDevOpsProvider {}
impl<'a> DuckProvider<'a> for AzureDevOpsProvider {
    fn get_collectors(&self, config: &Configuration) -> DuckResult<Vec<Box<dyn Collector>>> {
        let mut result = Vec::<Box<dyn Collector>>::new();
        for item in config.collectors.iter() {
            if let CollectorConfiguration::Azure(c) = item {
                c.validate()?;
                result.push(Box::new(AzureDevOpsCollector::new(&c)));
            }
        }
        return Ok(result);
    }
}

pub struct TeamCityProvider {}
impl<'a> DuckProvider<'a> for TeamCityProvider {
    fn get_collectors(&self, config: &Configuration) -> DuckResult<Vec<Box<dyn Collector>>> {
        let mut result = Vec::<Box<dyn Collector>>::new();
        for item in config.collectors.iter() {
            if let CollectorConfiguration::TeamCity(c) = item {
                c.validate()?;
                result.push(Box::new(TeamCityCollector::new(&c)));
            }
        }
        return Ok(result);
    }
}

pub struct OctopusDeployProvider {}
impl<'a> DuckProvider<'a> for OctopusDeployProvider {
    fn get_collectors(&self, config: &Configuration) -> DuckResult<Vec<Box<dyn Collector>>> {
        let mut result = Vec::<Box<dyn Collector>>::new();
        for item in config.collectors.iter() {
            if let CollectorConfiguration::OctopusDeploy(c) = item {
                c.validate()?;
                result.push(Box::new(OctopusDeployCollector::new(&c)));
            }
        }
        return Ok(result);
    }
}
