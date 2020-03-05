use crate::config::TeamCityConfiguration;
use crate::providers::{Collector, CollectorLoader};
use crate::DuckResult;

///////////////////////////////////////////////////////////
// Loader

impl CollectorLoader for TeamCityConfiguration {
    fn validate(&self) -> DuckResult<()> {
        Ok(())
    }

    fn load(&self) -> DuckResult<Box<dyn Collector>> {
        Ok(Box::new(TeamCityCollector::new(
            self.id.clone(),
            match self.enabled {
                Option::None => true,
                Option::Some(e) => e,
            },
        )))
    }
}

///////////////////////////////////////////////////////////
// Collector

pub struct TeamCityCollector {
    id: String,
    kind: String,
    enabled: bool,
}

impl TeamCityCollector {
    pub fn new(id: String, enabled: bool) -> Self {
        TeamCityCollector {
            id,
            kind: "TeamCity".to_owned(),
            enabled,
        }
    }
}

impl Collector for TeamCityCollector {
    fn id(&self) -> &str {
        &self.id
    }

    fn kind(&self) -> &str {
        &self.kind
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn collect(&self, _handle: std::sync::Arc<dyn waithandle::WaitHandle>) {
        unimplemented!()
    }
}
