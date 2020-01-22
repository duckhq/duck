pub mod collectors;
pub mod observers;

use crate::config::Configuration;
use crate::utils::DuckResult;

use self::collectors::*;
use self::observers::*;

pub trait DuckProvider<'a>: Send {
    fn get_observers(&self, _config: &Configuration) -> DuckResult<Vec<Box<dyn Observer>>> {
        Ok(Vec::<Box<dyn Observer>>::new())
    }

    fn get_collectors(&self, _config: &Configuration) -> DuckResult<Vec<Box<dyn Collector>>> {
        Ok(Vec::<Box<dyn Collector>>::new())
    }
}

pub struct DuckProviderCollection<'a> {
    providers: Vec<&'a dyn DuckProvider<'a>>,
}

impl<'a> DuckProviderCollection<'a> {
    pub fn new() -> Self {
        let mut providers = Vec::<&'a dyn DuckProvider>::new();
        providers.push(&AzureDevOpsProvider {});
        providers.push(&TeamCityProvider {});
        providers.push(&OctopusDeployProvider {});
        providers.push(&HueProvider {});
        providers.push(&SlackProvider {});
        providers.push(&MattermostProvider {});

        DuckProviderCollection { providers }
    }

    pub fn get_collectors(&self, config: &Configuration) -> DuckResult<Vec<Box<dyn Collector>>> {
        let mut result = Vec::<Box<dyn Collector>>::new();
        for provider in self.providers.iter() {
            let collectors = provider.get_collectors(config)?;
            for collector in collectors {
                if collector.info().enabled {
                    result.push(collector);
                }
            }
        }
        Ok(result)
    }

    pub fn get_observers(&self, config: &Configuration) -> DuckResult<Vec<Box<dyn Observer>>> {
        let mut result = Vec::<Box<dyn Observer>>::new();
        for provider in self.providers.iter() {
            let observers = provider.get_observers(config)?;
            for observer in observers {
                if observer.info().enabled {
                    result.push(observer);
                }
            }
        }
        Ok(result)
    }
}
