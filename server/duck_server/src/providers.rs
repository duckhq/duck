use std::sync::Arc;

use waithandle::WaitHandle;

use crate::builds::Build;
use crate::config::{CollectorConfiguration, Configuration};
use crate::DuckResult;

mod teamcity;

///////////////////////////////////////////////////////////
// Collecting

pub trait Collector {
    fn id(&self) -> &str;
    fn kind(&self) -> &str;
    fn enabled(&self) -> bool;
    fn collect(&self, handle: Arc<dyn WaitHandle>) -> Vec<Build>;
}

pub trait CollectorLoader {
    fn validate(&self) -> DuckResult<()>;
    fn load(&self) -> DuckResult<Box<dyn Collector>>;
}

pub fn create_collectors(config: &Configuration) -> DuckResult<Vec<Box<dyn Collector>>> {
    let mut collectors = Vec::<Box<dyn Collector>>::new();
    for config in config.collectors.iter() {
        let loader = get_collector_loader(&config);
        loader.validate()?;
        collectors.push(loader.load()?);
    }
    Ok(collectors)
}

fn get_collector_loader(config: &CollectorConfiguration) -> &impl CollectorLoader {
    match config {
        CollectorConfiguration::TeamCity(config) => config,
    }
}
